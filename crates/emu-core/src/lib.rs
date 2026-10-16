//! Shared kernel types for the ExR workspace.
//!
//! M1 scope: the minimum vocabulary `emu-vmm` needs (machine spec, accelerator
//! selection, display mode, instance identity). Everything else arrives with its
//! owning crate in later milestones.

use std::path::PathBuf;

/// Which host accelerator to use. `Auto` resolves per host OS at runtime
/// (Linux: KVM, Windows: WHPX, macOS: HVF, else TCG); explicit variants exist
/// for `doctor` output and forcing fallbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Accel {
    #[default]
    Auto,
    Kvm,
    Tcg,
    Whpx,
    Hvf,
}

/// Display attachment for a VM launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// No display device output (`-display none`). M1 headless proof.
    #[default]
    Headless,
}

/// How the guest boots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Boot {
    /// Boot the attached ISO through the guest bootloader (SeaBIOS/grub menu).
    Iso,
    /// Direct kernel boot: `-kernel/-initrd/-append`, bypassing any boot menu.
    /// The ISO stays attached as cdrom so the init script finds `system.sfs`.
    /// M1 uses this because the Android-x86 menu does not reliably self-boot
    /// headless; the kernel cmdline is taken verbatim from the ISO's own
    /// `isolinux.cfg` (`livem` entry).
    DirectKernel {
        kernel: PathBuf,
        initrd: PathBuf,
        append: String,
    },
}

/// Machine specification for one VM launch.
#[derive(Debug, Clone)]
pub struct VmSpec {
    /// Instance name (also scopes the runtime dir).
    pub name: String,
    /// Path to the boot ISO (attached as cdrom in both boot modes).
    pub boot_iso: PathBuf,
    /// How the guest boots (menu vs direct kernel).
    pub boot: Boot,
    /// vCPU count.
    pub cpus: u32,
    /// Guest RAM in MiB.
    pub ram_mb: u64,
    /// Accelerator selection.
    pub accel: Accel,
    /// Display attachment.
    pub display: DisplayMode,
    /// QEMU binary (host-provided, never bundled).
    pub qemu_bin: PathBuf,
}

impl VmSpec {
    /// Conservative M1 defaults: 2 vCPU, 2 GiB, KVM-on-Linux auto.
    pub fn headless(name: impl Into<String>, boot_iso: PathBuf, qemu_bin: PathBuf) -> Self {
        Self {
            name: name.into(),
            boot_iso,
            boot: Boot::Iso,
            cpus: 2,
            ram_mb: 2048,
            accel: Accel::Auto,
            display: DisplayMode::Headless,
            qemu_bin,
        }
    }
}

/// Runtime state of a VM handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Starting,
    Running,
    Stopped,
}
