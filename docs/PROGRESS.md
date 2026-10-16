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
