use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub const MAINNET_BETA: &str = "mainnet-beta";
pub const TESTNET: &str = "testnet";

pub const VALID_CLUSTERS: &[&str] = &[MAINNET_BETA, TESTNET];

pub fn cluster_rpc_url(name: &str) -> Option<&'static str> {
    match name {
        MAINNET_BETA => Some("https://api.mainnet-beta.solana.com"),
        TESTNET => Some("https://api.testnet.solana.com"),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Cluster {
    pub name: String,
    pub rpc_url: String,
}

impl Default for Cluster {
    fn default() -> Self {
        Self {
            name: MAINNET_BETA.into(),
            rpc_url: String::new(),
        }
    }
}

impl Cluster {
    pub fn validate(&self) -> Result<()> {
        if !VALID_CLUSTERS.contains(&self.name.as_str()) {
            bail!(
                "invalid cluster name {:?}, must be one of: {:?}",
                self.name,
                VALID_CLUSTERS
            );
        }
        Ok(())
    }

    pub fn effective_rpc_url(&self) -> String {
        if !self.rpc_url.is_empty() {
            return self.rpc_url.clone();
        }
        cluster_rpc_url(&self.name).unwrap_or("").to_string()
    }
}
