use std::str::FromStr;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use axum::http::uri::PathAndQuery;
use axum::Json;
use regex::Regex;
use crate::model::core::{ApiCore, RuleCore};
use crate::model::yaml::{ApiYaml, RuleYaml};

pub trait Extractor<Target> {
    fn extract(&self) -> Result<Target, String>;
}

impl Extractor<RuleCore> for RuleYaml {

    fn extract(&self) -> Result<RuleCore, String> {

        let regex_method_path: Regex = Regex::new(r"^(?<method>[A-Z]+)\s+(?<path>.+)$").unwrap();

        let (_, [method, path]) = regex_method_path
            .captures(&*self.matches)
            .expect(&*format!("Malformed match, should be like ’METHOD /path/to/resource’ but was {}", self.matches))
            .extract();

        let real_method = Method::from_str(method).unwrap();
        let real_path = PathAndQuery::from_str(path).unwrap();

        let regex_status: Regex = Regex::new(r"^(?<status>[0-9]+)/(?<format>[a-z]+)$").unwrap();

        let (_, [status, format]) = regex_status
            .captures(&*self.status)
            .expect(&*format!("Malformed status, should be like ’200/json’ but was {}", self.status))
            .extract();

        let real_status = StatusCode::from_str(status).unwrap();
        let body: Option<String> = self.body.clone().and_then(|b| {
            if b.is_empty() { None } else {
                Some(b.to_owned())
            }
        });

        Ok(RuleCore {
            route: real_path,
            method: real_method,
            status: real_status,
            format: format.to_string(),
            body
        })
    }
}

impl Extractor<ApiCore> for ApiYaml {
    fn extract(&self) -> Result<ApiCore, String> {

        let extracted_rules: Result<Vec<RuleCore>, String> = self.rules
            .clone()
            .into_iter()
            .map(|r| r.extract())
            .collect();

        extracted_rules.map(|rules| {
            ApiCore {
                name: self.name.to_owned(),
                header: None,
                rules
            }
        })
    }
}