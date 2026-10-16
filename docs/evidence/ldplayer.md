# LDPlayer evidence ledger (static, no execution, no network, no download)

Original: `LDPlayer9_ens_1001_ld.exe` in repo root (read-only input).

## E1. Input hash

- SHA-256 `e76e6696685037da3f115cf182877bf5901fef1db0b0a5aeafb810e9b0766440` (1294312 B).
- PE32 i386 GUI, 6 sections, linker 14.29 (VS2019), OS/Subsystem 6.0, created 2026-09-23.
  Version resource: JUST OKAY LIMITED 2016–2026, LDPlayer 1.0.0.1/1.0.0.6. SECURITY dir
  present (20968 B, signed); signature contents NOT verified.

## E2. Stub imports (objdump)

- `ADVAPI32 COMCTL32 GDI32 gdiplus IMM32 IPHLPAPI KERNEL32 ole32 OLEAUT32 SHELL32
  SHLWAPI USER32 WININET WS2_32` — downloader profile (WinINet/WinSock + GUI +
  adapter-info), no virtualization/driver APIs.

## E3. Skin archive (ZIPRES 7z, 218643 B → 65 files, 520 KiB)

- Extracted with `7z x` to gitignored `work/ldplayer/` (exit 0). Contents: downloader
  PNGs, `mainframe.xml`, `common/messagebox.xml`, `language.tsv` (20 locales;
  product named `LDPlayer` / downloader `LDPlayerX Installer`; 32-bit-OS
  incompatibility message; discord/facebook/vk/naver community links). No binaries.

## E4. Downloader behavior strings (inert data)

- Classes/methods: `CHttpClient/CHttpDownloader` (+`...2FileSink/...2MemSink`),
  `LDMultiDownloadItemInfo`, `DownloaderBox`, `XZDownloaderErrorBox`,
  `XZInstallErrorBox`; fields `download_url/app_download_url/downloader_package/
  downloader_version/channelOpenId/mnq_version`; PDB
  `D:\work\downloader_en_2026_1\ldsmaller\bin\ldsmaller.pdb` (name `ldsmaller`).
- WinINet API set: `HttpOpenRequestW/HttpSendRequestW/HttpQueryInfoW`.

## E5. Static URLs (inert — never fetched)

- Complete: `https://res.ldrescdn.com/download/package/LDPlayer_9.2.0.1.exe`,
  `https://static.ldrescdn.com/download/package/LDPlayer_9.2.0.1.exe`.
- Fragments (fixed-width config fields, runtime-assembled): `https://api`,
  `https://files.l`, `https://res.ldre`, `https://sg-datahub.chan`, `https://www.ld`,
  plus adjacent `cdn./player_files/ope…` path parts and a `Software\Xuan…` vendor-key
  prefix (field-truncated in hexdump at file offset `0xbce64`).
- Cert-chain URLs (DigiCert CRL/OCSP/cacerts) belong to the Authenticode signature,
  not to product behavior.

## E6. Toolchain

- Present host tools only: `file/7z/objdump/strings/python3`. No new tool installed.
  No execution, no network contact, no payload download.
