use crate::core::{ApiCore, HttpRoute, RuleCore, SystemCore};
use crate::http::{handler404, MochiRequestHandler};
use crate::MochiRouterState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{on, MethodFilter};
use axum::Router;
use std::collections::HashMap;

type SystemRulesMap = HashMap<HttpRoute, Vec<RuleCore>>;
impl SystemCore {
    pub fn generate_rules_map(&self) -> SystemRulesMap {
        let mut rules_map: SystemRulesMap = HashMap::new();

        // root api
        for ApiCore(rules) in self.root_api_set.apis.iter() {
            for rule in rules.iter() {
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
                    let http_route = HttpRoute {
                        route: format!("/{}{}", api_set.name, rule.endpoint.route),
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
    pub fn create_static_router(&self) -> Router<MochiRouterState> {
        let mut router = Router::new();
        let system_name = self.name.clone();
        // static sub router built from the ./config folder
        for (HttpRoute { route, method }, rules) in self.generate_rules_map().into_iter() {
            router = router.route(
                &route,
                on(MethodFilter::try_from(method.clone()).unwrap(), {
                    move |request: Request<Body>| async move {
                        match rules.handle_request(request).await {
                            Ok(res) => res.into_response(),
                            Err(e) => {
                                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
                            }
                        }
                    }
                }),
            )
        }

        router = router.fallback(
            move |m: State<MochiRouterState>, r: Request<Body>| async move {
                handler404(m, r, system_name).await
            },
        );

        router
    }
}
