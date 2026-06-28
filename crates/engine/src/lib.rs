//! Forgum engine library.
//!
//! The actual binary lives in `src/main.rs`. This library exposes the same
//! API surface so tests (and future embedders like the herder) can use the
//! engine without spawning a process.
//!
//! **Zero `#[cfg]` lives in this crate.** All platform branching is
//! delegated to `forgum-platform`. CI greps `engine/src/` and fails on hits.

pub mod cli;
pub mod config;
pub mod effects;
pub mod framebuffer;
pub mod protocol;
pub mod protocol_io;
pub mod render;
pub mod scheduler;

/// The current engine version, derived from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
