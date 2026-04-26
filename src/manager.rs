use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::keeper::Keeper;

const LOCK_FILENAME: &str = "solana-validator-snapshot-keeper.lock";

#[derive(Debug, Serialize, Deserialize)]
struct LockInfo {
    pid: i32,
    started_at: String,
}

pub struct Manager<'a> {
    cfg: &'a Config,
}

impl<'a> Manager<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self { cfg }
    }

    pub async fn run_once(&self) -> Result<()> {
        tracing::info!("running snapshot keeper (once)");
        let _guard = LockGuard::acquire(self.lock_path())?;
        Keeper::new(self.cfg).run().await
    }

    pub async fn run_on_interval(&self, interval: Duration) -> Result<()> {
        tracing::info!(?interval, "running snapshot keeper on interval");
        loop {
            let now = Utc::now();
            let next = next_boundary(now, interval);
            let sleep = (next - now).to_std().unwrap_or(Duration::from_secs(0));
            tracing::info!(
                next = %next.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                sleep_seconds = sleep.as_secs(),
                "next run"
            );
            tokio::time::sleep(sleep).await;

            match LockGuard::acquire(self.lock_path()) {
                Ok(_guard) => {
                    if let Err(e) = Keeper::new(self.cfg).run().await {
                        tracing::error!(error = %e, "run failed");
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "skipping cycle, lock held by another process");
                }
            }
        }
    }

    fn lock_path(&self) -> PathBuf {
        Path::new(&self.cfg.snapshots.directory).join(LOCK_FILENAME)
    }
}

struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    fn acquire(path: PathBuf) -> Result<Self> {
        if let Ok(data) = std::fs::read(&path) {
            if let Ok(info) = serde_json::from_slice::<LockInfo>(&data) {
                if is_process_alive(info.pid) {
                    bail!(
                        "another instance is running (PID: {}, started: {})",
                        info.pid,
                        info.started_at
                    );
                }
                tracing::warn!(stale_pid = info.pid, "stale lock file found, overwriting");
            }
        }

        let info = LockInfo {
            pid: std::process::id() as i32,
            started_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        };
        let body = serde_json::to_vec_pretty(&info).context("marshalling lock info")?;
        std::fs::write(&path, body)
            .with_context(|| format!("writing lock file {}", path.display()))?;
        tracing::debug!(path = %path.display(), pid = info.pid, "lock acquired");
        Ok(Self { path })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        match std::fs::remove_file(&self.path) {
            Ok(_) => tracing::debug!(path = %self.path.display(), "lock released"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => tracing::error!(
                path = %self.path.display(),
                error = %e,
                "failed to remove lock file"
            ),
        }
    }
}

#[cfg(unix)]
fn is_process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // Signal 0 is the "is this PID alive?" probe — no signal is delivered.
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: i32) -> bool {
    true
}

fn next_boundary(now: DateTime<Utc>, interval: Duration) -> DateTime<Utc> {
    if interval.is_zero() {
        return now;
    }
    let midnight = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .unwrap_or(now);
    let elapsed_ns = (now - midnight)
        .num_nanoseconds()
        .unwrap_or(0)
        .max(0) as u128;
    let interval_ns = interval.as_nanos().max(1);
    let intervals = (elapsed_ns / interval_ns) as i32;
    let step = chrono::Duration::from_std(interval).unwrap_or_default();
    let mut next = midnight + step * (intervals + 1);
    if next <= now {
        next += step;
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_boundary_rounds_up_to_interval_grid() {
        let interval = Duration::from_secs(60 * 60);
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 3, 17, 0).unwrap();
        let next = next_boundary(now, interval);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 4, 26, 4, 0, 0).unwrap());
    }

    #[test]
    fn next_boundary_advances_when_on_grid() {
        let interval = Duration::from_secs(60 * 60);
        let now = Utc.with_ymd_and_hms(2026, 4, 26, 3, 0, 0).unwrap();
        let next = next_boundary(now, interval);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 4, 26, 4, 0, 0).unwrap());
    }
}
