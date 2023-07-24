use std::str::FromStr;
use axum::http::{Method, StatusCode};
use axum::http::uri::PathAndQuery;
use regex::Regex;
use itertools::Itertools;
use crate::model::core::{ApiCore, ApiSetCore, EndpointCore, RuleCore};
use crate::model::yaml::{ApiShapeYaml, ApiYaml, RuleYaml};

fn extract_endpoint(s: &str) -> Result<EndpointCore, String> {
    let regex_method_path: Regex = Regex::new(r"^(?<method>[A-Z]+)\s+(?<path>.+)$").unwrap();

    let (_, [method, path]) = regex_method_path
        .captures(s)
        .expect(&*format!("Malformed match, should be like ’METHOD /path/to/resource’ but was {}", s))
        .extract();

    match Method::from_str(method) {
        Ok(m) => match PathAndQuery::from_str(path) {
            Ok(p) => Ok(EndpointCore {
                route: p,
                method: m
            }),
            Err(e) => Err(e.to_string())
        },
        Err(e) => Err(e.to_string())
    }
}

pub trait Extractor<Target> {

    fn extract(&self) -> Result<Target, String>;
}

impl Extractor<RuleCore> for RuleYaml {

    fn extract(&self) -> Result<RuleCore, String> {

        let endpoint = extract_endpoint(&self.matches).unwrap();

        let regex_status: Regex = Regex::new(r"^(?<status>[0-9]+)/(?<format>[a-z]+)$").unwrap();

        let (_, [status, format]) = regex_status
            .captures(&*self.status)
            .expect(&*format!("Malformed status, should be like ’200/json’ but was {}", self.status))
            .extract();

        let real_status = StatusCode::from_str(status).unwrap();
        let body: Option<String> = self.body.to_owned().and_then(|b| {
            if b.is_empty() { None } else {
                Some(b)
            }
        });

        Ok(RuleCore {
            endpoint,
            status: real_status,
            format: format.to_string(),
            body
        })
    }
}

impl Extractor<ApiCore> for ApiYaml {
    fn extract(&self) -> Result<ApiCore, String> {

        let extracted_rules: Result<Vec<RuleCore>, String> = self.rules
            .iter()
            .map(|r| r.extract())
            .collect();

        let headers = self.headers.clone();

        extracted_rules.map(|rules| {
            ApiCore {
                headers,
                rules
            }
        })
    }
}

impl Extractor<Vec<EndpointCore>> for ApiShapeYaml {
    fn extract(&self) -> Result<Vec<EndpointCore>, String> {

        let extracted_endpoints: Result<Vec<EndpointCore>, String> = self.shape
            .clone()
            .into_iter()
            .map(|r| extract_endpoint(&*r))
            .collect();

        extracted_endpoints
    }
}

fn build(name: String, shape: Option<ApiShapeYaml>, apis: Vec<ApiYaml>) -> ApiSetCore {

    let apis: Result<Vec<ApiCore>, String> = apis
        .into_iter()
        .map(|api| api.extract())
        .collect();
    // TODO: Validate with shape

    ApiSetCore {
        name: name.to_owned(),
        shape: shape.and_then(|shape_yaml| shape_yaml.extract().ok()),
        apis: apis.unwrap()
    }
}

pub fn build_all(shapes: Vec<ApiShapeYaml>, apis: Vec<ApiYaml>) -> Vec<ApiSetCore> {
    apis.into_iter()
        .into_group_map_by(|x| x.name.to_owned())
        .into_iter()
        .map(|(key, values)| {
            let may_be_shape = shapes
                .clone()
                .into_iter()
                .find(|s| s.name.eq(&*key));
            build(key.to_string(), may_be_shape, values)
        })
        .collect::<Vec<ApiSetCore>>()
}
