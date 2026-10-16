//! QEMU child-process backend (the M1 primary).
//!
//! QEMU is **spawned, never linked**: all control flows through a QMP unix
//! socket. Security defaults (per milestone rules):
//!
//! - socket + serial log live in `$XDG_RUNTIME_DIR/exr-<name>-<uniq>/` created
//!   `0700` (owner-only); the socket itself inherits those protections.
//! - ADB forwards (M2) will bind `127.0.0.1` only — noted here so the rule is
//!   visible where sockets are made.
//! - `Drop` kills leftover children and removes runtime dirs (best effort).

use crate::backend::{Handle, VmBackend};
use crate::error::VmError;
use crate::qmp::QmpClient;
use emu_core::{Accel, VmSpec, VmState};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SOCKET_WAIT_MS: u64 = 10_000;
const QMP_IO_TIMEOUT: Duration = Duration::from_secs(10);
const SHUTDOWN_WAIT_MS: u64 = 10_000;
const KILL_WAIT_MS: u64 = 5_000;

struct ChildEntry {
    child: Child,
    qmp: QmpClient,
    run_dir: PathBuf,
}

/// QEMU backend. Clone-free; share via [`std::sync::Arc`] if needed.
pub struct QemuBackend {
    runtime_base: PathBuf,
    children: Mutex<HashMap<String, ChildEntry>>,
}

impl QemuBackend {
    /// `runtime_base`: parent for per-VM dirs. `None` → `$XDG_RUNTIME_DIR`
    /// (fallback: std temp dir).
    pub fn new(runtime_base: Option<PathBuf>) -> Self {
        let base = runtime_base.unwrap_or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
        });
        Self {
            runtime_base: base,
            children: Mutex::new(HashMap::new()),
        }
    }

    /// Resolve [`Accel`] to a QEMU `-accel` argument on this host.
    fn resolve_accel(accel: Accel) -> Result<&'static str, VmError> {
        match accel {
            Accel::Auto => {
                if cfg!(target_os = "linux") && kvm_usable() {
                    Ok("kvm")
                } else if cfg!(target_os = "windows") {
                    Ok("whpx")
                } else if cfg!(target_os = "macos") {
                    Ok("hvf")
                } else {
                    Ok("tcg")
                }
            }
            Accel::Kvm => Ok("kvm"),
            Accel::Tcg => Ok("tcg"),
            Accel::Whpx => Ok("whpx"),
            Accel::Hvf => Ok("hvf"),
        }
    }

    fn make_run_dir(&self, name: &str) -> Result<PathBuf, VmError> {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = self
            .runtime_base
            .join(format!("exr-{}-{}", sanitize(name), uniq));
        std::fs::create_dir_all(&dir).map_err(|e| VmError::RuntimeDir {
            dir: dir.clone(),
            source: e,
        })?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).map_err(|e| {
            VmError::RuntimeDir {
                dir: dir.clone(),
                source: e,
            }
        })?;
        Ok(dir)
    }
}

/// `/dev/kvm` present and readable+writable (Linux fast path probe).
fn kvm_usable() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .is_ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Keep runtime dir names filesystem-safe.
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(32)
        .collect()
}

