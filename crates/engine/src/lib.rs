//! Forgum engine library.
//!
//! The actual binary lives in `src/main.rs`. This library exposes the same
//! API surface so tests (and future embedders like the herder) can use the
//! engine without spawning a process.
//!
//! **Zero `#[cfg]` lives in this crate.** All platform branching is
//! delegated to `forgum-platform`. CI greps `engine/src/` and fails on hits.

pub mod battle;
pub mod cli;
pub mod color;
pub mod completions;
pub mod config;
pub mod config_tui;
pub mod control_socket;
pub mod cow;
pub mod daemon;
pub mod demo;
pub mod dna;
pub mod easing;
pub mod effects;
pub mod fortune;
pub mod framebuffer;
pub mod herd;
pub mod init;
pub mod metrics;
pub mod particles;
pub mod protocol;
pub mod protocol_io;
pub mod remote;
pub mod render;
pub mod renderer;
pub mod say;
pub mod scheduler;
pub mod shader;
pub mod showcase;
pub mod status_line;
pub mod theme;
pub mod timer;
pub mod verlet;

/// The current engine version, derived from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
