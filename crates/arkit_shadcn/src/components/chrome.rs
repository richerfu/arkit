//! shadcn overlay shells. Overlay / dialog numbers come from official
//! `dialog.tsx` (`bg-black/50`, `sm:max-w-lg`, `p-6`, `rounded-lg`).

use arkit_component::components::{use_dialog_close, DialogClose};
use arkit_hooks::{ModalPortal, ModalPresentation};
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

pub(crate) fn centered_portal(open: bool, panel: Element, on_dismiss: EventHandler<()>) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: true,
            backdrop_color: spec::OVERLAY,
            viewport_inset: 16.0,
            on_dismiss,
            {panel}
        }
    }
}

pub(crate) fn bottom_portal(open: bool, panel: Element, on_dismiss: EventHandler<()>) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::BottomDrawer,
            dismiss_on_backdrop: true,
            backdrop_color: spec::OVERLAY,
            viewport_inset: 0.0,
            on_dismiss,
            {panel}
        }
    }
}

pub(crate) fn right_portal(open: bool, panel: Element, on_dismiss: EventHandler<()>) -> Element {
    rsx! {
        ModalPortal {
            open,
            presentation: ModalPresentation::RightSheet,
            dismiss_on_backdrop: true,
            backdrop_color: spec::OVERLAY,
            viewport_inset: 0.0,
            on_dismiss,
            {panel}
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

/// Official DialogContent: `max-w-lg`, `rounded-lg`, `border`, `bg-background`, `p-6`, `shadow-lg`.
#[component]
pub(crate) fn DialogShell(children: Element) -> Element {
    let theme = use_theme();
    rsx! {
        column {
            width: "100%",
            max_width: spec::DIALOG_MAX_W,
            background_color: theme.colors.background,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: spec::RADIUS_LG,
            shadow: "sm",
            padding_top: spec::DIALOG_PAD,
            padding_right: spec::DIALOG_PAD,
            padding_bottom: spec::DIALOG_PAD,
            padding_left: spec::DIALOG_PAD,
            {children}
        }
    }
}

#[component]
pub(crate) fn DialogCloseButton() -> Element {
    let theme = use_theme();
    let close = use_dialog_close();
    rsx! {
        row {
            width: "100%",
            justify_content: "end",
            button {
                button_type: "normal",
                width: spec::DIALOG_CLOSE_SIZE,
                height: spec::DIALOG_CLOSE_SIZE,
                padding: 0.0,
                background_color: 0x00000000u32,
                border_width: 0.0,
                opacity: 0.7,
                focusable: false,
                focus_on_touch: false,
                alignment: "center",
                onclick: move |_| {
                    if let Some(close) = close.as_ref() {
                        close.call();
                    }
                },
                {arkit_icon::icon("x", 16.0, theme.colors.muted_foreground)}
            }
        }
    }
}
