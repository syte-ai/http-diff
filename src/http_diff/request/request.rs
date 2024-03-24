use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use similar::ChangeTag;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
use tracing::error;
use url::Url;

use super::super::types::{HeadersMap, HttpMethod, JobStatus};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Response {
    pub status_code: u16,
    pub content_length: Option<u64>,
    #[serde(serialize_with = "ordered_headers")]
    pub headers: HeadersMap,
    pub body: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ResponseVariant {
    Success(Response),
    Fail(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub uri: Url,
    pub status: JobStatus,
    pub http_method: HttpMethod,
    pub headers: Option<HeadersMap>,
    pub body: Option<Value>,
    pub job_duration: Option<Duration>,
    pub response: Option<ResponseVariant>,
    pub diffs: Vec<(ChangeTag, String)>,
    pub has_diffs: bool,
}

impl Request {
    pub fn new(
        uri: &Url,
        http_method: &HttpMethod,
        headers: Option<HeadersMap>,
        body: Option<Value>,
    ) -> Self {
        Request {
            uri: uri.clone(),
            http_method: http_method.clone(),
            status: JobStatus::Pending,
            job_duration: None,
            response: None,
            diffs: Vec::new(),
            headers,
            body,
            has_diffs: false,
        }
    }

    pub fn reset(&mut self) {
        self.status = JobStatus::Pending;
        self.job_duration = None;
        self.response = None;
        self.diffs = Vec::new();
    }

    pub async fn start(&mut self) {
        self.status = JobStatus::Running;
        let client = reqwest::Client::new();
        let url = self.uri.as_str();

        let mut request_builder = match self.http_method {
            HttpMethod::DELETE => client.delete(url),
            HttpMethod::POST => client.post(url),
            HttpMethod::PUT => client.put(url),
            HttpMethod::PATCH => client.patch(url),
            HttpMethod::GET => client.get(url),
        };

        if let Some(body) = &self.body {
            match serde_json::to_string(body) {
                Ok(json_string) => {
                    request_builder = request_builder.body(json_string)
                }
                Err(error) => error!("error converting body {error}"),
            };
        }

        match &self.headers {
            Some(headers_map) => {
                for (key, value) in headers_map {
                    request_builder = request_builder.header(key, value);
                }
            }
            None => {}
        };

        let started_at = Instant::now();
        let result = request_builder.send().await;

        self.job_duration = Some(started_at.elapsed());

        match result {
            Ok(response) => {
                self.response = Some(ResponseVariant::Success(Response {
                    status_code: response.status().as_u16().to_owned(),
                    content_length: response.content_length(),
                    headers: reqwest_headers_to_hashmap(response.headers()),
                    body: response.json().await.ok(),
                }));
            }
            Err(err) => {
                error!("Request failed for url {}:{}", self.uri.as_str(), err);
                self.response = Some(ResponseVariant::Fail(err.to_string()));
            }
        }
    }

    pub fn apply_request_builder_dto(&mut self, dto: RequestBuilderDTO) {
        self.uri = dto.uri;
        self.body = dto.body;

        self.headers = dto.headers;
        self.http_method = dto.http_method;
    }

    pub fn set_diffs_and_calculate_status(
        &mut self,
        diffs: Vec<(ChangeTag, String)>,
    ) {
        let has_diffs = diffs.iter().any(|(tag, _)| tag != &ChangeTag::Equal);

        self.has_diffs = has_diffs;

        self.diffs = diffs;

        if has_diffs {
            self.status = JobStatus::Failed;
        } else {
            let request_failed = match &self.response {
                Some(ResponseVariant::Fail(_)) => true,
                _ => false,
            };

            self.status = if request_failed {
                JobStatus::Failed
            } else {
                JobStatus::Finished
            };
        }
    }

    pub fn get_status_text(&self) -> String {
        let is_success = match &self.status {
            JobStatus::Finished => " SUCCESS",
            JobStatus::Failed => " FAIL",
            _ => " PENDING",
        };

        let base_text = match &self.response {
            Some(ResponseVariant::Success(response)) => {
                format!(
                    "{} - {}",
                    is_success,
                    &response.status_code.to_string()
                )
            }
            _ => is_success.to_owned(),
        };

        match self.job_duration {
            Some(duration) => {
                format!("{} - in {:.2} sec", base_text, duration.as_secs_f64())
            }
            None => base_text,
        }
    }
}

fn reqwest_headers_to_hashmap(reqwest_headers: &HeaderMap) -> HeadersMap {
    let mut headers = HeadersMap::default();

    for (key, value) in reqwest_headers.iter() {
        headers.insert(key.to_string(), value.into());
    }

    headers
}

fn ordered_headers<S>(
    value: &HeadersMap,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RequestBuilderDTO {
    pub uri: Url,
    pub http_method: HttpMethod,
    pub headers: Option<HeadersMap>,
    pub body: Option<Value>,
}

impl From<&Request> for RequestBuilderDTO {
    fn from(request: &Request) -> Self {
        RequestBuilderDTO {
            uri: request.uri.clone(),
            http_method: request.http_method.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
        }
    }
}
