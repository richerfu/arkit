# arkit_terminal

Embedded terminal for arkit, powered by **rio-vt** and rendered through
**wgpu 30** into an OHOS XComponent.

## Architecture

rio-vt parses VT and owns the grid. **This crate does not open a PTY or SSH.**
The embedder owns host I/O:

```text
  keyboard/IME ── on_input / encode_* ──► your host (SSH, PTY, shell)
  host output   ── feed_vt ─────────────► VT parser (CPU, sequential)
                                      └─► render worker
                                          ├─► GPU cell instances (one per grid cell)
                                          ├─► GPU glyph atlas
                                          └─► wgpu GLES/EGL
                                              └─► XComponent
  write_pty     ── on_write_pty ────────► same host
```

VT parsing stays on the CPU: an ANSI state machine is sequential and cannot
be a compute shader. Grid layout, decorations, cursor, and glyph sampling
run in GPU shaders. OHOS Typography rasterizes a grapheme only on an atlas
miss so HarmonyOS CJK/emoji fallback remains correct; it never paints a
full NativeWindow frame.

The previous libghostty-vt path required a Zig toolchain and only used the
GPU for the final present. This crate depends on rio-vt (pure Rust, no Zig)
and walks the cell buffer in the vertex shader.

### Component (`Terminal`)

- Paints `TerminalFrame` into one wgpu-backed XComponent on-screen surface
- Owns the wgpu device/queue/surface, glyph atlas, and cell drawing on a
  dedicated worker; terminal output and scroll updates never create or diff
  ArkUI `Text` nodes
- Coalesces pending viewport snapshots by replacement, so a blocked surface
  presents the newest scroll position instead of replaying stale frames
- Optionally captures soft-keyboard / pointer and emits **host-bound** bytes
  via `on_input` (never injects them into VT)
- Uses Arkit native-element layout subscriptions; changing `config` updates
  the engine and painted cell geometry even when the surface size is unchanged

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

See `examples/terminal` for a real local PTY, a built-in demo shell, and SSH.
The example uses Arkit `use_layout_size` and `use_window_metrics` to derive
the grid from its measured viewport, update `Terminal.config`, and send the
same columns/rows to the local PTY or SSH session. Window and keyboard state
remain owned by Arkit. Host I/O lives in the example, not in this crate.

`Local PTY` opens an OS master/slave pair and starts an interactive shell
(`/system/bin/sh` on OHOS). `PTY probe` checks the tty, working directory,
`ls -la`, and a real pipeline before emitting its success marker;
PTY errors are displayed without substituting the built-in demo shell.
The backend uses nonblocking I/O, bounded queues, `TIOCSWINSZ`/`SIGWINCH`, and
closes and reaps its child when the session is dropped. Ordinary HAP sandbox
access to the PTY device and shell must be verified on the target OS.

On OHOS, the shell starts with `HOME` and its working directory set to
`/data/storage/el2/base/files/terminal`, inside the application's storage.
The sandbox root `/` cannot be listed by a normal app; launching there makes
`ls` report `Permission denied` even when the command itself runs correctly.
System and user directories still follow their existing access permissions.

Validated on DevEco's official MateBook emulator, HarmonyOS 7.0.0.107/API 26,
with an unsigned normal HAP: `/dev/pts/0`, `ls -la`, a `printf | cat` pipeline,
physical keyboard input, and Arkit layout resizing from 106×13 to 160×28.

Run the production host tests on macOS/Linux without linking OHOS UI libraries
(Python 3.11+ and Rust are required):

```sh
python3 scripts/test-terminal-host.py --toolchain 1.97.1 --offline \
  --target-dir /tmp/arkit-terminal-host-target
```

This covers tty allocation, `TIOCSWINSZ`, Ctrl-C, exit status, child cleanup
under output backpressure, and the production key/mouse encoders. The runner
uses a temporary Cargo project and the workspace lockfile; it does not modify
workspace manifests or test against a separate PTY implementation.

## Renderer

The workspace pins the current stable `wgpu = 30.0.0`, built with the GLES
backend. `raw-window-handle`'s OHOS handle passes the retained
`OHNativeWindow` to wgpu, which creates the EGL window surface used by the
XComponent.

OHOS GLES adapters often expose no compute shaders, so the cell buffer is an
instance vertex buffer rather than a storage buffer. The vertex shader maps
`instance_index` to (column, row) from a grid uniform.

The render worker is latest-frame driven:

1. Terminal mutations capture the visible grid.
2. A single mailbox replaces an unpresented snapshot instead of queueing it.
3. The worker uploads one GPU instance per cell and rasterizes missing glyphs
   into the atlas.
4. One GPU pass presents it to the current XComponent surface.
