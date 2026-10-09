//! Exercise the production VT encoders without loading GPU/native surfaces.
#![allow(dead_code)]

#[path = "../src/config.rs"]
mod config;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/frame.rs"]
mod frame;
#[path = "../src/input.rs"]
mod input;

use input::{EncodeState, KeyChord, KeyMods, MouseAction, MouseButton, MouseInput};

#[test]
fn physical_ctrl_and_alt_letters_are_encoded_as_terminal_chords() {
    let ctrl = KeyMods {
        ctrl: true,
        ..Default::default()
    };
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("c").with_utf8("c").with_mods(ctrl)
        )
        .unwrap(),
        b"\x03"
    );
    let alt = KeyMods {
        alt: true,
        ..Default::default()
    };
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("x").with_utf8("x").with_mods(alt)
        )
        .unwrap(),
        b"\x1bx"
    );
    let both = KeyMods {
        ctrl: true,
        alt: true,
        ..Default::default()
    };
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("c").with_utf8("c").with_mods(both)
        )
        .unwrap(),
        b"\x1b\x03"
    );
}

#[test]
fn navigation_and_function_keys_respect_application_cursor_and_modifiers() {
    let app = EncodeState {
        app_cursor: true,
        ..Default::default()
    };
    assert_eq!(input::encode_named_key(app, "up").unwrap(), b"\x1bOA");
    let ctrl = KeyMods {
        ctrl: true,
        ..Default::default()
    };
    assert_eq!(
        input::encode_key_chord(app, KeyChord::named("up").with_mods(ctrl)).unwrap(),
        b"\x1b[1;5A"
    );
    assert_eq!(input::encode_named_key(app, "f1").unwrap(), b"\x1bOP");
    assert_eq!(
        input::encode_key_chord(app, KeyChord::named("f12").with_mods(ctrl)).unwrap(),
        b"\x1b[24;5~"
    );
}

#[test]
fn shifted_tab_ctrl_space_and_unicode_keep_their_distinct_wire_values() {
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("tab").with_mods(KeyMods {
                shift: true,
                ..Default::default()
            })
        )
        .unwrap(),
        b"\x1b[Z"
    );
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("space").with_mods(KeyMods {
                ctrl: true,
                ..Default::default()
            })
        )
        .unwrap(),
        [0]
    );
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            KeyChord::named("界").with_utf8("界")
        )
        .unwrap(),
        "界".as_bytes()
    );
}

#[test]
fn wheel_reports_use_sgr_button_codes_and_shift_preserves_local_scrollback() {
    let state = EncodeState {
        mouse_reporting: true,
        sgr_mouse: true,
        ..Default::default()
    };
    let config = config::TerminalConfig::default();
    let mut event = MouseInput {
        action: MouseAction::Press,
        button: MouseButton::WheelUp,
        x: 0.0,
        y: 0.0,
        mods: Default::default(),
    };
    assert_eq!(
        input::encode_mouse(state, event, &config).unwrap(),
        b"\x1b[<64;1;1M"
    );
    event.button = MouseButton::WheelDown;
    assert_eq!(
        input::encode_mouse(state, event, &config).unwrap(),
        b"\x1b[<65;1;1M"
    );
    event.mods.shift = true;
    assert!(input::encode_mouse(state, event, &config)
        .unwrap()
        .is_empty());
}

#[test]
fn native_printable_key_fallback_keeps_ctrl_modifiers_when_unicode_is_absent() {
    use arkit_prelude::dioxus_elements::event::{KeyAction, KeyData, KeyboardKey};
    let key = KeyData {
        key: KeyboardKey::Character('c'),
        action: KeyAction::Down,
        modifiers: arkit_prelude::dioxus_elements::event::KeyModifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        input::encode_key_chord(
            EncodeState::default(),
            input::physical_key_chord(&key).unwrap()
        )
        .unwrap(),
        b"\x03"
    );
}
