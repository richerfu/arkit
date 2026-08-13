//! ColorUI chrome constants and shared modal / bar shells.
//!
//! Numbers follow weilanwl/ColorUI `main.css` at a 750upx design width
//! (1upx ≈ 0.5px). Overlay and dialog colors match `.cu-modal` / `.cu-dialog`.

use arkit_component::components::{use_dialog_close, DialogClose};
use arkit_hooks::{ModalPortal, ModalPresentation};
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_colorui_theme;

pub(crate) use crate::spec::{
    BAR_HEIGHT, BG_RED as CLOSE_RED, DIALOG_FILL, DIALOG_WIDTH, DRAWER_WIDTH, OVERLAY, PADDING,
    PADDING_XL,
};

pub(crate) const DIALOG_RADIUS: f32 = spec::RADIUS_CARD;

pub(crate) fn dialog_fill(dark: bool) -> u32 {
    if dark {
        0xFF1F1F1F
    } else {
        DIALOG_FILL
    }
}

pub(crate) fn bar_fill(dark: bool, card: u32) -> u32 {
    if dark {
        card
    } else {
        0xFFFFFFFF
    }
}

pub(crate) fn colorui_centered_portal(
    open: bool,
    panel: Element,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: true,
            backdrop_color: OVERLAY,
            viewport_inset: 16.0,
            on_dismiss,
            {panel}
        }
    }
}

pub(crate) fn colorui_bottom_portal(
    open: bool,
    panel: Element,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::BottomDrawer,
            dismiss_on_backdrop: true,
            backdrop_color: OVERLAY,
            viewport_inset: 0.0,
            on_dismiss,
            {panel}
        }
    }
}

pub(crate) fn colorui_right_portal(
    open: bool,
    panel: Element,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::RightSheet,
            dismiss_on_backdrop: true,
            backdrop_color: OVERLAY,
            viewport_inset: 0.0,
            on_dismiss,
            {panel}
        }
    }
}

/// `.cu-dialog` shell: 680upx, `#f8f8f8`, 10upx radius, overflow hidden.
#[component]
pub(crate) fn CuDialogShell(children: Element) -> Element {
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    rsx! {
        column {
            width: "100%",
            max_width: DIALOG_WIDTH,
            background_color: dialog_fill(dark),
            border_radius: DIALOG_RADIUS,
            clip: true,
            {children}
        }
    }
}

/// `.cu-bar.bg-white` with a centered title and optional red close.
#[component]
pub(crate) fn CuBarHeader(title: String, show_close: Option<bool>) -> Element {
    let theme = use_colorui_theme();
    let tokens = theme.tokens();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let close = use_dialog_close();
    let show_close = show_close.unwrap_or(true);
    rsx! {
        stack {
            width: "100%",
            height: BAR_HEIGHT,
            background_color: bar_fill(dark, tokens.colors.card),
            alignment: "center",
            text {
                width: "100%",
                content: title,
                font_size: 16.0,
                font_weight: 500,
                font_color: tokens.colors.foreground,
                text_align: "center",
            }
            if show_close {
                row {
                    width: "100%",
                    height: BAR_HEIGHT,
                    justify_content: "end",
                    align_items: "center",
                    padding_right: PADDING,
                    button {
                        button_type: "normal",
                        width: 32.0,
                        height: 32.0,
                        padding: 0.0,
                        background_color: 0x00000000u32,
                        border_width: 0.0,
                        focusable: false,
                        focus_on_touch: false,
                        alignment: "center",
                        onclick: move |_| {
                            if let Some(close) = close.as_ref() {
                                close.call();
                            }
                        },
                        {arkit_icon::icon("x", 18.0, CLOSE_RED)}
                    }
                }
            }
        }
    }
}

/// `.cu-bar.bg-white.justify-end` footer for dialog actions.
#[component]
pub(crate) fn CuBarFooter(children: Element) -> Element {
    let theme = use_colorui_theme();
    let tokens = theme.tokens();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    rsx! {
        row {
            width: "100%",
            min_height: BAR_HEIGHT,
            align_items: "center",
            justify_content: "end",
            background_color: bar_fill(dark, tokens.colors.card),
            padding_left: PADDING,
            padding_right: PADDING,
            {children}
        }
    }
}

#[component]
pub(crate) fn ProvideClose(close: EventHandler<()>, children: Element) -> Element {
    use_context_provider(|| DialogClose(close));
    rsx! { {children} }
}

pub(crate) fn provide_close(close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        ProvideClose {
            close,
            {children}
        }
    }
}
