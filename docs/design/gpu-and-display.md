# GPU and display (Phase B0 research)

Date: 2026-10-16. Baseline GPU observation: `docs/spec/comparison.md` (encoder-style
GLES passthrough + host-side translators in both local-payload products).

## Options

### A. virtio-gpu + virglrenderer (OpenGL path)

- Guest stack: kernel KMS driver + Mesa virgl 3D driver; host renders via OpenGL.
  Source: https://docs.mesa3d.org/drivers/virgl.html — VERIFIED. Venus (Vulkan over
  virtio-gpu) also exists in Mesa (same docs site — UNVERIFIED detail).
- virglrenderer licence: UNVERIFIED (canonical repo bot-walled at fetch time;
  pointer: https://gitlab.freedesktop.org/virgl/virglrenderer). Re-verify before M3;
  do not vendor it until then.
- Compat: broad Linux-guest support; Android-guest virgl story is thinner than the
  emulator-style encoder stacks. Latency: in-process GPU, low. Effort: MEDIUM
  (QEMU flags + guest driver presence in chosen image).

### B. virtio-gpu + gfxstream (GLES/Vulkan streaming path)

- Streaming kit: guest→host rendering APIs; used by Android virtual-device tooling
  (Goldfish/Cuttlefish host backends). Source: https://github.com/google/gfxstream
  (README) — VERIFIED. Licence Apache 2.0: full text fetched from
  https://raw.githubusercontent.com/google/gfxstream/main/LICENSE — VERIFIED.
- Closer to the observed baseline (encoder-style GLES in guests). Needs a guest
  image carrying the gfxstream guest stack (Cuttlefish-style) or Android-x86 with
  compatible GL — verification task for M3, not assumed here.
- Latency: low (local transport). Effort: MEDIUM-HIGH (backend wiring + image fit).

### C. scrcpy-style stream (separate display/input plane)

- Mirrors an already-running Android over ADB: H.264/H.265 video (+audio on new
  Android), 30–120fps, 35–70ms latency, keyboard/mouse/gamepad/HID control, no root,
  Win/Linux/macOS. Source: https://raw.githubusercontent.com/Genymobile/scrcpy/master/README.md
  (v5.0.1) — VERIFIED. Licence Apache 2.0 (same source) — VERIFIED.
- Role in ExR: NOT the primary display (we own the VM window), but the reference
  design for (1) an M3 fallback if virtio-gpu stumbles, and (2) the input-injection
  and screen-record protocol shapes in `emu-display`/`emu-input`. No scrcpy code is
  vendored; protocols are reimplemented from its docs against our own ADB client.

## Tradeoffs

| | virgl | gfxstream | scrcpy-style |
|---|---|---|---|
| Latency | low | low | 35–70ms (VERIFIED figure) |
| Guest fit (Android-x86 9) | MEDIUM (UNVERIFIED) | MEDIUM (UNVERIFIED) | HIGH (any ADB Android) |
| Host GPU need | YES (GL-capable) | YES | NO (decodes stream) |
| Effort | MEDIUM | MEDIUM-HIGH | LOW (fallback only) |
| Licence friction | re-verify | Apache-2.0 ✓ | Apache-2.0 ✓ |

## Proposed first-milestone path (M1–M3)

- M1 (headless): no display at all — boot to serial/ADB proof. De-risks CI.
- M2: ADB plane up (install/shell/forward) — control before pixels.
- M3: window via virtio-gpu + virgl first (simplest QEMU-native path on Arch/KVM);
  fall back to gfxstream if the Android-x86 guest stack mismatches; scrcpy-style
  streaming stays a documented fallback, implemented only if both stall.
- Configurable resolution/FPS/DPI lands in M3 as display-server parameters, not as
  guest hacks (guest props are user data, not our code).
