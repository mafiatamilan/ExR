# Phase A1 — MuMu MSI/ZIP unpack report (static, Linux-only)

Status: A1 complete for `mumu-mnq-win.igd055.msi` / `.zip`. No execution.
Next layers (Inno payload listing) blocked on missing `innoextract` — see §7.

## 1. Inputs (originals, untouched)

| File | Size (bytes) | SHA-256 | `file` |
|---|---|---|---|
| `mumu-mnq-win.igd055.msi` | 110980667 | `aaeb6fc9…9b5609ab2db` | Composite Document File V2, Microsoft Installer, Advanced Installer 17.5 build 2e45d4a6, Template `;2052`, Subject `mumu-mnq-win` |
| `mumu-mnq-win.igd055.zip` | 109148179 | `e5415e16…401147b724` | Zip archive, deflate, 1 member |

Full hashes:
- MSI: `aaeb6fc997485658ba5647eda47c062ab29c89618f8995b20c79d9b5609ab2db`
- ZIP: `e5415e16a7fc7eeae9711a2fa33c0e8e198901d5b013383e803d2d401147b724`

ZIP member `mumu-mnq-win.igd055.msi` (stored date 1980-01-01 placeholder) extracts
to SHA-256 `aaeb6fc9…` — byte-identical to the standalone MSI. The ZIP is a thin
transport wrapper, not a separate version.

## 2. Tools used / missing

Present: `file` 5.48, `7z` 26.04, `cabextract` 1.11, `strings`, `objdump`,
REA MCP (artifact-graph + Ghidra 12.1.2 provider, Hopper absent, IDA unconfigured).
Missing (needed for next steps): `innoextract` (Inno payload), `msitools`
(`msiextract`, structured MSI tables), `binwalk`, `unshield`.

## 3. Procedure (read-only on originals)

1. `file` + `sha256sum` on both originals.
2. `7z l` on MSI (Compound) and ZIP; extracted ZIP → `work/mumu-zip/`.
3. `7z x msi → work/mumu-msi/` (54 streams; 1 warning: 59-byte trailer past
   Compound end, `Physical Size 110980608`, `Tail Size 59`).
4. `cabextract -l/-d work/mumu-msi/disk1.cab → work/mumu-payload/`.
5. `7z l/x` on both CAB members; `objdump -h/-p`, `strings` (ASCII + UTF-16LE),
   PE section math, ZIP listing of `skin.zip`.
6. Re-verified original hashes/mtimes after extraction.

Nothing was executed. No writes to originals.

## 4. Findings — layer map

```text
mumu-mnq-win.igd055.zip (105 MiB)
└── mumu-mnq-win.igd055.msi (106 MiB, Advanced Installer 17.5 MSI)
    ├── MSI streams (54): [!File !Media !Binary !Property !Directory
    │   !Component !Feature !CustomAction !Registry …] + !_StringData (62 KiB)
    │   + support PEs: Binary.aicustact.dll (x86), Binary.SoftwareDetector.dll,
    │   Binary.viewer.exe (x86) + viewer.exe_1 (x64), icons/banners
    └── disk1.cab (107797067 B, 2 files)
        ├── MuMu_6.0.1_R4r3SjC.exe (6083616 B, 2026-07-17) — 7zS2.sfx DOWNLOADER
        │   └── 7z payload: 55 files / 20173779 B (see §5)
        └── Setup.exe (102473004 B, 2026-09-24) — Inno Setup 6.7.0 installer (see §6)
```

MSI metadata (from Compound SummaryInformation + `!_StringData` strings):
- Builder: `Advanced Installer 17.5 build 2e45d4a6`; DB template timestamp
  `2009-12-11` (tool default, not build date).
- `ProductName=mumu-mnq-win`, `ProductVersion=1.0.0`, `ProductLanguage=2052`
  (zh-CN), `ProductCode={F8AB28A6-968C-47DB-AE4B-DB706CEEE2E5}`.
- `Manufacturer` value empty in string table; default paths
  `Software\[Manufacturer]\[ProductName]`, `APPDIR=[ProgramFilesFolder][Manufacturer]\[ProductName]`.
- `AI_APP_FILE=[#Setup.exe]`; finish custom action `LaunchFile_1` runs
  `"[#MuMu_6.0.1_R4r3SjC.exe]"` with `/RunAsAdmin /DontWait` (i.e. the MSI
  hands off to the downloader stub on completion).
- Prereq strings: `AI_REQUIRED_DOTNET 2.0`, `AI_BOOTSTRAPPER`, admin-elevated
  restart actions, `AI_DetectSoftware` / `SoftwareDetector.dll`.

