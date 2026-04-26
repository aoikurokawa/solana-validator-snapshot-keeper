pub mod cluster;
pub mod hooks;
pub mod log_cfg;
pub mod size;
pub mod snapshots;
pub mod validator;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use cluster::Cluster;
pub use hooks::Hooks;
pub use log_cfg::Log;
pub use snapshots::Snapshots;
pub use validator::Validator;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub log: Log,
    pub validator: Validator,
    pub cluster: Cluster,
    pub snapshots: Snapshots,
    pub hooks: Hooks,
    #[serde(skip)]
    pub file: PathBuf,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let mut cfg = match std::fs::read_to_string(path) {
            Ok(text) => serde_yaml_ng::from_str::<Config>(&text)
                .with_context(|| format!("parsing config file {}", path.display()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!(path = %path.display(), "config file not found, using defaults");
                Config::default()
            }
            Err(e) => {
                return Err(anyhow::Error::new(e)
                    .context(format!("loading config file {}", path.display())));
            }
        };
        cfg.file = path.to_path_buf();
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        self.log.validate().context("log config")?;
        self.validator.validate().context("validator config")?;
        self.cluster.validate().context("cluster config")?;
        self.snapshots.validate().context("snapshots config")?;
        Ok(())
    }
}

pub fn default_config_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join("solana-validator-snapshot-keeper").join("config.yml")
    } else {
        PathBuf::from("config.yml")
    }
}
