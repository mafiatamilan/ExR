# PROGRESS LOG — ExR clean-room clone

Running log. Static analysis on Linux only. No installer executed.
Originals in repo root are read-only inputs; all unpacking goes to `work/`.
`work/` is git-ignored and never committed.

## 2026-10-16 — Phase A1 (MuMu .msi/.zip) DONE, awaiting report-format approval
- Inputs fingerprinted (sha256, `file`): `mumu-mnq-win.igd055.msi` (106 MiB),
  `mumu-mnq-win.igd055.zip` (105 MiB). Zip contains byte-identical MSI.
- Unpacked MSI Compound with `7z` to `work/mumu-msi/` (54 streams).
  `disk1.cab` (103 MiB) holds 2 files: `MuMu_6.0.1_R4r3SjC.exe` (5.9 MiB, 7z SFX
  downloader) + `Setup.exe` (98 MiB, Inno Setup 6.7.0).
- Extracted CAB with `cabextract` to `work/mumu-payload/`.
- Extracted SFX with `7z` to `work/mumu-sfx/` (55 files, 20 MiB): `config.ini`,
  `skin.zip`, `nemu-downloader.exe`, `MuMuDownloader.exe` (aria2-based),
  `ColaBoxChecker.exe`, `HyperVChecker.exe` (x64), crash reporting
  (crashpad + sentry), VC runtimes, 7z.exe/dll.
- `Setup.exe` confirmed as Inno Setup 6.7.0 stub + ~97 MiB overlay; `7z` cannot
  open it. Full file listing BLOCKED on missing `innoextract`.
- Static strings only (no execution): downloader endpoints under
  `*.nie.netease.com`, `mumuglobal.com`, `163.com`, `adl.netease.com`;
  sentry DSN host `sentry.netease.com`; Hyper-V/VBox conflict checks.
- Missing tools noted: `innoextract`, `msitools`/`msiextract`, `binwalk`,
  `unshield`. Present: `file`, `7z`, `cabextract`, `strings`, `objdump`.
- Report: `docs/spec/mumu-msi-A1.md`. Originals untouched (hashes re-verified).
- STOPPED before other products (MEmu/Nox/LDPlayer) pending report-format approval.

## 2026-10-16 — Phase A1 REA pass (same MSI/ZIP scope)
- REA `binary_session`: Ghidra 12.1.2 available (Linux x64, JDK 27); Hopper
  missing; IDA unconfigured. No execution.
- `open_binary` ZIP + `inspect_artifact` summary + occurrence page: 2 nodes /
  2 occurrences / 1 edge, 0 contradictions, source_complete true; member sizes
  match manual section 1.
- `inspect_pe_resources` (target-free): Setup.exe (22 resources) and
  nemu-downloader.exe (23 resources), both coverage complete.
- Ghidra `binary_overview` on nemu-downloader.exe: 22395 procedures,
  10449 strings, image base 0x400000; `search_strings "api.mumu"` literal:
  7 endpoint hits with addresses. Session closed via `close_binary`.
- Spec updated: `docs/spec/mumu-msi-A1.md` section 2 + new section 9 (evidence IDs).
- `work/` added to `.gitignore`; derived bytes stay untracked.

## 2026-10-16 — MuMu A1 finished per approved template (no other products started)

- Env is Arch (pacman, no apt; sudo needs password) so tools went rootless to
  `/tmp/opencode/reatools/root` (outside repo): innoextract 1.9, msitools 0.106,
  binwalk 3.1.0, unshield 1.6.2. No system modification.
- `innoextract --list/--extract` on `Setup.exe`: FAILED as recorded —
  loader revision 2 / checksum mismatch / undetectable data version (1.9 covers Inno
  to 6.0.5; payload is 6.7.0). Payload still locked; no alt attempted without approval.
- `msiinfo` full tables + `msiextract` to `work/mumu-msiextract/`: 2 files, hashes match
  cabextract outputs; Manufacturer corrected to `mumu-mnq-win`; 4 components, 1 feature,
  elevated stub-launch finish action, .NET 2.0 + OS-version launch conditions.
- `binwalk` on `Setup.exe`: 2 generic hits only (no archive/disk signatures).
- Stub triage: `Setup.exe` strings show Delphi RTL only (payload LZMA-opaque); no VMM/ADB
  strings drawable from the stub. Ghidra stays parked per triage-first rule (earlier run
  justified: downloader-endpoint question).
