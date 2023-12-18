use crate::core::RuleBodyCore;
use crate::template::helper_xpath::XPATH_HELPER;
use crate::template::parameter::TemplateParameterExtractor;
use crate::template::variables::HasVariables;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query, Request};
use handlebars::template::TemplateElement;
use handlebars::Handlebars;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::collections::HashMap;

const TEMPLATE_KEY: &str = "tpl";
pub fn rule_body_from_str(content: String) -> RuleBodyCore {
    let mut registry: Handlebars<'static> = Handlebars::new();
    let _: () = registry
        .register_template_string(TEMPLATE_KEY, content.clone())
        .context("Registering template in registry")
        .unwrap();

    registry.register_helper("xpath", Box::new(XPATH_HELPER));
    match registry.get_template(TEMPLATE_KEY) {
        Some(template) => match template.elements.as_slice() {
            [TemplateElement::RawString(e)] => RuleBodyCore::Plain(e.to_owned()),
            _ => {
                let parameters = template.extract_parameters();
                RuleBodyCore::Templated {
                    url_query: parameters.has_url_query(),
                    url_path: parameters.has_url_path(),
                    headers: parameters.has_headers(),
                    request_body_text: parameters.has_body_text(),
                    request_body_json: parameters.has_body_json(),
                    registry,
                }
            }
        },
        None => RuleBodyCore::Plain(content),
    }
}

pub async fn rule_body_to_str(request: Request<Body>, content: RuleBodyCore) -> Result<String> {
    match content {
        RuleBodyCore::Plain(content) => Ok(content),
        RuleBodyCore::Templated {
            registry,
            request_body_json,
            request_body_text,
            url_path,
            url_query,
            headers,
        } => {
            let (mut parts, body) = request.into_parts();
            let bytes = body.collect().await.unwrap().to_bytes();
            let req_body_json: Option<Value> = if request_body_json {
                serde_json::from_slice(bytes.as_ref()).ok()
            } else {
                None
            };

            let req_body_text: Option<String> = if request_body_text {
                String::from_utf8(bytes.to_vec()).ok()
            } else {
                None
            };

            let json_headers: Option<Value> = if headers {
                Some(json!(parts
                    .headers
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_str().unwrap().to_string()))
                    .collect::<HashMap<String, String>>()))
            } else {
                None
            };

            let url_query_params: Option<Value> = if url_query {
                let Query(query_params): Query<HashMap<String, String>> =
                    Query::try_from_uri(&parts.uri).context("Parsing query")?;
                Some(json!(query_params))
            } else {
                None
            };

            let url_path_params: Option<Value> = if url_path {
                let Path(path_params): Path<HashMap<String, String>> =
                    Path::from_request_parts(&mut parts, &())
                        .await
                        .context("Parsing path parameters")?;
                Some(json!(path_params))
            } else {
                None
            };

            registry
                .render(
                    TEMPLATE_KEY,
                    &json!({
                        "headers": json_headers,
                        "url": {
                            "query": url_query_params,
                            "path": url_path_params,
                        },
                        "body":{
                            "json": req_body_json,
                            "text": req_body_text
                        }
                    }),
                )
                .context("Rendering template")
        }
    }
}
