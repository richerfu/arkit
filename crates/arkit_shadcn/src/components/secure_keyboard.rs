//! Application-rendered keyboard for PINs, passwords, and other secrets.
//!
//! `SecureKeyboard` never mounts a native text input, so entered characters are
//! not handed to the system IME. Optional per-open digit randomization uses the
//! operating system random source. This reduces shoulder-surfing and input
//! method exposure, but it is not a hardware-backed trusted input surface.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::{BottomSheet, Button, ButtonVariant, ARKUI_BORDER_STYLE_SOLID};
use crate::i18n::{use_component_i18n, ComponentI18n};
use crate::icon::icon_placeholder;
use crate::theme::{spacing, typography, use_theme};
use arkit_prelude::*;

const DEFAULT_MAX_LENGTH: usize = 6;
const MAX_SUPPORTED_LENGTH: usize = 32;
// Preserve the established type scale while reducing the vertical whitespace
// around each glyph. With the 4vp row inset this produces a 36vp key cap.
const KEY_HEIGHT: f32 = 44.0;
// ArkUI centers the font line box, whose baseline leaves textual labels
// visually low. Reserve space below letters and action labels so their optical
// center matches the key cap; numeric glyphs already center correctly.
const TEXT_KEY_BOTTOM_INSET: f32 = 8.0;
const ORDERED_DIGITS: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
const SHUFFLE_ENTROPY_BYTES: usize = 18;
const DIGIT_ROW: &str = "1234567890";
const LETTER_ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
const SYMBOL_ROWS: [&str; 3] = ["!@#$%^&*()", "-_=+[]{}\\|", ";:'\",.?/`~"];

/// Character set and page model used by [`SecureKeyboard`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SecureKeyboardMode {
    /// Digits only, preserving the original randomized PIN keypad.
    #[default]
    Numeric,
    /// ASCII letters plus space, with a QWERTY layout and case toggle.
    Alphabetic,
    /// ASCII letters, digits, and space with a dedicated digit row.
    Alphanumeric,
    /// ASCII letters, digits, common punctuation, and space with letter and
    /// symbol pages.
    Full,
}

impl SecureKeyboardMode {
    fn accepts(self, character: char) -> bool {
        match self {
            Self::Numeric => character.is_ascii_digit(),
            Self::Alphabetic => character.is_ascii_alphabetic() || character == ' ',
            Self::Alphanumeric => character.is_ascii_alphanumeric() || character == ' ',
            Self::Full => character.is_ascii_graphic() || character == ' ',
        }
    }

    const fn initial_page(self) -> KeyboardPage {
        match self {
            Self::Numeric => KeyboardPage::Numbers,
            Self::Alphabetic | Self::Alphanumeric | Self::Full => KeyboardPage::Letters,
        }
    }

