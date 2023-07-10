use serde::{Deserialize};
use serde_yaml::Mapping;

#[derive(Deserialize, Clone)]
pub struct ApiYaml {
    pub name: String,
    pub headers: Mapping,
    pub rules: Vec<RuleYaml>
}

#[derive(Deserialize, Clone)]
pub struct RuleYaml {
    pub matches: String,
    pub status: String,
    pub body: Option<String>
}


#[derive(Deserialize, Clone)]
pub struct ApiShapeYaml {
    pub name: String,
    pub shape: Vec<String>
}