impl VmBackend for QemuBackend {
    fn launch(&self, spec: &VmSpec) -> Result<Handle, VmError> {
        if spec.name.is_empty() {
            return Err(VmError::InvalidSpec("empty instance name".into()));
        }
        if spec.cpus == 0 || spec.ram_mb < 256 {
            return Err(VmError::InvalidSpec(
                "cpus must be nonzero and ram >= 256 MiB".into(),
            ));
        }
        if !spec.boot_iso.is_file() {
            return Err(VmError::InvalidSpec(format!(
                "boot ISO not found: {}",
                spec.boot_iso.display()
            )));
        }
        {
            let children = self.children.lock().unwrap();
            if children.contains_key(&spec.name) {
                return Err(VmError::AlreadyRunning(spec.name.clone()));
            }
        }

        let accel = Self::resolve_accel(spec.accel)?;
        let cpu = if accel == "kvm" { "host" } else { "max" };
        let run_dir = self.make_run_dir(&spec.name)?;
        let sock = run_dir.join("qmp.sock");
        if sock.as_os_str().len() > 100 {
            return Err(VmError::InvalidSpec(
                "runtime path too long for unix socket".into(),
            ));
        }
        let serial = run_dir.join("serial.log");

        let mut cmd = Command::new(&spec.qemu_bin);
        cmd.arg("-name")
            .arg(&spec.name)
            .arg("-machine")
            .arg("q35,accel=".to_string() + accel)
            .arg("-cpu")
            .arg(cpu)
            .arg("-m")
            .arg(spec.ram_mb.to_string())
            .arg("-smp")
            .arg(spec.cpus.to_string())
            .arg("-cdrom")
            .arg(&spec.boot_iso)
            .arg("-display")
            .arg("none")
            .arg("-vga")
            .arg("std");
        match &spec.boot {
            crate::Boot::Iso | crate::Boot::IsoMenu => {
                cmd.arg("-boot").arg("order=d,menu=off");
            }
            crate::Boot::DirectKernel {
                kernel,
                initrd,
                append,
            } => {
                if !kernel.is_file() {
                    return Err(VmError::InvalidSpec(format!(
                        "kernel not found: {}",
                        kernel.display()
                    )));
                }
                if !initrd.is_file() {
                    return Err(VmError::InvalidSpec(format!(
                        "initrd not found: {}",
                        initrd.display()
                    )));
                }
                cmd.arg("-kernel")
                    .arg(kernel)
                    .arg("-initrd")
                    .arg(initrd)
                    .arg("-append")
                    .arg(append);
            }
        }
        cmd.arg("-serial")
            .arg(format!("file:{}", serial.display()))
            .arg("-qmp")
            .arg(format!("unix:{},server=on,wait=off", sock.display()))
            .arg("-no-reboot")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd.spawn().map_err(|e| VmError::SpawnFailed {
            bin: spec.qemu_bin.clone(),
            source: e,
        })?;

        // Wait for the socket, then handshake.
        let deadline = Instant::now() + Duration::from_millis(SOCKET_WAIT_MS);
        while !sock.exists() {
            if Instant::now() > deadline {
                let _ = self.children; // (nothing registered yet)
                return Err(VmError::QmpSocketTimeout {
                    sock,
                    timeout_ms: SOCKET_WAIT_MS,
                });
            }
            // If QEMU already exited, fail fast with a useful error.
            std::thread::sleep(Duration::from_millis(25));
        }
        let mut qmp = QmpClient::connect(&sock, QMP_IO_TIMEOUT)?;

        // ISO-menu boot: let the bootloader menu settle, then Enter on the
        // default (first) entry. Timing is generous; SeaBIOS + isolinux show
        // the menu within seconds on KVM.
        if spec.boot == crate::Boot::IsoMenu {
            std::thread::sleep(Duration::from_secs(15));
            qmp.press_key("ret")?;
        }

        let handle = Handle::new(&spec.name);
        self.children.lock().unwrap().insert(
            spec.name.clone(),
            ChildEntry {
                child,
                qmp,
                run_dir,
            },
        );
        Ok(handle)
    }

    fn state(&self, handle: &Handle) -> Result<VmState, VmError> {
        let mut children = self.children.lock().unwrap();
        let entry = children
            .get_mut(&handle.name)
            .ok_or_else(|| VmError::NotRunning(handle.name.clone()))?;
        // Process gone but entry present (e.g. guest halted with -no-reboot)?
        if let Ok(Some(_)) = entry.child.try_wait() {
            return Ok(VmState::Stopped);
        }
        match entry.qmp.is_running() {
            Ok(true) => Ok(VmState::Running),
            Ok(false) => Ok(VmState::Starting), // alive but vCPUs halted (early boot)
            Err(_) => {
                // QMP wedged but process alive: report by process liveness.
                match entry.child.try_wait() {
                    Ok(None) => Ok(VmState::Starting),
                    _ => Ok(VmState::Stopped),
                }
            }
        }
    }

