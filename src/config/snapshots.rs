use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use super::size::ByteSize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshots {
    pub directory: String,
    pub discovery: Discovery,
    pub download: Download,
    pub age: Age,
}

impl Default for Snapshots {
    fn default() -> Self {
        Self {
            directory: "/mnt/accounts/snapshots".into(),
            discovery: Discovery::default(),
            download: Download::default(),
            age: Age::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Discovery {
    pub candidates: DiscoveryCandidates,
    pub probe: DiscoveryProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryCandidates {
    pub min_suitable_full: u32,
    pub min_suitable_incremental: u32,
    pub sort_order: String,
}

impl Default for DiscoveryCandidates {
    fn default() -> Self {
        Self {
            min_suitable_full: 3,
            min_suitable_incremental: 5,
            sort_order: "latency".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryProbe {
    pub concurrency: u32,
    #[serde(with = "humantime_serde")]
    pub max_latency: Duration,
}

impl Default for DiscoveryProbe {
    fn default() -> Self {
        Self {
            concurrency: 500,
            max_latency: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Download {
    pub min_speed: ByteSize,
    #[serde(with = "humantime_serde")]
    pub min_speed_check_delay: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    pub connections: u32,
}

impl Default for Download {
    fn default() -> Self {
        Self {
            min_speed: ByteSize(60 * 1024 * 1024),
            min_speed_check_delay: Duration::from_secs(7),
            timeout: Duration::from_secs(30 * 60),
            connections: 8,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Age {
    pub remote: AgeRemote,
    pub local: AgeLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgeRemote {
    pub max_slots: u64,
}

impl Default for AgeRemote {
    fn default() -> Self {
        Self { max_slots: 1300 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgeLocal {
    pub max_incremental_slots: u64,
}

impl Default for AgeLocal {
    fn default() -> Self {
        Self {
            max_incremental_slots: 1300,
        }
    }
}

impl Snapshots {
    pub fn validate(&self) -> Result<()> {
        match self.discovery.candidates.sort_order.as_str() {
            "latency" | "slot_age" => {}
            other => bail!(
                "snapshots.discovery.candidates.sort_order must be \"latency\" or \"slot_age\", got {other:?}"
            ),
        }
        if self.discovery.probe.max_latency.is_zero() {
            bail!("snapshots.discovery.probe.max_latency must be > 0");
        }

        if self.directory.is_empty() {
            bail!("snapshots.directory is required");
        }
        let meta = std::fs::metadata(&self.directory)
            .with_context(|| format!("snapshots.directory: {}", self.directory))?;
        if !meta.is_dir() {
            bail!(
                "snapshots.directory: {} is not a directory",
                self.directory
            );
        }
        let probe_path =
            std::path::Path::new(&self.directory).join(".snapshot-keeper-probe");
        std::fs::write(&probe_path, b"")
            .with_context(|| format!("snapshots.directory: not writable ({})", self.directory))?;
        let _ = std::fs::remove_file(&probe_path);

        if self.download.min_speed.as_u64() < 1 {
            bail!("snapshots.download.min_speed must be > 0");
        }
        if self.download.timeout.is_zero() {
            bail!("snapshots.download.timeout must be > 0");
        }
        if self.download.connections < 1 {
            bail!("snapshots.download.connections must be >= 1");
        }
        if self.age.remote.max_slots < 1 {
            bail!("snapshots.age.remote.max_slots must be >= 1");
        }
        if self.age.local.max_incremental_slots < 1 {
            bail!("snapshots.age.local.max_incremental_slots must be >= 1");
        }
        Ok(())
    }
}
