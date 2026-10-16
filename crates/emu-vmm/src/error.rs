//! Typed errors for `emu-vmm`.
//!
//! Every failure mode is a distinct variant so automation (and `doctor`-style
//! CLIs later) can match on causes instead of parsing strings.

use std::path::PathBuf;
use thiserror::Error;

/// All errors produced by this crate.
#[derive(Debug, Error)]
pub enum VmError {
    /// The VM spec itself is unusable (missing ISO, bad geometry, ...).
    #[error("invalid VM spec: {0}")]
    InvalidSpec(String),

    /// Spawning the QEMU child process failed.
    #[error("failed to spawn qemu at {bin:?}: {source}")]
    SpawnFailed {
        bin: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Setting up the runtime dir / socket failed.
    #[error("runtime dir setup failed at {dir:?}: {source}")]
    RuntimeDir {
        dir: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// The QMP socket never appeared after spawn.
    #[error("QMP socket {sock:?} did not appear within {timeout_ms} ms")]
    QmpSocketTimeout { sock: PathBuf, timeout_ms: u64 },

    /// QMP greeting / capabilities handshake failed.
    #[error("QMP handshake failed: {0}")]
    QmpHandshake(String),

    /// A QMP command returned an error object.
    #[error("QMP command `{cmd}` failed: {detail}")]
    QmpCommand { cmd: String, detail: String },

    /// QMP transport broke mid-session.
    #[error("QMP transport error: {0}")]
    QmpTransport(String),

    /// Graceful shutdown did not complete in time (caller killed the child).
    #[error("shutdown timed out after {timeout_ms} ms; child killed")]
    ShutdownTimeout { timeout_ms: u64 },

    /// An operation needs a running VM but found none.
    #[error("no such running VM: {0}")]
    NotRunning(String),

    /// A VM with this name is already running.
    #[error("VM already running: {0}")]
    AlreadyRunning(String),

    /// The requested screendump file never materialised.
    #[error("screendump file missing at {0:?}")]
    ScreendumpMissing(PathBuf),

    /// IO error with context.
    #[error("{ctx}: {source}")]
    Io {
        ctx: &'static str,
        #[source]
        source: std::io::Error,
    },
}
