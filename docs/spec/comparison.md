# Comparison — MuMu / MEmu / Nox / LDPlayer (Phase A)

Scope: static analysis on Linux only. Nothing executed, no network contact.

## Feature table (confidence per cell)

| Feature | MuMu (`mumu-mnq-win`) | MEmu 9.5.7.1 | Nox 7.0.6.2 | LDPlayer stub |
|---|---|---|---|---|
| Delivery | MSI + downloader stub + bundle payload | Full offline 7z bundle | Full offline 7z bundle | Downloader stub only |
| Installer tech | Advanced Installer MSI + Inno 6.7.0 bundle | Custom Qt stub, 7z-in-PE | Proprietary stub, 7z-in-PE | WinINet stub, skin-only 7z |
| VMM | UNKNOWN (offline has none) | VirtualBox 6.1.40 fork — CONFIRMED | VirtualBox-derived, passthrough device — CONFIRMED | UNKNOWN (stub-only) |
| Hyper-V stance | Disable-via-DISM helper — CONFIRMED | Native HCS mode + VBox mode — CONFIRMED | Disable-via-DISM/bcdedit, no native mode — CONFIRMED | UNKNOWN |
| Android guest | UNKNOWN offline (downloader-fetched) | 9 / x86_64 / kernel 4.19 — CONFIRMED | 7.1.2 / x86 32-bit / kernel 4.0 — CONFIRMED | UNKNOWN |
| ARM translation | UNKNOWN | Houdini-style + nb bridge — CONFIRMED | Houdini + nb bridge (32-bit) — CONFIRMED | UNKNOWN |
| GPU path | UNKNOWN | GLES-encoder passthrough + ANGLE — CONFIRMED | Host ANGLE + guest GLES — CONFIRMED | UNKNOWN |
| ADB | UNKNOWN | Client + Google USB drivers — CONFIRMED | Dual clients, pinned 2017 set — CONFIRMED | UNKNOWN |
| Manager / CLI | UNKNOWN | GUI + `##` protocol CLI — CONFIRMED | GUI + `-name/-index` CLI — CONFIRMED | UNKNOWN |
| Keymap format | UNKNOWN | UNKNOWN (no files) | UNKNOWN (no files) | UNKNOWN |
| Extras | Downloader APK staging only | Recorder, root, GPS/ID spoof, shared folders — CONFIRMED | aapt packaging; macro HYPOTHESIS | UNKNOWN |
| Guest net | UNKNOWN | NAT-style + filter drivers (names only) | Bridge installer (role unconfirmed) | UNKNOWN |
| Model spoof | UNKNOWN | `microvirt_vm_*` props (empty template) | `SM-G9550` build prop — CONFIRMED | UNKNOWN |

## Common baseline (all products with local payloads)

- x86-system-image Android (MEmu 9/x86_64, Nox 7.1/x86) with Intel-bridge ARM translation
  (`libhoudini` + `libnb` in both) — same translation family, HIGH confidence.
- VirtualBox-derived VMM in MEmu AND Nox (renamed fork vs unrenamed configs + custom
  PCI device) — same VMM family, HIGH confidence.
- Hyper-V conflict handling everywhere (DISM helpers in MuMu/Nox, dual-mode in MEmu) —
  shared platform constraint, HIGH confidence.
- ADB client + scriptable manager CLI in MEmu AND Nox — same management shape, HIGH.
- Bundled Qt/CEF UI stacks and media plugins — commodity, MEDIUM.

## Differentiators

- MuMu: only product with NO offline emulator (downloader-fetch + portal bundleware);
  MSI-based delivery is unique in this set.
- MEmu: only 64-bit guest, only UEFI boot, only native Hyper-V mode, richest extras.
- Nox: only 32-bit guest, only multi-Android template set (4/5/7/12 configs, one ships
  disks), only debugger-helper tools, only model-spoof build prop.
- LDPlayer: only pure-stub artifact; only 20-language skin; only runtime-assembled URLs.

## What we could not learn

- MuMu: the entire emulator core (VMM, Android version/ABI, GPU, ADB, manager, keymaps) —
  offline payload is third-party bundleware; truth lives behind the downloader manifest
  (network-adjacent, needs your direction). LDPlayer: likewise everything core — stub-only.
- All: keymap file formats (no keymap files in MEmu/Nox trees — likely generated at
  runtime or stored per-instance outside the installer); display frame protocol and ADB
  ports (need a running guest); exact upstream VBox version for Nox; guest data-image
  contents (sparse 119–256 GiB images deliberately unconverted).
- Signatures: PE-hash-vs-signature binding and revocation/reputation NOT evaluated
  anywhere (your lookups to do).

## Recommendation for the gaps

1. Keymaps: grep installed per-instance state (not installer) later — propose a disposable
   Windows VM with snapshotted guest, or parse manager GUI resources for the schema.
   Do NOT run anything on this host.
2. MuMu/LDPlayer cores: you decide acquisition (MuMu manifest fetch / LDPlayer package
   download). Until then, treat MEmu+Nox as the design baseline — they agree on VMM
   family, translation stack, ADB shape, and CLI-driven management.
3. Display/ADB-ports: requires a booted guest under observation — Phase B headless-guest
   work, not static Phase A.
4. No further static effort recommended on bundleware internals (MuMu `Setup.exe`
   blobs, LDPlayer URL fragments) — out of scope for the clone design.
