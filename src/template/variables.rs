use handlebars::template::Parameter;

pub mod constants {
    pub const HEADERS: &'static str = "headers";
    pub const URL_QUERY: &'static str = "url.query";
    pub const URL_PATH: &'static str = "url.path";
    pub const BODY_JSON: &'static str = "body.json";
    pub const BODY_TEXT: &'static str = "body.text";
}

pub trait HasVariables {
    fn has_headers(&self) -> bool;
    fn has_url_query(&self) -> bool;
    fn has_url_path(&self) -> bool;
    fn has_body_json(&self) -> bool;
    fn has_body_text(&self) -> bool;
}

impl HasVariables for Vec<Parameter> {
    fn has_headers(&self) -> bool {
        self.iter().any(|p| {
            p.as_name()
                .unwrap_or_default()
                .starts_with(constants::HEADERS)
        })
    }
    fn has_url_query(&self) -> bool {
        self.iter().any(|p| {
            p.as_name()
                .unwrap_or_default()
                .starts_with(constants::URL_QUERY)
        })
    }

    fn has_url_path(&self) -> bool {
        self.iter().any(|p| {
            p.as_name()
                .unwrap_or_default()
                .starts_with(constants::URL_PATH)
        })
    }

    fn has_body_json(&self) -> bool {
        self.iter().any(|p| {
            p.as_name()
                .unwrap_or_default()
                .starts_with(constants::BODY_JSON)
        })
    }

    fn has_body_text(&self) -> bool {
        self.iter().any(|p| {
            p.as_name()
                .unwrap_or_default()
                .starts_with(constants::BODY_TEXT)
        })
    }
}
