//! FFI surface for libghostty-vt.
//!
//! Native builds include bindgen output covering terminal, render-state,
//! style, and key-encoder APIs. Stub builds provide minimal stand-ins.

#![allow(non_camel_case_types, non_snake_case, dead_code, improper_ctypes)]

#[cfg(not(ghostty_vt_stub))]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/ghostty_vt_bindings.rs"));
}

#[cfg(not(ghostty_vt_stub))]
pub use bindings::*;

#[cfg(not(ghostty_vt_stub))]
pub use bindings::GhosttyResult::GHOSTTY_SUCCESS;

// ── stub surface ──────────────────────────────────────────────────────────

#[cfg(ghostty_vt_stub)]
mod stub {
    use std::os::raw::c_void;

    #[repr(i32)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum GhosttyResult {
        GHOSTTY_SUCCESS = 0,
        GHOSTTY_OUT_OF_MEMORY = -1,
        GHOSTTY_INVALID_VALUE = -2,
        GHOSTTY_OUT_OF_SPACE = -3,
        GHOSTTY_NO_VALUE = -4,
    }
    pub use GhosttyResult::GHOSTTY_SUCCESS;

    pub type GhosttyTerminal = *mut c_void;
    pub type GhosttyRenderState = *mut c_void;
    pub type GhosttyFormatter = *mut c_void;
    pub type GhosttyKeyEncoder = *mut c_void;
    pub type GhosttyKeyEvent = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct GhosttyTerminalOptions {
        pub cols: u16,
        pub rows: u16,
        pub max_scrollback: usize,
    }

    pub unsafe fn ghostty_terminal_new(
        _a: *const c_void,
        out: *mut GhosttyTerminal,
        _o: GhosttyTerminalOptions,
    ) -> GhosttyResult {
        // SAFETY: the caller supplies the same valid out-pointer required by
        // the native Ghostty ABI.
        unsafe {
            *out = 1 as GhosttyTerminal;
        }
        GhosttyResult::GHOSTTY_SUCCESS
    }
    pub unsafe fn ghostty_terminal_free(_: GhosttyTerminal) {}
    pub unsafe fn ghostty_terminal_vt_write(_: GhosttyTerminal, _: *const u8, _: usize) {}
    pub unsafe fn ghostty_render_state_new(
        _: *const c_void,
        out: *mut GhosttyRenderState,
    ) -> GhosttyResult {
        // SAFETY: the caller supplies the same valid out-pointer required by
        // the native Ghostty ABI.
        unsafe {
            *out = 1 as GhosttyRenderState;
        }
        GhosttyResult::GHOSTTY_SUCCESS
    }
    pub unsafe fn ghostty_render_state_free(_: GhosttyRenderState) {}
}

#[cfg(ghostty_vt_stub)]
pub use stub::*;
