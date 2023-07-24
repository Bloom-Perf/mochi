mod extractor;
mod file;
mod model;
mod routes;

use crate::extractor::build_all;
use crate::file::ConfigurationFolder;
use crate::model::core::SystemCore;
use crate::model::yaml::SystemFolder;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{on, patch, post, MethodFilter};
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // initialize tracing

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new("./config").load_systems();

    // build our application with a route

    let handlers_maps: Vec<_> = system_folders
        .into_iter()
        .map(|system| {
            let s = SystemCore {
                name: system.name.to_owned(),
                api_sets: build_all(system.shapes.to_owned(), system.apis),
            };
            s.generate_handlers_map()
        })
        .collect();

    // handlers_maps.clone().iter().for_each(|e| {
    //     for x in e.keys() {
    //         dbg!(x);
    //     }
    // });

    let app: Router = handlers_maps.iter().fold(Router::new(), move |r, map| {
        let subrouter = map.iter().fold(Router::new(), |acc, (endpoint, handler)| {
            // dbg!(endpoint.clone());
            acc.route(
                &endpoint.route,
                on(MethodFilter::try_from(endpoint.clone().method).unwrap(), {
                    let cloned = Arc::clone(handler);
                    move |request: Request<Body>| async move {
                        match cloned(&request) {
                            Some(res) => res,
                            None => StatusCode::NOT_FOUND.into_response(),
                        }
                    }
                }),
            )
        });
        r.merge(subrouter)
    });

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
