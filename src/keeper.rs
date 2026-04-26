use anyhow::Result;

use crate::config::Config;

pub struct Keeper<'a> {
    #[allow(dead_code)]
    cfg: &'a Config,
}

impl<'a> Keeper<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self { cfg }
    }

    /// Single keeper cycle. Stub for phase 1; later phases add discovery,
    /// download, and pruning.
    pub async fn run(&self) -> Result<()> {
        tracing::info!("keeper.run() — stub (phase 1 skeleton)");
        Ok(())
    }
}
