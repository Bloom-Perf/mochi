use crate::core::RuleBodyCore;
use crate::template::parameter::TemplateParameterExtractor;
use crate::template::variables::HasVariables;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query, Request};
use handlebars::template::TemplateElement;
use handlebars::{Handlebars, RenderContext, RenderError, Renderable, StringOutput, Template};
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

pub trait RenderEasy {
    fn render_easy<T>(&self, data: T) -> Result<String>
    where
        T: Serialize;
}

impl RenderEasy for Template {
    fn render_easy<T>(&self, data: T) -> Result<String>
    where
        T: Serialize,
    {
        let registry = Handlebars::new();
        let mut output = StringOutput::new();
        let ctx = handlebars::Context::wraps(data).context("Building context for template")?;
        let mut render_context = RenderContext::new(self.name.as_ref());
        self.render(&registry, &ctx, &mut render_context, &mut output)?;
        output
            .into_string()
            .map_err(RenderError::from)
            .context("Rendering template")
    }
}

pub fn rule_body_from_str(content: String) -> RuleBodyCore {
    let mut registry = Handlebars::new();
    let _: () = registry
        .register_template_string("tpl", content.clone())
        .context("Registering template in registry")
        .unwrap();

    match registry.get_template("tpl") {
        Some(template) => match template.elements.as_slice() {
            [TemplateElement::RawString(e)] => RuleBodyCore::Plain(e.to_owned()),
            _ => {
                let parameters = template.extract_parameters();
                RuleBodyCore::Templated {
                    url_query: parameters.has_url_query(),
                    url_path: parameters.has_url_path(),
                    headers: parameters.has_headers(),
                    request_body_json: parameters.has_body_json(),
                    template: template.clone(),
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
            template,
            request_body_json,
            url_path,
            url_query,
            headers,
        } => {
            let (mut parts, body) = request.into_parts();
            let req_body_json: Option<Value> = if request_body_json {
                serde_json::from_slice(body.collect().await.unwrap().to_bytes().as_ref()).ok()
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

            template
                .render_easy(json!({
                    "headers": json_headers,
                    "url": {
                        "query": url_query_params,
                        "path": url_path_params,
                    },
                    "body":{
                        "json": req_body_json
                    }
                }))
                .context("Rendering template")
        }
    }
}
