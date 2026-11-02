//! Runtime configuration.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Daemon configuration, built from CLI flags.
#[derive(Debug, Clone)]
pub struct Config {
    /// Address the front server listens on.
    pub bind: SocketAddr,
    /// Root directory with one subdirectory per hosted domain.
    pub sites_root: PathBuf,
    /// Number of worker threads. Upstream uses one per customer, we use one per core.
    pub workers: usize,
    /// When set, reproduce upstream behaviour by answering every request with 502.
    pub upstream_compat_502: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8080".parse().expect("static addr"),
            sites_root: PathBuf::from("./sites"),
            workers: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            upstream_compat_502: false,
        }
    }
}

impl Config {
    /// Parse `--flag value` style arguments on top of the defaults.
    pub fn from_args<I: Iterator<Item = String>>(mut args: I) -> Result<Self, String> {
        let mut cfg = Self::default();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bind" => {
                    let v = args.next().ok_or("--bind needs a value")?;
                    cfg.bind = v.parse().map_err(|e| format!("bad --bind: {e}"))?;
                }
                "--sites" => {
                    cfg.sites_root = args.next().ok_or("--sites needs a value")?.into();
                }
                "--workers" => {
                    let v = args.next().ok_or("--workers needs a value")?;
                    cfg.workers = v.parse().map_err(|e| format!("bad --workers: {e}"))?;
                }
                "--upstream-compat-502" => cfg.upstream_compat_502 = true,
                other => return Err(format!("unknown flag: {other}")),
            }
        }
        Ok(cfg)
    }
}
