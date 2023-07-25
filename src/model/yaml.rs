use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub enum LatencyYaml {
    Constant(u32),
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiYaml {
    pub name: String,
    pub headers: HashMap<String, String>,
    pub latency: Option<LatencyYaml>,
    pub rules: Vec<RuleYaml>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RuleYaml {
    pub matches: String,
    pub latency: Option<LatencyYaml>,
    pub status: String,
    pub body: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiShapeYaml {
    pub name: String,
    pub shape: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SystemFolder {
    pub name: String,
    pub shapes: Vec<ApiShapeYaml>,
    pub apis: Vec<ApiYaml>,
}
