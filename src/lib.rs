//! # beget-rs
//!
//! Clean-room reimplementation of the Beget shared hosting stack.
//!
//! The crate is split into the same subsystems the upstream platform exposes:
//!
//! - [`http`]: the front web server (vhosts, static files, 502 emulation)
//! - [`panel`]: the control panel JSON API, wire-compatible with `api.beget.com`
//! - [`begetception`]: recursive self-hosting (Beget inside Beget inside Beget)
//! - [`uptime`]: SLA accounting
//! - [`php`]: legacy PHP 5.6 compatibility shim
//!
//! Everything is `#![forbid(unsafe_code)]` except the PHP shim, for obvious reasons.

#![deny(missing_docs)]

pub mod begetception;
pub mod config;
pub mod http;
pub mod panel;
pub mod php;
pub mod uptime;
pub mod vhost;

/// Version string reported in the `Server:` header.
pub const SERVER_NAME: &str = concat!("beget-rs/", env!("CARGO_PKG_VERSION"));
