use super::super::utils::flatten_variables_map;
use crate::actions::AppAction;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use uuid::Uuid;

use super::super::config::Configuration;
use super::super::config::{DomainVariant, EndpointConfiguration};
use super::super::request::Request;
use super::super::types::{
    AppError, HeadersMap, HttpMethod, PathVariableValue,
    PlaceholderToValueMap, VariableGenerator, VariablesMap,
};
use super::super::utils::{
    get_placeholders_from_string, replace_placeholder_with_value,
};
use super::job::Job;

pub fn map_configuration_to_jobs(
    configuration: &Configuration,
    app_actions_sender: broadcast::Sender<AppAction>,
    requests_semaphore: Arc<Semaphore>,
    threads_semaphore: Arc<Semaphore>,
) -> Result<Vec<Job>, AppError> {
    let mut endpoints: Vec<Job> = Vec::new();

    for endpoint_config in &configuration.endpoints {
        let placeholders =
            get_placeholders_from_string(&endpoint_config.endpoint);

        let mut endpoint_variable_lookup: VariablesMap = HashMap::new();

        if let Some(global_optional_variables) = &configuration.variables {
            endpoint_variable_lookup.extend(global_optional_variables.clone());
        }

        if let Some(endpoint_optional_variables) = &endpoint_config.variables {
            endpoint_variable_lookup
                .extend(endpoint_optional_variables.clone());
        }

        let endpoint_placeholders_with_variables: Vec<&String> = placeholders
            .iter()
            .filter(|&placeholder| {
                endpoint_variable_lookup.contains_key(placeholder)
            })
            .collect();

        endpoint_variable_lookup.retain(|key, _| placeholders.contains(key));

        if endpoint_placeholders_with_variables.is_empty() {
            let new_job = map_job_with_no_variables(
                &configuration.domains,
                endpoint_config,
                app_actions_sender.clone(),
                requests_semaphore.clone(),
                threads_semaphore.clone(),
            )?;

            endpoints.push(new_job);
        } else {
            let variable_map_combinations =
                flatten_variables_map(endpoint_variable_lookup);

            for variables_combination in variable_map_combinations {
                let new_job = map_job_with_variables(
                    &variables_combination,
                    &endpoint_placeholders_with_variables,
                    &configuration.domains,
                    endpoint_config,
                    app_actions_sender.clone(),
                    requests_semaphore.clone(),
                    threads_semaphore.clone(),
                )?;

                endpoints.push(new_job);
            }
        }
    }

    Ok(endpoints)
}

fn map_job_with_no_variables(
    domains: &Vec<DomainVariant>,
    endpoint_config: &EndpointConfiguration,
    app_actions_sender: broadcast::Sender<AppAction>,
    requests_semaphore: Arc<Semaphore>,
    threads_semaphore: Arc<Semaphore>,
) -> Result<Job, AppError> {
    let mut jobs: Vec<Request> = Vec::new();

    for domain_variant in domains {
        let (domain, domain_headers) = match domain_variant {
            DomainVariant::Url(domain) => (domain.clone(), None),
            DomainVariant::UrlWithHeaders(domain_config) => {
                (domain_config.domain.clone(), domain_config.headers.clone())
            }
        };

        let uri = domain.join(&endpoint_config.endpoint).map_err(|_| {
            let error_message = format!(
                "{} with {}",
                domain.to_string(),
                &endpoint_config.endpoint
            );
            AppError::FailedToParseConfig(error_message)
        })?;

        let headers_mapped = build_endpoint_headers(
            domain_headers,
            endpoint_config.headers.clone(),
        );

        let http_method = endpoint_config
            .http_method
            .clone()
            .unwrap_or_else(|| HttpMethod::GET);

        let new_job = Request::new(
            &uri,
            &http_method,
            headers_mapped,
            endpoint_config.body.clone(),
        );

        jobs.push(new_job);
    }

    Ok(Job::new(
        jobs,
        &endpoint_config.endpoint,
        app_actions_sender,
        requests_semaphore,
        threads_semaphore,
        &endpoint_config.response_processor,
        &endpoint_config.request_builder,
    ))
}

fn map_job_with_variables(
    variables_combination: &PlaceholderToValueMap,
    endpoint_placeholders_with_variables: &Vec<&String>,
    domains: &Vec<DomainVariant>,
    endpoint_config: &EndpointConfiguration,
    app_actions_sender: broadcast::Sender<AppAction>,
    requests_semaphore: Arc<Semaphore>,
    threads_semaphore: Arc<Semaphore>,
) -> Result<Job, AppError> {
    let mut jobs: Vec<Request> = Vec::new();

    let mut formatted_string = endpoint_config.endpoint.clone();

    for placeholder in endpoint_placeholders_with_variables {
        let value = variables_combination.get(*placeholder);
        if value.is_none() {
            continue;
        }

        let value_for_replacement = match value.unwrap() {
            PathVariableValue::Generator(generator_type) => {
                match generator_type {
                    VariableGenerator::UUID => Uuid::new_v4().to_string(),
                }
            }
            PathVariableValue::String(string_value) => string_value.clone(),
            PathVariableValue::Int(int_value) => int_value.to_string(),
        };

        formatted_string = replace_placeholder_with_value(
            &formatted_string,
            placeholder,
            &value_for_replacement,
        );
    }
    for domain_variant in domains {
        let (domain, domain_headers) = match domain_variant {
            DomainVariant::Url(domain) => (domain.clone(), None),
            DomainVariant::UrlWithHeaders(domain_config) => {
                (domain_config.domain.clone(), domain_config.headers.clone())
            }
        };

        let uri = domain.join(&formatted_string).map_err(|_| {
            let error_message =
                format!("{} with {}", domain.to_string(), &formatted_string);
            AppError::FailedToParseConfig(error_message)
        })?;

        let headers_mapped = build_endpoint_headers(
            domain_headers,
            endpoint_config.headers.clone(),
        );

        let http_method = endpoint_config
            .http_method
            .clone()
            .unwrap_or_else(|| HttpMethod::GET);

        let new_job = Request::new(
            &uri,
            &http_method,
            headers_mapped,
            endpoint_config.body.clone(),
        );

        jobs.push(new_job);
    }
    Ok(Job::new(
        jobs,
        &formatted_string,
        app_actions_sender.clone(),
        requests_semaphore.clone(),
        threads_semaphore.clone(),
        &endpoint_config.response_processor,
        &endpoint_config.request_builder,
    ))
}

fn build_endpoint_headers(
    domain: Option<HeadersMap>,
    endpoint: Option<HeadersMap>,
) -> Option<HeadersMap> {
    let headers_mapped = match (domain, endpoint) {
        (Some(mut domain_headers), Some(endpoint_headers)) => {
            domain_headers.extend(endpoint_headers);

            Some(domain_headers)
        }
        (None, Some(endpoint_headers)) => Some(endpoint_headers),
        (Some(domain_headers), None) => Some(domain_headers),
        (None, None) => None,
    };

    headers_mapped
}
