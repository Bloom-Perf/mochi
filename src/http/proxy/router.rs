use crate::core::SystemCore;
use crate::http::handler404;
use crate::MochiRouterState;
use anyhow::Context;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::Router;
use itertools::Itertools;
use log::debug;
use std::convert::Infallible;

impl SystemCore {
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
                "Sending request/receiving response from {target_url}"
            ))?;

        Response::builder()
            .status(response.status())
            .header(
                "Content-Type",
                response
                    .headers()
                    .get("Content-Type")
                    .unwrap_or(&HeaderValue::from_static("text/plain")),
            )
            .body(Body::from(response.bytes().await.context(format!(
                "Getting bytes from http client response (from {})",
                &reconstructed_uri
            ))?))
            .context("Building response to http server request")
    }
    pub fn create_proxy_router(&self) -> Router<MochiRouterState> {
        let system = self;
        let mut proxy_router: Router<MochiRouterState> = Router::new();
        for api in system.api_sets.iter() {
            if let Some(p) = &api.proxy {
                let url = p.0.clone();
                let system_name = system.name.clone();
                let api_name = api.name.clone();

                proxy_router = proxy_router.route(
                    "/config",
                    get(|State(s): State<MochiRouterState>| async move {
                        let state = s.proxy.read().unwrap();
                        state
                            .routes
                            .iter()
                            .map(|r| r.display(0))
                            .format("\n")
                            .to_string()
                            .into_response()
                    }),
                );

                proxy_router = proxy_router.route(
                    &format!("/{}/*path", &api_name),
                    any(
                        move |s: State<MochiRouterState>,
                              method: Method,
                              uri: Uri,
                              Path(path): Path<String>,
                              body: String| async move {
                            s.metrics.mochi_proxy_request_counter(
                                &system_name,
                                Some(&api_name),
                                &url.to_string(),
                                &path,
                            );

                            let _ = s
                                .proxy
                                .write()
                                .map(|mut w| w.append_path(path.split('/').collect()));

                            dbg!(&s.proxy);

                            match SystemCore::handle_proxy_request(method, uri, url, path, body)
                                .await
                            {
                                Ok(content) => Ok::<_, Infallible>(content),
                                Err(e) => Ok::<_, Infallible>(
                                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                                        .into_response(),
                                ),
                            }
                        },
                    ),
                )
            }
        }

        let system_name = system.name.clone();

        proxy_router =
            proxy_router.fallback(move |m: State<MochiRouterState>, r: Request<Body>| {
                handler404(m, r, system_name)
            });

        proxy_router
    }
}
