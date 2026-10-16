# MuMu A1 — unpack + triage report

## a. TL;DR

- What: NetEase MuMu `mumu-mnq-win` bundle (zh-CN) — MSI wrapper around a downloader stub plus an offline `Setup.exe`.
- Installer type: Advanced Installer 17.5 MSI holding exactly 2 files in `disk1.cab`, finished by launching the downloader stub elevated.
- Payload type: 7z SFX downloader (fully extracted, 55 files) + Inno Setup 6.7.0 `Setup.exe`, now listed AND extracted with a pinned fork build (34 records, 26 files on disk).
- KEY FINDING: the offline `Setup.exe` payload is a third-party software bundle (dictionary app, browser engine, task loader, updater, obfuscated blobs) — it contains NO hypervisor, NO Android image, NO ADB, NO emulator binaries at all.
- Consequence: the real emulator arrives ONLY through the downloader's manifest fetch; all emulator-core questions stay UNKNOWN for the offline scope. VMM hypothesis unchanged (VirtualBox family, LOW, from Hyper-V/VBox preflight checks only).

## b. Layer tree

```text
mumu-mnq-win.igd055.zip (transport wrapper, byte-identical MSI inside)
└── mumu-mnq-win.igd055.msi (Advanced Installer MSI, 2 files + registry + elevated finish action)
    ├── disk1.cab
    │   ├── MuMu_6.0.1_R4r3SjC.exe (7z SFX downloader stub, signed)
    │   │   └── 7z payload, 55 files: config.ini, nemu-downloader.exe, MuMuDownloader.exe,
    │   │       ColaBoxChecker.exe, HyperVChecker.exe, skin.zip (38 UI layouts, 15 languages),
    │   │       crashpad/sentry, VC runtimes, 7z.exe/dll
    │   └── Setup.exe (Inno Setup 6.7.0, app name NPAPI_24.0.0.1.2, NOT encrypted, english-only)
    │       └── 26 files on disk in 3 dirs (commondocs / commonappdata / app):
    │           dictionary editor + browser engine + task loader + updater + VC redist +
    │           small image-processing DLLs + numbered data slots + encoded config blobs
    └── MSI support binaries (not installed to target): aicustact.dll, SoftwareDetector.dll,
        viewer.exe (x86) + viewer.exe_1 (x64 bootstrapper that launches the stub)
```

Install footprint from MSI tables: two files land in `APPDIR` (`[ProgramFilesFolder]\mumu-mnq-win\mumu-mnq-win`,
single `MainFeature`), two registry values (`Version`, `Path`) under the vendor key,
launch conditions block pre-Vista Windows and require .NET 2.0+.

## c. Component inventory

| Component | Role | Evidence ref | Confidence |
|---|---|---|---|
| `.zip` wrapper | Transport of the MSI, no version difference | ZIP central directory + hash comparison | CONFIRMED |
| MSI database | Declares 2 files, 4 components, 1 feature, registry, upgrade rules | MSI File/Component/Feature/Media/Registry/Upgrade tables | CONFIRMED |
| `viewer.exe_1` (x64) finish action | Launches downloader stub elevated at end of MSI install | MSI CustomAction table, `LaunchFile_1` entry | CONFIRMED |
| `config.ini` | Pins downloader channel, language, task id; no offline package bundled | SFX payload text file | CONFIRMED |
| `nemu-downloader.exe` | Manifest client: fetches package/icon URLs, installs emulator + APKs | SFX payload strings + Ghidra string inventory (see evidence ledger) | CONFIRMED |
| `MuMuDownloader.exe` | Download engine embedding aria2 | SFX payload strings (aria2 sources/options) | CONFIRMED |
| `HyperVChecker.exe` / `ColaBoxChecker.exe` | Preflight: detect Hyper-V/VBox environment, optionally disable Hyper-V via DISM | SFX payload strings (check outcomes, DISM feature names) | CONFIRMED |
| `skin.zip` | Downloader UI skin (dialogs, settings pages, 15 languages) | SFX payload archive listing | CONFIRMED |
| crashpad + sentry | Crash reporting from the downloader | SFX payload file set + Sentry host string | CONFIRMED |
| `Setup.exe` bundle root | Third-party bundle, NOT the emulator (see rows below) | Inno listing + extracted file set, zero emulator-named files, zero drivers | CONFIRMED |
| Dictionary + browser + task-loader set | Bundled third-party apps: dictionary editor, embedded-browser engine, task loader, updater, VC redist | Extracted `commondocs` + `app` dirs; the four biggest files share one non-PE magic (same packer family) | CONFIRMED |
| Disguised loader + encoded configs | One PE DLL misnamed as `.ini`, one random-bytes config, one hex-encoded blob, four numbered data slots, one runtime-DLL zip | Extracted file types + magic bytes + archive listing | CONFIRMED |
| Small image-processing DLLs | Legacy support libraries (2007–2016 timestamps) | Extracted PE headers | CONFIRMED |
| Guest Android image | Absent from every extracted layer | No disk-image names, no disk signatures, empty driver search | UNKNOWN (absent offline) |

## d. A3 checklist

- Hypervisor/VMM: HYPOTHESIS (VirtualBox family, LOW). Reason: only signal remains the
  downloader's Hyper-V/VBox environment checks plus DISM Hyper-V-disable paths. The fully
  extracted offline payload contains no hypervisor binary, no `.sys`, no service — consistent
  with the emulator never being in the offline bundle.
- Android base / ABI / ARM translation: UNKNOWN. Reason: no system image, `build.prop`,
  kernel, or translation-library names in MSI, CAB, SFX, or Inno layers.
- GPU path: UNKNOWN. Reason: no renderer strings or graphics libraries in offline scope.
- Display/stream: UNKNOWN. Reason: no frame-bridge binary or protocol observed.
- Input + keymap format: UNKNOWN (emulator side). Reason: downloader copies keymap/wallpaper
  configs at install time, but no keymap format or injection binary observed offline.
- ADB (ports/bridge): UNKNOWN. Reason: no ADB client, port, or bridge strings in offline scope.
- Multi-instance manager / CLI: UNKNOWN. Reason: no manager console, clone/snapshot config,
  or command-line interface binary outside the downloader.
- Extras (macro recorder, resolution/DPI/FPS/CPU/RAM, shared folders, GPS/sensors, root
  toggle, APK install): UNKNOWN, except downloader-side APK fetch/install staging which is
  CONFIRMED as installer behavior (manifest `packageUrl` gate, not a runtime feature).

## e. Open questions and blockers (need your decision)

1. RESOLVED: pinned fork build listed AND extracted the Inno payload (details in evidence
   ledger). No fork fallback needed; no hand-parsing or VM run performed.
2. NEW: the offline bundle looks like a download-portal monetization wrapper, not vendor
   packaging — do you want any provenance note (e.g. where this MSI was sourced), or should
   I treat it as-is and move on? I recommend moving on; bundler internals are out of scope.
3. No Ghidra run on the extracted set: no extracted binary maps to a named A3 question
   (VMM / display bridge / input injector are all absent offline). The disguised loader DLL
   answers only bundler questions — left untouched per scope. Object now if you disagree.
4. No guest image found offline, so no mount step. Emulator-core answers depend on the
   downloader manifest path (network-adjacent — needs your explicit direction).
5. Ready to start MEmu on your go-ahead (order: MEmu, then Nox, then LDPlayer strings-only).
