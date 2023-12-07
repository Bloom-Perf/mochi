pub(crate) mod filesystem;
pub(crate) mod from_files;
pub(crate) mod to_domain;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub enum LatencyYaml {
    Constant(u32),
}

#[derive(Deserialize, Clone, Debug)]
pub enum Response {
    File(String),
    Inline(u16, Option<String>, Option<String>),
}

#[derive(Deserialize, Clone, Debug)]
pub struct ResponseDataYaml {
    pub status: u16,
    pub description: Option<String>,
    pub format: Option<String>,
    pub data: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiYaml {
    pub headers: Option<HashMap<String, String>>,
    pub latency: Option<LatencyYaml>,
    pub rules: Vec<RuleYaml>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RuleYaml {
    pub matches: String,
    pub latency: Option<LatencyYaml>,
    pub response: Response,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApiShapeYaml {
    pub shape: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ApiFolder {
    pub name: String,
    pub shape: Option<ApiShapeYaml>,
    pub apis: Vec<ApiYaml>,
    pub data: HashMap<String, ResponseDataYaml>,
}

#[derive(Clone, Debug)]
pub struct SystemFolder {
    pub name: String,
    pub api_folders: Vec<ApiFolder>,
    pub shape: Option<ApiShapeYaml>,
    pub apis: Vec<ApiYaml>,
    pub data: HashMap<String, ResponseDataYaml>,
}

#[derive(Clone, Debug)]
pub struct ConfFolder {
    pub systems: Vec<SystemFolder>,
}
