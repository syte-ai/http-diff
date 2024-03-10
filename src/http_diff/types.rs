use reqwest::header::HeaderValue as ReqwestHeaderValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AppError {
    #[error("Bad file format `{0}`")]
    FailedToParseConfig(String),
    #[error("File does not exist: `{0}`")]
    FileNotFound(String),
    #[error("Validation error: `{0}`")]
    ValidationError(String),
    #[error("Runtime error: `{0}`")]
    Exception(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Failed,
    Finished,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum VariableGenerator {
    UUID,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum PathVariableValue {
    Generator(VariableGenerator),
    Int(usize),
    String(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum PathVariable {
    SingleValue(PathVariableValue),
    MultipleValues(Vec<PathVariableValue>),
}

pub type VariablesMap = HashMap<String, PathVariable>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HeaderValue {
    String(String),
    U64(u64),
}

impl From<&HeaderValue> for ReqwestHeaderValue {
    fn from(value: &HeaderValue) -> Self {
        match value {
            HeaderValue::String(s) => ReqwestHeaderValue::from_str(s).unwrap(),
            HeaderValue::U64(u) => ReqwestHeaderValue::from(u.clone()),
        }
    }
}

impl From<&ReqwestHeaderValue> for HeaderValue {
    fn from(value: &ReqwestHeaderValue) -> Self {
        HeaderValue::String(value.to_str().unwrap().to_owned())
    }
}

impl Into<ReqwestHeaderValue> for HeaderValue {
    fn into(self) -> ReqwestHeaderValue {
        match self {
            HeaderValue::String(s) => {
                ReqwestHeaderValue::from_str(&s).unwrap()
            }
            HeaderValue::U64(u) => ReqwestHeaderValue::from(u),
        }
    }
}

pub type HeadersMap = HashMap<String, HeaderValue>;

pub type PlaceholderToValueMap = HashMap<String, PathVariableValue>;