- No guest image at any extracted layer — mount step queued behind payload unlock.
- Output split: `docs/spec/mumu-A1.md` (template a–e, human-readable, no IDs/addresses),
  `docs/evidence/mumu.md` (ledger E1–E8). Old `mumu-msi-A1.md` moved to `mumu-A1.md`.
- .gitignore verified: `*.exe/*.msi/*.zip/*.apk/*.ipa/*.dmg/*.pkg/*.iso/*.vhd*/*.ova/*.img`,
  `build/out/dist`, `work/` — only text reports staged.

## 2026-10-16 — MuMu A1 payload unlock (Q1-a, fork build, network-isolated)

- Pinned third-party fork `github.com/crazy-max/innoextract` tag `v1.11.0` commit
  `e4d1cc35…` (verified); source build impossible rootless (no boost headers), so used
  release binary (SHA-256 `9c6a3b08…`, logged) + 5 boost-1.83 `.so` from Arch archive.
- `unshare -rn` list-only: NPAPI_24.0.0.1.2, Inno 6.7.0 unicode, NOT encrypted, 34 records,
  1084 MiB (< 2 GiB gate). Extract to `work/mumu-setup/`: 26 files, 2.1 GiB on disk
  (duplicated destinations), 26 multi-part checksum warnings, exit 0. Nothing executed.
- KEY FINDING: offline payload is a third-party bundle (StarDict/CEF/task-loader/updater,
  obfuscated blobs, disguised `.ini` PE) — zero emulator binaries, zero drivers, zero
  disk images. Emulator arrives only via downloader manifest; A3 core stays UNKNOWN.
- No new Ghidra (no binary maps to a named A3 question), no guest image to mount,
  no fallback needed (1.11.0 worked; innoextract-go untouched).
- Docs rewritten to template: `docs/spec/mumu-A1.md` (no IDs/addresses/snippets),
  `docs/evidence/mumu.md` E9. STOPPING before MEmu per instructions.

## 2026-10-16 — MEmu A1 DONE (full offline bundle, VBox-fork VMM CONFIRMED)

- Input `MEmu-Setup-9.5.7.1-hafb3411bb.exe` (730 MiB, sha256 `83225b53…`), Microvirt Qt stub;
  payload = solid 7z in PE resources (735 MiB → 234 files / 2.13 GiB).
- Extracted to gitignored `work/memu/` + VMM pack to `work/memu-hyperv/`; guest VHDs
  converted read-only to raw in `work/` (disk2 excluded: 256 GiB virtual).
- VMM = VirtualBox 6.1.40 fork CONFIRMED (renamed binaries, fork PDB paths, VBox-schema
  config); Hyper-V HCS mode CONFIRMED; Android 9 x86_64 + Houdini-style bridge CONFIRMED;
  GLES encoder passthrough CONFIRMED; memuc `##` CLI CONFIRMED; root/GPS/ID extras
  CONFIRMED. Keymap format UNKNOWN (no such files).
- Reports: `docs/spec/memu-A1.md` + `docs/evidence/memu.md`. Text only, nothing executed.

## 2026-10-16 — Nox A1 DONE (VBox-derived single-binary VMM + Android 7.1 32-bit guest)

- Input `nox_setup_v7.0.6.2_full_intl.exe` (572 MiB, sha256 `7995ce17…`), Duodian stub;
  payload = 7z in PE resources (593 MiB → 439 files / 0.92 GiB).
- VMM = VirtualBox-derived CONFIRMED (unrenamed VBox configs + noxptdevice passthrough +
  single-binary NOXVM engine); guest = Android 7.1.2 x86 32-bit + Houdini CONFIRMED
  (kernel 4.0.9, SDK 25); host ANGLE + dual ADB CONFIRMED; NoxConsole CLI CONFIRMED.
  No native Hyper-V mode (DISM/bcdedit disabler instead). Keymap UNKNOWN.
- Reports: `docs/spec/nox-A1.md` + `docs/evidence/nox.md`. Text only, nothing executed.

## 2026-10-16 — LDPlayer A1 DONE (stub-only, no download)

- Input `LDPlayer9_ens_1001_ld.exe` (1.3 MiB, sha256 `e76e6696…`), JUST OKAY `ldsmaller`
  WinINet downloader; skin-only 7z (65 files, 20 locales). Two complete static package
  URLs (`LDPlayer_9.2.0.1.exe` on vendor CDN, inert data); rest runtime-assembled.
  All A3 UNKNOWN. Reports: `docs/spec/ldplayer-A1.md` + `docs/evidence/ldplayer.md`.

## 2026-10-16 — Q2 provenance note + comparison DONE, STOPPING for review