    const fn supports(self, page: KeyboardPage) -> bool {
        matches!(
            (self, page),
            (Self::Numeric, KeyboardPage::Numbers)
                | (Self::Alphabetic | Self::Alphanumeric, KeyboardPage::Letters)
                | (Self::Full, KeyboardPage::Letters | KeyboardPage::Symbols)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardPage {
    Letters,
    Numbers,
    Symbols,
}

/// User-facing text used by [`SecureKeyboard`].
///
/// When omitted, labels follow the active `arkit_i18n` locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureKeyboardLabels {
    pub delete: String,
    pub confirm: String,
    pub space: String,
}

impl SecureKeyboardLabels {
    pub fn english() -> Self {
        Self {
            delete: "Delete".to_string(),
            confirm: "Done".to_string(),
            space: "Space".to_string(),
        }
    }

    fn localized(i18n: ComponentI18n) -> Self {
        Self {
            delete: i18n.secure_keyboard_delete(),
            confirm: i18n.secure_keyboard_confirm(),
            space: i18n.secure_keyboard_space(),
        }
    }
}

/// Optional visual overrides for [`SecureKeyboard`].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SecureKeyboardStyle {
    pub key_background_color: Option<u32>,
    pub key_foreground_color: Option<u32>,
    pub function_key_background_color: Option<u32>,
    pub function_key_foreground_color: Option<u32>,
    pub confirm_background_color: Option<u32>,
    pub confirm_foreground_color: Option<u32>,
}

/// Props for [`SecureKeyboard`].
#[derive(Props, Clone, PartialEq)]
pub struct SecureKeyboardProps {
    /// Controlled value. Omit to let the component own its value.
    pub value: Option<String>,
    #[props(default)]
    pub default_value: String,
    /// Maximum number of characters. Values are clamped to `1..=32`.
    #[props(default = DEFAULT_MAX_LENGTH)]
    pub max_length: usize,
    /// Accepted characters and available keyboard pages.
    #[props(default)]
    pub mode: SecureKeyboardMode,
    /// In numeric mode, request an operating-system-random digit permutation.
    /// Text modes retain familiar key positions. If the OS random source is
    /// unavailable, ordered digits are used and the hint is hidden.
    #[props(default = true)]
    pub randomized: bool,
    /// Keep Done disabled until `max_length` characters have been entered.
    /// Disable this for variable-length secrets.
    #[props(default = true)]
    pub confirm_requires_complete: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub labels: Option<SecureKeyboardLabels>,
    #[props(default)]
    pub style: SecureKeyboardStyle,
    #[props(default)]
    pub on_change: EventHandler<String>,
    /// Called whenever the value first reaches `max_length`.
    #[props(default)]
    pub on_complete: EventHandler<String>,
    /// Called when the user presses the localized Done button.
    #[props(default)]
    pub on_confirm: EventHandler<String>,
}

/// Props for the default bottom-sheet presentation of [`SecureKeyboard`].
#[derive(Props, Clone, PartialEq)]
pub struct SecureKeyboardSheetProps {
    /// Optional text shown beside the default shield title icon.
    pub title: Option<String>,
    /// Controlled sheet state.
    pub open: Option<bool>,
    #[props(default)]
    pub default_open: bool,
    pub value: Option<String>,
    #[props(default)]
    pub default_value: String,
    #[props(default = DEFAULT_MAX_LENGTH)]
    pub max_length: usize,
    #[props(default)]
    pub mode: SecureKeyboardMode,
    #[props(default = true)]
    pub randomized: bool,
    #[props(default = true)]
    pub confirm_requires_complete: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub labels: Option<SecureKeyboardLabels>,
    #[props(default)]
    pub style: SecureKeyboardStyle,
    #[props(default)]
    pub on_change: EventHandler<String>,
    #[props(default)]
    pub on_complete: EventHandler<String>,
    #[props(default)]
    pub on_confirm: EventHandler<String>,
    #[props(default)]
    pub on_open_change: EventHandler<bool>,
}

/// Default bottom-sheet presentation for [`SecureKeyboard`].
#[component]
pub fn SecureKeyboardSheet(props: SecureKeyboardSheetProps) -> Element {
    let theme = use_theme();
    let max_length = normalize_max_length(props.max_length);
    let mode = props.mode;
    let default_value = sanitize_value(&props.default_value, max_length, mode);
    let mut internal_value = use_signal(move || default_value);
    let mut internal_open = use_signal(|| props.default_open);
    let value_controlled = props.value.is_some();
    let open_controlled = props.open.is_some();
    let current_value = props
        .value
        .as_deref()
        .map(|value| sanitize_value(value, max_length, mode))
        .unwrap_or_else(|| sanitize_value(&internal_value.read(), max_length, mode));
    let open = props.open.unwrap_or_else(|| *internal_open.read());
    let on_change = props.on_change;
    let on_confirm = props.on_confirm;
    let on_open_change = props.on_open_change;
    let labels = props.labels.clone();
    let style = props.style;
    let title = props.title.clone().filter(|title| !title.is_empty());

    let set_open = EventHandler::new(move |next: bool| {
        if !open_controlled {
            internal_open.set(next);
        }
        on_open_change.call(next);
    });
    let commit_value = EventHandler::new(move |next: String| {
        if !value_controlled {
            internal_value.set(next.clone());
        }
        on_change.call(next);
    });
    use_secure_keyboard_back_press(open, set_open);
    let submit = EventHandler::new(move |value: String| {
        on_confirm.call(value);
        set_open.call(false);
    });

    rsx! {
        BottomSheet {
            title: String::new(),
            open: Some(open),
            default_open: Some(false),
            show_header: Some(false),
            show_backdrop: Some(false),
            show_handle: Some(false),
            on_close: move |_| set_open.call(false),
            column {
                width: "100%",
                align_items: "center",
                row {
                    height: 28.0,
                    align_items: "center",
                    justify_content: "center",
                    {icon_placeholder("shield-check", 18.0, theme.colors.muted_foreground)}
                    if let Some(title) = title {
                        row { width: spacing::XS }
                        text {
                            content: title,
                            font_size: typography::SM,
                            font_weight: 600_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                }
                column { height: spacing::SM }
                SecureKeyboard {
                    value: Some(current_value),
                    max_length,
                    mode,
                    randomized: props.randomized,
                    confirm_requires_complete: props.confirm_requires_complete,
                    disabled: props.disabled,
                    labels,
                    style,
                    on_change: move |value| commit_value.call(value),
                    on_complete: props.on_complete,
                    on_confirm: move |value| submit.call(value),
                }
            }
        }
    }
}

/// Application-rendered keyboard content without a trigger, sheet, or preview.
///
/// Mount it directly in a custom container, or use [`SecureKeyboardSheet`] for
/// the default bottom-sheet presentation.
#[component]
pub fn SecureKeyboard(props: SecureKeyboardProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let max_length = normalize_max_length(props.max_length);
    let mode = props.mode;
    let default_value = sanitize_value(&props.default_value, max_length, mode);
    let mut internal_value = use_signal(move || default_value);
    let mut keyboard_page = use_signal(|| mode.initial_page());
    let mut uppercase = use_signal(|| false);
    let value_controlled = props.value.is_some();
    let current_value = props
        .value
        .as_deref()
        .map(|value| sanitize_value(value, max_length, mode))
        .unwrap_or_else(|| sanitize_value(&internal_value.read(), max_length, mode));
    let labels = props
        .labels
        .clone()
        .unwrap_or_else(|| SecureKeyboardLabels::localized(i18n));
    let on_change = props.on_change;
    let on_complete = props.on_complete;
    let on_confirm = props.on_confirm;
    let commit_value = EventHandler::new(move |next: String| {
        if !value_controlled {
            internal_value.set(next.clone());
        }
        on_change.call(next);
    });
    let layout = use_digit_layout(
        true,
        props.randomized && mode == SecureKeyboardMode::Numeric,
    );
    let page = normalize_keyboard_page(mode, keyboard_page());
    let key_background = props
        .style
        .key_background_color
        .unwrap_or(theme.colors.secondary);
    let key_foreground = props
        .style
        .key_foreground_color
        .unwrap_or(theme.colors.secondary_foreground);
    let function_background = props
        .style
        .function_key_background_color
        .unwrap_or(theme.colors.muted);
    let function_foreground = props
        .style
        .function_key_foreground_color
        .unwrap_or(theme.colors.muted_foreground);
    let confirm_background = props
        .style
        .confirm_background_color
        .unwrap_or(theme.colors.primary);
    let confirm_foreground = props
        .style
        .confirm_foreground_color
        .unwrap_or(theme.colors.primary_foreground);
    let current_length = current_value.chars().count();
    let confirm_disabled = props.disabled
        || current_value.is_empty()
        || (props.confirm_requires_complete && current_length < max_length);
    let confirm_value = current_value.clone();
    let confirm = EventHandler::new(move |()| on_confirm.call(confirm_value.clone()));

    rsx! {
        column {
            width: "100%",
            if page == KeyboardPage::Letters {
                SecureKeyboardLetterPad {
                    mode,
                    uppercase: uppercase(),
                    value: current_value.clone(),
                    max_length,
                    disabled: props.disabled,
                    space_label: labels.space.clone(),
                    delete_label: labels.delete.clone(),
                    confirm_label: labels.confirm.clone(),
                    confirm_disabled,
                    background_color: key_background,
                    foreground_color: key_foreground,
                    muted_background_color: function_background,
                    muted_foreground_color: function_foreground,
                    confirm_background_color: confirm_background,
                    confirm_foreground_color: confirm_foreground,
                    commit_value,
                    on_complete,
                    on_toggle_case: move |_| uppercase.toggle(),
                    on_page_change: move |page| keyboard_page.set(page),
                    on_confirm: confirm,
                }
            } else if page == KeyboardPage::Symbols {
                SecureKeyboardSymbolPad {
                    mode,
                    value: current_value.clone(),
                    max_length,
                    disabled: props.disabled,
                    space_label: labels.space.clone(),
                    delete_label: labels.delete.clone(),
                    confirm_label: labels.confirm.clone(),
                    confirm_disabled,
                    background_color: key_background,
                    foreground_color: key_foreground,
                    muted_background_color: function_background,
                    muted_foreground_color: function_foreground,
                    confirm_background_color: confirm_background,
                    confirm_foreground_color: confirm_foreground,
                    commit_value,
                    on_complete,
                    on_page_change: move |page| keyboard_page.set(page),
                    on_confirm: confirm,
                }
            } else {
                SecureKeyboardNumberPad {
                    mode,
                    layout,
                    value: current_value.clone(),
                    max_length,
                    disabled: props.disabled,
                    delete_label: labels.delete.clone(),
                    background_color: key_background,
                    foreground_color: key_foreground,
                    muted_background_color: function_background,
                    muted_foreground_color: function_foreground,
                    commit_value,
                    on_complete,
                }
            }
            if mode == SecureKeyboardMode::Numeric {
                column { height: spacing::SM }
                Button {
                    width: "100%",
                    variant: ButtonVariant::Default,
                    disabled: Some(confirm_disabled),
                    onclick: move |_| confirm.call(()),
                    "{labels.confirm}"
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DigitLayout {
    digits: [u8; 10],
}

impl DigitLayout {
    const fn ordered() -> Self {
        Self {
            digits: ORDERED_DIGITS,
        }
    }

    fn requested(randomized: bool) -> Self {
        if !randomized {
            return Self::ordered();
        }

        let mut entropy = [0_u8; SHUFFLE_ENTROPY_BYTES];
        if getrandom::fill(&mut entropy).is_err() {
            return Self::ordered();
        }
        Self::from_entropy(entropy)
    }

    fn from_entropy(entropy: [u8; SHUFFLE_ENTROPY_BYTES]) -> Self {
        let mut digits = ORDERED_DIGITS;
        for position in (1..digits.len()).rev() {
            let entropy_index = (digits.len() - 1 - position) * 2;
            let random = u16::from_le_bytes([entropy[entropy_index], entropy[entropy_index + 1]]);
            let swap_with = usize::from(random) % (position + 1);
            digits.swap(position, swap_with);
        }
        Self { digits }
    }
}

#[derive(Default)]
struct SecureKeyboardBackState {
    open: Cell<bool>,
    close: RefCell<Option<EventHandler<bool>>>,
}

fn use_secure_keyboard_back_press(open: bool, close: EventHandler<bool>) {
    let runtime = arkit_runtime::use_runtime_handle();
    let state = use_hook(|| Rc::new(SecureKeyboardBackState::default()));
    state.open.set(open);
    state.close.replace(Some(close));

    let registration_state = state.clone();
    let _registration = use_hook(move || {
        let handler: Rc<dyn Fn() -> bool> = Rc::new(move || {
            if !registration_state.open.replace(false) {
                return false;
            }
            let close = *registration_state.close.borrow();
            if let Some(close) = close {
                close.call(false);
            }
            true
        });
        Rc::new(runtime.register_back_handler(handler))
    });
}

fn use_digit_layout(open: bool, randomized: bool) -> DigitLayout {
    let layout = use_hook(|| Rc::new(Cell::new(DigitLayout::requested(randomized))));
    let last_open = use_hook(|| Rc::new(Cell::new(false)));
    let last_randomized = use_hook(|| Rc::new(Cell::new(randomized)));
    let was_open = last_open.replace(open);
    let changed_mode = last_randomized.replace(randomized) != randomized;
    if open && (!was_open || changed_mode) {
        layout.set(DigitLayout::requested(randomized));
    }
    layout.get()
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardLetterPadProps {
    mode: SecureKeyboardMode,
    uppercase: bool,
    value: String,
    max_length: usize,
    disabled: bool,
    space_label: String,
    delete_label: String,
    confirm_label: String,
    confirm_disabled: bool,
    background_color: u32,
    foreground_color: u32,
    muted_background_color: u32,
    muted_foreground_color: u32,
    confirm_background_color: u32,
    confirm_foreground_color: u32,
    commit_value: EventHandler<String>,
    on_complete: EventHandler<String>,
    on_toggle_case: EventHandler<()>,
    on_page_change: EventHandler<KeyboardPage>,
    on_confirm: EventHandler<()>,
}

#[component]
fn SecureKeyboardLetterPad(props: SecureKeyboardLetterPadProps) -> Element {
    let rows = LETTER_ROWS.map(|row| row.chars().collect::<Vec<_>>());
    let digits = DIGIT_ROW.chars().collect::<Vec<_>>();
    let value_empty = props.value.is_empty();
    let delete_value = props.value.clone();
    let mode = props.mode;
    let on_page_change = props.on_page_change;
    let on_toggle_case = props.on_toggle_case;
    let on_confirm = props.on_confirm;
    let space_width = if mode == SecureKeyboardMode::Full {
        "55%"
    } else {
        "75%"
    };

    rsx! {
        if matches!(
            mode,
            SecureKeyboardMode::Alphanumeric | SecureKeyboardMode::Full
        ) {
            row {
                width: "100%",
                align_items: "center",
                justify_content: "center",
                for character in digits {
                    SecureKeyboardCharacterKey {
                        key: "digit-row-{character}",
                        character,
                        label: character.to_string(),
                        width: "10%".to_string(),
                        mode,
                        value: props.value.clone(),
                        max_length: props.max_length,
                        disabled: props.disabled,
                        background_color: props.muted_background_color,
                        foreground_color: props.muted_foreground_color,
                        commit_value: props.commit_value,
                        on_complete: props.on_complete,
                    }
                }
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            for character in rows[0].iter().copied() {
                SecureKeyboardCharacterKey {
                    key: "letter-{character}",
                    character: apply_letter_case(character, props.uppercase),
                    label: apply_letter_case(character, props.uppercase).to_string(),
                    width: "10%".to_string(),
                    mode,
                    value: props.value.clone(),
                    max_length: props.max_length,
                    disabled: props.disabled,
                    background_color: props.background_color,
                    foreground_color: props.foreground_color,
                    commit_value: props.commit_value,
                    on_complete: props.on_complete,
                }
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            row {
                width: "90%",
                align_items: "center",
                justify_content: "center",
                for character in rows[1].iter().copied() {
                    SecureKeyboardCharacterKey {
                        key: "letter-{character}",
                        character: apply_letter_case(character, props.uppercase),
                        label: apply_letter_case(character, props.uppercase).to_string(),
                        width: "11.111%".to_string(),
                        mode,
                        value: props.value.clone(),
                        max_length: props.max_length,
                        disabled: props.disabled,
                        background_color: props.background_color,
                        foreground_color: props.foreground_color,
                        commit_value: props.commit_value,
                        on_complete: props.on_complete,
                    }
                }
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            SecureKeyboardKey {
                label: Some("⇧".to_string()),
                icon: None,
                accessibility_label: None,
                width: "15%".to_string(),
                disabled: props.disabled,
                selected: props.uppercase,
                background_color: props.muted_background_color,
                foreground_color: props.muted_foreground_color,
                onclick: move |_| on_toggle_case.call(()),
            }
            for character in rows[2].iter().copied() {
                SecureKeyboardCharacterKey {
                    key: "letter-{character}",
                    character: apply_letter_case(character, props.uppercase),
                    label: apply_letter_case(character, props.uppercase).to_string(),
                    width: "10%".to_string(),
                    mode,
                    value: props.value.clone(),
                    max_length: props.max_length,
                    disabled: props.disabled,
                    background_color: props.background_color,
                    foreground_color: props.foreground_color,
                    commit_value: props.commit_value,
                    on_complete: props.on_complete,
                }
            }
            SecureKeyboardKey {
                label: None,
                icon: Some("delete"),
                accessibility_label: Some(props.delete_label.clone()),
                width: "15%".to_string(),
                disabled: props.disabled || value_empty,
                background_color: props.muted_background_color,
                foreground_color: props.muted_foreground_color,
                onclick: move |_| {
                    props.commit_value.call(delete_last_character(&delete_value));
                },
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            if mode.supports(KeyboardPage::Symbols) {
                SecureKeyboardKey {
                    label: Some("#+=".to_string()),
                    icon: None,
                    accessibility_label: None,
                    width: "20%".to_string(),
                    disabled: props.disabled,
                    background_color: props.muted_background_color,
                    foreground_color: props.muted_foreground_color,
                    onclick: move |_| on_page_change.call(KeyboardPage::Symbols),
                }
            }
            SecureKeyboardCharacterKey {
                character: ' ',
                label: props.space_label.clone(),
                width: space_width.to_string(),
                mode,
                value: props.value.clone(),
                max_length: props.max_length,
                disabled: props.disabled,
                background_color: props.background_color,
                foreground_color: props.foreground_color,
                commit_value: props.commit_value,
                on_complete: props.on_complete,
            }
            SecureKeyboardKey {
                label: Some(props.confirm_label),
                icon: None,
                accessibility_label: None,
                width: "25%".to_string(),
                disabled: props.confirm_disabled,
                background_color: props.confirm_background_color,
                foreground_color: props.confirm_foreground_color,
                onclick: move |_| on_confirm.call(()),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardNumberPadProps {
    mode: SecureKeyboardMode,
    layout: DigitLayout,
    value: String,
    max_length: usize,
    disabled: bool,
    delete_label: String,
    background_color: u32,
    foreground_color: u32,
    muted_background_color: u32,
    muted_foreground_color: u32,
    commit_value: EventHandler<String>,
    on_complete: EventHandler<String>,
}

#[component]
fn SecureKeyboardNumberPad(props: SecureKeyboardNumberPadProps) -> Element {
    let mode = props.mode;
    let value_empty = props.value.is_empty();
    let delete_value = props.value.clone();

    rsx! {
        for row_index in 0..3 {
            row {
                key: "digit-row-{row_index}",
                width: "100%",
                align_items: "center",
                justify_content: "center",
                for column_index in 0..3 {
                    SecureKeyboardDigitKey {
                        key: "digit-{row_index}-{column_index}",
                        digit: props.layout.digits[row_index * 3 + column_index],
                        width: "33.333%".to_string(),
                        mode,
                        value: props.value.clone(),
                        max_length: props.max_length,
                        disabled: props.disabled,
                        background_color: props.background_color,
                        foreground_color: props.foreground_color,
                        commit_value: props.commit_value,
                        on_complete: props.on_complete,
                    }
                }
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            row { width: "33.333%", height: KEY_HEIGHT }
            SecureKeyboardDigitKey {
                digit: props.layout.digits[9],
                width: "33.333%".to_string(),
                mode,
                value: props.value.clone(),
                max_length: props.max_length,
                disabled: props.disabled,
                background_color: props.background_color,
                foreground_color: props.foreground_color,
                commit_value: props.commit_value,
                on_complete: props.on_complete,
            }
            SecureKeyboardKey {
                label: None,
                icon: Some("delete"),
                accessibility_label: Some(props.delete_label),
                width: "33.333%".to_string(),
                disabled: props.disabled || value_empty,
                background_color: props.muted_background_color,
                foreground_color: props.muted_foreground_color,
                onclick: move |_| {
                    props.commit_value.call(delete_last_character(&delete_value));
                },
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardSymbolPadProps {
    mode: SecureKeyboardMode,
    value: String,
    max_length: usize,
    disabled: bool,
    space_label: String,
    delete_label: String,
    confirm_label: String,
    confirm_disabled: bool,
    background_color: u32,
    foreground_color: u32,
    muted_background_color: u32,
    muted_foreground_color: u32,
    confirm_background_color: u32,
    confirm_foreground_color: u32,
    commit_value: EventHandler<String>,
    on_complete: EventHandler<String>,
    on_page_change: EventHandler<KeyboardPage>,
    on_confirm: EventHandler<()>,
}

#[component]
fn SecureKeyboardSymbolPad(props: SecureKeyboardSymbolPadProps) -> Element {
    let rows = SYMBOL_ROWS.map(|row| row.chars().collect::<Vec<_>>());
    let value_empty = props.value.is_empty();
    let delete_value = props.value.clone();
    let mode = props.mode;
    let on_page_change = props.on_page_change;
    let on_confirm = props.on_confirm;

    rsx! {
        for (row_index, row) in rows.iter().take(2).enumerate() {
            row {
                key: "symbol-row-{row_index}",
                width: "100%",
                align_items: "center",
                justify_content: "center",
                for character in row.iter().copied() {
                    SecureKeyboardCharacterKey {
                        key: "symbol-{character}",
                        character,
                        label: character.to_string(),
                        width: "10%".to_string(),
                        mode,
                        value: props.value.clone(),
                        max_length: props.max_length,
                        disabled: props.disabled,
                        background_color: props.background_color,
                        foreground_color: props.foreground_color,
                        commit_value: props.commit_value,
                        on_complete: props.on_complete,
                    }
                }
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            for character in rows[2].iter().copied() {
                SecureKeyboardCharacterKey {
                    key: "symbol-{character}",
                    character,
                    label: character.to_string(),
                    width: "8.5%".to_string(),
                    mode,
                    value: props.value.clone(),
                    max_length: props.max_length,
                    disabled: props.disabled,
                    background_color: props.background_color,
                    foreground_color: props.foreground_color,
                    commit_value: props.commit_value,
                    on_complete: props.on_complete,
                }
            }
            SecureKeyboardKey {
                label: None,
                icon: Some("delete"),
                accessibility_label: Some(props.delete_label),
                width: "15%".to_string(),
                disabled: props.disabled || value_empty,
                background_color: props.muted_background_color,
                foreground_color: props.muted_foreground_color,
                onclick: move |_| {
                    props.commit_value.call(delete_last_character(&delete_value));
                },
            }
        }
        row {
            width: "100%",
            align_items: "center",
            justify_content: "center",
            SecureKeyboardKey {
                label: Some("ABC".to_string()),
                icon: None,
                accessibility_label: None,
                width: "20%".to_string(),
                disabled: props.disabled,
                background_color: props.muted_background_color,
                foreground_color: props.muted_foreground_color,
                onclick: move |_| on_page_change.call(KeyboardPage::Letters),
            }
            SecureKeyboardCharacterKey {
                character: ' ',
                label: props.space_label,
                width: "55%".to_string(),
                mode,
                value: props.value.clone(),
                max_length: props.max_length,
                disabled: props.disabled,
                background_color: props.background_color,
                foreground_color: props.foreground_color,
                commit_value: props.commit_value,
                on_complete: props.on_complete,
            }
            SecureKeyboardKey {
                label: Some(props.confirm_label),
                icon: None,
                accessibility_label: None,
                width: "25%".to_string(),
                disabled: props.confirm_disabled,
                background_color: props.confirm_background_color,
                foreground_color: props.confirm_foreground_color,
                onclick: move |_| on_confirm.call(()),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardDigitKeyProps {
    digit: u8,
    width: String,
    mode: SecureKeyboardMode,
    value: String,
    max_length: usize,
    disabled: bool,
    background_color: u32,
    foreground_color: u32,
    commit_value: EventHandler<String>,
    on_complete: EventHandler<String>,
}

#[component]
fn SecureKeyboardDigitKey(props: SecureKeyboardDigitKeyProps) -> Element {
    let digit = props.digit;
    let disabled = props.disabled || props.value.chars().count() >= props.max_length;
    let value = props.value;
    let max_length = props.max_length;
    let mode = props.mode;
    let commit_value = props.commit_value;
    let on_complete = props.on_complete;

    rsx! {
        SecureKeyboardKey {
            label: Some(digit.to_string()),
            icon: None,
            accessibility_label: None,
            width: props.width,
            disabled,
            background_color: props.background_color,
            foreground_color: props.foreground_color,
            onclick: move |_| {
                let Some(next) =
                    append_character(&value, char::from(b'0' + digit), max_length, mode)
                else {
                    return;
                };
                commit_value.call(next.clone());
                if next.chars().count() == max_length {
                    on_complete.call(next);
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardCharacterKeyProps {
    character: char,
    label: String,
    width: String,
    mode: SecureKeyboardMode,
    value: String,
    max_length: usize,
    disabled: bool,
    background_color: u32,
    foreground_color: u32,
    commit_value: EventHandler<String>,
    on_complete: EventHandler<String>,
}

#[component]
fn SecureKeyboardCharacterKey(props: SecureKeyboardCharacterKeyProps) -> Element {
    let disabled = props.disabled || props.value.chars().count() >= props.max_length;
    let character = props.character;
    let mode = props.mode;
    let value = props.value;
    let max_length = props.max_length;
    let commit_value = props.commit_value;
    let on_complete = props.on_complete;

    rsx! {
        SecureKeyboardKey {
            label: Some(props.label),
            icon: None,
            accessibility_label: None,
            width: props.width,
            disabled,
            background_color: props.background_color,
            foreground_color: props.foreground_color,
            onclick: move |_| {
                let Some(next) = append_character(&value, character, max_length, mode) else {
                    return;
                };
                commit_value.call(next.clone());
                if next.chars().count() == max_length {
                    on_complete.call(next);
                }
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SecureKeyboardKeyProps {
    label: Option<String>,
    icon: Option<&'static str>,
    accessibility_label: Option<String>,
    width: String,
    disabled: bool,
    #[props(default)]
    selected: bool,
    background_color: u32,
    foreground_color: u32,
    onclick: EventHandler<()>,
}

#[component]
fn SecureKeyboardKey(props: SecureKeyboardKeyProps) -> Element {
    let theme = use_theme();
    let disabled = props.disabled;
    let onclick = props.onclick;
    let label_is_long = props
        .label
        .as_ref()
        .is_some_and(|label| label.chars().count() > 1);
    let label_needs_optical_centering = props.label.as_ref().is_some_and(|label| {
        let mut characters = label.chars();
        let first_is_ascii_letter = characters
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic());
        let has_more_characters = characters.next().is_some();
        first_is_ascii_letter || has_more_characters
    });

    rsx! {
        row {
            width: props.width,
            height: KEY_HEIGHT,
            align_items: "center",
            justify_content: "center",
            padding: spacing::XXS,
            button {
                button_type: "normal",
                width: "100%",
                height: "100%",
                padding_top: 0.0,
                padding_right: 0.0,
                padding_bottom: if label_needs_optical_centering {
                    TEXT_KEY_BOTTOM_INSET
                } else {
                    0.0
                },
                padding_left: 0.0,
                alignment: "center",
                background_color: if props.selected {
                    theme.colors.accent
                } else {
                    props.background_color
                },
                foreground_color: props.foreground_color,
                border_width: 0.0,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_radius: theme.radii.md,
                clip: true,
                enabled: !disabled,
                focusable: false,
                focus_on_touch: false,
                opacity: if disabled { 0.4 } else { 1.0 },
                onclick: move |_| {
                    if !disabled {
                        onclick.call(());
                    }
                },
                if let Some(icon) = props.icon {
                    column {
                        align_items: "center",
                        justify_content: "center",
                        {icon_placeholder(icon, 22.0, props.foreground_color)}
                    }
                } else if let Some(label) = props.label {
                    text {
                        content: label,
                        width: "100%",
                        font_size: if label_is_long {
                            typography::SM
                        } else {
                            typography::XL
                        },
                        font_weight: 600_i32,
                        font_color: props.foreground_color,
                        line_height: 26.0,
                        text_align: "center",
                    }
                }
            }
        }
    }
}

fn normalize_max_length(max_length: usize) -> usize {
    max_length.clamp(1, MAX_SUPPORTED_LENGTH)
}

fn normalize_keyboard_page(mode: SecureKeyboardMode, page: KeyboardPage) -> KeyboardPage {
    if mode.supports(page) {
        page
    } else {
        mode.initial_page()
    }
}

fn sanitize_value(value: &str, max_length: usize, mode: SecureKeyboardMode) -> String {
    value
        .chars()
        .filter(|character| mode.accepts(*character))
        .take(max_length)
        .collect()
}

fn append_character(
    value: &str,
    character: char,
    max_length: usize,
    mode: SecureKeyboardMode,
) -> Option<String> {
    if !mode.accepts(character) || value.chars().count() >= max_length {
        return None;
    }
    let mut next = String::with_capacity(value.len() + character.len_utf8());
    next.push_str(value);
    next.push(character);
    Some(next)
}

fn delete_last_character(value: &str) -> String {
    let mut next = value.to_string();
    next.pop();
    next
}

fn apply_letter_case(character: char, uppercase: bool) -> char {
    if uppercase {
        character.to_ascii_uppercase()
    } else {
        character.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_each_mode_and_clamps_external_values() {
        assert_eq!(
            sanitize_value("a12-34 B", 4, SecureKeyboardMode::Numeric),
            "1234"
        );
        assert_eq!(
            sanitize_value("a1_B z", 8, SecureKeyboardMode::Alphabetic),
            "aB z"
        );
        assert_eq!(
            sanitize_value("a1_B z", 8, SecureKeyboardMode::Alphanumeric),
            "a1B z"
        );
        assert_eq!(
            sanitize_value("a1_B z", 8, SecureKeyboardMode::Full),
            "a1_B z"
        );
        assert_eq!(normalize_max_length(0), 1);
        assert_eq!(normalize_max_length(usize::MAX), MAX_SUPPORTED_LENGTH);
    }

    #[test]
    fn append_and_delete_respect_mode_and_limit() {
        assert_eq!(
            append_character("12", '3', 3, SecureKeyboardMode::Numeric),
            Some("123".to_string())
        );
        assert_eq!(
            append_character("123", '4', 3, SecureKeyboardMode::Numeric),
            None
        );
        assert_eq!(
            append_character("12", 'a', 3, SecureKeyboardMode::Numeric),
            None
        );
        assert_eq!(
            append_character("A", ' ', 3, SecureKeyboardMode::Alphabetic),
            Some("A ".to_string())
        );
        assert_eq!(
            append_character("A", '@', 3, SecureKeyboardMode::Alphanumeric),
            None
        );
        assert_eq!(
            append_character("A", '@', 3, SecureKeyboardMode::Full),
            Some("A@".to_string())
        );
        assert_eq!(delete_last_character("A@"), "A");
        assert_eq!(delete_last_character(""), "");
    }

    #[test]
    fn pages_are_normalized_to_the_selected_mode() {
        assert_eq!(
            normalize_keyboard_page(SecureKeyboardMode::Numeric, KeyboardPage::Letters),
            KeyboardPage::Numbers
        );
        assert_eq!(
            normalize_keyboard_page(SecureKeyboardMode::Alphabetic, KeyboardPage::Symbols),
            KeyboardPage::Letters
        );
        assert_eq!(
            normalize_keyboard_page(SecureKeyboardMode::Alphanumeric, KeyboardPage::Numbers),
            KeyboardPage::Letters
        );
        assert_eq!(
            normalize_keyboard_page(SecureKeyboardMode::Full, KeyboardPage::Symbols),
            KeyboardPage::Symbols
        );
    }

    #[test]
    fn letter_case_is_applied_before_insertion() {
        assert_eq!(apply_letter_case('q', false), 'q');
        assert_eq!(apply_letter_case('q', true), 'Q');
    }

    #[test]
    fn deterministic_entropy_produces_a_digit_permutation() {
        let entropy = [9, 0, 8, 0, 7, 0, 6, 0, 5, 0, 4, 0, 3, 0, 2, 0, 1, 0];
        let layout = DigitLayout::from_entropy(entropy);
        let mut sorted = layout.digits;
        sorted.sort_unstable();

        assert_eq!(sorted, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_ne!(layout.digits, ORDERED_DIGITS);
    }

    #[test]
    fn ordered_layout_is_used_when_randomization_is_disabled() {
        let layout = DigitLayout::requested(false);

        assert_eq!(layout.digits, ORDERED_DIGITS);
    }
}
