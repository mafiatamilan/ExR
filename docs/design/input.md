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

## Coordinate, orientation, and aspect rules

- All gesture coordinates live in a 1000x1000 resolution-independent space: `x` runs
  0 (left) to 1000 (right), `y` runs 0 (top) to 1000 (bottom), regardless of the
  guest resolution chosen in M3. The engine scales to physical pixels at inject time.
- `orientation` on `[window]` declares the guest orientation the profile was authored
  for (`landscape` | `portrait` | `any`). If the live display orientation mismatches a
  non-`any` profile, the profile is refused with an error (never silently remapped).
- Aspect: coordinates are NOT aspect-corrected across orientations. A profile authored
  for 16:9 applied to a 4:3 window keeps identical 1000-space positions; authors ship
  separate profiles per aspect class. The loader warns when window aspect differs by
  more than 15% from the profile's recorded `authored_aspect` (optional field,
  e.g. `authored_aspect = "16:9"`).

## Mouse-look recentering spec

- Pointer-lock mode (`mode = "pointer_lock"`): the host cursor is captured and hidden;
  relative host motion emits relative-move events; when the cursor reaches the window
  edge it is recentered and the recenter delta is SUBTRACTED from the stream (no jump).
- Sensitivity is a profile scalar (`sensitivity = 1.0`, range 0.1–10.0, default 1.0);
  optional per-axis scale (`sensitivity_x`, `sensitivity_y`). Invert flags
  (`invert_y = false` default) are explicit booleans, never magic negatives.
- Escape from lock is always the same host key (`Escape` releases capture; the game
  never sees it). Click-to-shoot uses the primary button mapped like any `tap`.

## Touch-slot budget and conflict rule

- The engine tracks at most 10 concurrent touch slots (slots 0–9, Android-compatible
  range). A gesture that needs more slots than are free is REJECTED at bind time with
  a named error (`slot-exhaustion`), never silently dropped.
- Conflict rule: two active bindings may not claim the same host input (same key,
  button, or gamepad control) unless one is a strict superset chord declared via
  `modifier`. On conflict the loader errors naming both bindings and their lines.
- Slot assignment is deterministic (lowest free slot, gestures release in LIFO order
  of completion) so golden streams (below) are reproducible.

## Strict validation

- Unknown keys anywhere in a profile = hard error naming file, table, and line.
- Unknown `format_version` MAJOR version = refuse to load (`unsupported keymap
  version N (engine supports 1)`); unknown MINOR additions within a supported major
  are ignored with a warning. (v1 has no minor split yet; rule takes effect at v1.1+.)
- Out-of-range coordinates, empty chords, duplicate binding names, and zero-radius
  sticks are all load-time errors. Validation runs before any VM contact.

## Golden-event-stream tests

- Every profile in the repo's fixture set ships with a recorded golden event stream:
  a scripted host-input sequence plus the EXACT expected injected-event sequence
  (slot ids, coordinates in 1000-space, order). Tests replay the sequence through the
  engine against a fake sink and byte-compare (order-sensitive, timing-insensitive).
- Goldens live next to fixtures (`keymaps/tests/<name>.golden.json`) and are reviewed
  like code. A engine change that alters output must regenerate goldens explicitly
  (`UPDATE_GOLDENS=1`), never silently.

## QMP multitouch transport (verified 2026-10-16, local QEMU 11.1.2)

- The local QEMU's QMP schema for `input-send-event` carries an `mtt`
  (multi-touch) event case alongside key/btn/rel/abs (probed live via
  `query-qmp-schema`; enum values `[key, btn, rel, abs, mtt]`). The `mtt` payload
  fields are resolved in M4 against the schema before wiring the engine to QMP.
- ADB input stays the documented fallback for guests/transports without multitouch.
