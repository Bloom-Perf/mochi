mod extractor;
mod file;
mod model;
mod routes;

use crate::extractor::build_all;
use crate::file::ConfigurationFolder;
use crate::model::core::SystemCore;
use crate::model::yaml::SystemFolder;
use crate::routes::handle_request;
use axum::body::Body;
use axum::http::Request;
use axum::routing::{on, MethodFilter};
use axum::Router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // initialize tracing

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new("./config").load_systems();

    // build our application with a route

    let rules_maps: Vec<_> = system_folders
        .into_iter()
        .map(|system| {
            let s = SystemCore {
                name: system.name.to_owned(),
                api_sets: build_all(system.shapes.to_owned(), system.apis),
            };
            s.generate_rules_map()
        })
        .collect();

    // handlers_maps.clone().iter().for_each(|e| {
    //     for x in e.keys() {
    //         dbg!(x);
    //     }
    // });

    let app: Router = rules_maps.into_iter().fold(Router::new(), move |r, map| {
        let subrouter = map
            .into_iter()
            .fold(Router::new(), |acc, (endpoint, rules)| {
                // dbg!(endpoint.clone());
                acc.route(
                    &endpoint.route,
                    on(MethodFilter::try_from(endpoint.clone().method).unwrap(), {
                        move |request: Request<Body>| handle_request(request, rules.to_owned())
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
