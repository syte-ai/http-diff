use super::super::request::{Request, RequestBuilderDTO, ResponseVariant};
use super::super::types::{AppError, JobStatus};
use super::super::utils::clean_special_chars_for_filename;
use crate::actions::AppAction;
use anyhow::{bail, Result};
use futures::future::join_all;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, Semaphore};
use tokio::task;
use tracing::{debug, error, info};

#[derive(Clone, Debug, PartialEq)]
pub struct JobDTO {
    pub requests: Vec<Request>,
    pub status: JobStatus,
    pub job_duration: Option<Duration>,
    pub job_name: String,
}

impl JobDTO {
    pub fn is_failed(&self) -> bool {
        return self.status == JobStatus::Failed;
    }

    pub fn get_requests_with_diffs(&self) -> Vec<Request> {
        self.requests
            .iter()
            .filter(|request| {
                !request.diffs.is_empty()
                    && request.status == JobStatus::Failed
            })
            .map(|request| request.clone())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Job {
    pub requests_semaphore: Arc<Semaphore>,
    pub threads_semaphore: Arc<Semaphore>,
    pub requests: Vec<Request>,
    pub status: JobStatus,
    pub job_duration: Option<Duration>,
    pub job_name: String,
    pub app_actions_sender: broadcast::Sender<AppAction>,
    pub response_processor: Option<Vec<String>>,
    pub request_builder: Option<Vec<String>>,
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.requests == other.requests
            && self.status == other.status
            && self.job_duration == other.job_duration
            && self.job_name == other.job_name
    }
}

impl Job {
    pub fn new(
        requests: Vec<Request>,
        job_name: &str,
        app_actions_sender: broadcast::Sender<AppAction>,
        requests_semaphore: Arc<Semaphore>,
        threads_semaphore: Arc<Semaphore>,
        response_processor: &Option<Vec<String>>,
        request_builder: &Option<Vec<String>>,
    ) -> Self {
        Job {
            requests,
            status: JobStatus::Pending,
            job_duration: None,
            job_name: job_name.to_string(),
            app_actions_sender,
            requests_semaphore,
            threads_semaphore,
            response_processor: response_processor.clone(),
            request_builder: request_builder.clone(),
        }
    }

    pub fn publish_self(&self) {
        let _ = self
            .app_actions_sender
            .send(AppAction::JobsUpdated(vec![self.clone().into()]));
    }

    pub fn reset(&mut self) {
        self.status = JobStatus::Pending;
        self.job_duration = None;
        for job in self.requests.iter_mut() {
            job.reset();
        }
    }

    pub fn is_failed(&self) -> bool {
        self.status == JobStatus::Failed
    }

    pub async fn start(&mut self) -> Result<()> {
        self.reset();

        self.publish_self();

        let a_permit = self.requests_semaphore.acquire().await?;

        self.status = JobStatus::Running;
        self.publish_self();

        let handles = self.requests.iter_mut().map(|request| {
            let mut request = request.clone();

            tokio::spawn(async move {
                request.start().await;

                request
            })
        });

        let results = join_all(handles).await;

        for handle_result in results.iter() {
            match handle_result {
                Ok(updated_job) => {
                    for job in self.requests.iter_mut() {
                        if job.uri == updated_job.uri {
                            *job = updated_job.clone()
                        }
                    }
                }
                Err(e) => {
                    return Err(AppError::Exception(format!(
                        "Exception during request execution: {}",
                        e
                    ))
                    .into());
                }
            }
        }

        drop(a_permit);

        self.job_duration = self
            .requests
            .iter()
            .map(|job| job.job_duration)
            .filter_map(|duration_option| duration_option)
            .max();

        if let Some(duration) = self.job_duration {
            info!(
                "Finished endpoint job: {} in {:.2} sec",
                &self.job_name,
                duration.as_secs_f64()
            );
        }

        self.publish_self();

        self.calculate_job_diffs().await?;

        let some_failed =
            self.requests.iter().any(|job| job.status == JobStatus::Failed);

        if some_failed {
            self.status = JobStatus::Failed;
        } else {
            self.status = JobStatus::Finished;
        };

        self.publish_self();

        Ok(())
    }

    pub async fn apply_request_builder_to_request(
        request_builder_command: &Vec<String>,
        request: &Request,
    ) -> Result<Option<RequestBuilderDTO>> {
        debug!("request_builder: {:?}", request_builder_command);

        let job_name = request.uri.to_string();

        let request_dto: RequestBuilderDTO = request.into();

        let request_serialized =
            match serde_json::to_string_pretty(&request_dto) {
                Ok(res) => res,
                Err(error) => {
                    error!("request_serialization failed: {error}");

                    bail!(AppError::ValidationError(format!(
                        "Failed to serialize {} request",
                        job_name
                    )));
                }
            };

        let request_serialized_after_builder_process =
            match Job::execute_external_process(
                request_builder_command,
                Some(request_serialized.as_str()),
            )
            .await
            {
                Ok(output) => output,
                Err(error) => {
                    error!("request builder process failed: {error}");

                    bail!(AppError::ValidationError(format!(
                        "request builder process failed for job {}",
                        job_name
                    )));
                }
            };

        let request_deserialized_after_builder_process =
            match serde_json::from_str::<RequestBuilderDTO>(
                &request_serialized_after_builder_process,
            ) {
                Ok(dto) => dto,
                Err(error) => {
                    error!("Failed to deserialize request {job_name} after applying builder command: {error}");

                    bail!(AppError::ValidationError(format!(
                        "Failed to deserialize request {} after applying builder command",
                        job_name
                    )));
                }
            };

        return Ok(Some(request_deserialized_after_builder_process));
    }

