use crate::model::core::{ApiCore, ApiSetCore, RuleCore, SystemCore};
use axum::body::{Body, BoxBody};
use axum::http::{Method, Request, Response};
use replace_with::replace_with;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HttpRoute {
    pub route: String,
    pub method: Method,
}

fn compute_response(
    request: &Request<Body>,
    rule: RuleCore,
    api_headers: HashMap<String, String>,
) -> Option<Response<BoxBody>> {
    dbg!(api_headers.clone());
    // All api headers must match the corresponding headers in the received request
    let matching_request = api_headers.into_iter().all(|(key, value)| {
        request
            .headers()
            .get(key)
            .map(move |req_header_value| req_header_value.to_str().unwrap() == &*value)
            .unwrap_or(false)
    });

    if matching_request {
        let body = rule.body.clone().map(Body::from).unwrap_or(Body::empty());
        let body = axum::body::boxed(body);
        let res = Response::builder().status(rule.status).body(body).unwrap();
        Some(res)
    } else {
        None
    }
}

pub type GeneratedHandler<'a> =
    dyn 'a + Sync + Send + Fn(&Request<Body>) -> Option<Response<BoxBody>>;
pub type HandlersMap<'a> = HashMap<HttpRoute, Arc<GeneratedHandler<'a>>>;

impl<'a> SystemCore {
    pub fn generate_handlers_map(self) -> HandlersMap<'a> {
        let mut handlers: HandlersMap<'a> = HashMap::new();
        for ApiSetCore {
            name: api_set_name,
            apis,
            ..
        } in self.api_sets.into_iter()
        {
            for ApiCore { headers, rules } in apis.into_iter() {
                for rule in rules.into_iter() {
                    // dbg!(rule.clone());
                    let http_route = HttpRoute {
                        route: format!(
                            "/{}/{}{}",
                            self.name,
                            api_set_name,
                            rule.endpoint.route.to_owned()
                        ),
                        method: rule.endpoint.method.to_owned(),
                    };

                    // dbg!(http_route.clone());

                    let headers_default = headers.clone();
                    let rule_default = rule.clone();

                    let default_create: Arc<GeneratedHandler> = Arc::new(move |req| {
                        compute_response(req, rule_default.to_owned(), headers_default.to_owned())
                    });

                    if let Some(handlerplace) = handlers.get_mut(&http_route) {
                        let headers_replace = headers.clone();
                        let rule_replace = rule.clone();
                        replace_with(
                            handlerplace,
                            || default_create,
                            |old| {
                                Arc::new(move |req| {
                                    old(req).or(compute_response(
                                        req,
                                        rule_replace.to_owned(),
                                        headers_replace.to_owned(),
                                    ))
                                })
                            },
                        );
                    } else {
                        handlers.insert(http_route, default_create);
                    }
                }
            }
        }

        handlers
    }
}
