use crate::core::{LatencyCore, RuleBodyCore, RuleCore};
use crate::http::MochiRequestHandler;
use crate::template::render::build_templated_response_body;
use anyhow::{bail, Context};
use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use std::time::Duration;
use tokio::time::sleep;

impl LatencyCore {
    async fn compute_latency(&self) {
        match self {
            LatencyCore::Constant(v) => sleep(Duration::from_millis(v.clone().into())).await,
        }
    }
}

impl RuleBodyCore {
    async fn generate_response_body(&self, request: Request<Body>) -> anyhow::Result<Body> {
        let uri = request.uri().clone();
        let method = request.method().clone();

        Ok(Body::from(match self {
            RuleBodyCore::Plain(content) => content.clone(),
            RuleBodyCore::Templated {
                has_variables,
                registry,
            } => build_templated_response_body(registry, has_variables, request)
                .await
                .context(format!(
                    "Generating response body for request received on [{method}] {uri}"
                ))?,
        }))
    }
}

impl MochiRequestHandler for Vec<RuleCore> {
    async fn handle_request(&self, request: Request<Body>) -> anyhow::Result<Response<Body>> {
        for rule in self.iter() {
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
                    value.compute_latency().await
                };

                let body = match &rule.body {
                    Some(b) => b.generate_response_body(request).await?,
                    None => Body::empty(),
                };

                return Response::builder()
                    .header("Content-Type", rule.format.to_owned())
                    .status(rule.status)
                    .body(body)
                    .context("Could not generate response body");
            }
        }

        let uri = request.uri();
        let method = request.method();

        bail!("No rule matched for request [{method}] {uri}");
    }
}
