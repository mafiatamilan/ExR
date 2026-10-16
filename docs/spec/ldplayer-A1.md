# LDPlayer A1 — stub-only report (no payload download)

## a. TL;DR

- What: JUST OKAY `LDPlayer9_ens_1001_ld` 1.3 MiB downloader stub ("ldsmaller") for LDPlayer 9 — a GUI downloader, not an installer.
- Installer type: WinINet-based multi-part downloader with 20-language skin, channel/version gates; real payload fetched at runtime.
- Payload type: NOT obtained (per instructions). Two complete static package URLs reference `LDPlayer_9.2.0.1.exe` on the vendor CDN; remaining endpoints are runtime-assembled fragments.
- Core-stack hypothesis: NONE offered — a 1.3 MiB stub cannot answer VMM/Android/GPU/ADB questions.
- Gaps: everything emulator-core is UNKNOWN; payload acquisition is your decision.

## b. Layer tree

```text
LDPlayer9_ens_1001_ld.exe (1.3 MiB, JUST OKAY, signed, VS2019 linker)
├── PE imports: WinINet + WinSock (downloader), GUI stack, adapter-info API
└── .rsrc/ZIPRES 7z (219 KiB → 65 files): downloader skin only —
    PNGs, message-box/main-frame XML, 20-language string table. No binaries.
```

Real payload (`LDPlayer_9.2.0.1.exe`, ~hundreds of MiB presumed) is NOT in this artifact.

## c. Component inventory

| Component | Role | Evidence ref | Confidence |
|---|---|---|---|
| Stub executable | Multi-part HTTP downloader + install wizard | Imports, downloader class/method names, PDB path | CONFIRMED |
| Skin archive | 20-language installer UI (incl. LDPlayerX naming) | Extracted PNG/XML/TSV set | CONFIRMED |
| Package URLs | Two complete static URLs for the full installer on vendor CDN | UTF-16 string extraction | CONFIRMED |
| Runtime URL builder | Remaining endpoints stored as fragments, assembled at runtime | Adjacent fixed-width config fields | CONFIRMED |
| Channel/version gate | `mnq_version`-style channel tag + version checks | Behavior strings (shared `mnq` tag with MuMu bundle noted) | CONFIRMED |
| Emulator core | No binaries, images, drivers, or configs in stub scope | Empty searches over stub + skin | UNKNOWN |

## d. A3 checklist

- Hypervisor/VMM: UNKNOWN. Reason: stub-only; no VMM binary, driver, or config present.
- Android base / ABI / ARM translation: UNKNOWN. Reason: stub-only.
- GPU path: UNKNOWN. Reason: stub-only.
- Display/stream: UNKNOWN. Reason: stub-only.
- Input + keymap format: UNKNOWN. Reason: stub-only.
- ADB: UNKNOWN. Reason: stub-only.
- Multi-instance/CLI: UNKNOWN. Reason: stub-only.
- Extras: UNKNOWN. Reason: stub-only (32-bit-OS incompatibility message noted as installer behavior only).

## e. Open questions and blockers

1. Payload acquisition is YOUR decision (per instructions, nothing downloaded). Static
   package URLs are in the evidence ledger as inert data.
2. The `mnq` channel tag shared with the MuMu MSI filename is unexplained — possible
   shared portal/channel infra. Recorded, not investigated (no network contact).
3. No blockers on my side; LDPlayer stays UNKNOWN until you provide the payload.
