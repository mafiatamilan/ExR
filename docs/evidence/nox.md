# Nox evidence ledger (static, no execution, no network)

Original: `nox_setup_v7.0.6.2_full_intl.exe` in repo root (read-only input).

## E1. Input hash

- SHA-256 `7995ce172f7c25caf508755e412bbd69b9d905b945ea805dead6106846177953` (598932728 B).
- PE32 i386 GUI, 5 sections, linker 14.0 (VS2015), OS/Subsystem 6.0, created 2025-03-14.
  Version resource: Duodian Technology, NoxPlayer Installer 7.0.6.2. SECURITY dir
  present (12024 B, signed); signature contents NOT verified.

## E2. Payload container

- innoextract 1.11.0 `--info`: `Not a supported Inno Setup installer` (correct).
- 7z listing: PE with `.rsrc/2052/APP/BIGNOXLITE` = 7z solid, 593062165 B, LZMA2:26/
  LZMA:20/BCJ2, 2 blocks, 7191 B headers. Full listing outside repo at
  `/tmp/opencode/reatools/nox-list.txt`: 439 files + 33 folders, 983483233 B
  (0.92 GiB — under the 20 GiB gate, not encrypted, free disk ~285 GiB).
  Extracted with `7z x` to gitignored `work/nox/` (939 MiB), exit 0.
- Largest: `bin/base7-disk` 395.1 MiB, `bin/libcef.dll` 68.4 MiB,
  `bin/libQt5WebKit.dll` 50.8 MiB, `bin/base-disk2.vmdk` 44.6 MiB,
  `noxvm/NOXVM.exe` 43.6 MiB.

## E3. VMM + machine configs

- `noxvm/` holds ONLY `NOXVM.exe` (44 MiB, PE32 i386 GUI 5.01): single-binary VMM.
  Strings: `vbox v is %s` marker; version values not recovered (keys only in pass).
- `bin/base{,4,5,7,12}.vbox` (+`pipe/base4.vbox`): genuine VirtualBox XML
  (`xmlns="http://www.innotek.de/VirtualBox-settings"`, `version="1.12-windows"`,
  VBoxManage comment intact). `base7` (uuid `d79ffde7-...`): 2 vCPU (VirtEx/NestedPaging/
  VPID/UX, PAE, LargePages), 1536 MB PageFusion, ICH9, USBTablet+PS2Keyboard, VRAM 32 MB
  (3D off in template), USB on, VRDE off (3389). Custom device `noxptdevice`
  (PCI bus 0/dev 6, Trusted) with PDM path `...\Bignox\BigNoxVM\RT/NOXa.dll`
  (that DLL is NOT in the payload — install-time/runtime path, recorded absent).
- `disableHV.bat`: DISM disable of `Containers-DisposableClientVM/Microsoft-Hyper-V-All/
  VirtualMachinePlatform/HypervisorPlatform/Windows-Defender-ApplicationGuard/
  Microsoft-Windows-Subsystem-Linux` + `bcdedit /set hypervisorlaunchtype off`.
  No HCS JSON / Hyper-V provider DLL anywhere in tree (searched names).

## E4. Guest images (read-only userspace inspection)

- `bin/base7-disk` is itself 7z (LZMA2:24, 414253046 B → 1064966132 B): `base7-disk1.vmdk`
  953483264 B + `base7-disk2.vmdk` 111476736 B + `base7.vbox` 6132 B. Extracted to
  gitignored `work/nox-guest/`.
- `qemu-img info` (real `vmdk`): disk1 virtual 2.03 GiB (909 MiB), disk2 virtual
  119 GiB (106 MiB sparse — NOT converted, would balloon).
- disk1 converted to `work/nox-disk1.raw` (2.0 GiB): MBR, p1/p2/p5 8 MiB, p3 6.1 MiB,
  p6 2 GiB Linux (same Android-x86 scheme as MEmu).
- p3 (ext2): `cmdline` (`init=/init ... cpuidle.off=1 androidboot.console=tty0`),
  `kernel` (4669984 B), `ramdisk` (1668876 B). Kernel banner: `4.0.9+ (sam@topaz0)
  #166 SMP PREEMPT ... Jan 2024`.
- p6 (ext4, system-as-root, no `lib64`): `build.prop` = Android 7.1.2 (N2G48H, SDK 25,
  Feb 2025, builder `dpi`), `cpu.abi=x86`, `abilist=x86,armeabi-v7a,armeabi`,
  `abilist64=` EMPTY (32-bit), model `SM-G9550`. ARM: `libhoudini.so` (4.9 MiB, 2021)
  + `libnb/nbaio.so`. GLES: `libEGL/GLESv1_CM/v2/v3`, `libvulkan`; vendor tree has
  `etc+lib` only, NO `_enc`/renderControl/Goldfish stack (differs from MEmu).
- `bin/base-disk2.vmdk` (44.6 MiB): shared data disk, not opened further.
- Derived copies (`nox-disk1.raw/noxp6.ext4/noxp3.ext4`, ramdisk/kernel extracts in
  `/tmp/opencode/reatools/noxp3`) are chmod 444 / read-only opens; originals untouched.

## E5. Manager / CLI / ADB / extras

- `bin/NoxConsole.exe`: `-name:nox_name | -index:nox_index` selectors; verbs `backup/
  restore/remove/clone/add/import/launch/quit(all)/reboot/runapp/installapp/uninstallapp/
  killapp(all)` (+`shell am/pm` pass-throughs, `tasklist` polling, cert strings).
- `bin/MultiPlayerManager.exe` + `bin/resources/dist*` web UI + `Nox.exe` (10.9 MiB GUI).
- ADB: `bin/adb.exe` + `bin/nox_adb.exe` + `AdbWinApi/AdbWinUsbApi.dll`; `bin/adb.7z`
  pins 2017 set (adb.exe 828672 B + nox_adb.exe + API libs).
- GPU host: `bin/egl.7z` pins 2017 ANGLE (`libEGL.dll` 135168 B + `libGLESv2.dll` 4181504 B).
- APK: `bin/aapt.exe` + `bin/aapt_x.exe` (5.2 MiB). Debug: `kntd32.exe` (40 KiB PE32 GUI)
  + `kntd64.exe` (63 KiB PE32+): cdb-wrapper strings (`cdb -p`, CONIN/OUT, dispatch
  callbacks) — debugger helpers, NOT drivers. Network: `NetBridgeInstaller.exe`,
  `AudioRouter.exe`, `firewall_mgr.dll`, `audio_sniffer.dll`. Macro: `naction.exe`
  (PE32 console) — no macro/record strings in first pass, role unconfirmed.
- No `*keymap*/*keyboard*/*.kcm` names in the 439-file tree.

## E6. Toolchain

- Present host tools only: `file/7z/objdump/strings/qemu-img/fdisk/debugfs`.
  innoextract used once for the negative `--info` probe. No new tool installed.
  No execution, no network, no mounts.
