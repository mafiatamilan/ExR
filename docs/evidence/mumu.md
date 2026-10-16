# MuMu evidence ledger (REA + CLI tool log)

Static only. No execution, no network contact. Endpoint strings are data; no URL was fetched.
Originals: `mumu-mnq-win.igd055.msi` / `.zip` in repo root (read-only inputs).

## E1. Input hashes

- MSI `aaeb6fc997485658ba5647eda47c062ab29c89618f8995b20c79d9b5609ab2db` (110980667 B)
- ZIP `e5415e16a7fc7eeae9711a2fa33c0e8e198901d5b013383e803d2d401147b724` (109148179 B)
- CAB member `Setup.exe` `7e373a6fe7e0921a4e5b87f3089ace296678d9e089b509ac2eedb3f09be4a2af` (102473004 B)
- CAB member `MuMu_6.0.1_R4r3SjC.exe` `a669f46ef1fec7425068184b33921def5ca9cf5dc0ca71d787bce37c018aa420` (6083616 B)
- `nemu-downloader.exe` `dea4d6f911c056bc25e21beeabeadba0fe2d46483009315ca488093a3a239e6f` (3719160 B)
- msiextract outputs hash-match cabextract outputs (both pairs identical).

## E2. Tool versions

- `file` 5.48, `7z` 26.04, `cabextract` 1.11, `strings`/`objdump` (binutils).
- Rootless prefix `/tmp/opencode/reatools/root` (outside repo, user-local, no system modification):
  `innoextract` 1.9 (banner: Inno 1.2.10–6.0.5), `msitools` 0.106 (`msiextract`/`msiinfo`),
  `binwalk` 3.1.0, `unshield` 1.6.2. Deps satisfied by system `boost-libs` 1.92.0 + `xz` 5.8.4;
  `gcab` 1.6 + `libgsf` 1.14.60 extracted alongside for msitools.
- REA: artifact-graph provider v1; Ghidra provider 12.1.2 (bridge verified vs 12.1.4),
  Linux x64, JDK 27. Hopper unavailable (no launcher); IDA unconfigured. Ghidra session
  closed via `close_binary` after use (ephemeral DB discarded).

## E3. Commands (read-only on originals)

- `file` + `sha256sum` on both originals.
- `7z l` MSI + ZIP; `7z x` ZIP → `work/mumu-zip/`; `7z x` MSI → `work/mumu-msi/` (54 streams,
  1 warning: 59-byte trailer, Physical 110980608 / Tail 59).
- `cabextract -l` / `-d work/mumu-payload` on `disk1.cab` (2 files).
- `7z l/x` on SFX → `work/mumu-sfx/` (55 files / 20173779 B); `7z l` on `skin.zip`.
- `objdump -h/-p` on `Setup.exe` (11 sections, linker 2.25, OS/Subsystem 6.1, entry 0xb1e60,
  headers end 0xCDA00/842240, overlay 101630764 B).
- `innoextract --list` + `--extract --output-dir work/mumu-setup`: both fail —
  `Warning: Unexpected setup loader revision: 2`, `Warning: Setup loader checksum mismatch!`,
  `Could not determine setup data version!` (+ upstream-issue pointer). Exit path recorded,
  payload not listed.
- `msiinfo tables/suminfo/export` (Property, File, Directory, Component, Feature, Media,
  CustomAction, Registry, LaunchCondition, Upgrade); `msiextract --directory work/mumu-msiextract`.
- `binwalk work/mumu-payload/Setup.exe` (85 signatures / 187 patterns, 549 ms): 2 hits —
  Copyright text at 0xA4826, SHA256 constants LE at 0xB3D10. No archive/disk signatures.
- Triage `strings` greps over `Setup.exe` stub (VMM/driver/ADB/input patterns): stub yields only
  Delphi RTL (`System.SysUtils`) + generic `ReadBuffer*` — payload strings are LZMA-opaque.

## E4. MSI tables (structured, via msitools)

- Property: `ProductName=mumu-mnq-win`, `Manufacturer=mumu-mnq-win`, `ProductVersion=1.0.0`,
  `ProductCode={F8AB28A6-968C-47DB-AE4B-DB706CEEE2E5}`, `ProductLanguage=2052`,
  `UpgradeCode={C18B11B7-D4F9-4513-9E5A-828E379A20D3}`, `AI_APP_FILE=[#Setup.exe]`,
  `AI_REQUIRED_DOTNET_VERSION/DISPLAY=2.0`, `AI_PACKAGE_TYPE=Intel`, `AI_BOOTSTRAPPERLANGS=2052`.
- File: 2 rows — `MuMu_6.0.1_R4r3SjC.exe` (6083616 B, v22.0.0.0, lang 1033, seq 1),
  `Setup.exe` (102473004 B, v0.0.0.0 unversioned, seq 2). Media: `DiskId 1, LastSequence 2,
  Cabinet #disk1.cab`.
- Directory: `APPDIR → TARGETDIR`, `TARGETDIR → SourceDir` (=root). Component: 4 rows
  (`APPDIR {8C71472C…}`, `ProductInformation {AEBD815A…}`, file components `{6614ED36…}` /
  `{30452230…}`). Feature: single `MainFeature` (Display 1, Level 1, `APPDIR`).
- CustomAction: `LaunchFile_1` type 1730, source `viewer.exe_1`,
  target `/EnforcedRunAsAdmin /DontWait /RunAsAdmin "[#MuMu_6.0.1_R4r3SjC.exe]"`;
  `LaunchFile` type 2 via `viewer.exe` for `<target file>`; `AI_*` suite via `aicustact.dll`
  (`UpdateInstallMode`, `LaunchApp`, `DetectModernWindows`, `PrepareUpgrade`, …) and
  `SoftwareDetector.dll:OnDetectSoftware`.
