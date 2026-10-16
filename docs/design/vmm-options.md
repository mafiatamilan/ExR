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
