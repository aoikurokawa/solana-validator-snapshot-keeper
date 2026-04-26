use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::default_config_path;

#[derive(Debug, Parser)]
#[command(
    name = "solana-validator-snapshot-keeper",
    version,
    about = "Keeps fresh Solana snapshots on disk"
)]
pub struct Cli {
    #[arg(short = 'c', long = "config", default_value_os_t = default_config_path(), global = true)]
    pub config: PathBuf,

    #[arg(long = "log-level", global = true)]
    pub log_level: Option<String>,

    #[arg(long = "log-disable-timestamps", global = true)]
    pub log_disable_timestamps: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the snapshot keeper (once or on an interval).
    Run {
        /// Run on an interval (e.g. 4h, 30m).
        #[arg(short = 'i', long = "on-interval")]
        on_interval: Option<String>,
    },
}
