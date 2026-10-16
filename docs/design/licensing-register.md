# Licensing register (Phase B0)

Date: 2026-10-16. Rule: process boundary where possible; never link GPL code into
our crates; never bundle vendor/proprietary bits. Every claim: URL + verified/unverified.

## VMM / GPU stack

| Dependency | Licence | Implication |
|---|---|---|
| QEMU (spawned process, QMP-driven) | GPLv2 overall (parts vary, GPLv2-compatible) — https://gitlab.com/qemu-project/qemu/-/raw/master/LICENSE — VERIFIED | Process boundary: GPL contained in the external binary; our crates only spawn it + speak JSON. Host must provide QEMU (like git/docker model). Never vendor QEMU binaries. |
| crosvm (secondary option) | BSD-3-Clause — https://github.com/google/crosvm — VERIFIED | Permissive; integration allowed if ever chosen. |
| rust-vmm crates (possible helpers) | Apache-2.0 OR BSD-3-Clause (suggested per-crate) — https://github.com/rust-vmm/community — VERIFIED | Permissive; safe to depend on. |
| gfxstream (evaluated M3) | Apache-2.0 — https://raw.githubusercontent.com/google/gfxstream/main/LICENSE — VERIFIED | Permissive; may link/vendor per terms. |
| virglrenderer (evaluated M3) | UNVERIFIED (canonical repo bot-walled 2026-10-16; pointer https://gitlab.freedesktop.org/virgl/virglrenderer) | Do NOT vendor or link before licence re-verified. QEMU-side use needs no linking from us. |
| VirtualBox (rejected) | Base OSE GPLv3 + Extension Pack proprietary — UNVERIFIED (pointer https://www.virtualbox.org/wiki/Licensing_FAQ) | Rejected partly on this split; no VBox code, images, or configs in our tree. |

## Guest side

| Item | Status |
|---|---|
| Android-x86 / Bliss / Lineage images | Open (Apache-2.0 family — UNVERIFIED per-image; confirm hash+licence at fetch time). Fetched by the user tool from publishers, never vendored. |
| GApps / Play Store | Proprietary. NEVER bundled, fetched, or installed by our tooling. |
| ARM translators | Proprietary. User-supplied files only (see `guest-images.md`); hash-recorded, never hosted/fetched. |
| AOSP/Cuttlefish references | Apache-2.0 (AOSP standard — UNVERIFIED here; pointer https://source.android.com/docs/setup/about/licenses). |

## Rust ecosystem (planned; all UNVERIFIED — pin + re-verify at first use)

 tokio, serde/serde_json, anyhow, thiserror, tracing (+tracing-subscriber), clap,
 wgpu, winit, egui (MIT verified via https://raw.githubusercontent.com/emilk/egui/master/LICENSE-MIT
 — VERIFIED MIT), iced, tauri, adb_client (MIT — VERIFIED via crates.io API 2026-10-16),
 gilrs-class gamepad crate, qmp-client crate. Expectation: all MIT/Apache-2.0 dual —
 confirm each `license` field in Cargo.lock review before M1 ends (CI check: `cargo deny`
 with an allowlist of MIT/Apache-2.0/BSD-3/MPL? no — MIT/Apache-2.0/BSD-3-Clause only).

## Non-goals with licence impact (explicit)

- No GPL code linked into ExR crates (QEMU stays a child process).
- No `cargo deny` exceptions without your approval.
- No vendor assets/images/configs/strings in repo or releases (clean-room, Phase A rule
  carries forward).
- scrcpy (Apache-2.0 — VERIFIED) is a protocol reference only; no code vendored.
