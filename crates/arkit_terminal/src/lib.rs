//! Embedded terminal for arkit, powered by **libghostty-vt**.
//!
//! ## Responsibility split (Ghostty model)
//!
//! libghostty-vt is a VT parser + render-state snapshot. **Host I/O is yours.**
//!
//! | Concern | API |
//! |---------|-----|
//! | Host → paint | [`TerminalController::feed_vt`] / [`TerminalEngine::write_bytes`] |
//! | Keyboard → host | [`TerminalProps::on_input`] + [`TerminalController::encode_key`] |
//! | Terminal → host | [`TerminalProps::on_write_pty`] (DA/DSR/…) |
//! | Paint | [`Terminal`] XComponent + wgpu render worker |
//!
//! ```text
//!   UI / IME ──on_input──► your PTY | SSH | local shell
//!   host output ──feed_vt──► libghostty-vt ──► latest snapshot
//!                                           └─► wgpu ──► XComponent surface
//!   write_pty effect ──on_write_pty──► same host
//! ```
//!
//! Do **not** feed typed keys into `feed_vt` — that injects bytes as if the
//! remote produced them and breaks delete/echo.
//!
//! ## Capability map
//!
//! | Concern | Ghostty API | arkit_terminal |
//! |---------|-------------|----------------|
//! | Create / scrollback | `GhosttyTerminalOptions` | [`TerminalConfig`] / [`TerminalEngine`] |
//! | Resize + cell px | `ghostty_terminal_resize` | [`TerminalConfig::cell_width_px`] |
//! | Colors | `OPT_COLOR_*` | theme fields on [`TerminalConfig`] |
//! | VT write | `ghostty_terminal_vt_write` | [`feed_vt`](TerminalController::feed_vt) |
//! | Scrollback | `ghostty_terminal_scroll_viewport` | [`scroll_by`](TerminalController::scroll_by) / pan |
//! | Effects | `OPT_WRITE_PTY` / bell / title | [`TerminalEffects`] + props |
//! | Paint | `ghostty_render_state_*` (viewport only) | [`TerminalFrame`] |
//! | Keys / mouse / focus | encoders | `encode_*` → host bytes |
//!
//! ## Building
//!
//! Submodule `vendor/ghostty` + Zig (or `GHOSTTY_VT_LIB_DIR` prebuilt).

mod capture;
mod component;
mod config;
mod effects;
mod engine;
mod error;
mod ffi;
mod frame;
mod ime;
mod input;
mod native_surface;
mod renderer;
mod surface;
mod worker;

pub use component::{Terminal, TerminalController, TerminalProps};
pub use config::{Rgb, TerminalConfig, TerminalEffects};
pub use engine::{TerminalEngine, TerminalSize};
pub use error::{TerminalError, TerminalErrorKind, TerminalResult};
pub use frame::{
    rgb_to_argb, CursorVisualStyle, TerminalCell, TerminalCursor, TerminalFrame, TerminalRun,
    TerminalScrollbar,
};
pub use input::{KeyChord, KeyMods, MouseAction, MouseButton, MouseInput};
