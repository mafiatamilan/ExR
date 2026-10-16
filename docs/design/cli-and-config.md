# CLI and config (Phase B0 design)

Date: 2026-10-16. Baseline shape: `docs/spec/comparison.md` (both local-payload
products pair a manager GUI with a scriptable instance CLI using name/index
addressing and lifecycle/clone/app verbs). Our surface is original; only the
*shape* (scriptable, addressable) is inherited.

## CLI surface (`emu-cli`)

Addressing: every command takes exactly one of `--name <instance>` or
`--index <n>` (stable per registry order). All commands are scriptable
(exit codes + `--json` output mode) and mirror nothing vendor-specific.

```text
exr list [--json]                          # instances: name, index, state, android, cpus, ram
exr create --name <n> [--android 9] [--cpus 4] [--ram 4096] [--dpi N] [--keymap file]
exr clone --name <src> --new-name <dst>    # qcow2 overlay copy (seconds, not GBs)
exr snapshot [--name N] create|list|restore|delete <snap>
exr start|stop|reboot [--name N]           # lifecycle via QMP + guest state watch
exr install --name N <apk>                 # adb install passthrough
exr adb --name N -- <adb args...>          # full adb passthrough (shell, forward, ...). 
exr input --name N --keymap <file>         # attach a keymap profile (see input.md)
exr display --name N [--res WxH] [--fps N] [--dpi N]
exr delete --name N [--yes]                # destroys overlays + state (asks first)
exr doctor                                # host checks: /dev/kvm, accel, qemu present, image registry
```

Snapshots use the QMP/block-layer path (see `vmm-options.md`), not guest hacks.

## TOML config schema

`~/.config/exr/config.toml` (global) + `instances/<name>/instance.toml` (per instance).
Versioned (`schema_version = 1`); unknown fields are hard errors.

```toml
# global
schema_version = 1
[qemu]
bin = "/usr/bin/qemu-system-x86_64"   # never bundled; host-provided
accel = "auto"                        # auto | kvm | tcg (win: whpx, mac: hvf in trait)
[images]
registry = "~/.local/share/exr/images"
[network]
mode = "user"                         # user-mode NAT first; bridged later
adb_base_port = 5555                  # per-instance: base + index (documented, no clash)

# per instance (instances/<name>/instance.toml)
schema_version = 1
name = "arena"
android = "9"
cpus = 4
ram_mb = 4096
dpi = 240
[display]
width = 1280
height = 720
fps = 60
[keymap]
active = "arena-v1.toml"
[translator]
# user-supplied only (see guest-images.md); absent = x86-only guest
path = null
sha256 = null
```

## Per-instance state (registry, not the image)

- `instances/registry.toml`: name ↔ index ↔ uuid ↔ state (stopped/running/paused) ↔
  ports ↔ overlay paths. Source of truth for addressing; rewritten atomically.
- `instances/<name>/`: `instance.toml`, `overlay.qcow2` (+snapshot chains),
  `keymaps/`, `manifest.toml` (image hash, translator hash-or-absent, created-by).
- Base images stay read-only; `clone` = new overlay (qcow2 backing chain), never a
  full copy. `delete` removes overlays + state after confirmation.
