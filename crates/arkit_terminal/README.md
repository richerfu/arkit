# arkit_terminal

Embedded terminal for arkit, powered by **libghostty-vt** and rendered through
**wgpu 30** into an OHOS XComponent.

## Architecture

libghostty-vt only parses VT and exposes render-state. **This crate does not
open a PTY or SSH.** The embedder owns host I/O:

```text
  keyboard/IME ── on_input / encode_* ──► your host (SSH, PTY, shell)
  host output   ── feed_vt ─────────────► VT parser ──► latest-frame mailbox
                                                    └─► render worker
                                                        ├─► GPU rect instances
                                                        ├─► GPU glyph atlas
                                                        └─► wgpu GLES/EGL
                                                            └─► XComponent
  write_pty     ── on_write_pty ────────► same host
```

This follows Ghostty's renderer boundaries rather than embedding its desktop
Metal/OpenGL renderer verbatim: VT state is captured independently, one render
thread owns the surface and caches, cell decorations are instanced GPU quads,
and rasterized glyphs are cached in a GPU atlas. OHOS Typography is used only
to rasterize a styled run on an atlas miss so HarmonyOS system CJK/emoji
fallback remains correct; it never paints a full NativeWindow frame.

### Component (`Terminal`)

- Paints `TerminalFrame` into one wgpu-backed XComponent on-screen surface
- Owns the wgpu device/queue/surface, glyph atlas, and text/cursor drawing on a
  dedicated worker; terminal output and
  scroll updates never create or diff ArkUI `Text` nodes
- Coalesces pending viewport snapshots by replacement, so a blocked surface
  presents the newest scroll position instead of replaying stale frames
- Optionally captures soft-keyboard / pointer and emits **host-bound** bytes
  via `on_input` (never injects them into VT)

### Controller

| Method | Direction |
|--------|-----------|
| `feed_vt` / `write_bytes` | Host → terminal |
| `encode_key` / `encode_text` / `encode_mouse` / `encode_focus` | Returns host-bound bytes |

### Example host wiring

```rust
// on_input from Terminal → write to SSH/PTY
// SSH/PTY reader task → controller.feed_vt(&chunk)
// on_write_pty → write to SSH/PTY
```

See `examples/terminal` for a local cooked shell + SSH demo (host lives in the
example, not in this crate).

## Renderer

The workspace pins the current stable `wgpu = 30.0.0`, built with the GLES
backend. `raw-window-handle`'s OHOS handle passes the retained
`OHNativeWindow` to wgpu, which creates the EGL window surface used by the
XComponent.

The render worker is deliberately latest-frame driven:

1. Terminal mutations capture viewport-only render state.
2. A single mailbox replaces an unpresented snapshot instead of queueing it.
3. The worker converts the newest snapshot to rectangle and glyph instances.
4. One GPU pass presents it to the current XComponent surface.

This bounds work during rapid output or scrolling: stale intermediate frames
are discarded before scene construction and GPU submission.

## Ghostty dependency

Ghostty is vendored as a git submodule at `vendor/ghostty`.

```sh
git submodule update --init --recursive crates/arkit_terminal/vendor/ghostty
```

| Variable | Purpose |
|----------|---------|
| `GHOSTTY_SRC` | Override Ghostty tree (default: `vendor/ghostty`) |
| `GHOSTTY_VT_LIB_DIR` | Prebuilt `libghostty-vt` root (single-arch dir **or** multi-arch root) |
| `ZIG` / `ZIG_TARGET` | Zig binary and cross target (e.g. `aarch64-linux-ohos`) |

### Multi-arch OHOS (arm64 / armv7 / x86_64)

`build.rs` resolves prebuilts per `CARGO_CFG_TARGET_ARCH`:

```text
$GHOSTTY_VT_LIB_DIR/
  aarch64/lib/libghostty-vt.a
  armv7/lib/libghostty-vt.a
  x86_64/lib/libghostty-vt.a
```

Workspace default cache: `target/ghostty-vt-ohos/{aarch64,armv7,x86_64}/lib`.

```sh
# build libghostty-vt for all three
for t in aarch64-linux-ohos:aarch64 arm-linux-ohoseabi:armv7 x86_64-linux-ohos:x86_64; do
  zt=${t%%:*}; out=${t##*:}
  ZIG_TARGET=$zt OUT_DIR=target/ghostty-vt-ohos/$out \
    crates/arkit_terminal/scripts/build-ghostty-vt.sh
done

export GHOSTTY_VT_LIB_DIR=$PWD/target/ghostty-vt-ohos
cd examples/terminal
ohrs build --arch aarch   # arm64-v8a
ohrs build --arch arm     # armeabi-v7a
ohrs build --arch x64     # x86_64
```

On non-OHOS hosts, missing sources/Zig/prebuilts fall back to **stub** mode for
type-checking. OHOS builds fail fast instead of silently shipping a non-working
terminal; explicitly enable the `stub` feature only for a non-production check.
