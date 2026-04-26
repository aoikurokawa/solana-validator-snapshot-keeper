use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Validator {
    pub rpc_url: String,
    pub active_identity_pubkey: String,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:8899".into(),
            active_identity_pubkey: String::new(),
        }
    }
}

impl Validator {
    pub fn validate(&self) -> Result<()> {
        if self.rpc_url.is_empty() {
            bail!("validator.rpc_url is required");
        }
        if self.active_identity_pubkey.is_empty() {
            bail!("validator.active_identity_pubkey is required");
        }
        Ok(())
    }
}