## 5. Downloader SFX (`MuMu_6.0.1_R4r3SjC.exe`)

- PE32 i386 GUI, 4 sections; version resource: Company `NetEase, Inc.`,
  Description `MuMu模拟器`, InternalName `7zS2.sfx`, OriginalFilename
  `NemuDownloader.exe`, versions `22.0.0.0` / `1.0.0.0`; SECURITY directory
  present (signed, 10744 B) — contrast `Setup.exe` Security dir = 0.
- 7z archive at offset 88: `LZMA2:6m BCJ`, solid, 11 blocks, 55 files.
- Payload (`work/mumu-sfx/`, 20 MiB):
  - `config.ini` (`[General] channel=nochannel-mumu12`, `fchannel` same,
    `language=zh-Hans`, `taskid=6a4723215672d839d6deb78a`, `apkid=None`,
    `offlinepackage*=None`) — channel/task selectors, no bundled offline image.
  - `nemu-downloader.exe` (3719160 B, PE32 i386 GUI) — Qt-linked (layered-window
    / PNG-chunk strings), JSON-manifest client (`packageUrl`, `apk_id`,
    `logoIcoUrl`, `parseFromJson`, `fetchDownloadInfo`, `doInstall`,
    `copyWallpaperIntoDest`, `moveKeymapConfig`).
  - `MuMuDownloader.exe` (5956856 B, PE32 i386 console, PDB-stripped) — embeds
    **aria2** (`DownloadContext.cc/.h`, `download_helper.cc`, aria2 option
    strings). Download engine evidence, not a guess.
  - `ColaBoxChecker.exe` (4198576 B, PE32 i386 console), `HyperVChecker.exe`
    (120488 B, PE32+ x86-64 console) — preflight checks; observed strings:
    `hyperv`, `hyperv_error`, `vbox_env_error`, `close_hyperv_error`,
    `[HyperVChecker::process]`, `[HyperVClose::_closeHyperV]`, DISM
    `/Disable-Feature:microsoft-hyper-v-all` and `Microsoft-Hyper-V-Hypervisor`.
    Implication (to verify in A2/A3): host VMM conflicts with Hyper-V, consistent
    with a VirtualBox-family VMM — recorded as hypothesis, not conclusion.
  - `skin.zip` (395215 B) — downloader UI skin (PNGs + XML, e.g.
    `CancelStartWindow.xml`, `btn_*.png`).
  - Crash reporting: `crashpad_handler.exe` + `sentry.dll`; Sentry DSN host
    `sentry.netease.com` (project 324) in downloader strings.
  - Runtimes: `msvcrt.dll`, `msvcp_win.dll`, `msvcp140.dll`, `vcruntime140.dll`,
    `ucrtbase.dll`, `api-ms-win-*` set, `7z.exe`/`7z.dll` (21.05).
- Network endpoints observed in `nemu-downloader.exe` static strings (no traffic
  generated): `https://api.mumu3.nie.netease.com/api/`,
  `https://api.mumu.nie.netease.com/api/`, `https://api.mumuglobal.com/api/`,
  `http://nemu-api.game163.dev.webapp.163.com:8400/api/`,
  `http://api.mumu3.dev.webcgi.163.com:8400/api/`,
  `https://adl.netease.com/d/g/a11/c/offline_installer_64`,
  `https://dns.update.netease.com/hdserver`,
  `https://dns.update.easebar.com/hdserver`, `mumu.163.com` redirects,
  `mumuglobal.com` help URLs. Behavior strings: HTTP DNS task for overseas,
  sync-GET with status-code handling, `ServerApiUtil::getNemu/MuMu3ServerRoot`
  (official vs overseas), game-launcher icon download with `packageUrl` gate.

## 6. Main installer (`Setup.exe`, Inno payload — listing pending)

- `102473004` B; PE32 i386 GUI; 11 sections (`.text .itext .data .idata .didata
  .edata .rdata .reloc .rsrc` …); linker 2.25; OS/Subsystem 6.1; entry
  `0xb1e60`; `SizeOfImage 0xdc000`; headers end `0xCDA00` (842240 B);
  **overlay 101630764 B (~97 MiB)** = compressed payload.
- Identity strings: `Inno Setup Setup Data (6.7.0)`, `Inno Setup Messages
  (6.5.0) (u)`, `name="JR.Inno.Setup"`, `This installation was built with Inno
  Setup.`, `https://jrsoftware.org/ishelp/index.php?topic=setupcmdline`,
  Delphi RTTI (`System.Rtti`, `System.Generics.Collections`), UTF-16 title
  fragment `NPAPI_24.0.0.1.2 Setup`. `7z` reports `Cannot open the file as
  archive` — expected for Inno.
