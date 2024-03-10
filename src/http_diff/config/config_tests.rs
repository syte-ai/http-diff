use super::config::{
    DomainVariant, EndpointConfiguration, UrlWithOptionalHeaders,
};

use crate::http_diff::types::{HeaderValue, HttpMethod};

#[test]
pub fn test_parses_config_string() {
    use std::collections::HashMap;

    use url::Url;

    use crate::http_diff::types::{
        PathVariable, PathVariableValue, VariableGenerator, VariablesMap,
    };

    use super::config::Configuration;

    let config_json_string = r#"
    {
      "domains": [
        "http://domain-a.com",
        {
          "domain": "http://domain-b.com/",
          "headers": {
            "cookie": "auth=test"
          }
        }
      ],
      "endpoints": [
        {
          "endpoint": "/health",
          "request_builder": [
            "python3",
            "script.py"
          ],
          "headers": {
            "X-test": "custom-endpoint-header"
          }
        },
        {
          "endpoint": "/api/v1/users/<userId>",
          "http_method": "POST"
        },
        {
          "endpoint": "/api/v1/products?omit_id=<productId>&include_empty=<include_empty>",
          "response_processor": ["jq", "del(.headers.date)"],
          "variables": {
            "include_empty": [
              "true",
              "false"
            ]
          }
        },
        {
          "endpoint": "/api/v1/carts?status=<status>",
          "request_builder": [
            "python3",
            "script.py"
          ],
          "response_processor": ["jq", "del(.headers.date)"],
          "variables": {
            "status": [
              "pending",
              "empty"
            ]
          }
        }
      ],
      "variables": {
        "userId": [
          123,
          444
        ],
        "productId": "UUID"
      }
    }    
    "#;

    let actual: Configuration =
        serde_json::from_str(config_json_string).unwrap();

    let mut expected_global_variables: VariablesMap = HashMap::new();

    expected_global_variables.insert(
        "userId".to_string(),
        PathVariable::MultipleValues(vec![
            PathVariableValue::Int(123),
            PathVariableValue::Int(444),
        ]),
    );

    expected_global_variables.insert(
        "productId".to_string(),
        PathVariable::SingleValue(PathVariableValue::Generator(
            VariableGenerator::UUID,
        )),
    );

    let mut expected_product_variables: VariablesMap = HashMap::new();

    expected_product_variables.insert(
        "include_empty".to_string(),
        PathVariable::MultipleValues(vec![
            PathVariableValue::String("true".to_string()),
            PathVariableValue::String("false".to_string()),
        ]),
    );

    let mut expected_cart_variables: VariablesMap = HashMap::new();

    expected_cart_variables.insert(
        "status".to_string(),
        PathVariable::MultipleValues(vec![
            PathVariableValue::String("pending".to_string()),
            PathVariableValue::String("empty".to_string()),
        ]),
    );

    let mut second_domain_headers = HashMap::new();

    second_domain_headers.insert(
        "cookie".to_owned(),
        HeaderValue::String("auth=test".to_owned()),
    );

    let second_domain_config = UrlWithOptionalHeaders {
        domain: Url::parse("http://domain-b.com").unwrap(),
        headers: Some(second_domain_headers),
    };

    let mut health_endpoint_headers = HashMap::new();

    health_endpoint_headers.insert(
        "X-test".to_owned(),
        HeaderValue::String("custom-endpoint-header".to_owned()),
    );

    let expected = Configuration {
        domains: vec![
            DomainVariant::Url(Url::parse("http://domain-a.com").unwrap()),
            DomainVariant::UrlWithHeaders(second_domain_config),
        ],
        endpoints: vec![
            EndpointConfiguration {
                endpoint: "/health".to_string(),
                variables: None,
                http_method: None,
                headers: Some(health_endpoint_headers),
                body: None,
                response_processor: None,
                request_builder: Some(vec!["python3".to_owned(), "script.py".to_owned()]),
            },
            EndpointConfiguration {
                endpoint: "/api/v1/users/<userId>".to_string(),
                variables: None,
                http_method: Some(HttpMethod::POST),
                headers: None,
                body: None,
                response_processor: None,
                request_builder: None,
            },
            EndpointConfiguration {
                endpoint: "/api/v1/products?omit_id=<productId>&include_empty=<include_empty>"
                    .to_string(),
                variables: Some(expected_product_variables),
                http_method: None,
                headers: None,
                body: None,
                response_processor: Some(vec!["jq".to_owned(), "del(.headers.date)".to_owned()]),
                request_builder: None,
            },
            EndpointConfiguration {
                endpoint: "/api/v1/carts?status=<status>".to_string(),
                variables: Some(expected_cart_variables),
                http_method: None,
                headers: None,
                body: None,
                response_processor: Some(vec!["jq".to_owned(), "del(.headers.date)".to_owned()]),
                request_builder: Some(vec!["python3".to_owned(), "script.py".to_owned()]),
            },
        ],
        variables: Some(expected_global_variables),
        concurrent_jobs: 20,
    };

    assert_eq!(actual, expected)
}
