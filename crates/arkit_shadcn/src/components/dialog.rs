//! Dialog — centered modal panel with a close button, plus `DialogHeader` and
//! `DialogFooter` building blocks.
//!
//! The panel is published through `arkit_hooks::use_overlay`; the hook layer
//! owns the full-screen modal/backdrop native tree so dialog content does not
//! participate in the page layout.

use std::cell::Cell;
use std::rc::Rc;

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

pub(crate) const DIALOG_MAX_WIDTH: f32 = 512.0;
const DIALOG_VIEWPORT_INSET: f32 = spacing::SM;
const OVERLAY_BACKDROP_COLOR: u32 = 0x80000000;
const DIALOG_CLOSE_BUTTON_SIZE: f32 = 28.0;
const DIALOG_CLOSE_EDGE_INSET: f32 = 14.0;
const DIALOG_HEADER_CLOSE_RESERVE: f32 = DIALOG_CLOSE_BUTTON_SIZE + spacing::XS;

/// Close handle for dialog / alert-dialog content rendered inside the overlay
/// portal. Buttons in `cancel` / `action` slots (or custom footer content)
/// should call this so uncontrolled dialogs can dismiss without a parent
/// `open` signal.
#[derive(Clone)]
pub struct DialogClose(pub EventHandler<()>);

impl DialogClose {
    pub fn call(&self) {
        self.0.call(());
    }
}

/// Resolve the active dialog close handle, if the caller is inside a dialog
/// panel (including overlay-mounted content).
pub fn use_dialog_close() -> Option<DialogClose> {
    try_use_context::<DialogClose>()
}

/// Provide [`DialogClose`] to overlay-mounted panel content.
#[component]
pub(crate) fn DialogCloseProvider(close: EventHandler<()>, children: Element) -> Element {
    use_context_provider(|| DialogClose(close));
    rsx! { {children} }
}

pub(crate) fn use_dialog_overlay(open: bool, panel: Element, on_dismiss: EventHandler<()>) {
    let overlay = arkit_hooks::use_overlay();
    let last_open = use_hook(|| Rc::new(Cell::new(None::<bool>)));
    let changed = last_open.get() != Some(open);
    last_open.set(Some(open));

    let spec = arkit_hooks::ModalOverlaySpec {
        open,
        presentation: arkit_hooks::ModalPresentation::CenteredDialog,
        dismiss_on_backdrop: true,
        backdrop_color: OVERLAY_BACKDROP_COLOR,
        viewport_inset: DIALOG_VIEWPORT_INSET,
    };

    let effect_overlay = overlay.clone();
    use_effect(use_reactive((&open,), move |(open,)| {
        if !changed {
            return;
        }

        if open {
            let panel = panel.clone();
            effect_overlay.show_modal_with_dismiss(
                spec,
                move || {
                    rsx! {
                        arkit_animation::MountTransition {
                            preset: Some(arkit_animation::TransitionPreset::Fade),
                            duration_ms: Some(160),
                            {panel.clone()}
                        }
                    }
                },
                move || on_dismiss.call(()),
            );
        } else {
            effect_overlay.dismiss();
        }
    }));

    let cleanup_overlay = overlay.clone();
    let cleanup_last_open = last_open.clone();
    use_drop(move || {
        if cleanup_last_open.get() == Some(true) {
            cleanup_overlay.dismiss();
        }
    });
}

/// Modal dialog panel.
#[component]
pub fn Dialog(
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    let panel = rsx! {
        DialogCloseProvider {
            close,
            stack {
                width: "100%",
                max_width: DIALOG_MAX_WIDTH,
                alignment: "top-start",
                border_radius: theme.radii.lg,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                shadow: "sm",
                column {
                    width: "100%",
                    align_items: "start",
                    padding_top: spacing::XXL,
                    padding_right: spacing::XXL,
                    padding_bottom: spacing::XXL,
                    padding_left: spacing::XXL,
                    {children}
                }
                row {
                    width: "100%",
                    justify_content: "end",
                    padding_top: DIALOG_CLOSE_EDGE_INSET,
                    padding_right: DIALOG_CLOSE_EDGE_INSET,
                    hit_test_behavior: "transparent",
                    button {
                        button_type: "normal",
                        width: DIALOG_CLOSE_BUTTON_SIZE,
                        height: DIALOG_CLOSE_BUTTON_SIZE,
                        padding: 0.0,
                        background_color: "#00000000",
                        border_width: 0.0,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_radius: theme.radii.sm,
                        clip: true,
                        focusable: false,
                        focus_on_touch: false,
                        alignment: "center",
                        opacity: 0.7_f32,
                        onclick: move |_| close.call(()),
                        {icon_placeholder("x", 18.0, theme.colors.muted_foreground)}
                    }
                }
            }
        }
    };

    use_dialog_overlay(current, panel, close);
    rsx! {}
}

/// Dialog header — start-aligned `native:text-xl` title and `native:text-base`
/// muted description with `gap-1.5` spacing.
#[component]
pub fn DialogHeader(title: String, description: Option<String>) -> Element {
    let theme = use_theme();

    rsx! {
        column {
            width: "100%",
            align_items: "start",
            row {
                width: "100%",
                align_items: "start",
                row {
                    layout_weight: 1.0,
                    text {
                        width: "100%",
                        font_size: typography::XL,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                        text_align: "start",
                        "{title}"
                    }
                }
                row {
                    width: DIALOG_HEADER_CLOSE_RESERVE,
                    height: 1.0,
                }
            }
            if let Some(description) = description.as_ref() {
                if !description.is_empty() {
                    text {
                        width: "100%",
                        margin_top: spacing::XS,
                        font_size: typography::MD,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: "start",
                        "{description}"
                    }
                }
            }
        }
    }
}

/// Dialog footer — full-width end-aligned row holding action children. Components with
/// semantic action/cancel slots, such as `AlertDialog`, own their native visual
/// ordering explicitly instead of trying to reverse opaque Dioxus children.
#[component]
pub fn DialogFooter(children: Element) -> Element {
    rsx! {
        row {
            width: "100%",
            margin_top: spacing::LG,
            align_items: "center",
            justify_content: "end",
            {children}
        }
    }
}
