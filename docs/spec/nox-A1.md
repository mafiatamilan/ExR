# Nox A1 — unpack + triage report

## a. TL;DR

- What: Duodian NoxPlayer 7.0.6.2 full offline installer (custom stub + 7z payload, 439 files, 0.9 GiB).
- Installer type: proprietary stub (Duodian, signed) with the payload as a solid 7z block in the PE resource fork.
- Payload type: complete emulator tree — VirtualBox-derived VMM, four versioned base configs (only Android 7.1 ships disks), ADB pair, console CLI, multi-player manager.
- Core-stack hypothesis CONFIRMED at high confidence: VMM is VirtualBox-derived (genuine VBox configs + custom PCI passthrough device); guest is Android 7.1.2 32-bit x86 with Houdini ARM bridge.
- Gaps: Nox-side VMM version number unrecovered; keymap format not found; no native Hyper-V mode (helper script disables Hyper-V instead).

## b. Layer tree

```text
nox_setup_v7.0.6.2_full_intl.exe (572 MiB, Duodian stub 7.0.6.2, signed)
└── .rsrc/2052/APP/BIGNOXLITE: solid 7z (LZMA2/BCJ2, 593 MiB → 0.9 GiB, 439 files + 33 dirs)
    ├── bin/ — Nox GUI, NoxConsole CLI, MultiPlayerManager, nox_adb+adb, aapt pair,
    │   Qt5 + WebKit/CEF stack, media plugins, updater, bridge/debug helpers, .bat tools
    ├── bin/base{,4,5,7,12}.vbox — versioned machine configs (only base7 ships disks)
    ├── bin/base7-disk (7z) — guest disks: 953 MiB system VMDK + 111 MiB data VMDK + machine config
    ├── bin/base-disk2.vmdk — shared data disk image (44.6 MiB)
    ├── bin/adb.7z + bin/egl.7z — pinned 2017 ADB set and ANGLE GLES set
    ├── noxvm/NOXVM.exe — the VMM engine, single 44 MiB binary
    └── plugins/ — Qt platform/media plugin tree
```

## c. Component inventory

| Component | Role | Evidence ref | Confidence |
|---|---|---|---|
| Installer stub | Installs the tree (Duodian 7.0.6.2) | PE version resource, payload resource fork | CONFIRMED |
| NOXVM engine | VirtualBox-derived VMM in one binary | Genuine VBox machine configs, passthrough device, VBox-flavored strings | CONFIRMED |
| `noxptdevice` PCI device | Guest/host channel (input, display, control plane) | Machine config device entries + host driver path | CONFIRMED |
| Versioned base configs | Android 4/5/7/12 machine templates; only 7.1 has local disks | Five `.vbox` files, single disk archive | CONFIRMED |
| Guest system image | Android 7.1.2 (Nougat, SDK 25) 32-bit x86, GRUB boot, kernel 4.0 custom | Partition table, build properties, kernel banner | CONFIRMED |
| ARM translation | Houdini bridge (32-bit), ARMv7 ABIs advertised | Bridge + Houdini lib names, ABI list | CONFIRMED |
| GPU path | Host ANGLE GLES set + guest GLES libs | Pinned ANGLE archive, guest GL lib names | CONFIRMED |
| ADB pair | `adb.exe` + `nox_adb.exe` + API libs (2017 vintage) | ADB binaries, pinned ADB archive | CONFIRMED |
| NoxConsole CLI | Scriptable manager (`-name/-index` + lifecycle/app/backup verbs) | CLI verb strings with argument syntax | CONFIRMED |
| MultiPlayerManager | Multi-instance GUI front end | Manager binary + web UI resource tree | CONFIRMED |
| Helpers | Hyper-V disabler, debugger-based tools, network-bridge installer, APK packagers | Batch contents, tool strings, packager binaries | CONFIRMED |
| Keymap format | No keymap-named files in 439-file tree | Empty filename search | UNKNOWN |
| Macro recorder | `naction` console binary, role unconfirmed from strings | Binary presence only | HYPOTHESIS |

## d. A3 checklist

- Hypervisor/VMM: CONFIRMED (VirtualBox-derived + custom passthrough device).
  Reason: unrenamed VBox config namespace/schema, Nox PCI device entries, VMM binary strings.
- Android base / ABI / ARM translation: CONFIRMED (7.1.2, x86 32-bit, Houdini bridge).
  Reason: build properties (SDK 25, empty 64-bit ABI list, S8 model spoof), bridge/Houdini names.
- GPU path: CONFIRMED pattern (host ANGLE + guest GLES).
  Reason: pinned ANGLE lib set, guest EGL/GLES/Vulkan names; encoder-stack shape differs
  from MEmu (no `_enc` stack in vendor tree).
- Display/stream: HYPOTHESIS (framebuffer via passthrough/GUI window).
  Reason: passthrough device + GUI exist; frame protocol unobserved statically.
- Input + keymap format: PARTIAL (HID config + console verbs observed; file format UNKNOWN).
  Reason: USB-tablet/PS2 HID in machine config; no keymap file found.
- ADB: CONFIRMED (dual clients + pinned set). Reason: binaries + archive; ports unobserved.
- Multi-instance/CLI: CONFIRMED (console CLI + multi-player GUI + per-version configs).
  Reason: verb set with instance selectors, manager binary, versioned templates.
- Extras: PARTIAL (APK packaging via aapt pair CONFIRMED; macro recorder HYPOTHESIS;
  GPS/root/sensors not inventoried in Nox scope).

## e. Open questions and blockers

1. VMM exact upstream version unrecovered (version values need deeper resource parsing) —
   non-blocking for comparison.
2. No native Hyper-V mode: the helper disables Hyper-V/WSL/VMP via DISM+bcdedit instead.
   No HCS-style definition found — recorded as absence, not a gap to chase.
3. Only Android 7.1 ships disks; 4/5/12 configs are diskless templates (download-on-demand
   presumed, not verified — no network contact made).
4. No blockers: no new tool needed; nothing executed; disk2-style sparse image
   (119 GiB virtual) left unconverted deliberately.
