//! The `VmBackend` trait: the single portability seam of ExR.
//!
//! M1 ships two implementations: [`crate::fake::FakeBackend`] (unit tests) and
//! [`crate::qemu::QemuBackend`] (real QEMU child process over QMP).
//! Windows (WHPX) and macOS (HVF) backends are trait-conformant stubs in later
//! milestones — the trait below is the contract they will implement.

use crate::error::VmError;
use emu_core::{VmSpec, VmState};
use std::path::Path;

/// Opaque handle to a launched VM. Backends resolve it to their own state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle {
    /// Instance name from the launching [`VmSpec`].
    pub name: String,
}

impl Handle {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Lifecycle + introspection contract every backend implements.
///
/// All methods are synchronous and blocking (M1 scope); async bindings arrive
/// when a second caller needs them. Implementations must be safe to call from
/// any thread.
pub trait VmBackend: Send + Sync {
    /// Launch `spec` (spare no validation: reject bad specs here, not in QEMU
    /// argv). Returns a handle; the VM is starting or running.
    fn launch(&self, spec: &VmSpec) -> Result<Handle, VmError>;

    /// Current state of `handle`.
    fn state(&self, handle: &Handle) -> Result<VmState, VmError>;

    /// Graceful stop: ACPI powerdown → wait → process kill escalation.
    /// Removes the runtime dir on success.
    fn stop(&self, handle: &Handle) -> Result<(), VmError>;

    /// Capture the guest framebuffer to `out_ppm` (QMP `screendump`, P6 PPM).
    fn screendump(&self, handle: &Handle, out_ppm: &Path) -> Result<(), VmError>;

    /// Tail (up to ~64 KiB) of the guest serial log (`-serial file:`).
    /// Used by the M1 boot proof and `doctor`-style diagnostics.
    fn serial_log(&self, handle: &Handle) -> Result<String, VmError>;
}
