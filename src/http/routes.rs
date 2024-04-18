use crate::core::{ApiCore, ConfCore, HttpRoute, LatencyCore, RuleBodyCore, RuleCore, SystemCore};
use axum::body::Body;
use axum::http::{Request, StatusCode, Uri};

use crate::metrics::MochiMetrics;
use crate::template::render::rule_body_to_str;
use anyhow::Context;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, on, MethodFilter};
use axum::Router;
use log::{debug, warn};
use reqwest::Method;
use std::collections::HashMap;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;

async fn compute_latency(latency: LatencyCore) {
    match latency {
        LatencyCore::Constant(v) => sleep(Duration::from_millis(v.into())).await,
    }
}

async fn generate_response_body(
    request: Request<Body>,
    rule_body_core: Option<RuleBodyCore>,
) -> Body {
    match rule_body_core {
        None => Body::empty(),
        Some(rule_body_core) => {
            Body::from(rule_body_to_str(request, rule_body_core).await.unwrap())
        }
    }
}

pub async fn handle_request<'reg>(request: Request<Body>, rules: Vec<RuleCore>) -> Response<Body> {
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
            if let Some(value) = rule.latency.clone() {
                compute_latency(value).await
            };

            let body = generate_response_body(request, rule.body.clone()).await;

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

        // root api
        for ApiCore(rules) in self.root_api_set.apis.iter() {
            for rule in rules.iter() {
                // dbg!(rule.clone());
                let http_route = HttpRoute {
                    route: format!("{}", rule.endpoint.route.to_owned()),
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

async fn handle_proxy_request(
    request_method: Method,
    request_uri: Uri,
    target_url: Uri,
    target_path: String,
    request_body: String,
) -> anyhow::Result<Response> {
    let query_params = match request_uri.query() {
        Some(str) => format!("?{str}"),
        None => "".to_string(),
    };

    let reconstructed_uri = format!("{target_url}{target_path}{query_params}");

    debug!("Request received {request_uri} and redirecting to {reconstructed_uri}");

    let new_url = reqwest::Url::parse(reconstructed_uri.as_str())
        .context(format!("Reconstructing target uri {reconstructed_uri}"))?;

    let response = reqwest::Client::new()
        .request(request_method, new_url)
        .body(request_body)
        .send()
        .await
        .context(format!(
            "Sending request / receiving response from {target_url}"
        ))?;

    Response::builder()
        .status(response.status())
        .header(
            "Content-Type",
            response.headers().get("Content-Type").unwrap(),
        )
        .body(Body::from(response.bytes().await.unwrap()))
        .context("Build response")
}

impl ConfCore {
    pub fn build_router(&self, initial_router: Router<MochiMetrics>) -> Router<MochiMetrics> {
        self.systems
            .iter()
            .fold(initial_router, move |r, system| {
                let system_name_subrouter = system.name.clone();

                // static sub router built from the ./config folder
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

                let system_name_subrouter = system.name.clone();

                let proxy_router =
                    system
                        .api_sets
                        .iter()
                        .fold(Router::new(), move |acc, api| match &api.proxy {
                            Some(p) => {
                                let url = p.0.clone();
                                acc.route(
                                &format!("/{}/*path", api.name),
                                any(move |method: Method,
                                       uri: Uri,
                                       Path(path): Path<String>,
                                       body: String| async {
                                    match handle_proxy_request(method, uri, url, path, body).await {
                                        Ok(content) => Ok::<_, Infallible>(content),
                                        Err(e) => Ok::<_, Infallible>(
                                            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                                                .into_response(),
                                        ),
                                    }
                                }),
                            )
                            }
                            None => acc,
                        })
                        .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
                            handler404(m, r, system_name_subrouter)
                        });

                r.nest(&format!("/static/{}", system.name), subrouter)
                    .nest(&format!("/proxy/{}", system.name), proxy_router)
            })
            .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
                handler404(m, r, "Mochi System".to_string())
            })
    }
}