- Registry: `Version=[ProductVersion]`, `Path=[APPDIR]` under `Software\[Manufacturer]\[ProductName]`
  (root -1 = HKLM per msitools convention, component `ProductInformation`).
- LaunchCondition: blocks Win9x (`VersionNT`), NT4 (`<>400`), 2000 (`<>500`), XP/2003
  (`<>501 AND <>502`); requires `AI_DETECTED_DOTNET_VERSION >= AI_REQUIRED_DOTNET_VERSION`.
- Upgrade: same UpgradeCode removes `0.0.1–1.0.0` (`OLDPRODUCTS`, attr 257), detects newer
  (`AI_NEWERPRODUCTFOUND`, attr 2).

## E5. REA artifact inventory (ZIP)

- `open_binary /home/david/proj/ExR/mumu-mnq-win.igd055.zip` → `{format: zip, kind: archive,
  sha256: e5415e16…}`.
- `inspect_artifact detail: summary` → parent `ev_c5d904f670c2770f3af64e1207a97d80e915c7c9c32854d6c20bd940011aad45`;
  manifest `agm_e79d6669c694bbf31153791678e21af09ed47ea50149a129e4b2eca6a38d45d4`,
  graph `1172140bfbb594adcd6ca2058f58d9ac93b8896329cc88485ed52ffc3c571fca`;
  2 nodes / 2 occurrences / 1 edge, 0 integrity contradictions, `source_complete: true`.
- Occurrence page `ev_76928ccda976038fc535be4ad5156d1785fbb7d767297782c0e99ccdbfb64357`
  (view_digest `cd374c08…`): root `.` ZIP 109148179 B verified +
  `mumu-mnq-win.igd055.msi` declared 110980667 / compressed 109148035, executable true,
  hash verified.

## E6. REA PE resources (target-free, no disassembler)

- `Setup.exe` → `ev_ed580dc56db07bc69cb0768336671390ed65060768f0ba778b34b4c0404fb096`,
  pe32, 102473004 B, 22 resources / 50 entries, coverage complete.
- `nemu-downloader.exe` → `ev_05ac9d380774e6253673299a20343aa68b69307315c750f834a626ac5b29ff6a`,
  pe32, 3719160 B, 23 resources / 54 entries, coverage complete.
- Limitation (both): non-icon payloads opaque; icon candidates are table relationships only.

## E7. REA Ghidra on nemu-downloader.exe (question-driven)

- Question answered: "which network endpoints + manifest protocol does the downloader stub use?"
  (triage-first justification; no VMM/display/input binary was available to target instead).
- `open_binary work/mumu-sfx/nemu-downloader.exe` → PE x86.
- `binary_overview` → `ev_14e61ce578299fa8ac15b2c26af00edd871ad16996a73c12ccceeaea55179e7a`;
  image base `0x400000`; 7 segments (Headers 0x400000–0x400400; `.text` 0x401000–0x675000;
  `.rdata` 0x675000–0x715200; `.data` 0x716000–0x735c28; `.rsrc` 0x736000–0x784400;
  `.reloc` 0x785000–0x7a2400; `tdb` 0xffdff000–0xffe00000); **22395 procedures,
  10449 defined strings**. PDB ref `nemu-downloader.pdb`
  GUID `22eb9237-553c-413f-9f65-60c084a40f35` age 0x32, load state unknown.
- `search_strings` literal `api.mumu` → `ev_45dbb9058e78fc8c7bc907e9450d011f983bc28f5049ae27e3aacdcfbb5f6743`
  (authority shipped-artifact, observed), 7 hits:
  0x6cb0c0 + 0x6cb144 `https://api.mumu3.nie.netease.com/api/`,
  0x6cb114 `http://api.mumu3.dev.webcgi.163.com:8400/api/`,
  0x6cb16c `https://api.mumu.nie.netease.com/api/`,
  0x6cb194 `https://api.mumuglobal.com/api/`,
  0x6e797c `https://api.mumu.nie.netease.com/`,
  0x6e79d0 `https://api.mumuglobal.com/`.
- Manual-strings corroboration (same binary): `nemu-api.game163.dev.webapp.163.com:8400`,
  `adl.netease.com/d/g/a11/c/offline_installer_64`, `dns.update.netease.com/easebar`,
  `sentry.netease.com` DSN `02fc884050864ef68fb1d7655e21d3a1@sentry.netease.com/324`,
  manifest keys `packageUrl/apk_id/logoIcoUrl`, methods `fetchDownloadInfo/parseFromJson/doInstall`,
  aria2 sources in sibling `MuMuDownloader.exe`, DISM `Disable-Feature:microsoft-hyper-v-all`
  + `Microsoft-Hyper-V-Hypervisor` in `HyperVChecker.exe`.
- Ghidra limitation: defined-data strings only; empty search ≠ absence; session closed,
  ephemeral DB discarded, originals unmodified.

## E8. SFX payload hashes/dates (for cross-check)

- `config.ini` 356 B (nochannel-mumu12 / zh-Hans / taskid `6a4723215672d839d6deb78a`).
- `nemu-downloader.exe` 3719160 B, `MuMuDownloader.exe` 5956856 B, `skin.zip` 395215 B
  (38 XML layouts incl. 15 `lan_*` languages), `ColaBoxChecker.exe` 4198576 B,
  `HyperVChecker.exe` 120488 B (PE32+ x64), `crashpad_handler.exe` 556024 B,
  `sentry.dll` 228856 B, `7z.exe` 299776 B / `7z.dll` 1148160 B (21.05), VC/UCRT set.
