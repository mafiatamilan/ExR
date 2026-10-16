# VMM options (Phase B0 research)

Date: 2026-10-16. All external claims carry URL + verified/unverified.
Linux-first (Arch/KVM). Windows/macOS are trait targets, not built in v1.

## Candidates

### 1. QEMU (system emulation, spawned + driven over QMP)

- Licence: emulator as a whole GPLv2 (parts vary, all GPLv2-compatible; TCG mostly
  BSD/MIT). Source: https://gitlab.com/qemu-project/qemu/-/raw/master/LICENSE — VERIFIED.
- Accel per host OS: KVM/Linux, hvf/macOS, whpx/Windows, TCG fallback everywhere.
  Source: https://www.qemu.org/docs/master/system/introduction.html (docs v11.1.50) — VERIFIED.
- GPU options: virtio devices incl. virtio-gpu; 3D via host backends (see
  `gpu-and-display.md`). Block layer supports snapshots/migration. Same source — VERIFIED.
- Android guest support: QEMU boots x86/x86_64 Android images (Android-x86 project
  documents QEMU among run options — UNVERIFIED detail, re-check before M1;
  pointer: https://www.android-x86.org/documentation/how-to-run-android-x86-in-qemu.html).
- Rust driving: QMP is a versioned JSON machine API over a socket, used by libvirt-class
  tooling. Same qemu.org source — VERIFIED. Implementation: spawn `qemu-system-x86_64`
  as a child process, speak QMP (serde_json) for lifecycle/snapshots; no FFI needed.
  QMP client crates exist on crates.io (evaluate in M1; none pinned here — UNVERIFIED).
- Maturity: highest (decades, libvirt ecosystem). Effort: LOW-MEDIUM (process spawn +
  QMP client + config builder).

### 2. crosvm

- Licence: BSD-3-Clause. Source: https://github.com/google/crosvm — VERIFIED.
- Accel: Linux/Android KVM (+Gunyah/GenieZone/Halla); Windows WHPX (+HAXM). No macOS
  hypervisor listed. Same source — VERIFIED.
- GPU: virtio-gpu 2D/3D via virglrenderer, gfxstream, or vulkano. Same source — VERIFIED.
- Android guests: designed for them (ARCVM, Cuttlefish, Android TerminalApp).
  Same source — VERIFIED.
- Rust driving: crosvm IS Rust; driving = library/CLI integration, but its device model
  targets ChromeOS/Android flows (Minijail/seccomp sandboxing, cros_async runtime) rather
  than a gaming-emulator control plane. Architectural fit: MEDIUM. Effort: MEDIUM
  (vendoring/integrating a second large Rust workspace + its build).
- Maturity: high inside Google/ChromeOS; smaller external operator community.

### 3. Custom rust-vmm build

- Licence of crates: suggested Apache-2.0 OR BSD-3-Clause (per-repo). Source:
  https://github.com/rust-vmm/community — VERIFIED. Available pieces (VERIFIED, same
  source): kvm-ioctls, vm-memory, linux-loader, virtio-queue/devices, vhost,
  seccompiler, vm-allocator, vmm-reference; Hyper-V bindings exist (`mshv`) but the
  ecosystem is KVM/Linux-first.
- Accel: KVM only for a from-scratch build (macOS/Windows would need new backends).
- GPU: no virtio-gpu/GPU story in the component set — must be built or borrowed.
- Android guests: possible (Firecracker/Cloud-Hypervisor lineage) but no Android
  gaming-emulator precedent with display/input/ADB integration.
- Effort: HIGH (a VMM is a product, not a crate). Rejected as primary.

### 4. VirtualBox via VBoxManage