    pub async fn execute_external_process(
        raw_command: &Vec<String>,
        input: Option<&str>,
    ) -> Result<String> {
        let command = raw_command.first().cloned().unwrap_or("echo".into());

        let (_, arguments) = raw_command.split_at(1);

        let mut child = Command::new(command)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(input) = input {
            let mut stdin = child.stdin.take().ok_or_else(|| {
                AppError::ValidationError(
                    "Failed to take stdin in external process".into(),
                )
            })?;

            let input = input.to_owned();

            tokio::spawn(async move {
                stdin.write_all(input.as_bytes()).await.unwrap();

                drop(stdin);
            });
        }

        let child_stdout = child.stdout.take().ok_or_else(|| {
            AppError::ValidationError(
                "Failed to take stdout in external process".into(),
            )
        })?;

        let mut child_stderr = child.stderr.take().ok_or_else(|| {
            AppError::ValidationError(
                "Failed to take stderr in external process".into(),
            )
        })?;

        let mut reader = BufReader::new(child_stdout).lines();

        let command_handle = tokio::spawn(async move {
            let status = child.wait().await.or_else(|_| {
                Err(AppError::Exception(
                    "Failed to await external process".into(),
                ))
            });

            status
        });

        let mut capture = String::new();

        while let Some(line) = reader.next_line().await? {
            capture.push_str(&line);
            capture.push_str("\n");
        }

        let exit_status = command_handle.await??;

        match exit_status.success() {
            false => {
                let mut output_string = String::new();

                child_stderr.read_to_string(&mut output_string).await?;

                return Err(AppError::ValidationError(format!(
                    "External command failed:\n{}",
                    output_string
                ))
                .into());
            }
            _ => {}
        }

        Ok(capture)
    }

    pub async fn calculate_job_diffs(&mut self) -> Result<()> {
        let first_request = match self.requests.first_mut() {
            Some(request) => request,
            None => {
                return Err(AppError::ValidationError(
                    "missing first job".into(),
                )
                .into())
            }
        };

        let first_response = match &first_request.response {
            Some(res) => res,
            None => {
                return Err(AppError::ValidationError(
                    "missing first job response".into(),
                )
                .into())
            }
        };

        let old = Job::apply_response_processor(
            &self.response_processor,
            &first_response,
        )
        .await?;

        let first_response_lines = old.lines();

        let (lines_count, _) = first_response_lines.size_hint();

        let mut first_request_diffs: Vec<(ChangeTag, String)> =
            Vec::with_capacity(lines_count);

        for line in first_response_lines {
            first_request_diffs.push((ChangeTag::Equal, line.to_string()));
        }

        first_request.set_diffs_and_calculate_status(first_request_diffs);

        for request in self.requests.iter_mut().skip(1) {
            let old = old.clone();

            let second_response = match &request.response {
                Some(res) => res,
                None => {
                    return Err(AppError::ValidationError(format!(
                        "missing response for job: {}",
                        request.uri.to_string()
                    ))
                    .into())
                }
            };

            let new = Job::apply_response_processor(
                &self.response_processor,
                &second_response,
            )
            .await?;

            let permit = self.threads_semaphore.acquire().await?;

            let diffs = task::spawn_blocking(move || {
                let diff = TextDiff::from_lines(&old, &new);

                diff.iter_all_changes()
                    .map(|change| {
                        (
                            change.tag(),
                            change.value_ref().to_owned().to_owned(),
                        )
                    })
                    .collect()
            })
            .await?;

            drop(permit);

            request.set_diffs_and_calculate_status(diffs);
        }

        if self.requests.iter().any(|job| job.status == JobStatus::Failed) {
            self.status = JobStatus::Failed
        } else {
            self.status = JobStatus::Finished
        }

        Ok(())
    }

    pub async fn apply_response_processor(
        response_processor: &Option<Vec<String>>,
        response: &ResponseVariant,
    ) -> Result<String> {
        let stringified_response = match serde_json::to_string_pretty(response)
        {
            Ok(res) => res,
            Err(error) => {
                return Err(AppError::ValidationError(format!(
                    "Failed to stringify the response, error: {}",
                    error
                ))
                .into());
            }
        };

        match (&response_processor, response) {
            (Some(command), ResponseVariant::Success(_)) => {
                return Job::execute_external_process(
                    command,
                    Some(&stringified_response),
                )
                .await
            }
            _ => {}
        };

        Ok(stringified_response)
    }
}

impl From<Job> for JobDTO {
    fn from(job: Job) -> JobDTO {
        JobDTO {
            requests: job.requests,
            status: job.status,
            job_duration: job.job_duration,
            job_name: job.job_name,
        }
    }
}

impl JobDTO {
    pub async fn save(&self, base_directory: &PathBuf) -> Result<()> {
        let base_path = base_directory
            .join(clean_special_chars_for_filename(&self.job_name));

        if !base_path.exists() {
            create_dir_all(&base_path).await?;
        }

        for job in &self.requests {
            let file_name = format!(
                "{}.json",
                clean_special_chars_for_filename(job.uri.as_str())
            );
            let job_file_path = base_path.join(file_name);

            let mut file = File::create(&job_file_path).await?;

            let content = serde_json::to_string_pretty(&job.response)?;

            file.write_all(content.as_bytes()).await?;

            debug!("response saved to: {:?}", job_file_path.to_str());
        }

        Ok(())
    }
}
