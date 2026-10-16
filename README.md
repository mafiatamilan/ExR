# ExR — cross-platform Android gaming emulator (early scaffold)

> Status: **M1** — headless QEMU boot proof. Everything else is a stub.
> See `docs/DESIGN.md` for the architecture and `docs/PROGRESS.md` for the log.

## M1 demo: boot Android-x86 headless, prove it with screendumps

Prerequisites: Linux with `/dev/kvm` readable+writable, `qemu-system-x86_64`
installed (host-provided, never bundled), Rust via rustup.

1. Fetch the guest image **once** (the only approved download):
   `android-x86_64-9.0-r2.iso` into `~/.local/share/exr/images/` and verify
   the published SHA1 `1cc85b5ed7c830ff71aecf8405c7281a9c995aa0`
   (see `docs/design/guest-images.md`). Then extract the kernel pair next
   to it (direct-boot needs them; the ISO menu does not self-boot headless):
   ```sh
   cd ~/.local/share/exr/images
   7z x -y android-x86_64-9.0-r2.iso kernel initrd.img
   ```
2. Run the demo (polls the framebuffer up to 240 s, passes on a lit frame
   plus an inter-capture change, prints the serial tail, stops the VM):
   ```sh
   cargo run -p emu-vmm --example m1_boot -- \
     --iso ~/.local/share/exr/images/android-x86_64-9.0-r2.iso
   ```
   Options: `--name <instance>` (default `m1demo`), `--wait-secs <n>`
   (default 240), `--append "..."` (kernel cmdline; default is the ISO's own
   `isolinux.cfg` entry plus dual consoles and `vga=791`), `--iso-boot`
   (menu boot instead of direct kernel boot), `EXR_QEMU_BIN` and
   `EXR_IMAGE_DIR` overrides.
3. Run the same proof as an integration test (ignored by default):
   ```sh
   EXR_TEST_ISO=~/.local/share/exr/images/android-x86_64-9.0-r2.iso \
     cargo test -p emu-vmm --test integration_boot -- --ignored --nocapture
   ```

Security defaults: QMP lives on a unix socket in a `0700` dir under
`$XDG_RUNTIME_DIR`; QEMU is spawned as a child process, never linked;
sockets and runtime dirs are removed on stop/`Drop`.

## Workspace layout

- `crates/emu-vmm` — real code (M1): `VmBackend` trait, `QemuBackend` (QMP),
  `FakeBackend` (unit tests), PPM frame helpers.
- `crates/emu-core` — shared kernel types (`VmSpec`, `Accel`, ...).
- `crates/emu-{config,image,adb,display,input,manager,cli,gui}` — empty stubs
  for M2+. `emu-cli` becomes the `exr` binary in M5.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```
`cargo deny` runs in CI from `deny.toml` (allowlist MIT/Apache-2.0/BSD-3-Clause);
the `cargo-deny` binary is not vendored. Licence: MIT OR Apache-2.0
(`LICENSE-MIT`, `LICENSE-APACHE`).
