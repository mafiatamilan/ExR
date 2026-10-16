# ExR design (Phase B0)

Date: 2026-10-16. Linux-first (Arch/KVM). Windows (WHPX) and macOS (HVF) exist as
trait targets only — not built in v1. No implementation code in this phase.
Evidence basis: `docs/spec/comparison.md` + `docs/design/*.md`. Clean-room: no
`docs/evidence/` content, no vendor code/strings/assets/configs anywhere here.

## Cargo workspace

```text
exr/                      # cargo workspace (edition 2021+, rust-version pinned in M1)
├── crates/
│   ├── emu-config   # TOML schema, validation, registry (atomic writes)
│   ├── emu-image    # open-image fetch/verify/registry, qcow2 overlays, user-translator slots
│   ├── emu-vmm      # VmBackend trait + QemuBackend (QMP); WhpxBackend/HvfBackend stubs
│   ├── emu-adb      # Rust ADB client (protocol, install, shell, forward) over adb_client-class crate
│   ├── emu-display  # window + frame path (virtio-gpu/virgl → gfxstream → fallback), res/fps/dpi
│   ├── emu-input    # keymap v1 engine, touch/multitouch, gamepad, mouse-look
│   ├── emu-manager  # registry, lifecycle, clone, snapshots, per-instance state
│   ├── emu-cli      # `exr` binary (clap), --json output, doctor
│   └── emu-gui      # GUI shell (framework picked in M6 spike: egui vs iced vs tauri)
└── tests/                # headless-guest integration suite (see below)
```

## Trait boundaries (the portability seam)

```rust
// emu-vmm (sketch — all multi-OS surface lives behind this)
trait VmBackend {
    fn launch(&mut self, spec: &VmSpec) -> Result<Handle>;   // spawn + QMP hello
    fn stop(&mut self, h: &Handle) -> Result<()>;
    fn snapshot_create/list/restore/delete(...);
    fn set_display(&mut self, h: &Handle, mode: DisplayMode) -> Result<()>;
}
struct QemuBackend { bin: PathBuf, accel: Accel };           // M1 builds this
struct WhpxBackend; struct HvfBackend;                       // trait-conformant stubs (compile, untested)
enum Accel { Auto, Kvm, Tcg, Whpx, Hvf }                     // Auto resolves per host OS
```

`emu-display` consumes frames behind a `FrameSource` trait (virtio-gpu fd →
stream fallback); `emu-input` injects behind an `InputSink` trait (virtio-input →
ADB fallback). Backends are chosen at runtime from `doctor` probes, never by cfg
sprawl in feature code.

## Cross-cutting choices

- Errors: `thiserror` for crate error enums (machine-readable variants per failure
  mode), `anyhow` at binary edges (`emu-cli`, `emu-gui`) with context chains.
- Async: tokio (single runtime; QMP socket, ADB connections, display pump). If the M1
  spike shows tokio fights QEMU child-process IO, documented fallback: blocking IO on
  dedicated threads — decision logged, not silently flipped.
- Logging: `tracing` + `tracing-subscriber` (spans per instance/QMP session); `--json`
  CLI output is data on stdout, logs on stderr — never mixed.
- Config: TOML, `schema_version` everywhere, unknown-field rejection (see
  `cli-and-config.md`). Licences: `cargo deny` allowlist MIT/Apache-2.0/BSD-3-Clause,
  wired in M1 CI (see `licensing-register.md`).

## Test strategy (headless guest)

- Unit: config validation (reject unknown fields, bad geometry), keymap schema
  (every example in `input.md` is a test fixture), QMP message codecs, ADB protocol
  framing against recorded transcripts (no live device in unit tests).
- Integration (headless, KVM): boot open Android-x86 image with `-display none`,
  assert serial-console boot marker + `adb wait-for-device` + `shell getprop`
  within timeout; install a test APK (open-source, e.g. a trivial activity built
  in-tree); snapshot create/restore round-trip; clone produces independent overlay.
  These run in CI only where `/dev/kvm` exists, otherwise gracefully skipped with a
  `doctor` explanation.
- GUI tests: screenshot-compare the window at fixed virtual resolution against a
  recorded frame HASH (not pixels — font/GPU variance), plus input-injection
  round-trip (inject tap → guest `getevent` shows it).

## Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| Android-x86 guest lacks virtio-input/gpu drivers | MEDIUM | M1/M3 spikes verify first; fallbacks: ADB injection, scrcpy-style stream (designed, not built) |
| QMP surface gaps for snapshots on chosen QEMU | LOW | libvirt-proven path; spike in M1, crosvm secondary kept warm |
| ARM-translator absence limits app compat | CERTAIN (by design) | Explicit x86-only status + user-supplied slot (see `guest-images.md`) |
| GPL contamination via dependency | LOW | `cargo deny` gate from M1; QEMU stays a child process |
| Scope creep into vendor parity (macros, stores) | MEDIUM | Out-of-scope list enforced in review; M7extras are ours, not clones |
| GUI framework churn | LOW | Decision deferred to M6 spike with criteria (binary size, input latency, packaging) |

## Explicit out-of-scope

DRM/anti-cheat/anti-tamper bypass; vendor assets, images, configs, or strings;
Play Store/GApps redistribution; ARM-translator sourcing; running anything as root;
Windows/macOS builds in v1 (traits only); vendor-format keymap compatibility in v1.

## Baseline → crate + milestone map

Source features: `docs/spec/comparison.md` (common baseline + differentiators).

| Baseline feature | Crate | Milestone |
|---|---|---|
| Boot open Android image, home screen | emu-vmm + emu-image | M1 (headless boot proof; window arrives M3) |
| ADB connect/install/shell/forward | emu-adb | M2 |
| GPU-accelerated window, res/FPS/DPI | emu-display (+emu-vmm) | M3 |
| Keyboard→touch keymaps, mouse-look, gamepad | emu-input (+emu-display) | M4 |
| Multi-instance create/clone/start/stop/delete + CLI | emu-manager + emu-cli (+emu-config) | M5 |
| GUI shell | emu-gui | M6 |
| Recorder, snapshots, GPS/ID spoof props, root toggle, shared folders | emu-manager/emu-display/emu-input | M7 |

## Milestone re-order (vs the original M1–M7) and why

Original order built the window first (M1) and control second. Evidence from the
baseline (both local-payload products are CLI/manager-driven with ADB as the control
plane) justifies control-plane-first:

- M1 = headless boot (serial-marker proof, no window). Why: de-risks CI and the QEMU
  wiring before any pixel work; matches the headless-guest test strategy.
- M2 = ADB plane (unchanged in content, promoted). Why: everything downstream
  (install, forward, input fallback, record) leans on it.
- M3 = display window + GPU (old M1-window + old M3-GPU merged). Why: one milestone
  owns the whole pixel path incl. virgl→gfxstream→fallback decision.
- M4–M7 unchanged in content (input → manager/CLI → GUI → extras).
