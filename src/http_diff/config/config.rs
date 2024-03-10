use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::File, io::Read};
use url::Url;

use crate::http_diff::types::{
    AppError, HeadersMap, HttpMethod, VariablesMap,
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

pub fn load_config_from_file(file_path: &str) -> Result<Configuration> {
    let mut file = File::open(file_path)
        .map_err(|_| AppError::FileNotFound(file_path.to_string()))?;

    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;
    let configuration: Configuration = serde_json::from_str(&buffer)
        .map_err(|error| AppError::FailedToParseConfig(error.to_string()))?;

    validate_config(&configuration)?;

    Ok(configuration)
}

pub fn validate_config(configuration: &Configuration) -> Result<(), AppError> {
    if configuration.domains.len() < 2 {
        return Err(AppError::ValidationError(
            "Minimum 2 domains required".to_string(),
        ));
    }

    if configuration.endpoints.is_empty() {
        return Err(AppError::ValidationError(
            "No endpoints were specified".to_string(),
        ));
    }

    Ok(())
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
