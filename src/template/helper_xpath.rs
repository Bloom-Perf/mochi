use handlebars::{
    to_json, BlockContext, Context, Handlebars, Helper, HelperDef, HelperResult, Output,
    PathAndJson, RenderContext, RenderError, RenderErrorReason, Renderable,
};
use serde_json::value::Value as Json;
use std::collections::HashMap;
use sxd_document::parser;
use sxd_xpath::evaluate_xpath;
use sxd_xpath::nodeset::Node;

fn create_block<'rc>(param: &PathAndJson<'rc>) -> BlockContext<'rc> {
    let mut block = BlockContext::new();

    if let Some(new_path) = param.context_path() {
        *block.base_path_mut() = new_path.clone();
    } else {
        // use clone for now
        block.set_base_value(param.value().clone());
    }

    block
}

#[derive(Clone, Copy)]
pub struct XpathHelper;

impl HelperDef for XpathHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let body = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("xpath", 0))?;
        let query = h
            .param(1)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("xpath", 1))?;

        let template = h.template();

        match template {
            Some(t) => match *body.value() {
                Json::String(ref body_str)
                    if !body_str.is_empty() || (body_str.is_empty() && h.inverse().is_none()) =>
                {
                    let block_context = create_block(body);
                    rc.push_block(block_context);

                    let package = parser::parse(body_str)
                        .ok()
                        .ok_or(RenderErrorReason::Other("problem parsing xml".to_string()))?;
                    let document = package.as_document();

                    let query_xpath = match *query.value() {
                        Json::String(ref query_str) => Ok(query_str),
                        _ => Err(RenderErrorReason::InvalidParamType(
                            "xpath query must be a string",
                        )),
                    }
                    .ok()
                    .ok_or(RenderErrorReason::Other(
                        "problem parsing xpath query".to_string(),
                    ))?;

                    let value = evaluate_xpath(&document, query_xpath).ok().ok_or(
                        RenderErrorReason::Other("problem evaluating xpath query".to_string()),
                    )?;
                    match value {
                        sxd_xpath::Value::Nodeset(nodeset) => {
                            dbg!(&nodeset);
                            if let Some(ref mut block) = rc.block_mut() {
                                let mut values: Vec<serde_json::Value> = vec![];

                                for el in nodeset.iter() {
                                    match el {
                                        Node::Text(v) => {
                                            values.push(to_json(v.text()));
                                        }
                                        Node::Element(e) => values.push(to_json(
                                            e.attributes()
                                                .iter()
                                                .map(|attr| {
                                                    (
                                                        attr.name().local_part().to_string(),
                                                        attr.value(),
                                                    )
                                                })
                                                .collect::<HashMap<String, &str>>(),
                                        )),
                                        _ => (),
                                    }
                                }

                                block.set_local_var("results", to_json(values));
                            }

                            t.render(r, ctx, rc, out)?;
                        }
                        sxd_xpath::Value::String(v) => {
                            if let Some(ref mut block) = rc.block_mut() {
                                block.set_local_var("results", to_json(vec![v]));
                            }

                            t.render(r, ctx, rc, out)?;
                        }
                        sxd_xpath::Value::Number(v) => {
                            if let Some(ref mut block) = rc.block_mut() {
                                block.set_local_var("results", to_json(vec![v]));
                            }

                            t.render(r, ctx, rc, out)?;
                        }
                        sxd_xpath::Value::Boolean(v) => {
                            if let Some(ref mut block) = rc.block_mut() {
                                block.set_local_var("results", to_json(vec![v]));
                            }

                            t.render(r, ctx, rc, out)?;
                        }
                    }

                    rc.pop_block();
                    Ok(())
                }
                _ => {
                    if let Some(else_template) = h.inverse() {
                        else_template.render(r, ctx, rc, out)
                    } else if r.strict_mode() {
                        Err(RenderError::strict_error(body.relative_path()))
                    } else {
                        Ok(())
                    }
                }
            },
            None => Ok(()),
        }
    }
}

pub static XPATH_HELPER: XpathHelper = XpathHelper;