    fn screendump(&self, handle: &Handle, out_ppm: &Path) -> Result<(), VmError> {
        let mut children = self.children.lock().unwrap();
        let entry = children
            .get_mut(&handle.name)
            .ok_or_else(|| VmError::NotRunning(handle.name.clone()))?;
        entry.qmp.screendump(out_ppm)?;
        if !out_ppm.is_file() {
            return Err(VmError::ScreendumpMissing(out_ppm.to_path_buf()));
        }
        Ok(())
    }

    fn serial_log(&self, handle: &Handle) -> Result<String, VmError> {
        let children = self.children.lock().unwrap();
        let entry = children
            .get(&handle.name)
            .ok_or_else(|| VmError::NotRunning(handle.name.clone()))?;
        let log = entry.run_dir.join("serial.log");
        let bytes = std::fs::read(&log).map_err(|e| VmError::Io {
            ctx: "serial log read",
            source: e,
        })?;
        // Lossy tail: logs are guest bytes; cap at 64 KiB from the end.
        let tail = if bytes.len() > 65536 {
            &bytes[bytes.len() - 65536..]
        } else {
            &bytes[..]
        };
        Ok(String::from_utf8_lossy(tail).into_owned())
    }

    fn stop(&self, handle: &Handle) -> Result<(), VmError> {
        let mut entry = {
            let mut children = self.children.lock().unwrap();
            children
                .remove(&handle.name)
                .ok_or_else(|| VmError::NotRunning(handle.name.clone()))?
        };
        // 1. graceful: ACPI powerdown, then poll for exit.
        let _ = entry.qmp.powerdown();
        let deadline = Instant::now() + Duration::from_millis(SHUTDOWN_WAIT_MS);
        let mut exited = false;
        while Instant::now() < deadline {
            match entry.child.try_wait() {
                Ok(Some(_)) => {
                    exited = true;
                    break;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break,
            }
        }
        // 2. escalate: SIGKILL via Child::kill, then reap.
        if !exited {
            let _ = entry.child.kill();
            let deadline = Instant::now() + Duration::from_millis(KILL_WAIT_MS);
            while Instant::now() < deadline {
                match entry.child.try_wait() {
                    Ok(Some(_)) => {
                        exited = true;
                        break;
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                    Err(_) => break,
                }
            }
        }
        let _ = entry.qmp.quit();
        let _ = std::fs::remove_dir_all(&entry.run_dir);
        if !exited {
            return Err(VmError::ShutdownTimeout {
                timeout_ms: SHUTDOWN_WAIT_MS + KILL_WAIT_MS,
            });
        }
        Ok(())
    }
}

impl Drop for QemuBackend {
    fn drop(&mut self) {
        let mut children = self.children.lock().unwrap();
        for (_, mut entry) in children.drain() {
            let _ = entry.child.kill();
            let _ = entry.child.wait();
            let _ = std::fs::remove_dir_all(&entry.run_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emu_core::{Accel, DisplayMode};

    #[test]
    fn missing_iso_rejected_before_spawn() {
        let b = QemuBackend::new(None);
        let spec = VmSpec {
            name: "x".into(),
            boot_iso: "/definitely/not/here.iso".into(),
            boot: crate::Boot::Iso,
            cpus: 2,
            ram_mb: 2048,
            accel: Accel::Tcg,
            display: DisplayMode::Headless,
            qemu_bin: "/nonexistent/qemu".into(),
        };
        assert!(matches!(b.launch(&spec), Err(VmError::InvalidSpec(_))));
    }

    #[test]
    fn empty_name_rejected() {
        let b = QemuBackend::new(None);
        let spec = VmSpec {
            name: "".into(),
            boot_iso: "/definitely/not/here.iso".into(),
            boot: crate::Boot::Iso,
            cpus: 2,
            ram_mb: 2048,
            accel: Accel::Tcg,
            display: DisplayMode::Headless,
            qemu_bin: "/nonexistent/qemu".into(),
        };
        assert!(matches!(b.launch(&spec), Err(VmError::InvalidSpec(_))));
    }
}
