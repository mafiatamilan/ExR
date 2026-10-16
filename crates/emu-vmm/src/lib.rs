//! `emu-vmm`: QEMU-child-process VM backend for ExR.
//!
//! - [`backend::VmBackend`] is the portability seam (QEMU now; WHPX/HVF later).
//! - [`qemu::QemuBackend`] spawns QEMU as a child process and drives it over a
//!   QMP unix socket. QEMU is never linked.
//! - [`fake::FakeBackend`] is an in-memory double for unit tests.
//! - [`frame`] parses QMP `screendump` PPMs for the headless boot proof.
//!
//! Security defaults: runtime dirs `0700` under `$XDG_RUNTIME_DIR`, sockets
//! inherited-owner-only, graceful-shutdown escalation, full cleanup on
//! stop/`Drop`. ADB forwards (M2) bind `127.0.0.1` only.

pub mod backend;
pub mod error;
pub mod fake;
pub mod frame;
pub mod qemu;
pub mod qmp;

pub use backend::{Handle, VmBackend};
pub use emu_core::{Accel, Boot, DisplayMode, VmSpec, VmState};
pub use error::VmError;
pub use fake::FakeBackend;
pub use qemu::QemuBackend;
