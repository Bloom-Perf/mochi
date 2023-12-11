use handlebars::template::{DecoratorTemplate, HelperTemplate, Parameter, TemplateElement};
use handlebars::Template;

struct TemplateShape {
    pub name: Parameter,
    pub params: Vec<Parameter>,
    pub template: Option<Template>,
    pub inverse: Option<Template>,
}

impl TemplateShape {
    fn register_params(self, params: &mut Vec<Parameter>) {
        params.push(self.name);
        for el in self.params.into_iter() {
            params.push(el);
        }
        if let Some(t) = self.template {
            for p in t.extract_parameters() {
                params.push(p);
            }
        }

        if let Some(t) = self.inverse {
            for p in t.extract_parameters() {
                params.push(p);
            }
        }
    }
}

trait RegisterParams {
    fn register_params(&self, params: &mut Vec<Parameter>);
}

impl RegisterParams for HelperTemplate {
    fn register_params(&self, params: &mut Vec<Parameter>) {
        TemplateShape {
            params: self.params.clone(),
            name: self.name.clone(),
            template: self.template.clone(),
            inverse: self.inverse.clone(),
        }
        .register_params(params);
    }
}

impl RegisterParams for DecoratorTemplate {
    fn register_params(&self, params: &mut Vec<Parameter>) {
        TemplateShape {
            params: self.params.clone(),
            name: self.name.clone(),
            template: self.template.clone(),
            inverse: None,
        }
        .register_params(params);
    }
}

pub trait TemplateParameterExtractor {
    fn extract_parameters(&self) -> Vec<Parameter>;
}
impl TemplateParameterExtractor for Template {
    fn extract_parameters(&self) -> Vec<Parameter> {
        let mut params: Vec<Parameter> = vec![];

        for el in self.elements.iter() {
            match el {
                TemplateElement::Expression(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::DecoratorBlock(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::DecoratorExpression(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::HelperBlock(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::HtmlExpression(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::PartialBlock(e) => {
                    e.register_params(&mut params);
                }
                TemplateElement::PartialExpression(e) => {
                    e.register_params(&mut params);
                }
                _ => (),
            }
        }

        params
    }
}
