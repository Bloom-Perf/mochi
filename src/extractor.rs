use crate::model::core::{ApiCore, ApiSetCore, EndpointCore, LatencyCore, RuleCore};
use crate::model::yaml::{
    ApiShapeYaml, ApiYaml, LatencyYaml, Response, ResponseDataYaml, RuleYaml,
};
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

    let m = Method::from_str(method).map_err(|e| e.to_string())?;
    let p = PathAndQuery::from_str(path).map_err(|e| e.to_string())?;

    Ok(EndpointCore {
        route: p,
        method: m,
    })
}

fn extract_rule(
    rule: &RuleYaml,
    api_latency: Option<LatencyYaml>,
    api_headers: HashMap<String, String>,
    data: HashMap<String, ResponseDataYaml>,
) -> Result<RuleCore, String> {
    let endpoint = extract_endpoint(rule.matches.to_owned())?;

    let (real_status, opt_body, opt_format) = match rule.response.clone() {
        Response::File(path) => {
            let file = data.get(&path).unwrap();
            (
                StatusCode::from_u16(file.status).ok(),
                file.data
                    .to_owned()
                    .and_then(|b| if b.is_empty() { None } else { Some(b) }),
                file.format.to_owned(),
            )
        }
        Response::Inline(status, body, format) => (
            StatusCode::from_u16(status).ok(),
            body.to_owned()
                .and_then(|b| if b.is_empty() { None } else { Some(b) }),
            format,
        ),
    };

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
        status: real_status.unwrap(),
        format: opt_format.unwrap_or(String::from("text/plain")),
        body: opt_body,
    })
}

fn extract_api(api: &ApiYaml, data: &HashMap<String, ResponseDataYaml>) -> Result<ApiCore, String> {
    let extracted_rules: Result<Vec<RuleCore>, String> = api
        .rules
        .iter()
        .map(|r| extract_rule(&r, api.latency.clone(), api.headers.clone(), data.clone()))
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

fn build(
    name: String,
    shape: Option<ApiShapeYaml>,
    apis: Vec<ApiYaml>,
    data: &HashMap<String, ResponseDataYaml>,
) -> ApiSetCore {
    let apis: Result<Vec<ApiCore>, String> = apis
        .into_iter()
        .map(|api| extract_api(&api, data))
        .collect();
    // TODO: Validate with shape

    ApiSetCore {
        name: name.to_owned(),
        shape: shape.and_then(|shape_yaml| extract_api_shape(&shape_yaml).ok()),
        apis: apis.unwrap(),
    }
}

pub fn build_all(
    shapes: Vec<ApiShapeYaml>,
    apis: Vec<ApiYaml>,
    data: HashMap<String, ResponseDataYaml>,
) -> Vec<ApiSetCore> {
    dbg!(data.keys().clone());
    dbg!(apis.clone());
    apis.into_iter()
        .into_group_map_by(|x| x.name.to_owned())
        .into_iter()
        .map(|(key, values)| {
            let may_be_shape = shapes.clone().into_iter().find(|s| s.name.eq(&*key));
            build(key.to_string(), may_be_shape, values, &data)
        })
        .collect::<Vec<ApiSetCore>>()
}
