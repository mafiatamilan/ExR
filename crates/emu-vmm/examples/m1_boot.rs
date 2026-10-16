//! M1 demo: headless boot proof against a real Android-x86 ISO.
//!
//! Run: `cargo run -p emu-vmm --example m1_boot -- --iso <path> [--name N]
//! [--wait-secs S] [--append "..."] [--iso-boot]`.
//!
//! Proof strategy: Android sleeps its display when idle, so one late capture
//! can be legitimately black. The demo polls the framebuffer every 10 s and
//! passes when it observes a lit frame AND a change between consecutive
//! captures inside the window. Serial log tail is printed as boot evidence.

use emu_vmm::{QemuBackend, VmBackend, VmSpec};
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn arg(flag: &str) -> Option<String> {
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == flag {
            return it.next();
        }
        if let Some(v) = a.strip_prefix(&format!("{flag}=")) {
            return Some(v.to_string());
        }
    }
    None
}

fn main() -> Result<(), String> {
    let iso = arg("--iso").ok_or("--iso <path-to-android-x86.iso> required")?;
    let name = arg("--name").unwrap_or_else(|| "m1demo".to_string());
    let wait_secs: u64 = arg("--wait-secs")
        .and_then(|s| s.parse().ok())
        .unwrap_or(240);
    let qemu_bin = std::env::var_os("EXR_QEMU_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/bin/qemu-system-x86_64"));
    let mut spec = VmSpec::headless(name, PathBuf::from(iso), qemu_bin);
    if let Some(ram) = arg("--ram-mb").and_then(|s| s.parse::<u64>().ok()) {
        spec.ram_mb = ram;
    }
    // Direct kernel boot by default (deterministic headless path; the ISO menu
    // does not reliably self-boot). Pass --iso-boot to use menu boot instead.
    // Kernel cmdline is verbatim from the ISO's isolinux.cfg `livem` entry,
    // plus dual consoles (VGA text + serial capture) and a VESA mode so the
    // framebuffer exists under direct boot.
    if arg("--iso-boot").is_some() {
        spec.boot = emu_vmm::Boot::IsoMenu;
    } else {
        let imgdir = std::env::var_os("EXR_IMAGE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default();
                home.join(".local/share/exr/images")
            });
        spec.boot = emu_vmm::Boot::DirectKernel {
            kernel: imgdir.join("kernel"),
            initrd: imgdir.join("initrd.img"),
            append: arg("--append").unwrap_or_else(|| {
                "root=/dev/ram0 console=tty0 console=ttyS0,115200 vga=791 SRC= DATA=".to_string()
            }),
        };
    }
    let backend = QemuBackend::new(None);

    println!("[m1] launching...");
    let h = backend.launch(&spec).map_err(|e| format!("launch: {e}"))?;
    println!("[m1] launched; polling framebuffer for up to {wait_secs}s...");

    let shot = std::env::temp_dir().join("exr-m1-shot.ppm");
    let deadline = Instant::now() + Duration::from_secs(wait_secs);
    let mut prev: Option<emu_vmm::frame::Ppm> = None;
    let (mut saw_lit, mut saw_change, mut captures) = (false, false, 0u32);
    while Instant::now() < deadline && !(saw_lit && saw_change) {
        std::thread::sleep(Duration::from_secs(10));
        if backend.screendump(&h, &shot).is_err() {
            continue;
        }
        let Ok(ppm) = emu_vmm::frame::parse_ppm(&shot) else {
            continue;
        };
        captures += 1;
        let lit = emu_vmm::frame::is_nonblank(&ppm);
        let kind = emu_vmm::frame::classify(&ppm);
        saw_lit |= lit;
        if let Some(p) = prev {
            if emu_vmm::frame::changed_fraction(&p, &ppm).is_some_and(|c| c > 0.0) {
                saw_change = true;
            }
        }
        prev = Some(ppm);
        let left = deadline.saturating_duration_since(Instant::now()).as_secs();
        println!("[m1] capture #{captures} lit={lit} kind={kind:?} ({left}s left)");
    }

    match backend.serial_log(&h) {
        Ok(log) => {
            // Full log to /tmp for analysis (outside the repo); tail on stdout.
            let _ = std::fs::write("/tmp/exr-m1-serial.log", &log);
            let lines: Vec<&str> = log.lines().collect();
            let tail = lines.iter().rev().take(60).rev();
            println!(
                "[m1] serial: {} bytes, {} lines (full: /tmp/exr-m1-serial.log):",
                log.len(),
                lines.len()
            );
            for l in tail {
                println!("  | {l}");
            }
        }
        Err(e) => println!("[m1] serial unavailable: {e}"),
    }
    println!("[m1] stopping...");
    backend.stop(&h).map_err(|e| format!("stop: {e}"))?;
    if saw_lit && saw_change {
        println!("[m1] PASS: lit frame observed and framebuffer changed over time");
        Ok(())
    } else {
        Err(format!(
            "FAIL: lit={saw_lit} changed={saw_change} after {captures} captures"
        ))
    }
}
