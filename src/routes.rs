use crate::model::core::{ApiCore, LatencyCore, RuleCore, SystemCore};
use axum::body::{Body, BoxBody};
use axum::http::{Method, Request, Response, StatusCode};
use axum::response::IntoResponse;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HttpRoute {
    pub route: String,
    pub method: Method,
}

async fn compute_latency(latency: LatencyCore) {
    match latency {
        LatencyCore::Constant(v) => sleep(Duration::from_millis(v.into())).await,
    }
}

pub async fn handle_request(request: Request<Body>, rules: Vec<RuleCore>) -> Response<BoxBody> {
    for rule in rules.iter() {
        dbg!(rule.clone());
        // All api headers must match the corresponding headers in the received request
        let matching_request = rule.headers.iter().all(|(key, value)| {
            request
                .headers()
                .get(key)
                .map(move |req_header_value| req_header_value.to_str().unwrap() == &*value)
                .unwrap_or(false)
        });

        if matching_request {
            if let Some(value) = rule.clone().latency {
                compute_latency(value).await
            }

            let body = rule.body.clone().map(Body::from).unwrap_or(Body::empty());
            let body = axum::body::boxed(body);
            return Response::builder()
                .header("Content-Type", rule.format.to_owned())
                .status(rule.status)
                .body(body)
                .unwrap();
        }
    }
    return StatusCode::NOT_FOUND.into_response();
}

pub type RulesMap = HashMap<HttpRoute, Vec<RuleCore>>;

impl SystemCore {
    pub fn generate_rules_map(self) -> RulesMap {
        let mut rules_map: RulesMap = HashMap::new();
        for api_set in self.api_sets.into_iter() {
            for ApiCore(rules) in api_set.apis.into_iter() {
                for rule in rules.into_iter() {
                    // dbg!(rule.clone());
                    let http_route = HttpRoute {
                        route: format!(
                            "/{}/{}{}",
                            self.name,
                            api_set.name,
                            rule.endpoint.route.to_owned()
                        ),
                        method: rule.endpoint.method.to_owned(),
                    };

                    dbg!(rule.clone());

                    rules_map
                        .entry(http_route)
                        .and_modify(|v| v.push(rule.to_owned()))
                        .or_insert(vec![rule.to_owned()]);
                }
            }
        }

        rules_map
    }
}
