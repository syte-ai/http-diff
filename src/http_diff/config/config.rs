use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string_pretty, Value};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use url::Url;

use crate::http_diff::types::{
    AppError, HeaderValue, HeadersMap, HttpMethod, PathVariable,
    PathVariableValue, VariableGenerator, VariablesMap,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EndpointConfiguration {
    pub endpoint: String,
    pub variables: Option<VariablesMap>,
    pub http_method: Option<HttpMethod>,
    pub headers: Option<HeadersMap>,
    pub body: Option<Value>,
    pub response_processor: Option<Vec<String>>,
    pub request_builder: Option<Vec<String>>,
}

fn default_concurrent_jobs() -> usize {
    20
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UrlWithOptionalHeaders {
    pub domain: Url,
    pub headers: Option<HeadersMap>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum DomainVariant {
    Url(Url),
    UrlWithHeaders(UrlWithOptionalHeaders),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Configuration {
    pub domains: Vec<DomainVariant>,
    pub endpoints: Vec<EndpointConfiguration>,
    pub variables: Option<VariablesMap>,
    #[serde(default = "default_concurrent_jobs")]
    pub concurrent_jobs: usize,
}

impl Configuration {
    pub fn default() -> Configuration {
        let mut second_domain_headers = HashMap::new();

        second_domain_headers.insert(
            "cookie".to_owned(),
            HeaderValue::String("auth=test".to_owned()),
        );

        let mut health_headers = HashMap::new();

        health_headers.insert(
            "x-test".to_owned(),
            HeaderValue::String("true".to_owned()),
        );

        let mut user_variables: VariablesMap = HashMap::new();

        user_variables.insert(
            "userId".to_string(),
            PathVariable::MultipleValues(vec![
                PathVariableValue::Int(123),
                PathVariableValue::String("true".to_string()),
                PathVariableValue::Generator(VariableGenerator::UUID),
            ]),
        );

        user_variables.insert(
            "skip".to_string(),
            PathVariable::SingleValue(PathVariableValue::Generator(
                VariableGenerator::UUID,
            )),
        );

        let create_user_payload = json!({ "username": "test" });

        Self {
            domains: vec![
                DomainVariant::Url(
                    Url::parse("http://localhost:3000").unwrap(),
                ),
                DomainVariant::UrlWithHeaders(UrlWithOptionalHeaders {
                    domain: Url::parse("http://localhost:3001").unwrap(),
                    headers: Some(second_domain_headers),
                }),
            ],
            endpoints: vec![
                EndpointConfiguration {
                    endpoint: "/health".to_string(),
                    variables: None,
                    http_method: None,
                    headers: Some(health_headers),
                    body: None,
                    response_processor: None,
                    request_builder: Some(vec![
                        "python3".to_owned(),
                        "script.py".to_owned(),
                    ]),
                },
                EndpointConfiguration {
                    endpoint: "/api/v1/users/<userId>?skip=<skip>".to_string(),
                    variables: Some(user_variables),
                    http_method: Some(HttpMethod::GET),
                    headers: None,
                    body: None,
                    response_processor: Some(vec![
                        "jq".to_owned(),
                        "del(.headers.auth)".to_owned(),
                    ]),
                    request_builder: None,
                },
                EndpointConfiguration {
                    endpoint: "/api/v1/users".to_string(),
                    variables: None,
                    http_method: Some(HttpMethod::POST),
                    headers: None,
                    body: Some(create_user_payload),
                    response_processor: Some(vec![
                        "jq".to_owned(),
                        "del(.headers.auth, .body.id)".to_owned(),
                    ]),
                    request_builder: None,
                },
            ],
            variables: None,
            concurrent_jobs: default_concurrent_jobs(),
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.domains.len() < 2 {
            return Err(AppError::ValidationError(
                "Minimum 2 domains required".to_string(),
            ));
        }

        if self.endpoints.is_empty() {
            return Err(AppError::ValidationError(
                "No endpoints were specified".to_string(),
            ));
        }

        Ok(())
    }

    pub fn save(&self, file_path: &Path) -> Result<()> {
        let stringified_config = to_string_pretty(self)?;

        let mut file = File::create(&file_path)?;

        file.write_all(stringified_config.as_bytes())?;

        Ok(())
    }
}

pub fn load_config_from_file(
    file_path: &str,
) -> Result<Configuration, AppError> {
    let mut file = File::open(file_path)
        .map_err(|_| AppError::FileNotFound(file_path.to_string()))?;

    let mut buffer = String::new();

    file.read_to_string(&mut buffer)
        .map_err(|error| AppError::FailedToParseConfig(error.to_string()))?;

    let configuration: Configuration = serde_json::from_str(&buffer)
        .map_err(|error| AppError::FailedToParseConfig(error.to_string()))?;

    configuration.validate()?;

    Ok(configuration)
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
