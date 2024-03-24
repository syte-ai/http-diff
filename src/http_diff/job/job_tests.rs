use super::super::job::Job;
use crate::actions::AppAction;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, Semaphore};
use url::Url;

use super::super::config::{
    Configuration, DomainVariant, EndpointConfiguration,
    UrlWithOptionalHeaders,
};
use super::super::request::Request;
use super::super::types::{
    HeaderValue, HeadersMap, HttpMethod, JobStatus, PathVariable,
    PathVariableValue, VariablesMap,
};

use super::job_mapper::map_configuration_to_jobs;

#[test]
pub fn test_convert_config_jobs() {
    let jobs_semaphore = Arc::new(Semaphore::new(40));

    let (app_actions_sender, _) = broadcast::channel::<AppAction>(100);

    let mut expected_global_variables: VariablesMap = HashMap::new();

    expected_global_variables.insert(
        "userId".to_string(),
        PathVariable::MultipleValues(vec![
            PathVariableValue::Int(123),
            PathVariableValue::Int(444),
        ]),
    );

    expected_global_variables.insert(
        "status".to_string(),
        PathVariable::SingleValue(PathVariableValue::String(
            "deleted".to_string(),
        )),
    );

    let mut domain_headers = HashMap::new();

    domain_headers.insert(
        "cookie".to_owned(),
        HeaderValue::String("auth=check".to_owned()),
    );

    let domain_with_headers_config = UrlWithOptionalHeaders {
        domain: Url::parse("http://domain-with-specific-headers.com").unwrap(),
        headers: Some(domain_headers),
    };

    let mut endpoint_headers = HeadersMap::default();

    endpoint_headers.insert(
        "X-test".to_owned(),
        HeaderValue::String("test=true".to_owned()),
    );

    let configuration = Configuration {
        domains: vec![
            DomainVariant::Url(Url::parse("http://domain-a.com").unwrap()),
            DomainVariant::Url(Url::parse("http://domain-b.com").unwrap()),
            DomainVariant::UrlWithHeaders(domain_with_headers_config),
        ],
        endpoints: vec![
            EndpointConfiguration {
                endpoint: "/health".to_string(),
                variables: None,
                http_method: None,
                headers: None,
                body: None,
                response_processor: None,
                request_builder: None,
            },
            EndpointConfiguration {
                endpoint: "/api/v1/users/<userId>?status=<status>".to_string(),
                variables: None,
                http_method: Some(HttpMethod::POST),
                headers: None,
                body: None,
                response_processor: None,
                request_builder: None,
            },
            EndpointConfiguration {
                endpoint: "/api/v1/accounts/<accountId>?admin=<admin_flag>"
                    .to_string(),
                variables: Some(HashMap::from([
                    (
                        "admin_flag".to_owned(),
                        PathVariable::MultipleValues(vec![
                            PathVariableValue::String("true".to_string()),
                            PathVariableValue::String("false".to_string()),
                        ]),
                    ),
                    (
                        "accountId".to_owned(),
                        PathVariable::SingleValue(PathVariableValue::Int(123)),
                    ),
                ])),
                http_method: Some(HttpMethod::GET),
                headers: Some(endpoint_headers),
                body: None,
                response_processor: None,
                request_builder: None,
            },
        ],
        variables: Some(expected_global_variables),
        concurrent_jobs: 20,
    };

    let actual_jobs = map_configuration_to_jobs(
        &configuration,
        app_actions_sender.clone(),
        jobs_semaphore.clone(),
    )
    .unwrap();

    let expected_jobs = vec![
        Job {
            semaphore: jobs_semaphore.clone(),
            requests: vec![
                Request {
                    uri: Url::parse("http://domain-a.com/health").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-b.com/health").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-with-specific-headers.com/health").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "cookie".to_owned(),
                        HeaderValue::String("auth=check".to_owned()),
                    )])),
                    body: None,
                },
            ],
            status: JobStatus::Pending,
            job_duration: None,
            job_name: "/health".to_string(),
            app_actions_sender: app_actions_sender.clone(),
            response_processor: None,
            request_builder: None,
        },
        Job {
            semaphore: jobs_semaphore.clone(),
            requests: vec![
                Request {
                    uri: Url::parse("http://domain-a.com/api/v1/users/123?status=deleted").unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-b.com/api/v1/users/123?status=deleted").unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse(
                        "http://domain-with-specific-headers.com/api/v1/users/123?status=deleted",
                    )
                    .unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "cookie".to_owned(),
                        HeaderValue::String("auth=check".to_owned()),
                    )])),
                    body: None,
                },
            ],
            status: JobStatus::Pending,
            job_duration: None,
            job_name: "/api/v1/users/123?status=deleted".to_string(),
            app_actions_sender: app_actions_sender.clone(),
            response_processor: None,
            request_builder: None,
        },
        Job {
            semaphore: jobs_semaphore.clone(),
            requests: vec![
                Request {
                    uri: Url::parse("http://domain-a.com/api/v1/users/444?status=deleted").unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-b.com/api/v1/users/444?status=deleted").unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: None,
                    body: None,
                },
                Request {
                    uri: Url::parse(
                        "http://domain-with-specific-headers.com/api/v1/users/444?status=deleted",
                    )
                    .unwrap(),
                    http_method: HttpMethod::POST,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "cookie".to_owned(),
                        HeaderValue::String("auth=check".to_owned()),
                    )])),
                    body: None,
                },
            ],
            status: JobStatus::Pending,
            job_duration: None,
            job_name: "/api/v1/users/444?status=deleted".to_string(),
            app_actions_sender: app_actions_sender.clone(),
            response_processor: None,
            request_builder: None,
        },
        Job {
            semaphore: jobs_semaphore.clone(),
            requests: vec![
                Request {
                    uri: Url::parse("http://domain-a.com/api/v1/accounts/123?admin=true").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "X-test".to_owned(),
                        HeaderValue::String("test=true".to_owned()),
                    )])),
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-b.com/api/v1/accounts/123?admin=true").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "X-test".to_owned(),
                        HeaderValue::String("test=true".to_owned()),
                    )])),
                    body: None,
                },
                Request {
                    uri: Url::parse(
                        "http://domain-with-specific-headers.com/api/v1/accounts/123?admin=true",
                    )
                    .unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([
                        (
                            "X-test".to_owned(),
                            HeaderValue::String("test=true".to_owned()),
                        ),
                        (
                            "cookie".to_owned(),
                            HeaderValue::String("auth=check".to_owned()),
                        ),
                    ])),
                    body: None,
                },
            ],
            status: JobStatus::Pending,
            job_duration: None,
            job_name: "/api/v1/accounts/123?admin=true".to_string(),
            app_actions_sender: app_actions_sender.clone(),
            response_processor: None,
            request_builder: None,
        },
        Job {
            semaphore: jobs_semaphore.clone(),
            requests: vec![
                Request {
                    uri: Url::parse("http://domain-a.com/api/v1/accounts/123?admin=false").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "X-test".to_owned(),
                        HeaderValue::String("test=true".to_owned()),
                    )])),
                    body: None,
                },
                Request {
                    uri: Url::parse("http://domain-b.com/api/v1/accounts/123?admin=false").unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([(
                        "X-test".to_owned(),
                        HeaderValue::String("test=true".to_owned()),
                    )])),
                    body: None,
                },
                Request {
                    uri: Url::parse(
                        "http://domain-with-specific-headers.com/api/v1/accounts/123?admin=false",
                    )
                    .unwrap(),
                    http_method: HttpMethod::GET,
                    status: JobStatus::Pending,
                    job_duration: None,
                    response: None,
                    diffs: Vec::new(),
                    has_diffs: false,
                    headers: Some(HashMap::from([
                        (
                            "X-test".to_owned(),
                            HeaderValue::String("test=true".to_owned()),
                        ),
                        (
                            "cookie".to_owned(),
                            HeaderValue::String("auth=check".to_owned()),
                        ),
                    ])),
                    body: None,
                },
            ],
            status: JobStatus::Pending,
            job_duration: None,
            job_name: "/api/v1/accounts/123?admin=false".to_string(),
            app_actions_sender,
            response_processor: None,
            request_builder: None,
        },
    ];

    assert_eq!(actual_jobs, expected_jobs)
}
