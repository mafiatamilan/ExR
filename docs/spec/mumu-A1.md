# MuMu A1 — unpack + triage report

## a. TL;DR

- What: NetEase MuMu `mumu-mnq-win` bundle (zh-CN) — MSI wrapper around a downloader stub plus an offline installer.
- Installer type: Advanced Installer 17.5 MSI holding exactly 2 files in `disk1.cab`, finished by launching the downloader stub elevated.
- Payload type: 7z SFX downloader (fully extracted, 55 files) + Inno Setup 6.7.0 `Setup.exe` (97 MiB overlay, locked — innoextract 1.9 cannot parse its loader).
- Downloader behavior CONFIRMED: channel-pinned JSON manifest fetch over HTTPS, aria2-powered download, Hyper-V/VBox preflight checks, Sentry/crashpad reporting.
- Core-stack hypothesis, LOW confidence: VirtualBox-family VMM (only signal is Hyper-V/VBox conflict checks; no hypervisor binary observed — guest image still inside the locked Inno payload).

## b. Layer tree

```text
mumu-mnq-win.igd055.zip (transport wrapper, byte-identical MSI inside)
└── mumu-mnq-win.igd055.msi (Advanced Installer MSI, 2 files + registry + elevated finish action)
    ├── disk1.cab
    │   ├── MuMu_6.0.1_R4r3SjC.exe (7z SFX downloader stub, signed)
    │   │   └── 7z payload, 55 files: config.ini, nemu-downloader.exe, MuMuDownloader.exe,
    │   │       ColaBoxChecker.exe, HyperVChecker.exe, skin.zip (38 UI layouts, 15 languages),
    │   │       crashpad/sentry, VC runtimes, 7z.exe/dll
    │   └── Setup.exe (Inno Setup 6.7.0 installer, 97 MiB compressed overlay — NOT YET LISTED)
    │       └── ??? (VMM, Android image, drivers expected here; locked)
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
| `config.ini` | Pins downloader channel (`nochannel-mumu12`), language, task id; no offline package bundled | SFX payload text file | CONFIRMED |
| `nemu-downloader.exe` | Manifest client: fetches package/icon URLs, installs emulator + APKs, copies wallpaper/keymap configs | SFX payload strings (client class/method names, endpoint hosts) + Ghidra string inventory (see evidence ledger) | CONFIRMED |
| `MuMuDownloader.exe` | Download engine embedding aria2 | SFX payload strings (aria2 sources/options) | CONFIRMED |
| `HyperVChecker.exe` / `ColaBoxChecker.exe` | Preflight: detect Hyper-V/VBox environment, optionally disable Hyper-V via DISM | SFX payload strings (check outcomes, DISM feature names) | CONFIRMED |
| `skin.zip` | Downloader UI skin (dialogs, settings pages, 15 languages) | SFX payload archive listing | CONFIRMED |
| crashpad + sentry | Crash reporting from the downloader | SFX payload file set + Sentry host string | CONFIRMED |
| `Setup.exe` stub | Inno 6.7.0 loader; real payload compressed in overlay | PE section math + Inno version strings | CONFIRMED |
| `Setup.exe` payload | Expected VMM, drivers, Android image — content unknown | Blocked: innoextract 1.9 rejects the loader revision | UNKNOWN |
| Guest Android image | No image file observed at any extracted layer | No `.vmdk/.vdi/.img/.iso/.qcow2` in MSI, CAB, or SFX trees; binwalk on `Setup.exe` shows no disk signatures | UNKNOWN |

## d. A3 checklist

- Hypervisor/VMM: HYPOTHESIS (VirtualBox family, LOW). Reason: only signal is the
  downloader's Hyper-V/VBox environment checks plus DISM Hyper-V-disable paths. No
  hypervisor binary, driver, or service observed — MSI installs no `.sys`/service and the
  Inno payload is still locked.
- Android base / ABI / ARM translation: UNKNOWN. Reason: no system image, `build.prop`,
  kernel, or `libhoudini`-class library name observed at any extracted layer.
- GPU path: UNKNOWN. Reason: no renderer strings in MSI/SFX scope; guest-side graphics
  libraries live in the locked payload.
- Display/stream: UNKNOWN. Reason: no frame-bridge binary or protocol observed; downloader
  is an installer UI, not the emulator window.
- Input + keymap format: UNKNOWN (emulator side). Reason: downloader references copying
  keymap/wallpaper configs during install, but no keymap file format or injection binary observed.
- ADB (ports/bridge): UNKNOWN. Reason: no ADB client, port, or bridge strings in MSI/SFX scope.
- Multi-instance manager / CLI: UNKNOWN. Reason: no manager console, clone/snapshot config,
  or command-line interface binary observed outside the locked payload.
- Extras (macro recorder, resolution/DPI/FPS/CPU/RAM, shared folders, GPS/sensors, root
  toggle, APK install): UNKNOWN, except downloader-side APK fetch/install staging which is
  CONFIRMED as installer behavior (manifest `packageUrl` gate, not a runtime feature).

## e. Open questions and blockers (need your decision)

1. `Setup.exe` (Inno 6.7.0) vs innoextract 1.9 (`supports up to 6.0.5`, rejects this
   loader with revision/checksum/version errors): approve an alternative? Options I will
   NOT take without approval: (a) fetch a newer innoextract build, (b) parse the Inno
   overlay by hand, (c) run the installer in a disposable VM/Wine prefix. Recommend (a) first.
2. Confirm the A3 UNKNOWNs above stay UNKNOWN until the Inno payload opens — I will not
   promote the VirtualBox-family hint past LOW without a hypervisor binary.
3. Guest-image step is queued behind (1): once the payload lists, I will mount any
   system/data image read-only and record `build.prop`, kernel version/config, init rc
   files, and GPU/HAL/ARM-translation library NAMES only.
