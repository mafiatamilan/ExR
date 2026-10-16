# Guest images (Phase B0 research)

Date: 2026-10-16. Baseline: `docs/spec/comparison.md` (MEmu: Android 9/x86_64;
Nox: Android 7.1.2/x86-32; both with bridge-style ARM translation, NOT shipped).

## Open images (version / ABI)

- Android-x86: newest stable is 9.0-r2 (Pie), then 8.1-r6, 7.1-r5.
  Source: https://www.android-x86.org/releases.html — VERIFIED.
  Matches the MEmu baseline (Android 9) directly; Nox's 7.1 also covered.
  Licence: Apache 2.0 (project standard — UNVERIFIED, confirm from image sources
  before M1; pointer: https://www.android-x86.org/source.html).
- Bliss OS (newer Android on x86): public downloadable images are PAUSED; source
  trees/installer/kernels still active. Source: https://blissos.org/ — VERIFIED.
  Implication: do NOT plan around Bliss ISOs; track source only.
- Waydroid images: LineageOS-based Android 13, container (LXC) model, GNU/Linux only.
  Source: https://docs.waydro.id/ — VERIFIED. Not directly bootable under QEMU
  (container images, binder-dependent); useful as a build reference, not as M1 images.
- AOSP Cuttlefish: virtual Android device for KVM hosts (crosvm/QEMU backends per
  crosvm docs). UNVERIFIED detail (not fetched); evaluate as the Android 12+ path
  after M1. Pointer: https://github.com/google/android-cuttlefish.

## What cannot be redistributed

- Google Apps / Play Store: proprietary; must never be bundled or fetched by our
  tooling. MicroG (open re-implementation) is the documented alternative for users
  who need push/account shims — UNVERIFIED detail, pointer: https://microg.org.
- ARM translators (vendor bridge libraries of the observed family): proprietary,
  device/system-image-bound; must never be copied, bundled, or downloaded by us.

## User-supplied-translator mechanism (design)

- `emu-image` manages ONLY open images (fetch + SHA-256 verify + local registry).
- Translators are user-supplied files: the user points config at a local path
  (extracted by the user from hardware they own); ExR verifies presence + records
  the file hash in the instance manifest, and bind-mounts/injects it into the guest
  image copy at instance creation. ExR never downloads, hosts, or names a source.
- If no translator is configured, the instance is created x86-only and ARM-only apps
  are reported as uninstallable/unrunnable (explicit status, not silent failure).
- Same pattern covers GApps/MicroG: user-supplied packages only, installed via
  `emu-adb` after first boot, recorded in the instance manifest.

## Image handling rules

- Every image: fetch over HTTPS from the project publisher, SHA-256 pinned in
  `emu-image`, verified before first use; qcow2 overlays per instance (base image
  stays read-only; clones/snapshots are overlay operations — maps to QEMU block
  layer, see `vmm-options.md`).
- No vendor images, no GApps, no translators in this repo or our releases — enforced
  by review + the licensing register.
