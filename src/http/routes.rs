use crate::core::{ApiCore, ConfCore, HttpRoute, LatencyCore, RuleBodyCore, RuleCore, SystemCore};
use axum::body::Body;
use axum::http::{Request, StatusCode};

use crate::http::handler404;
use crate::template::render::rule_body_to_str;
use crate::MochiRouterState;
use anyhow::Context;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::{on, MethodFilter};
use axum::Router;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

async fn compute_latency(latency: &LatencyCore) {
    match latency {
        LatencyCore::Constant(v) => sleep(Duration::from_millis(v.clone().into())).await,
    }
}

async fn generate_response_body(
    request: Request<Body>,
    rule_body_core: &Option<RuleBodyCore>,
) -> anyhow::Result<Body> {
    match rule_body_core {
        None => Ok(Body::empty()),
        Some(rule_body_core) => Ok(Body::from(
            rule_body_to_str(request, rule_body_core)
                .await
                .context("Generating response body")?,
        )),
    }
}

pub async fn handle_request(request: Request<Body>, rules: Vec<RuleCore>) -> Response<Body> {
    for rule in rules.iter() {
        // All api headers must match the corresponding headers in the received request
        let matching_request = rule.headers.iter().all(|(key, value)| {
            request
                .headers()
                .get(key)
                .map(move |req_header_value| req_header_value.to_str().unwrap() == *value)
                .unwrap_or(false)
        });

        if matching_request {
            if let Some(value) = &rule.latency {
                compute_latency(value).await
            };

            let body = generate_response_body(request, &rule.body).await;

            return match body {
                Ok(b) => Response::builder()
                    .header("Content-Type", rule.format.to_owned())
                    .status(rule.status)
                    .body(b)
                    .unwrap(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Could not generate response body: {:#?}", e),
                )
                    .into_response(),
            };
        }
    }
    StatusCode::NOT_FOUND.into_response()
}

pub type SystemRulesMap = HashMap<HttpRoute, Vec<RuleCore>>;

impl SystemCore {
    pub fn generate_rules_map(&self) -> SystemRulesMap {
        let mut rules_map: SystemRulesMap = HashMap::new();

        // root api
        for ApiCore(rules) in self.root_api_set.apis.iter() {
            for rule in rules.iter() {
                // dbg!(rule.clone());
                let http_route = HttpRoute {
                    route: rule.endpoint.route.to_string(),
                    method: rule.endpoint.method.to_owned(),
                };

                rules_map
                    .entry(http_route)
                    .and_modify(|v| v.push(rule.to_owned()))
                    .or_insert(vec![rule.to_owned()]);
            }
        }

        // api folders
        for api_set in self.api_sets.iter() {
            for ApiCore(rules) in api_set.apis.iter() {
                for rule in rules.iter() {
                    // dbg!(rule.clone());
                    let http_route = HttpRoute {
                        route: format!("/{}{}", api_set.name, rule.endpoint.route.to_string()),
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

//fn register_request_response_into_proxy_state()

impl ConfCore {
    pub fn build_router(
        &self,
        initial_router: Router<MochiRouterState>,
    ) -> Router<MochiRouterState> {
        let mut global_router: Router<MochiRouterState> = initial_router;

        for system in self.systems.iter() {
            let system_name = system.name.clone();

            // static sub router built from the ./config folder
            let subrouter = system
                .generate_rules_map()
                .into_iter()
                .fold(Router::new(), |acc, (endpoint, rules)| {
                    acc.route(
                        &endpoint.route,
                        on(MethodFilter::try_from(endpoint.method).unwrap(), {
                            move |request: Request<Body>| handle_request(request, rules.to_owned())
                        }),
                    )
                })
                .fallback(move |m: State<MochiRouterState>, r: Request<Body>| {
                    handler404(m, r, system_name)
                });

            let proxy_router = system.create_proxy_router();

            // Proxy setup

            global_router = global_router
                .nest(&format!("/static/{}", &system.name), subrouter)
                .nest(&format!("/proxy/{}", &system.name), proxy_router)
        }

        global_router.fallback(move |m: State<MochiRouterState>, r: Request<Body>| {
            handler404(m, r, "Mochi System".to_string())
        })
    }
}