- Imports (stub loader): `kernel32.dll` surface (`CreateProcessW`,
  `CreateThread`, `VirtualProtect`, `FindResourceW`, `LoadResource`,
  `LockResource`, file/path/version helpers) — no VMM conclusion drawable from
  the stub alone.
- Staged output dir `work/mumu-setup/` reserved for `innoextract` output.

## 7. Gaps / blockers (need approval)

1. `Setup.exe` file listing + version table need `innoextract` (not installed;
   small package, no >1 GB download). Proposed: `apt install innoextract`
   (or equivalent) then `innoextract -d work/mumu-setup work/mumu-payload/Setup.exe`.
2. Structured MSI tables (`msitools`/`msiextract`) would replace raw 7z-stream
   reading for A2 (`drivers/.sys`, services, registry, runtimes). Current
   findings use `!_StringData` + CAB/file evidence only.
3. `binwalk` absent — only needed if Inno payload or later products hide nested
   firmware/disk images; not yet required.
4. No VMM/Android/GPU/input/ADB conclusions yet — correctly deferred to A2/A3
   after the Inno payload is listed. Do **not** treat Hyper-V-checker strings as
   proof of VirtualBox.

## 8. Safety & clean-room notes

- Static only: `file/7z/cabextract/strings/objdump/7z-l`. No execution of
  MSI/CAB/Setup/SFX members; no Wine/VM used.
- No decompiled code, assets, icons, branding, or images copied into the new
  project — this report records paths, sizes, hashes, and short factual strings
  (endpoint URLs, version fields) as behavior evidence only.
- Out of scope untouched: licensing/DRM/anti-cheat/anti-tamper bypass.
- Originals re-verified unchanged after unpack (hashes in §1 match pre-extract
  values; `work/` holds all derived bytes).

## Appendix — paths

- `work/mumu-msi/` — MSI Compound streams + `disk1.cab` (106 MiB, git-ignored).
- `work/mumu-payload/` — `Setup.exe`, `MuMu_6.0.1_R4r3SjC.exe` (104 MiB).
- `work/mumu-sfx/` — SFX contents (20 MiB), incl. `config.ini`, `skin.zip`.
- `work/mumu-zip/` — ZIP-extracted MSI copy (hash-verified identical).
- `work/mumu-setup/` — empty, reserved for Inno extraction.

## 9. REA evidence ledger (static, no execution)

- ZIP inventory (`inspect_artifact`, summary): parent
  `ev_c5d904f670c2770f3af64e1207a97d80e915c7c9c32854d6c20bd940011aad45`;
  manifest `agm_e79d6669…`, graph `1172140b…`; 2 nodes / 2 occurrences / 1 edge,
  0 integrity contradictions, `source_complete: true`. Occurrence page
  `ev_76928ccd…`: root `.` (ZIP, 109148179 B, verified) +
  `mumu-mnq-win.igd055.msi` (declared 110980667, compressed 109148035,
  `executable: true`, `hash_status: verified`). Corroborates §1 byte-identity.
- PE resources (`inspect_pe_resources`, portable, no disassembler):
  `Setup.exe` → `ev_ed580dc5…`, pe32, 102473004 B,
  sha256 `7e373a6f…be4a2af`, 22 resources / 50 entries, complete;
  `nemu-downloader.exe` → `ev_05ac9d38…`, pe32, 3719160 B,
  sha256 `dea4d6f9…a239e6f`, 23 resources / 54 entries, complete.
- Ghidra 12.1.2 on `work/mumu-sfx/nemu-downloader.exe`
  (sha256 `dea4d6f9…`, PE x86, image base `0x400000`):
  overview `ev_14e61ce5…` — 7 segments
  (Headers/`.text` 2572288 B / `.rdata` / `.data` / `.rsrc` / `.reloc` / `tdb`),
  **22395 procedures, 10449 defined strings**; PDB ref
  `nemu-downloader.pdb` GUID `22eb9237-553c-413f-9f65-60c084a40f35` age `0x32`
  (load state unknown — names may be degraded).
- Ghidra `search_strings` literal `api.mumu` → `ev_45dbb905…`, 7 observed hits
  with addresses: `0x6cb0c0` + `0x6cb144` (`api.mumu3.nie.netease.com`),
  `0x6cb114` (dev `webcgi.163.com:8400`), `0x6cb16c` (`api.mumu.nie…`),
  `0x6cb194` (`api.mumuglobal.com`), `0x6e797c` / `0x6e79d0` (roots).
  Authority `shipped-artifact`, confidence `observed`. Session closed with
  `close_binary`; ephemeral Ghidra DB discarded, originals unmodified.