- Licence: base package OSE under GPLv3; Extension Pack under a proprietary licence
  (PUEL-family). UNVERIFIED (not fetched; pointer: https://www.virtualbox.org/wiki/Licensing_FAQ)
  — re-verify before any use. GPLv3 + proprietary-pack split is awkward for a
  permissive-licensed Rust project that wants to bundle or automate installs.
- Accel: its own hypervisor per host OS (KVM-independent). Version skew across user
  machines; headless control via CLI/XML scraping rather than a versioned API.
- Fit vs evidence: closest to the observed baseline family (see `docs/spec/comparison.md`),
  but API, licensing, and per-host kernel-module friction make it the worst primary.
  Effort: LOW to prototype, HIGH to productize cross-platform.

## Recommendation

- PRIMARY: QEMU, driven as a child process over QMP from Rust. Reasons: GPLv2 is
  process-isolated (no linking; licence contained — see `licensing-register.md`), all
  three host accels exist behind one interface, virtio-gpu + snapshot story is upstream,
  and effort is lowest for a Linux-first M1.
- SECONDARY: crosvm. Reasons: BSD-3-Clause, Rust-native, Android-first device model;
  revisit if QEMU's process/API weight hurts, or for a hardened/sandboxed edition.
  Not primary because its ChromeOS-centric architecture and build system cost more
  than QMP-driving QEMU.
- REJECTED for v1: custom rust-vmm build (effort, no GPU story), VirtualBox driving
  (licence split, CLI-scraping control plane, host-module friction).

## M1 validated flags (QEMU 11.1.2, Arch/KVM, Android-x86 9.0-r2)

Proven working 2026-10-16 (headless boot proof in ~30 s wall):

```text
-machine q35,accel=kvm -cpu host -m 2048 -smp 2
-cdrom <iso> -kernel <kernel> -initrd <initrd.img>
-append "root=/dev/ram0 console=tty0 console=ttyS0,115200 vga=791 SRC= DATA="
-display none -vga std -serial file:<run>/serial.log
-qmp unix:<run>/qmp.sock,server=on,wait=off -no-reboot
```

Notes: direct kernel boot (the ISO's isolinux menu does not self-boot headless —
stuck on the orange menu, verified by screendump); kernel cmdline is the ISO's own
`livem` entry plus dual consoles and `vga=791` (without it the direct-boot
framebuffer never lights); QMP `input-send-event` on this QEMU carries
key/btn/rel/abs/**mtt** cases (probed via `query-qmp-schema`) — multitouch
transport confirmed present for M4.

## M1b findings (2026-10-16): Android userspace proven

ISO boot entries used verbatim (`isolinux.cfg`): live =
`initrd=/initrd.img root=/dev/ram0 quiet SRC= DATA=`; debug adds `DEBUG=2`;
software-rendering (`vesa`) = `nomodeset vga=ask` — `vga=ask` is interactive and
`nomodeset` removes the framebuffer the proof needs, so the vesa entry was NOT
attempted physically (recorded rationale, not an oversight).

Serial analysis (349-line boot log; last-60 are kernel driver chatter with no
errors — ata/sdhci/usbhid/e1000e/input, then):
`Run /init as init process` → `Detecting Android-x86... found at /dev/sr0` →
a root shell prompt. The initramfs `init` script ends in an emergency shell loop
*only if* `exec switch_root` fails — but interactive probing (pty serial, shell
is pre-existing, not injected) proved the opposite: `/proc/1/cmdline` is `/init`,
`/proc` holds 1000+ PIDs, mounts show the full Android world (cgroups, FUSE
emulated storage), and `getprop sys.boot_completed` returns `1`. So switch_root
SUCCEEDED; the coexisting root console shell comes from the live image itself
(Android-side console service, exact unit unconfirmed — does not affect acceptance).

PRIMARY markers (exact strings from the kernel log, first at t≈1.63 s):
`init: init first stage started!`, `init: Using Android DT directory
/proc/device-tree/firmware/android/`, `init: First stage mount skipped
(missing/incompatible fstab in device tree)`, `init: Skipped setting
INIT_AVB_VERSION (not in recovery mode)`, `init: Loading SELinux policy`,
`init: init second stage started!`, `init: Running restorecon...`,
`init: waitid failed: No child processes`. `sys.boot_completed=1` by t≈25 s.

Variations, one change at a time: `consoleblank=0` — display STILL sleeps between
t+100 s (lit, 565 colors) and t+150 s (black): the sleeper is Android, not the
kernel blanker → polling proof is required design, not a workaround. RAM 4096 —
no behavioral difference vs 2048 (default stays 2048). `-vga virtio` — guest runs
but the kernel log shows zero virtio-gpu driver activity and the framebuffer stays
dark: virtio-gpu needs guest-driver work, stays M3. qcow2 install — deferred
(live-boot per scope); `Boot::IsoMenu` (QMP Enter on the menu) implemented and
verified as the kept alternative to `Boot::DirectKernel`.

SECONDARY frame signal: `classify()` colour-count heuristic — black 1 colour,
VGA text 2 colours, isolinux photo-menu 565 colours; thresholds text 2–16,
graphical ≥100 (calibrated on real captures, unit-tested). Informational in M1b;
acceptance rests on the serial markers. FINAL proof (`adb getprop
sys.boot_completed`) is M2.

ADB headless enablement (docs only, no code): image is userdebug
(`ro.debuggable=1`, `persist.sys.usb.config=adb` preset, `/bin/adbd` present,
`ro.secure=1`, no `ro.adb.secure` override); no ADB kernel params in ISO entries.
Route: root serial shell → `setprop service.adb.tcp.port 5555` (+ adbd restart if
needed) → QEMU user-net `hostfwd=tcp:127.0.0.1:5555-:5555` → host adb. No Settings
toggle and no input path needed for bring-up. Open for M2: RSA-auth behaviour
(`ro.adb.secure` default on userdebug → host-key provisioning or test disabling).
