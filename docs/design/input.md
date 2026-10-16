# Input (Phase B0 research + keymap format)

Date: 2026-10-16. Baseline: `docs/spec/comparison.md` (input planes observed by
name only — pipe devices, console verbs; NO keymap file format recovered anywhere).

## Injection options

- virtio-input (keyboard/tablet/multitouch passthrough into the guest): the
  architecturally clean path under QEMU; host-side events from winit/gamepad APIs,
  guest-side standard Linux input. Guest kernel support required in the chosen image
  (verify in M4 — UNVERIFIED). Effort: MEDIUM.
- uinput on the host + guest-side agent: alternative if virtio-input is missing from
  the guest kernel; heavier (needs a tiny guest-side open-source agent over ADB).
  Fallback, not primary.
- ADB input commands (`input tap/swipe/keyevent`) and scrcpy-style injection: always
  available once `emu-adb` is up; coarse (no multitouch gestures, higher latency) but
  universal. Use for M4 bring-up tests and as fallback. scrcpy documents keyboard,
  mouse, and gamepad control shapes (source in `gpu-and-display.md` — VERIFIED).
- Mouse-look (FPS-style pointer lock): host pointer-capture in the window (winit)
  mapped to relative-move injection; needs the same path as multitouch (primary:
  virtio-input/tablet). Designed in M4, tuned after M3 window exists.
- Gamepad: host gamepad (gilrs-class crate, to be pinned in M4 spike — UNVERIFIED)
  mapped to guest gamepad/axis events via the primary injection path; scrcpy's
  gamepad support is the observed prior art for the mapping shape.

## Our OWN keymap format (v1) — no vendor compatibility

- Format: TOML, versioned (`format_version = 1`), documented below. Chosen over JSON:
  comments + tables suit hand-edited game profiles; serde support is identical.
- Semantics: named bindings map host inputs → guest touch/key events with screen
  geometry in a resolution-independent 1000×1000 space (scales to any M3 resolution).
- This is an original design. No vendor keymap file was recovered in Phase A and none
  is referenced.

```toml
# ExR keymap v1 — original format, no vendor compatibility
format_version = 1
game = "Example Arena"
profile_author = "user"

[window]
# optional: restrict profile to a guest orientation
orientation = "landscape"   # landscape | portrait | any

[[tap]]
# keyboard key -> screen tap
key = "KeyW"
x = 120
y = 780

[[tap]]
key = "Space"
x = 880
y = 800

[[swipe]]
# mouse-drag region -> swipe vector (e.g. right-stick look)
button = "Right"
from_x = 750
from_y = 500
to_x = 950
to_y = 500
duration_ms = 120

[[multitouch]]
# chord: held key + click = two-finger gesture (e.g. sprint+skill)
modifier = "ShiftLeft"
key = "KeyJ"
points = [{ x = 200, y = 700 }, { x = 820, y = 300 }]

[[stick]]
# WASD cluster -> virtual joystick anchored at a point
up = "KeyW"
down = "KeyS"
left = "KeyA"
right = "KeyD"
center_x = 150
center_y = 750
radius = 120

[[gamepad]]
# host gamepad button/axis -> guest event
button = "South"
tap_x = 880
tap_y = 800
```

- Validation: schema-checked on load (unknown keys rejected with file/line errors);
  profiles live per-instance (`instances/<name>/keymaps/*.toml`) plus user library.
- Mouse-look profile = a `stick`-less variant using pointer-capture + relative mode
  (field `mode = "pointer_lock"` on the profile; exact schema fixed in M4).
