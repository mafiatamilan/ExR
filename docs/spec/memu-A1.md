# MEmu A1 — unpack + triage report

## a. TL;DR

- What: Microvirt MEmu 9.5.7.1 full offline installer (Qt stub + 7z payload, 234 files, 2.1 GiB) — the opposite of MuMu: everything ships locally.
- Installer type: custom Qt/WebEngine stub (Microvirt, signed) with the payload as a solid 7z block in the PE resource section.
- Payload type: complete emulator tree — VirtualBox-fork VMM, dual Hyper-V mode, Android 9 guest images, ADB set, scriptable CLI, manager GUI.
- Core-stack hypothesis CONFIRMED at high confidence: VMM is a VirtualBox 6.1.40 fork ("MemuHyperv", renamed binaries/config/PDB paths); guest is Android 9 x86_64 with Houdini-style ARM translation and emulator GLES passthrough.
- Gaps: keymap file format not found; installer stub internals unexamined (installer, not emulator).

## b. Layer tree

```text
MEmu-Setup-9.5.7.1-hafb3411bb.exe (730 MiB, Qt stub by Microvirt 9.5.7.1, signed)
└── .rsrc/SETUPZIP/104: solid 7z (LZMA2/BCJ2, 735 MiB → 2.1 GiB, 234 files + 40 dirs)
    ├── MEmu/ — manager GUI, console, CLI, service, ADB set, Qt5 stack, skins, uninstaller
    ├── MEmu/image/96/ — guest: UEFI boot image, 2.5 GiB system VHD, 256 GiB sparse data VHD,
    │   Hyper-V HCS definition, VirtualBox-fork machine config
    ├── MEmuHyperv32.7z / MEmuHyperv64.7z — VMM packs: fork binaries, ring-0 modules,
    │   host drivers (.sys/.inf/.cat), UEFI firmware, ANGLE/GL translators
    └── adbdrv/ — Google USB drivers (.inf/.cat + WDF co-installers, 32/64-bit) + devcon
```

## c. Component inventory

| Component | Role | Evidence ref | Confidence |
|---|---|---|---|
| Qt installer stub | Installs the tree, VT/GPU/ARM preflight messages | PE sections (Qt metadata), version resource, message strings | CONFIRMED |
| MemuHyperv VMM pack | VirtualBox 6.1.40 fork: headless VM runner, manager CLI, COM service, runtime, device/DD packs, guest-prop service, record/shared-folder APIs | Renamed binary set, PDB source paths, headless banner string, machine config schema | CONFIRMED |
| Host drivers | Kernel + network filter drivers with signed catalogs | `.sys` + `.inf` + `.cat` file set in VMM pack | CONFIRMED |
| Hyper-V mode | Native Hyper-V VM definition + provider DLL | HCS JSON (virtual switch, UEFI boot, vCPU/RAM), provider binary | CONFIRMED |
| Guest system image | Android 9 (Pie, SDK 28) x86_64, GRUB+UEFI boot, kernel 4.19 custom | Partition table, build properties, kernel banner, boot config | CONFIRMED |
| ARM translation | Houdini-style bridge (`libhoudini` + `libnb` native bridge), ARM ABIs advertised | Guest lib names + ABI list property | CONFIRMED |
| GPU path | Emulator GLES encoder stack in guest + ANGLE/GL translators + render libs on host | Guest encoder lib names, host translator/render lib names | CONFIRMED |
| ADB set | Standard ADB client + Windows USB/API libs + Google USB driver package | ADB binary, API libs, driver `.inf`/`.cat` | CONFIRMED |
| memuc CLI | Scriptable manager: VM lifecycle, clone/import/export, APK ops, app control, input, sensors, network, files | `##`-delimited command protocol strings | CONFIRMED |
| MEmuConsole GUI | Multi-instance manager front end | Manager binary + skin/resource tree | CONFIRMED |
| Extras | Screen recorder + API, root toggle, GPS/IMEI/operator spoofing props, shared folders | Recorder binary/API, guest properties, superuser init fragment | CONFIRMED |
| Keymap format | No keymap-named files anywhere in payload | Empty filename search over 234 files | UNKNOWN |

## d. A3 checklist

- Hypervisor/VMM: CONFIRMED (VirtualBox 6.1.40 fork "MemuHyperv" + native Hyper-V mode).
  Reason: renamed VBox binary family, fork PDB paths with upstream version, VBox-schema
  machine config with renamed devices, HCS Hyper-V definition + provider DLL.
- Android base / ABI / ARM translation: CONFIRMED (Android 9, x86_64 primary, Houdini-style bridge).
  Reason: build properties (SDK 28, ABI lists), bridge + Houdini lib names, 64/32-bit zygote config.
- GPU path: CONFIRMED (emulator GLES encoder passthrough + host ANGLE/GL render).
  Reason: guest encoder/codec lib names, host translator + render lib names, hardware-GL guest property.
- Display/stream: HYPOTHESIS (guest framebuffer via VMM display device to host window).
  Reason: VMM display-device code present, but no frame-protocol observation; window path untested.
- Input + keymap format: PARTIAL (CLI `input` + pipe-device names observed; file format UNKNOWN).
  Reason: input command + guest pipe-device names exist, but no keymap file found to document a format.
- ADB: CONFIRMED (client + drivers + enablement property). Reason: binary + driver package +
  guest ADB property; ports unobserved (no running guest).
- Multi-instance/CLI: CONFIRMED (manager GUI + `##` CLI protocol + per-VM config).
  Reason: manager binary, full lifecycle/clone/snapshot-adjacent command set, XML machine config.
- Extras: CONFIRMED (recorder, root toggle, GPS/ID spoofing, shared folders).
  Reason: recorder binary/API, superuser init + enable flag, geo/identity guest properties.

## e. Open questions and blockers

1. Keymap format unanswered — likely inside the manager GUI resources or guest data image;
   needs a follow-up pass (resource/config hunt), not a blocker for comparison.
2. Display frame path and ADB ports need a running guest — out of scope for static Phase A.
3. The `96` in `image/96` and the `44..120` VM-create variants are unexplained identifiers —
   recorded as-is, no guess.
4. No blockers: no new tool was needed (qemu-img/debugfs/7z all present); nothing executed.
