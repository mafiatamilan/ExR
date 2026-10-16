# MEmu evidence ledger (static, no execution, no network)

Original: `MEmu-Setup-9.5.7.1-hafb3411bb.exe` in repo root (read-only input).

## E1. Input hash

- SHA-256 `83225b53845964b03976c1309c300f2655aee08bf510c22adde71e1b2f954042` (764603136 B).
- PE32 i386 GUI, 7 sections (`.text .rdata .data .qtmetad .qtmimed .rsrc .reloc`),
  linker 14.29, OS/Subsystem 6.0, created 2026-09-16. Version resource: Microvirt
  Software Technology, MEmu Installer 9.5.7.1. SECURITY dir present (21248 B, signed);
  signature contents NOT verified (no verification run).

## E2. Payload container

- innoextract 1.11.0 `--info`: `Not a supported Inno Setup installer` (correct — Qt stub).
- 7z listing: PE with `.rsrc/SETUPZIP/104` = 7z solid, 735312022 B, LZMA2:26/LZMA:20/BCJ2,
  2 blocks, 4514 B headers. Full listing saved outside repo at
  `/tmp/opencode/reatools/memu-list.txt` (349 lines): 234 files + 40 folders,
  2292177559 B uncompressed (2.13 GiB — under the 20 GiB gate, not encrypted,
  free disk ~285 GiB). Extracted with `7z x` to gitignored `work/memu/` (2.2 GiB), exit 0.
- Largest members: system disk VHD 1844.5 MiB, Qt5WebEngineCore 71.6 MiB, data disk VHD
  56.5 MiB, uninstaller 19 MiB, MEmuHyperv64.7z 18.4 MiB, software-GL 15.3 MiB.

## E3. VMM pack (MEmuHyperv64.7z, 19337833 B → 89779707 B, 74 files)

- Extracted to gitignored `work/memu-hyperv/`.
- Renamed VirtualBox family: `MEmuHeadless.exe` (232704 B), `MEmuManage.exe` (1397504 B),
  `MEmuSVC.exe`, `MEmuC.dll`, `MEmuRT.dll`, `MEmuDD(2/U).dll`, `MEmuGuestPropSvc.dll`,
  `MEmuSharedFolders.dll`, `MEmuRecordApi.dll`, `MEmuProxyStub(.Legacy).dll`,
  `MEmuRes.dll`, `MEmuAuth.dll`, `MEmuNetFltNobj.dll`.
- Ring-0 + drivers: `HPVR0.r0` (1569480 B), `MEmuDDR0.r0` (236736 B), `MEmuDrv.sys`
  (374880 B, 2023-02-20) + `MEmuDrv.inf/.cat`, `MEmuNetFlt.sys` (231056 B) + infs/cats,
  `MEmuNetLwf.sys` (252560 B) + inf, `MEmuDrvInst.exe`, `NetFltInstall.exe`, `difxapi.dll`.
- Firmware: `MEmuEFI64.fd` (4194304 B). Hyper-V: `MEmuHPV.dll` (3695872 B),
  `MEmuHyper.exe` (136960 B). Machine default: `.MemuHyperv/MemuHyperv.xml`.
- GPU translators: `libEGL_translator`, `libGLES_CM_translator`, `libGLES_V2_translator`,
  `libOpenglRender(2).dll`, `libEGL/libGLESv2`, `libshadertranslator`, `libxysprt`,
  `d3dcompiler_47.dll`.
- Version evidence (PDB paths in binaries): `E:\develop\memuhyperv-6.1.40\src\MEmu\...`
  (Main/Runtime/Frontends trees, `win.amd64\release` objects); headless banner
  `Maiwei VM MemuHyperv Headless Interface 6.1.40`. Upstream VirtualBox 6.1.40 is the
  matching public release line.

## E4. Machine + Hyper-V configs

- `MEmu/image/96/MEmu.memu`: XML, `xmlns="http://www.memuhyperv.org/"`,
  `version="1.16-windows"`, Machine uuid `{44da1481-...}`, OSType Linux, CPU with
  HardwareVirtEx/NestedPaging/VPID/PAE/X2APIC, `MEmuInternal/Devices/MEmuPipe{Audio,
  Command,Device,Ime,Memud,VInput}` trusted pipes, Display VRAM 12 MB (3D off in
  template), Audio Null/disabled. Guest properties include `enable_su`,
  `graphics_render_mode`, `hardware_opengl`, `latitude/longitude` (Shanghai coords),
  `imei/imsi/operator_*`, `microvirt_vm_{brand,manufacturer,model}`, `phone_layout`.
