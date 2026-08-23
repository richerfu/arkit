//! Embedded terminal for arkit, powered by **rio-vt** and a wgpu cell renderer.
//!
//! ## Responsibility split
//!
//! rio-vt is a VT parser + grid. **Host I/O is yours.** This crate paints the
//! grid on the GPU: a vertex shader walks the cell buffer, a fragment shader
//! samples the glyph atlas and draws decorations.
//!
//! | Concern | API |
//! |---------|-----|
//! | Host → paint | [`TerminalController::feed_vt`] (mailbox; parse is off the UI thread) |
//! | Keyboard → host | [`TerminalProps::on_input`] + [`TerminalController::encode_key`] |
//! | Terminal → host | [`TerminalProps::on_write_pty`] (DA/DSR/…) |
//! | Paint | [`Terminal`] XComponent vsync → wgpu render worker |
//!
//! ```text
//!   UI / IME ──on_input──► your PTY | SSH | local shell
//!   host output ──feed_vt──► byte mailbox (no parse on the UI thread)
//!                 XComponent vsync ──► rio-vt on the render worker
//!                                   └──► GPU cell instances + glyph atlas
//!                                        └──► wgpu GLES/EGL ──► XComponent
//!   write_pty effect ──on_write_pty──► same host
//! ```
//!
//! Do **not** feed typed keys into `feed_vt` — that injects bytes as if the
//! remote produced them and breaks delete/echo.
//!
//! VT parsing stays on the CPU, but not on the ArkUI thread: ANSI state
//! machines are sequential and run on `arkit-terminal`. Grid layout,
//! decorations, cursor, and glyph sampling run in GPU shaders. Present is
//! paced by `OH_NativeXComponent_RegisterOnFrameCallback`.

mod capture;
mod component;
mod config;
mod effects;
mod engine;
mod error;
mod frame;
mod ime;
mod input;
mod native_surface;
mod renderer;
mod surface;
mod worker;

pub use component::{Terminal, TerminalController, TerminalInbox, TerminalProps};
pub use config::{Rgb, TerminalConfig, TerminalEffects};
pub use engine::{TerminalEngine, TerminalSize};
pub use error::{TerminalError, TerminalErrorKind, TerminalResult};
pub use frame::{
    rgb_to_argb, CursorVisualStyle, TerminalCell, TerminalCursor, TerminalFrame, TerminalRun,
    TerminalScrollbar,
};
pub use input::{KeyChord, KeyMods, MouseAction, MouseButton, MouseInput};
