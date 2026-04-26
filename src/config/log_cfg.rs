use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Log {
    pub level: String,
    pub format: String,
    pub disable_timestamps: bool,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            level: "info".into(),
            format: "text".into(),
            disable_timestamps: false,
        }
    }
}

impl Log {
    pub fn validate(&self) -> Result<()> {
        parse_level(&self.level)?;
        match self.format.as_str() {
            "text" | "json" | "logfmt" => {}
            other => bail!(
                "log.format must be one of text, json, logfmt - got: {other}"
            ),
        }
        Ok(())
    }

    pub fn install(&self, level_override: Option<&str>, disable_timestamps_override: bool) {
        let level_str = level_override
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.level);
        let level = parse_level(level_str).unwrap_or(Level::INFO);
        let disable_ts = self.disable_timestamps || disable_timestamps_override;
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level.to_string()));

        let builder = fmt().with_env_filter(filter);

        // Single-shot install. tracing's global subscriber can only be set once.
        match self.format.as_str() {
            "json" => {
                let s = builder.json();
                if disable_ts {
                    let _ = s.without_time().try_init();
                } else {
                    let _ = s.try_init();
                }
            }
            _ => {
                if disable_ts {
                    let _ = builder.without_time().try_init();
                } else {
                    let _ = builder.try_init();
                }
            }
        }
    }
}

fn parse_level(s: &str) -> Result<Level> {
    match s.to_ascii_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        // tracing has no FATAL — collapse into ERROR
        "fatal" => Ok(Level::ERROR),
        other => bail!(
            "log.level must be one of debug, info, warn, error, fatal - got: {other}"
        ),
    }
}