- `MEmu/image/96/hyperv.json`: HCS definition — HCN network `MEmuSwitch`,
  owner `MEmu_Android_hyperv`, schema 2.1, UEFI SCSI boot disk 2, 4 vCPU / 4096 MB.

## E5. Guest images (read-only userspace inspection)

- `qemu-img info` (all report `vpc`/VHD despite `.vmdk` names): disk1 virtual 2.53 GiB
  (file 1.8 GiB), disk2 virtual 256 GiB (file 56.5 MiB, `qemu-img map` 29 extents —
  mostly sparse), boot.vhd virtual 128 MiB (file 4 MiB).
- Converted disk1 + boot to raw in gitignored `work/` (`disk1.raw` 2.6 GiB,
  `boot.raw` 128 MiB); disk2 NOT converted (256 GiB virtual would balloon).
- disk1 partition table (MBR+GRUB): p1/p2/p3 8 MiB, p5 8 MiB, p6 2.5 GiB Linux.
  p3 (ext2) holds `cmdline`, `kernel`, `kernel-hv` (7414880 B), `ramdisk` (1750024 B).
  Kernel banner: `Linux ... 4.19.71+ (cjw@dev) #846 SMP PREEMPT ... 2026, bzImage x86_64,
  EFI handoff`. Boot cmdline: `init=/init ... androidboot.hardware=intel`.
  ramdisk = gzip→cpio with `init` (2.2 MiB), `init.rc`, `init.intel.rc`,
  `init.superuser.rc` (root), `init.usb*`, `init.zygote64_32`, `ueventd*.rc`, `fstab.intel`.
- p6 (ext4, system-as-root): `build.prop` = Android 9 (release 9, SDK 28, PI, May 2022,
  `cjw` builder), `cpu.abi=x86_64`, abilists include `arm64-v8a/x86/armeabi-v7a`,
  `ro.dalvik.vm.native.bridge=libnb.so`, tablet characteristics.
  ARM bridge: `libhoudini.so` (6.0/7.5 MiB) + `libnb/nbaio/nblog.so`.
  GLES: `libEGL/GLESv1_CM/v2/v3`, `libvulkan`, `egl/libGLES_android.so` (fallback),
  vendor `libGLESv1/v2_enc(2)`, `libOpenglCodecCommon(2)/libOpenglSystemCommon(2)`,
  `lib_renderControl_enc`, `libGoldfishProfiler2`, vendor `egl/lib*EGL*_emulation`,
  `hw/` Treble impls (composer 2.1, allocator 2.0, mapper 2.0, sensors, gnss, camera...).
- boot.vhd p1 is FAT16 (7z-readable): `EFI/boot/bootx64.efi` + `EFI/ubuntu/grub.cfg`
  (`linux (hd0,msdos3)/kernel-hv`, `initrd (hd0,msdos3)/ramdisk`).
- Derived partition copies in `work/` (`disk1.raw/boot.raw/p6.ext4/p3.ext4/bootp1.ext4`)
  are chmod 444 and were opened read-only (debugfs/7z list); originals untouched.

## E6. Manager / CLI / ADB

- `memuc.exe` (PE32 console): `##`-delimited protocol — `CreateVm/CloneVm/RemoveVm/
  RenameVm/GetAllVmInfo/ExportVm/ImportVm`, `installapk/installpkgname/uninstallapk/
  listapps/launchapp/stopapp`, `createfile/removefile/downloadfile/uploadfile`,
  `input/rotate/zoomin/out/accelerometer/network/togglewifi`, `setconfigex/getconfigex`,
  `create vm(44, 51, 71, 76, 90, 96, 120)` variants, `RandomizeDevice/SetBTSInfo`.
- Siblings: `MEmuConsole.exe` (+`.ico`, skins `.rcc`), `MEmu.exe` (4.7 MiB GUI),
  `MemuService.exe`, `MEmuPush.exe`, `MEmuRepair.exe`, `clearRemnants.exe`,
  `screenrecord.exe` + `MEmuRecordApi.dll`, `aapt.exe`, `7za.exe`, `MEmuDef.db` (data).
- ADB: `adb.exe` 5.7 MiB + `AdbWinApi/AdbWinUsbApi.dll` + `adbdrv/` Google USB set
  (android_winusb inf/cat 32+64-bit, WDF co-installers, devcon) + guest
  `persist.service.adb.enable` property slot.
- No `*keymap*/*keyboard*/*.kcm` names anywhere in the 234-file tree.

## E7. Toolchain

- Present host tools only: `file/7z/objdump/strings/qemu-img/debugfs/fuse2fs/fdisk`.
  No new tool installed. No execution, no network, no mounts (user-space reads only).
