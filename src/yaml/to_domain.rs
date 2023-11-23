use crate::core::{ApiCore, ApiSetCore, EndpointCore, LatencyCore, RuleCore};
use crate::yaml::{ApiShapeYaml, ApiYaml, LatencyYaml, Response, ResponseDataYaml, RuleYaml};
use anyhow::{Context, Result};
use axum::http::uri::PathAndQuery;
use axum::http::{Method, StatusCode};
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

// Parse endpoints like this "POST /route/to/my/endpoint"
fn extract_endpoint(s: &String) -> Result<EndpointCore> {
    let regex_method_path: Regex = Regex::new(r"^(?<method>[A-Z]+)\s+(?<path>.+)$")?;

    let captured_result = regex_method_path.captures(s).context(format!(
        "Could not parse endpoint '{}' (should be like 'METHOD /path/to/resource')",
        s
    ))?;

    let (_, [method_raw, path_raw]) = captured_result.extract();

    Ok(EndpointCore {
        route: PathAndQuery::from_str(path_raw)?,
        method: Method::from_str(method_raw)?,
    })
}

fn extract_rule(
    rule: &RuleYaml,
    api_latency: Option<LatencyYaml>,
    api_headers: HashMap<String, String>,
    data: HashMap<String, ResponseDataYaml>,
) -> Result<RuleCore> {
    let endpoint = extract_endpoint(&rule.matches)?;

    let (real_status, opt_body, opt_format) = match rule.response.clone() {
        Response::File(path) => {
            let file = data
                .get(&path)
                .context(format!("Getting file content of '{}'", path))?;
            (
                StatusCode::from_u16(file.status)
                    .context(format!("Parsing file status '{}'", file.status))?,
                file.data
                    .to_owned()
                    .and_then(|b| if b.is_empty() { None } else { Some(b) }),
                file.format.to_owned(),
            )
        }
        Response::Inline(status, body, format) => (
            StatusCode::from_u16(status).context(format!("Parsing file status '{}'", status))?,
            body.and_then(|b| if b.is_empty() { None } else { Some(b) }),
            format,
        ),
    };

    Ok(RuleCore {
        endpoint,
        headers: api_headers,
        latency: rule
            .latency
            .clone()
            .or(api_latency)
            .map(|latency| match latency {
                LatencyYaml::Constant(value) => LatencyCore::Constant(value),
            }),
        status: real_status,
        format: opt_format.unwrap_or(String::from("text/plain")),
        body: opt_body,
    })
}

fn extract_api(api: &ApiYaml, data: &HashMap<String, ResponseDataYaml>) -> Result<ApiCore> {
    let extracted_rules: Result<Vec<RuleCore>> = api
        .rules
        .iter()
        .map(|r| extract_rule(r, api.latency.clone(), api.headers.clone(), data.clone()))
        .collect();

    extracted_rules.map(ApiCore)
}

fn extract_api_shape(api_shape: &ApiShapeYaml) -> Result<Vec<EndpointCore>> {
    let extracted_endpoints: Result<Vec<EndpointCore>> =
        api_shape.shape.iter().map(extract_endpoint).collect();

    extracted_endpoints
}

fn build(
    name: String,
    shape: Option<ApiShapeYaml>,
    apis: Vec<ApiYaml>,
    data: &HashMap<String, ResponseDataYaml>,
) -> Result<ApiSetCore> {
    let apis: Result<Vec<ApiCore>> = apis.iter().map(|api| extract_api(api, data)).collect();
    // TODO: Validate with shape

    Ok(ApiSetCore {
        name,
        shape: shape.and_then(|shape_yaml| extract_api_shape(&shape_yaml).ok()),
        apis: apis.context("Extracting apis".to_string())?,
    })
}

pub fn build_all(
    shapes: Vec<ApiShapeYaml>,
    apis: Vec<ApiYaml>,
    data: HashMap<String, ResponseDataYaml>,
) -> Result<Vec<ApiSetCore>> {
    apis.into_iter()
        .into_group_map_by(|x| x.name.to_owned())
        .into_iter()
        .map(|(key, values)| {
            let may_be_shape = shapes.clone().into_iter().find(|s| s.name.eq(&*key));
            build(key, may_be_shape, values, &data)
        })
        .collect()
}
