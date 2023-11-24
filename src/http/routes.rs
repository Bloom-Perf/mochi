use crate::core::{ApiCore, ConfCore, HttpRoute, LatencyCore, RuleCore, SystemCore};
use axum::body::{Body, BoxBody};
use axum::http::{Request, StatusCode};

use crate::metrics::MochiMetrics;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::{on, MethodFilter};
use axum::Router;
use log::warn;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

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
                .map(move |req_header_value| req_header_value.to_str().unwrap() == *value)
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
    StatusCode::NOT_FOUND.into_response()
}

pub type SystemRulesMap = HashMap<HttpRoute, Vec<RuleCore>>;

impl SystemCore {
    pub fn generate_rules_map(&self) -> SystemRulesMap {
        let mut rules_map: SystemRulesMap = HashMap::new();
        for api_set in self.api_sets.iter() {
            for ApiCore(rules) in api_set.apis.iter() {
                for rule in rules.iter() {
                    // dbg!(rule.clone());
                    let http_route = HttpRoute {
                        route: format!("/{}{}", api_set.name, rule.endpoint.route.to_owned()),
                        method: rule.endpoint.method.to_owned(),
                    };

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

async fn handler404(
    State(metrics): State<MochiMetrics>,
    request: Request<Body>,
    system_name: String,
) -> Response {
    warn!(
        "Request with route --- \n\t[{}] {}\n --- did not match any route of the configuration of system \"{}\"",
        request.method(),
        request.uri(),
        system_name
    );
    metrics.mochi_route_not_found(system_name);
    StatusCode::NOT_FOUND.into_response()
}

impl ConfCore {
    pub fn build_router(&self, initial_router: Router<MochiMetrics>) -> Router<MochiMetrics> {
        self.systems
            .iter()
            .fold(initial_router, move |r, system| {
                let system_name_subrouter = system.name.clone();

                let subrouter = system
                    .generate_rules_map()
                    .into_iter()
                    .fold(Router::new(), |acc, (endpoint, rules)| {
                        acc.route(
                            &endpoint.route,
                            on(MethodFilter::try_from(endpoint.clone().method).unwrap(), {
                                move |request: Request<Body>| {
                                    handle_request(request, rules.to_owned())
                                }
                            }),
                        )
                    })
                    .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
                        handler404(m, r, system_name_subrouter)
                    });

                r.nest(&format!("/{}", system.name), subrouter)
            })
            .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
                handler404(m, r, "Mochi System".to_string())
            })
    }
}
