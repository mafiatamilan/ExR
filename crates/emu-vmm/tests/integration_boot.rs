//! M1 headless-boot integration test. IGNORED by default (needs KVM + ISO).
//!
//! Run: `EXR_TEST_ISO=~/.local/share/exr/images/android-x86_64-9.0-r2.iso \
//!        cargo test -p emu-vmm --test integration_boot -- --ignored --nocapture`
//!
//! Polls the framebuffer (Android sleeps its display when idle) and passes on
//! a lit frame plus an inter-capture change; always stops the VM afterwards.

use emu_vmm::{Boot, QemuBackend, VmBackend, VmSpec};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn iso_path() -> PathBuf {
    if let Some(p) = std::env::var_os("EXR_TEST_ISO") {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".local/share/exr/images/android-x86_64-9.0-r2.iso")
}

#[test]
#[ignore]
fn headless_boot_proof() {
    let iso = iso_path();
    assert!(iso.is_file(), "missing ISO: {}", iso.display());
    let imgdir = iso.parent().unwrap().to_path_buf();
    let mut spec = VmSpec::headless("m1test", iso, PathBuf::from("/usr/bin/qemu-system-x86_64"));
    spec.boot = Boot::DirectKernel {
        kernel: imgdir.join("kernel"),
        initrd: imgdir.join("initrd.img"),
        append: "root=/dev/ram0 console=tty0 console=ttyS0,115200 vga=791 SRC= DATA=".to_string(),
    };
    // Precondition: kernel/initrd must sit next to the ISO (see README).
    assert!(
        imgdir.join("kernel").is_file(),
        "missing kernel next to ISO"
    );
    assert!(
        imgdir.join("initrd.img").is_file(),
        "missing initrd next to ISO"
    );

    let backend = QemuBackend::new(None);
    let h = backend.launch(&spec).expect("launch");
    let shot = std::env::temp_dir().join("exr-it-shot.ppm");
    let deadline = Instant::now() + Duration::from_secs(300);
    let (mut saw_lit, mut saw_change) = (false, false);
    let mut prev: Option<emu_vmm::frame::Ppm> = None;
    while Instant::now() < deadline && !(saw_lit && saw_change) {
        std::thread::sleep(Duration::from_secs(10));
        if backend.screendump(&h, &shot).is_err() {
            continue;
        }
        let Ok(ppm) = emu_vmm::frame::parse_ppm(&shot) else {
            continue;
        };
        saw_lit |= emu_vmm::frame::is_nonblank(&ppm);
        if let Some(p) = prev {
            if emu_vmm::frame::changed_fraction(&p, &ppm).is_some_and(|c| c > 0.0) {
                saw_change = true;
            }
        }
        prev = Some(ppm);
    }
    let log = backend.serial_log(&h).unwrap_or_default();
    assert!(
        log.contains("Android-x86"),
        "serial shows no Android boot ({} bytes)",
        log.len()
    );
    backend.stop(&h).expect("stop");
    let _ = std::fs::remove_file(&shot);
    assert!(saw_lit, "never saw a lit frame");
    assert!(saw_change, "framebuffer never changed");
}
