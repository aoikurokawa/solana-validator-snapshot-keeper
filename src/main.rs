mod cli;
mod config;
mod keeper;
mod manager;

use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::manager::Manager;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();
    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Logger may or may not be installed depending on where we failed; print the
            // error chain unconditionally so the user always sees it.
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Cli) -> Result<()> {
    let cfg = Config::load(&args.config)
        .with_context(|| format!("loading config from {}", args.config.display()))?;
    cfg.log.install(args.log_level.as_deref(), args.log_disable_timestamps);

    let manager = Manager::new(&cfg);

    match args.command {
        Command::Run { on_interval } => match on_interval {
            Some(s) => {
                let dur: Duration = humantime::parse_duration(&s)
                    .with_context(|| format!("invalid interval {s:?}"))?;
                manager.run_on_interval(dur).await
            }
            None => manager.run_once().await,
        },
    }
}
