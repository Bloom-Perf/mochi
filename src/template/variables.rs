use handlebars::template::Parameter;

pub mod constants {
    pub const HEADERS: &'static str = "headers";
    pub const URL_QUERY: &'static str = "url.query";
    pub const URL_PATH: &'static str = "url.path";
    pub const BODY_JSON: &'static str = "body.json";
    pub const BODY_TEXT: &'static str = "body.text";
}

#[derive(Clone, Debug)]
pub struct HasVariables {
    pub has_headers: bool,
    pub has_url_query: bool,
    pub has_url_path: bool,
    pub has_body_json: bool,
    pub has_body_text: bool,
}

pub trait FindVariables {
    fn find_present_variables(&self) -> HasVariables;
}

impl FindVariables for Vec<Parameter> {
    fn find_present_variables(&self) -> HasVariables {
        let mut has_variables = HasVariables {
            has_headers: false,
            has_url_query: false,
            has_url_path: false,
            has_body_json: false,
            has_body_text: false,
        };

        for p in self.iter() {
            let name = p.as_name().unwrap_or_default();

            if name.starts_with(constants::HEADERS) {
                has_variables.has_headers = true;
                continue;
            }

            if name.starts_with(constants::URL_QUERY) {
                has_variables.has_url_query = true;
                continue;
            }

            if name.starts_with(constants::URL_PATH) {
                has_variables.has_url_path = true;
                continue;
            }

            if name.starts_with(constants::BODY_JSON) {
                has_variables.has_body_json = true;
                continue;
            }

            if name.starts_with(constants::BODY_TEXT) {
                has_variables.has_body_text = true;
                continue;
            }
        }

        has_variables
    }
}
