use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashSet, time::Duration};

use anyhow::Result;
use futures_util::future::join_all;
use tokio::{
    select,
    sync::{broadcast, Semaphore},
};
use tracing::error;

use crate::actions::AppAction;
use crate::ui::notification::{
    Notification, NotificationId, NotificationType,
};

use super::config::load_config_from_file;
use super::job::{map_configuration_to_jobs, Job, JobDTO};
use super::{
    types::AppError,
    utils::{get_random_emoji, prettify_duration, EmojiType},
};

const MAX_RETRY_COUNT: u32 = 10;

#[derive(Clone, Debug)]
enum JobEvent {
    Restart(String),
    Finished(String),
    Terminate,
}

pub struct App {
    pub jobs: Vec<Job>,
    pub jobs_semaphore: Arc<Semaphore>,
    pub max_threads_semaphore: Arc<Semaphore>,
    pub app_actions_sender: broadcast::Sender<AppAction>,
}

impl App {
    pub fn new(
        app_actions_sender: broadcast::Sender<AppAction>,
    ) -> Result<App, AppError> {
        let jobs_semaphore = Arc::new(Semaphore::new(0));
        let max_threads_semaphore =
            Arc::new(Semaphore::new(num_cpus::get() * 2));

        Ok(App {
            jobs_semaphore,
            max_threads_semaphore,
            jobs: Vec::new(),
            app_actions_sender,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.reset_all_jobs_and_publish();

        let total_jobs_count = self.jobs.len();
        let (events_sender, mut events_receiver) =
            broadcast::channel::<JobEvent>(total_jobs_count);

        let events_sender_ref = events_sender.clone();

        tokio::spawn(async move {
            let mut finished_jobs = HashSet::new();

            loop {
                let action = match events_receiver.recv().await {
                    Ok(action) => action,
                    Err(_) => continue,
                };

                match action {
                    JobEvent::Restart(name) => {
                        finished_jobs.remove(&name);
                    }
                    JobEvent::Finished(name) => {
                        finished_jobs.insert(name);
                    }
                    _ => {}
                }

                if finished_jobs.len() == total_jobs_count {
                    let _ = events_sender_ref.send(JobEvent::Terminate);

                    break;
                }
            }
        });

        let handles = self.jobs.iter_mut().map(|job| {
            let mut job_ref = job.clone();
            let job_name = job_ref.job_name.clone();
            let mut app_actions_receiver = self.app_actions_sender.clone().subscribe();
            let events_sender = events_sender.clone();

            let mut events_receiver = events_sender.subscribe();

            tokio::spawn(async move {
                let mut should_run = true;
                let mut result: Option<Result<Job>> = None;

                let mut retry_count = 0;

                while should_run {
                    select! {
                        _ = async {
                                loop {
                                    let action = match app_actions_receiver.recv().await {
                                        Ok(action) => action,
                                        Err(_) => continue,
                                    };

                                    match action {
                                        AppAction::StartOneJob(name) => {

                                            if job_name == name {
                                                let _ = events_sender.send(JobEvent::Restart(name));
                                                break
                                            }
                                        }
                                        _ => {}
                                    }
                                 }
                            } => {
                                retry_count = 0;
                            }

                        job = async {
                            job_ref.start().await?;

                            let _ = events_sender.send(JobEvent::Finished(job_name.clone()));

                            loop {
                                let action = match events_receiver.recv().await {
                                    Ok(action) => action,
                                    Err(_) => continue,
                                };

                                match action {
                                    JobEvent::Terminate =>
                                        {
                                            break
                                        }
                                    _ => {}
                                }

                            }

                            should_run = false;


                            Ok(job_ref.clone())
                        } => {
                            match job {
                                Err(_) => {
                                    if retry_count > MAX_RETRY_COUNT {
                                        should_run = false;
                                    }
                                    retry_count += 1;
                                }
                                _=> {
                                }
                            }

                            result = Some(job);
                        }
                    };
                }

                return result;
            })
        });

        let started_at = Instant::now();

        let results = join_all(handles).await;

        for handle_result in results.iter() {
            match handle_result {
                Ok(updated_job) => {
                    for job in self.jobs.iter_mut() {
                        match updated_job {
                            Some(Ok(updated_job)) => {
                                if job.job_name == updated_job.job_name {
                                    *job = updated_job.clone()
                                }
                            }
                            _ => {
                                error!("Should not be reachable. Expected job to be defined");

                                return Err(AppError::Exception(
                                    "Critical runtime error occurred".into(),
                                )
                                .into());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("critical error: {}", e);

                    return Err(AppError::Exception(
                        "Failed to start job execution".into(),
                    )
                    .into());
                }
            }
        }

        let failed_jobs: Vec<&Job> =
            self.jobs.iter().filter(|job| job.is_failed()).collect();

        let time_took = prettify_duration(started_at.elapsed());

        let notification;
        if failed_jobs.is_empty() {
            notification = Notification::new(
                NotificationId::AllRequestsFinishedWithoutFails,
                &format!(
                    "All requests are finished in {} {}",
                    time_took,
                    get_random_emoji(EmojiType::Happy)
                ),
                Some(Duration::from_secs(5)),
                NotificationType::Success,
            );
        } else {
            notification = Notification::new(
                NotificationId::AllRequestsFinishedWithFails,
                &format!(
                    "All requests are finished in {}. {} failed {}.",
                    time_took,
                    failed_jobs.len(),
                    get_random_emoji(EmojiType::Sad)
                ),
                Some(Duration::from_secs(5)),
                NotificationType::Warning,
            );
        }

        let _ = self
            .app_actions_sender
            .send(AppAction::SetNotification(notification));

        Ok(())
    }

    pub async fn start_by_name(&mut self, name: &str) {
        let mut app_actions_receiver = self.app_actions_sender.subscribe();

        if let Some(target_job) =
            self.jobs.iter_mut().find(|job| job.job_name == name)
        {
            let job_name = target_job.job_name.clone();

            select! {
                _ = async {
                    loop {
                        let action = match app_actions_receiver.recv().await {
                            Ok(action) => action,
                            Err(_) => continue,
                        };

                        match action {
                            AppAction::StartOneJob(name) => {
                                if job_name == name {
                                    break
                                }
                            }
                            _ => {}
                        }
                    };
                } => {
                }
                _ = target_job.start() => {}
            }
        }
    }

    pub fn reset_all_jobs_and_publish(&mut self) {
        for job in self.jobs.iter_mut() {
            job.reset()
        }

        let jobs: Vec<JobDTO> =
            self.jobs.iter().map(|job| job.clone().into()).collect();

        let _ = self.app_actions_sender.send(AppAction::JobsUpdated(jobs));
    }

    pub async fn load_configuration_file(
        &mut self,
        path_to_file: &str,
    ) -> Result<()> {
        let configuration = load_config_from_file(path_to_file)?;

        self.jobs_semaphore =
            Arc::new(Semaphore::new(configuration.concurrent_jobs));

        let jobs = map_configuration_to_jobs(
            &configuration,
            self.app_actions_sender.clone(),
            self.jobs_semaphore.clone(),
            self.max_threads_semaphore.clone(),
        )?;

        let _ = self
            .app_actions_sender
            .send(AppAction::ConfigurationLoaded(configuration));

        self.jobs = jobs;

        let total_requests: usize =
            self.jobs.iter().map(|job| job.requests.len()).sum();

        let mut current_processing_job_index = 0;

        for job in self.jobs.iter_mut() {
            for request in job.requests.iter_mut() {
                current_processing_job_index += 1;

                let _ = self.app_actions_sender.send(
                    AppAction::LoadingJobsProgress((
                        current_processing_job_index,
                        total_requests,
                    )),
                );

                let command = job.request_builder.clone();

                if let Some(command) = command {
                    let _ = self.max_threads_semaphore.acquire().await?;

                    match Job::apply_request_builder_to_request(&command, &request).await {
                        Ok(Some(request_builder_dto)) => {
                            request.apply_request_builder_dto(request_builder_dto)
                        }
                        _ => {
                            return Err(AppError::Exception(format!(
                                "Failed to apply request builder: '{}' to request: '{}'",
                                command.join(" "),
                                request.uri
                            )).into())
                        }
                    };
                }
            }
        }

        let mapped = self.jobs.iter().map(|job| job.clone().into()).collect();

        let _ = self.app_actions_sender.send(AppAction::JobsUpdated(mapped));
        let _ = self.app_actions_sender.send(AppAction::StartAllJobs);

        Ok(())
    }

    pub async fn reload_configuration_file(
        &mut self,
        path_to_file: &str,
    ) -> Result<()> {
        let notification = Notification::new(
            NotificationId::ReloadingConfiguration,
            "Reloading configuration file as it was changed.",
            Some(Duration::from_secs(5)),
            NotificationType::Warning,
        );

        let _ = self
            .app_actions_sender
            .send(AppAction::SetNotification(notification));

        return self.load_configuration_file(path_to_file).await;
    }
}
