//! Dialog — centered modal panel with a close button, plus `DialogHeader` and
//! `DialogFooter` building blocks.
//!
//! The panel is published through `arkit_hooks::use_overlay`; the hook layer
//! owns the full-screen modal/backdrop native tree so dialog content does not
//! participate in the page layout.

use std::cell::Cell;
use std::rc::Rc;

use super::floating_layer::SHADOW_SM;
use super::{ARKUI_BORDER_STYLE_SOLID, ARKUI_BUTTON_TYPE_NORMAL};
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

pub(crate) const DIALOG_MAX_WIDTH: f32 = 512.0;
const DIALOG_VIEWPORT_INSET: f32 = spacing::SM;
const OVERLAY_BACKDROP_COLOR: u32 = 0x80000000;

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
        stack {
            percent_width: 1.0,
            max_width_constraint: DIALOG_MAX_WIDTH,
            alignment: 0_i32,
            border_radius: theme.radii.lg,
            border_width: 1.0,
            border_color: theme.colors.border,
            background_color: theme.colors.background,
            shadow: SHADOW_SM,
            column {
                percent_width: 1.0,
                padding_top: spacing::XXL,
                padding_right: spacing::XXL,
                padding_bottom: spacing::XXL,
                padding_left: spacing::XXL,
                {children}
            }
            row {
                percent_width: 1.0,
                justify_content: "end",
                padding_top: 14.0,
                padding_right: 14.0,
                hit_test_behavior: 2_i32,
                button {
                    button_type: ARKUI_BUTTON_TYPE_NORMAL,
                    width: 28.0,
                    height: 28.0,
                    padding: 0.0,
                    background_color: 0x00000000,
                    border_width: 0.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    border_radius: theme.radii.sm,
                    clip: true,
                    focusable: false,
                    focus_on_touch: false,
                    alignment: 4,
                    opacity: 0.7_f32,
                    onclick: move |_| close.call(()),
                    {icon_placeholder("x", 18.0, theme.colors.muted_foreground)}
                }
            }
        }
    };

    use_dialog_overlay(current, panel, close);
    rsx! {}
}

/// Dialog header — RN Reusables native defaults: centered, `native:text-xl`
/// title and `native:text-base` muted description with `gap-1.5` spacing.
#[component]
pub fn DialogHeader(title: String, description: Option<String>) -> Element {
    let theme = use_theme();

    rsx! {
        column {
            percent_width: 1.0,
            align_items: "center",
            text {
                percent_width: 1.0,
                font_size: typography::XL,
                font_weight: 600_i32,
                font_color: theme.colors.foreground,
                line_height: 20.0,
                text_align: 1,
                "{title}"
            }
            if let Some(description) = description.as_ref() {
                if !description.is_empty() {
                    text {
                        percent_width: 1.0,
                        margin_top: spacing::XS,
                        font_size: typography::MD,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: 1,
                        "{description}"
                    }
                }
            }
        }
    }
}

/// Dialog footer — full-width column holding action children. Components with
/// semantic action/cancel slots, such as `AlertDialog`, own their native visual
/// ordering explicitly instead of trying to reverse opaque Dioxus children.
#[component]
pub fn DialogFooter(children: Element) -> Element {
    rsx! {
        column {
            percent_width: 1.0,
            margin_top: spacing::LG,
            {children}
        }
    }
}
