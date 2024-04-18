use crate::core::{
    ApiCore, ApiSetCore, ApiSetRootCore, ConfCore, EndpointCore, LatencyCore, ProxyCore, RuleCore,
    SystemCore,
};
use crate::template::render::rule_body_from_str;
use crate::yaml::{
    ApiShapeYaml, ApiYaml, ConfFolder, LatencyYaml, ProxyYaml, Response, ResponseDataYaml,
    RuleYaml, SystemFolder,
};
use anyhow::{bail, Context, Result};
use axum::http::uri::PathAndQuery;
use axum::http::{Method, StatusCode, Uri};
use itertools::Itertools;
use regex::Regex;
use std::collections::{HashMap, LinkedList};
use std::str::FromStr;

// Parse endpoints like this "POST /route/to/my/endpoint"
fn extract_endpoint(s: &String) -> Result<EndpointCore> {
    let regex_method_path: Regex = Regex::new(r"^(?<method>[A-Z]+)\s+(?<path>.+)$")?;

    let captured_result = regex_method_path.captures(s).context(format!(
        "Could not parse endpoint '{s}' (should be like 'METHOD /path/to/resource')"
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
                .context(format!("Getting file content of '{path}'"))?;
            (
                StatusCode::from_u16(file.status)
                    .context(format!("Parsing file status '{}'", file.status))?,
                file.data
                    .clone()
                    .and_then(|b| if b.is_empty() { None } else { Some(b) }),
                file.format.clone(),
            )
        }
        Response::Inline(status, body, format) => (
            StatusCode::from_u16(status).context(format!("Parsing file status '{status}'"))?,
            body.and_then(|b| if b.is_empty() { None } else { Some(b) }),
            format,
        ),
        Response::OkText(body) => (StatusCode::OK, Some(body), Some("text/plain".to_string())),
        Response::OkJson(body) => (
            StatusCode::OK,
            Some(body),
            Some("application/json".to_string()),
        ),
        Response::OkXml(body) => (
            StatusCode::OK,
            Some(body),
            Some("application/xml".to_string()),
        ),
        Response::Ok => (StatusCode::NO_CONTENT, None, None),
    };

    let opt_rule_body = opt_body.map(rule_body_from_str);

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
        body: opt_rule_body,
    })
}

fn extract_api(api: &ApiYaml, data: &HashMap<String, ResponseDataYaml>) -> Result<ApiCore> {
    let extracted_rules: Result<Vec<RuleCore>> = api
        .rules
        .iter()
        .map(|r| {
            extract_rule(
                r,
                api.latency.clone(),
                api.headers.clone().unwrap_or_default(),
                data.clone(),
            )
        })
        .collect();

    Ok(ApiCore(extracted_rules?))
}

fn extract_api_shape(api_shape: &ApiShapeYaml) -> Result<Vec<EndpointCore>> {
    let extracted_endpoints: Result<Vec<EndpointCore>> =
        api_shape.shape.iter().map(extract_endpoint).collect();

    extracted_endpoints
}

pub fn build_api_set(
    name: &String,
    shape: &Option<ApiShapeYaml>,
    apis: &[ApiYaml],
    proxy: &Option<ProxyYaml>,
    data: &HashMap<String, ResponseDataYaml>,
) -> Result<ApiSetCore> {
    let apis_core: Vec<ApiCore> = apis
        .iter()
        .map(|api| extract_api(api, data))
        .collect::<Result<Vec<_>>>()?;

    let proxy_core = match proxy {
        Some(p) => {
            let uri = Uri::try_from(p.url.clone()).context(format!(
                "Parsing url '{}' while building api_set '{name}'",
                p.url
            ))?;
            Some(ProxyCore(uri))
        }
        None => None,
    };

    let shape_core = match shape {
        Some(s) => {
            let shape = extract_api_shape(s).context(format!(
                "Extracting api shape while building api_set '{name}'"
            ))?;
            for (pos, api) in apis_core.iter().enumerate() {
                validate_api_with_shape(name, &shape, api).context(format!(
                    "Validating api '{pos}' against shape while building api_set '{name}'"
                ))?
            }
            Some(shape)
        }
        None => None,
    };

    Ok(ApiSetCore {
        name: name.to_owned(),
        shape: shape_core,
        proxy: proxy_core,
        apis: apis_core,
    })
}

pub fn validate_api_with_shape(name: &String, shape: &[EndpointCore], api: &ApiCore) -> Result<()> {
    if shape.len() != api.0.len() {
        bail!(
            "Api name: {name}\n -> Shape and api don’t have the same number of endpoints: {} != {}",
            shape.len(),
            api.0.len()
        );
    }

    // Checking shape rules are all implemented by the api
    let mut messages = LinkedList::new();
    for el in shape.iter() {
        let shape_rule_implemented_by_api = api.0.iter().any(|rule| rule.endpoint == el.clone());

        if !shape_rule_implemented_by_api {
            messages.push_front(format!("Api does not implement shape rule '{el}'"))
        }
    }

    // Checking all api rules are present in the shape definition
    for el in api.0.iter() {
        let api_rule_present_in_shape = shape
            .iter()
            .any(|shape_endpoint| *shape_endpoint == el.endpoint);

        if !api_rule_present_in_shape {
            messages.push_front(format!(
                "Api contains rule '{}' not present in shape definition",
                el.endpoint
            ))
        }
    }

    if !messages.is_empty() {
        bail!(
            "Api name: {name}\n -> Shape/api contract mismatch:\n - {}",
            messages.iter().join("\n - ")
        );
    }

    Ok(())
}

pub fn build_root_api_set(system: &SystemFolder) -> Result<ApiSetRootCore> {
    let system_name = &system.name;

    let apis_core: Vec<ApiCore> = system
        .apis
        .iter()
        .map(|api| extract_api(api, &system.data))
        .collect::<Result<Vec<_>>>()?;

    let proxy_core = match &system.proxy {
        Some(p) => {
            let uri = Uri::try_from(p.url.clone()).context(format!(
                "Parsing url '{}' while building root api_set of system '{system_name}'",
                p.url
            ))?;
            Some(ProxyCore(uri))
        }
        None => None,
    };

    let shape_core = match &system.shape {
        Some(s) => {
            let shape = extract_api_shape(s).context(format!(
                "Extracting api shape while building root api_set of system '{system_name}'"
            ))?;
            for (pos, api) in apis_core.iter().enumerate() {
                validate_api_with_shape(&system.name, &shape, api).context(format!(
                    "Validating api '{pos}' against shape while building root api_set of system '{system_name}'"
                ))?
            }
            Some(shape)
        }
        None => None,
    };

    Ok(ApiSetRootCore {
        shape: shape_core,
        proxy: proxy_core,
        apis: apis_core,
    })
}

impl ConfFolder {
    pub fn extract(&self) -> Result<ConfCore> {
        let system_cores: Result<Vec<SystemCore>> = self
            .systems
            .iter()
            .map(|system| {
                let root_api_set = build_root_api_set(system)?;

                let api_sets = system
                    .api_folders
                    .iter()
                    .map(|f| {
                        // Merge data folder with the system data folder as a fallback
                        let merged_data_folders = f
                            .data
                            .clone()
                            .into_iter()
                            .chain(system.data.clone())
                            .collect();
                        build_api_set(&f.name, &f.shape, &f.apis, &f.proxy, &merged_data_folders)
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(SystemCore {
                    name: system.name.to_owned(),
                    root_api_set,
                    api_sets,
                })
            })
            .collect();

        let v = system_cores?;
        //dbg!(v.clone());

        Ok(ConfCore { systems: v })
    }
}
