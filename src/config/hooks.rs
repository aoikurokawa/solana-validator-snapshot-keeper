use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Hooks {
    pub on_success: Vec<HookCommand>,
    pub on_failure: Vec<HookCommand>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HookCommand {
    pub name: String,
    pub cmd: String,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub allow_failure: bool,
    pub stream_output: bool,
    pub disabled: bool,
}
