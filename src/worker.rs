use futures::future::join_all;
use notify::{
    event::{MetadataKind, ModifyKind},
    Config, Event, EventKind, PollWatcher, RecursiveMode, Watcher,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    select,
    sync::broadcast::{Receiver, Sender},
};
use tracing::{debug, error};

use crate::{
    actions::AppAction,
    http_diff::{app::App, types::AppError},
    ui::notification::{Notification, NotificationId, NotificationType},
};

pub async fn process_app_action(
    action: AppAction,
    worker_actions_sender: Sender<AppAction>,
    base_output_directory: PathBuf,
) {
    match action {
        AppAction::SaveCurrentJob(job) => {
            match job.save(&base_output_directory).await {
                Ok(()) => {
                    let notification = Notification::new(
                        NotificationId::SavedJob,
                        &format!(
                            "Saved job to {}",
                            &base_output_directory
                                .canonicalize()
                                .unwrap_or_else(|_| PathBuf::new())
                                .to_str()
                                .unwrap_or_else(|| "")
                        ),
                        Some(Duration::from_secs(5)),
                        NotificationType::Success,
                    );
                    let _ = worker_actions_sender
                        .send(AppAction::SetNotification(notification));
                }
                Err(error) => {
                    let notification = Notification::new(
                        NotificationId::FailedToSaveJobs,
                        "Failed to save job",
                        None,
                        NotificationType::Error,
                    );

                    let _ = worker_actions_sender
                        .send(AppAction::SetNotification(notification));

                    error!("error: {}", error);
                }
            };
        }
        AppAction::SaveFailedJobs(jobs) => {
            let jobs_count = jobs.len();

            let tasks: Vec<_> = jobs
                .into_iter()
                .map(|job| {
                    let output = base_output_directory.clone();

                    tokio::spawn(async move { job.save(&output).await })
                })
                .collect();

            join_all(tasks).await;

            let jobs_display_text_part =
                if jobs_count == 1 { "job" } else { "jobs" };

            let notification = Notification::new(
                NotificationId::SavedJobs,
                &format!(
                    "Saved {} {jobs_display_text_part} to {}",
                    jobs_count,
                    base_output_directory
                        .canonicalize()
                        .unwrap_or_else(|_| PathBuf::new())
                        .to_str()
                        .unwrap_or_else(|| "")
                ),
                Some(Duration::from_secs(5)),
                NotificationType::Success,
            );

            let _ = worker_actions_sender
                .send(AppAction::SetNotification(notification));
        }
        _ => {}
    }
}

pub async fn handle_commands_to_http_diff_loop(
    http_diff_actions_receiver: &mut Receiver<AppAction>,
    http_diff: &mut App,
) -> Result<(), AppError> {
    loop {
        let action = match http_diff_actions_receiver.recv().await {
            Ok(action) => action,
            Err(_) => continue,
        };

        match action {
            AppAction::StartAllJobs => {
                let mut should_run = true;

                while should_run {
                    select! {
                        _ = async {
                            let _ = http_diff.start().await;

                            should_run = false;
                        }    => {}
                    action = async {
                            loop {
                                let action = match http_diff_actions_receiver.recv().await {
                                    Ok(action) => action,
                                    Err(_) => continue,
                                };

                                match action {
                                    AppAction::StartAllJobs => {
                                        return None
                                    }
                                    AppAction::ReloadConfigurationFile(_) => {
                                        return Some(action);
                                    }
                                    _ => {
                                    }
                                }
                            }
                        } => {
                            match action {
                                Some(AppAction::ReloadConfigurationFile(path)) => {
                                    // terminates requests execution if config changed
                                    should_run = false;

                                    // pushes same action again, as previous was already consumed
                                    let _ = http_diff.app_actions_sender
                                        .send(AppAction::ReloadConfigurationFile(path));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            AppAction::StartOneJob(name) => {
                http_diff.start_by_name(&name).await;
            }
            AppAction::TryLoadConfigurationFile(path_to_configuration) => {
                http_diff.load_configuration_file(&path_to_configuration)?
            }
            AppAction::ReloadConfigurationFile(path_to_configuration) => {
                http_diff.reload_configuration_file(&path_to_configuration)?
            }
            _ => {}
        }
    }
}

pub async fn get_configuration_file_watcher<P: AsRef<Path>>(
    path: P,
    app_actions_sender: Sender<AppAction>,
) -> anyhow::Result<PollWatcher> {
    let path_to_file = path
        .as_ref()
        .to_str()
        .ok_or(AppError::ValidationError(
            "Could not validate configuration file path".into(),
        ))?
        .to_string();

    let mut watcher = PollWatcher::new(
        move |res: Result<Event, notify::Error>| match res {
            Ok(event) => match event.kind {
                EventKind::Modify(ModifyKind::Metadata(
                    MetadataKind::WriteTime,
                )) => {
                    debug!(
                        "configuration file was saved : {:?}",
                        path_to_file
                    );

                    let _ = app_actions_sender.send(
                        AppAction::ReloadConfigurationFile(
                            path_to_file.clone(),
                        ),
                    );
                }
                event_kind => {
                    debug!("configuration file event: {:?}", event_kind)
                }
            },

            Err(e) => error!(
                "Failed to consume update configuration file event: {:?}",
                e
            ),
        },
        Config::default().with_poll_interval(Duration::from_millis(200)),
    )?;

    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