- `docs/evidence/mumu.md` E10: bundleware assessment, `igd055`/`mnq` tag link, full
  Authenticode table (Caphyon/NetEase/Microsoft/Hefei-Nuola signers, validity +
  timestamp status; hash-binding + revocation left for your lookup).
- `docs/spec/comparison.md`: feature table with per-cell confidence, baseline vs
  differentiators, unlearned-items section, gap recommendations.
- No Phase B, no cloning. Awaiting your review of the comparison.

## 2026-10-16 — Phase B0 DONE (research + design, no code)

- Researched open projects only (network): QEMU (GPLv2, KVM/hvf/whpx/QMP), crosvm
  (BSD-3-Clause, WHPX, virtio-gpu), rust-vmm crates (Apache/BSD), scrcpy v5.0.1
  (Apache-2.0), gfxstream (Apache-2.0), virgl (mesa docs; licence unverified),
  Android-x86 9.0-r2, Waydroid (LXC/Lineage-13), Bliss (images paused), egui (MIT),
  adb_client 3.2.3 (MIT). Every claim cited URL + date + verified/unverified.
- Deliverables: `docs/design/` (vmm-options, guest-images, gpu-and-display, input
  with original TOML keymap v1, cli-and-config, licensing-register) + `docs/DESIGN.md`
  (workspace, 9 crates, VmBackend/FrameSource/InputSink traits, tokio/thiserror/tracing,
  headless-guest tests, risks, out-of-scope, baseline→crate map, reordered milestones
  M1-headless/M2-adb/M3-display with reasons).
- Clean-room: design cites `docs/spec/` only; verified no vendor strings/configs in
  design docs. No implementation code. Text only. STOPPING for review before any code.

## 2026-10-16 — M1 DONE (headless boot proof, ~30 s wall)

- Env: QEMU 11.1.2, `/dev/kvm` writable, rust 1.90 rootless — nothing missing.
- Workspace: 10 crates (emu-vmm + emu-core real, 8 stubs), MIT/Apache licences,
  deny.toml (added Unicode-3.0 for transitive unicode-ident; cargo-deny binary absent
  so CI-gated, manual tree audit clean: thiserror/serde/serde_json only).
- QMP: owned minimal blocking client (qmp 0.1.1 immature, qapi-qmp types-only —
  verified on crates.io, rationale in qmp.rs). Socket in 0700 `$XDG_RUNTIME_DIR` dir.
- ISO: OSDN unresolvable from IPv6-only host → official SourceForge mirror, 8-way
  ranged fetch, sha1 `1cc85b5e…995aa0` VERIFIED. Stored outside repo.
- Proof: direct kernel boot (ISO menu stalls headless — orange menu screenshotted);
  cmdline from ISO isolinux.cfg + consoles + vga=791; serial shows kernel boot and
  `Detecting Android-x86... found at /dev/sr0`; lit frame ≤10 s, lit+changed PASS
  at ~20–30 s; graceful stop + full cleanup. Demo + ignored integration test both pass.
- 19 unit tests green; fmt/clippy clean. Deviations: Boot::DirectKernel added,
  polling proof (display sleeps when idle), serial_log API, persistent qcow2
  deferred (live-boot per scope).

## 2026-10-16 — M1b DONE (Android userspace proven; approaches used: 2 of 3 budgeted)

- Diagnosis path: initramfs `init` script read (open Android-x86 code) → emergency
  shell means `exec switch_root` fell through; DEBUG=2 run parked at its checkpoint
  (mounts confirmed working); pty-driven interrogation proved switch_root SUCCEEDED
  (PID1=/init, 1000+ procs, FUSE mounts, boot_completed=1).
- Markers + timings in docs/design/vmm-options.md (M1b section). Variations:
  consoleblank=0 (Android still sleeps display), RAM4096 (no change), virtio-vga
  (no guest driver → dark; M3), vesa entry skipped with rationale.
- Product: Boot::IsoMenu + QMP send_key/press_key (verified via --iso-boot PASS),
  frame classify() heuristic (calibrated 1/2/565 colours), serial last-60 in demo.
- 21 unit tests green; fmt/clippy clean. Image manifest at docs/design/image-manifest.toml.
- cargo-deny 0.20.2 (rootless `cargo install --locked`): `cargo deny check` →
  advisories ok, bans ok, licenses ok, sources ok. Notes: syn 2.0.119 + 3.0.9 both
  present (informational); BSD-3-Clause allowance currently unmatched (kept for M2+).
