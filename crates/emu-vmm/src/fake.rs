//! In-memory backend for unit tests. No processes, no sockets.

use crate::backend::{Handle, VmBackend};
use crate::error::VmError;
use emu_core::{VmSpec, VmState};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// State machine: launch → Running, stop → removed (subsequent calls fail
/// with [`VmError::NotRunning`], mirroring the real backend's cleanup).
#[derive(Debug, Default)]
pub struct FakeBackend {
    states: Mutex<HashMap<String, VmState>>,
}

impl FakeBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VmBackend for FakeBackend {
    fn launch(&self, spec: &VmSpec) -> Result<Handle, VmError> {
        if spec.name.is_empty() {
            return Err(VmError::InvalidSpec("empty instance name".into()));
        }
        if spec.cpus == 0 || spec.ram_mb == 0 {
            return Err(VmError::InvalidSpec("cpus and ram must be nonzero".into()));
        }
        let mut states = self.states.lock().unwrap();
        if states.contains_key(&spec.name) {
            return Err(VmError::AlreadyRunning(spec.name.clone()));
        }
        states.insert(spec.name.clone(), VmState::Running);
        Ok(Handle::new(&spec.name))
    }

    fn state(&self, handle: &Handle) -> Result<VmState, VmError> {
        self.states
            .lock()
            .unwrap()
            .get(&handle.name)
            .copied()
            .ok_or_else(|| VmError::NotRunning(handle.name.clone()))
    }

    fn stop(&self, handle: &Handle) -> Result<(), VmError> {
        let mut states = self.states.lock().unwrap();
        states
            .remove(&handle.name)
            .map(|_| ())
            .ok_or_else(|| VmError::NotRunning(handle.name.clone()))
    }

    fn screendump(&self, handle: &Handle, out_ppm: &Path) -> Result<(), VmError> {
        if !self.states.lock().unwrap().contains_key(&handle.name) {
            return Err(VmError::NotRunning(handle.name.clone()));
        } // Deterministic 8x8 test pattern (white diagonal on black): valid P6,
          // non-blank, stable across calls — enough to exercise frame.rs end to end.
        let mut bytes = b"P6\n8 8\n255\n".to_vec();
        for y in 0..8u8 {
            for x in 0..8u8 {
                let v = if x == y { 255u8 } else { 0u8 };
                bytes.extend_from_slice(&[v, v, v]);
            }
        }
        std::fs::write(out_ppm, &bytes).map_err(|e| VmError::Io {
            ctx: "fake screendump write",
            source: e,
        })?;
        Ok(())
    }

    fn serial_log(&self, handle: &Handle) -> Result<String, VmError> {
        if !self.states.lock().unwrap().contains_key(&handle.name) {
            return Err(VmError::NotRunning(handle.name.clone()));
        }
        Ok("[fake serial: guest says hello]\n".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame;
    use emu_core::Accel;

    fn spec(name: &str) -> VmSpec {
        VmSpec {
            name: name.into(),
            boot_iso: "/nonexistent.iso".into(),
            boot: crate::Boot::Iso,
            cpus: 2,
            ram_mb: 2048,
            accel: Accel::Tcg,
            display: emu_core::DisplayMode::Headless,
            qemu_bin: "/nonexistent/qemu".into(),
        }
    }

    #[test]
    fn lifecycle() {
        let b = FakeBackend::new();
        let h = b.launch(&spec("a")).unwrap();
        assert_eq!(b.state(&h).unwrap(), VmState::Running);
        b.stop(&h).unwrap();
        assert!(matches!(b.state(&h), Err(VmError::NotRunning(_))));
    }

    #[test]
    fn double_launch_rejected() {
        let b = FakeBackend::new();
        b.launch(&spec("a")).unwrap();
        assert!(matches!(
            b.launch(&spec("a")),
            Err(VmError::AlreadyRunning(_))
        ));
    }

    #[test]
    fn stop_unknown_rejected() {
        let b = FakeBackend::new();
        assert!(matches!(
            b.stop(&Handle::new("ghost")),
            Err(VmError::NotRunning(_))
        ));
    }

    #[test]
    fn bad_spec_rejected() {
        let b = FakeBackend::new();
        let mut s = spec("");
        assert!(matches!(b.launch(&s), Err(VmError::InvalidSpec(_))));
        s = spec("x");
        s.cpus = 0;
        assert!(matches!(b.launch(&s), Err(VmError::InvalidSpec(_))));
    }

    #[test]
    fn fake_screendump_is_valid_nonblank_ppm() {
        let b = FakeBackend::new();
        let h = b.launch(&spec("a")).unwrap();
        let p = std::env::temp_dir().join("exr-fake-dump.ppm");
        b.screendump(&h, &p).unwrap();
        let ppm = frame::parse_ppm(&p).unwrap();
        assert_eq!((ppm.width, ppm.height), (8, 8));
        assert!(frame::is_nonblank(&ppm));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn fake_serial_log_present_while_running() {
        let b = FakeBackend::new();
        let h = b.launch(&spec("a")).unwrap();
        assert!(!b.serial_log(&h).unwrap().is_empty());
        b.stop(&h).unwrap();
        assert!(matches!(b.serial_log(&h), Err(VmError::NotRunning(_))));
    }
}
