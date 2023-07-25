use crate::model::core::{ApiCore, ApiSetCore, EndpointCore, LatencyCore, RuleCore};
use crate::model::yaml::{ApiShapeYaml, ApiYaml, LatencyYaml, RuleYaml};
use axum::http::uri::PathAndQuery;
use axum::http::{Method, StatusCode};
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

fn extract_endpoint(s: String) -> Result<EndpointCore, String> {
    let regex_method_path: Regex = Regex::new(r"^(?<method>[A-Z]+)\s+(?<path>.+)$").unwrap();

    let (_, [method, path]) = regex_method_path
        .captures(&*s)
        .expect(&*format!(
            "Malformed match, should be like ’METHOD /path/to/resource’ but was {}",
            s
        ))
        .extract();

    match Method::from_str(method) {
        Ok(m) => match PathAndQuery::from_str(path) {
            Ok(p) => Ok(EndpointCore {
                route: p,
                method: m,
            }),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

fn extract_rule(
    rule: &RuleYaml,
    api_latency: Option<LatencyYaml>,
    api_headers: HashMap<String, String>,
) -> Result<RuleCore, String> {
    let endpoint = extract_endpoint(rule.matches.to_owned()).unwrap();

    let regex_status: Regex = Regex::new(r"^(?<status>[0-9]+)/(?<format>[a-z]+)$").unwrap();

    let (_, [status, format]) = regex_status
        .captures(&*rule.status)
        .expect(&*format!(
            "Malformed status, should be like ’200/json’ but was {}",
            rule.status
        ))
        .extract();

    let real_status = StatusCode::from_str(status).unwrap();
    let body: Option<String> =
        rule.body
            .to_owned()
            .and_then(|b| if b.is_empty() { None } else { Some(b) });

    Ok(RuleCore {
        endpoint,
        headers: api_headers.clone(),
        latency: rule
            .latency
            .clone()
            .or(api_latency.clone())
            .map(|latency| match latency {
                LatencyYaml::Constant(value) => LatencyCore::Constant(value),
            }),
        status: real_status,
        format: format.to_string(),
        body,
    })
}

fn extract_api(api: &ApiYaml) -> Result<ApiCore, String> {
    let extracted_rules: Result<Vec<RuleCore>, String> = api
        .rules
        .iter()
        .map(|r| extract_rule(&r, api.latency.clone(), api.headers.clone()))
        .collect();

    extracted_rules.map(|rules| ApiCore(rules))
}

fn extract_api_shape(api_shape: &ApiShapeYaml) -> Result<Vec<EndpointCore>, String> {
    let extracted_endpoints: Result<Vec<EndpointCore>, String> = api_shape
        .shape
        .clone()
        .into_iter()
        .map(extract_endpoint)
        .collect();

    extracted_endpoints
}

fn build(name: String, shape: Option<ApiShapeYaml>, apis: Vec<ApiYaml>) -> ApiSetCore {
    let apis: Result<Vec<ApiCore>, String> =
        apis.into_iter().map(|api| extract_api(&api)).collect();
    // TODO: Validate with shape

    ApiSetCore {
        name: name.to_owned(),
        shape: shape.and_then(|shape_yaml| extract_api_shape(&shape_yaml).ok()),
        apis: apis.unwrap(),
    }
}

pub fn build_all(shapes: Vec<ApiShapeYaml>, apis: Vec<ApiYaml>) -> Vec<ApiSetCore> {
    apis.into_iter()
        .into_group_map_by(|x| x.name.to_owned())
        .into_iter()
        .map(|(key, values)| {
            let may_be_shape = shapes.clone().into_iter().find(|s| s.name.eq(&*key));
            build(key.to_string(), may_be_shape, values)
        })
        .collect::<Vec<ApiSetCore>>()
}
