//! Responsive bottom sheet / anchored desktop popover.
//!
//! The sheet is mounted through a root-projected portal, so it is not
//! clipped by the page or showcase canvas. Phone layouts mirror the React
//! Native Reusables sheet: optional dismissible backdrop and drag indicator,
//! rounded top corners, safe-area-aware padding, and pan-down dismissal. PC
//! layouts follow shadcn's responsive picker pattern: when an anchor is
//! supplied, the same content opens as an in-place popover next to its trigger.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::floating_layer::{
    trigger_frame_for_anchor, viewport_scale, FloatingAlign, FloatingPanelPlacement, FloatingSide,
    FLOATING_CAPTURE_COLOR,
};
use super::motion::{AnimatedModal, OVERLAY_ENTER_MS, OVERLAY_EXIT_MS, SHEET_DISTANCE};
use super::motion::{OverlayPresence, FLOATING_ENTER_MS, FLOATING_EXIT_MS};
use super::ARKUI_BORDER_STYLE_SOLID;
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

const BOTTOM_SHEET_HEADER_HEIGHT: f32 = 48.0;
const BOTTOM_SHEET_HANDLE_HEIGHT: f32 = 24.0;
const BOTTOM_SHEET_MIN_HEIGHT: f32 = 240.0;
const BOTTOM_SHEET_PC_DEFAULT_WIDTH: f32 = 420.0;
const BOTTOM_SHEET_PC_ESTIMATED_HEIGHT: f32 = 360.0;
const BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD: f32 = 72.0;

fn bottom_sheet_backdrop(theme: Theme) -> u32 {
    match theme.mode {
        ThemeMode::Light => 0x8F000000,
        ThemeMode::Dark => 0x3D000000,
    }
}

fn display_vp_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

#[derive(Default)]
struct BottomSheetBackState {
    open: Cell<bool>,
    close: RefCell<Option<EventHandler<()>>>,
}

fn use_bottom_sheet_back_press(open: bool, close: EventHandler<()>) {
    let runtime = arkit_runtime::use_runtime_handle();
    let state = use_hook(|| Rc::new(BottomSheetBackState::default()));
    state.open.set(open);
    state.close.replace(Some(close));

    let registration_state = state.clone();
    let _registration = use_hook(move || {
        let handler: Rc<dyn Fn() -> bool> = Rc::new(move || {
            if !registration_state.open.replace(false) {
                return false;
            }
            if let Some(close) = *registration_state.close.borrow() {
                close.call(());
            }
            true
        });
        Rc::new(runtime.register_back_handler(handler))
    });
}

fn modal_bottom_sheet_portal(
    open: bool,
    panel: Element,
    on_dismiss: EventHandler<()>,
    theme: Theme,
    show_backdrop: bool,
    desktop: bool,
) -> Element {
    let backdrop_color = if show_backdrop {
        bottom_sheet_backdrop(theme)
    } else {
        0x00000000
    };
    rsx! {
        AnimatedModal {
            open,
            presentation: if desktop {
                arkit_hooks::ModalPresentation::CenteredDialog
            } else {
                arkit_hooks::ModalPresentation::BottomDrawer
            },
            dismiss_on_backdrop: true,
            backdrop_color,
            viewport_inset: if desktop { spacing::LG } else { 0.0 },
            on_dismiss,
            preset: Some(if desktop {
                arkit_animation::TransitionPreset::ZoomIn
            } else {
                arkit_animation::TransitionPreset::SlideUp
            }),
            duration_ms: Some(OVERLAY_ENTER_MS),
            exit_duration_ms: Some(OVERLAY_EXIT_MS),
            distance: if desktop { None } else { Some(SHEET_DISTANCE) },
            {panel}
        }
    }
}

fn anchored_bottom_sheet_popover(
    open: bool,
    panel: Element,
    placement: FloatingPanelPlacement,
    on_dismiss: EventHandler<()>,
) -> Element {
    let left = placement.x.max(0.0);
    let top = placement.y.max(0.0);
    rsx! {
        OverlayPresence {
            open,
            preset: Some(arkit_animation::TransitionPreset::Fade),
            duration_ms: Some(FLOATING_ENTER_MS),
            exit_duration_ms: Some(FLOATING_EXIT_MS),
            fill: Some(true),
            layer: Some(arkit_hooks::OverlayLayer::Floating),
            stack {
                width: "100%",
                height: "100%",
                background_color: FLOATING_CAPTURE_COLOR,
                hit_test_behavior: "default",
                onclick: move |_| on_dismiss.call(()),
                column {
                    position: format!("{left},{top}"),
                    onclick: move |event| event.stop_propagation(),
                    {panel}
                }
            }
        }
    }
}

/// A controlled or uncontrolled bottom sheet.
///
/// The trigger remains caller-owned, matching the other modal components in
/// this crate. Set `open` from the trigger and handle `on_close` for backdrop,
/// close-button, save-button, and drag dismissal paths. On PC, pass the
/// trigger's `NativeElementRef` through `anchor` to use the shadcn-style
/// in-place popover presentation. Without an anchor, PC falls back to a
/// centered dialog. Phone layouts ignore `anchor` and retain the bottom sheet.
#[component]
pub fn BottomSheet(
    title: String,
    open: Option<bool>,
    default_open: Option<bool>,
    show_header: Option<bool>,
    show_backdrop: Option<bool>,
    show_handle: Option<bool>,
    anchor: Option<arkit_arkui::NativeElementRef>,
    pc_width: Option<f32>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let adaptive = arkit_hooks::use_adaptive_layout();
    let viewport = arkit_hooks::use_overlay_viewport();
    let fallback_anchor_ref = arkit_hooks::use_native_element_ref();
    let anchored = anchor.is_some();
    let anchor_ref = anchor.unwrap_or(fallback_anchor_ref);
    let anchor_frame = use_signal(arkit_arkui::LayoutFramePx::default);
    arkit_hooks::use_layout_frame(anchor_ref.clone(), move |frame| {
        let mut anchor_frame = anchor_frame;
        anchor_frame.set(frame);
    });
    let panel_ref = arkit_hooks::use_native_element_ref();
    let panel_frame = use_signal(arkit_arkui::LayoutFramePx::default);
    arkit_hooks::use_layout_frame(panel_ref.clone(), move |frame| {
        let mut panel_frame = panel_frame;
        panel_frame.set(frame);
    });
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(value) => value,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let desktop = adaptive.is_pc();
    let panel_width = pc_width
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or(BOTTOM_SHEET_PC_DEFAULT_WIDTH);

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });
    use_bottom_sheet_back_press(current, close);

    let panel = rsx! {
        BottomSheetPanel {
            title,
            show_header: show_header.unwrap_or(true),
            show_handle: show_handle.unwrap_or(true),
            desktop,
            pc_width: panel_width,
            native_ref: Some(panel_ref),
            on_close: close,
            {children}
        }
    };

    if desktop && anchored {
        let frame = if current {
            trigger_frame_for_anchor(&anchor_ref, *anchor_frame.read())
        } else {
            *anchor_frame.read()
        };
        let measured_panel = *panel_frame.read();
        let scale = viewport_scale(viewport);
        let panel_height = if measured_panel.is_measured() {
            measured_panel.height / scale
        } else {
            BOTTOM_SHEET_PC_ESTIMATED_HEIGHT
        };
        let placement = FloatingPanelPlacement::resolve(
            frame,
            viewport,
            panel_width,
            panel_height,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            spacing::XXS,
        );
        return anchored_bottom_sheet_popover(current, panel, placement, close);
    }

    modal_bottom_sheet_portal(
        current,
        panel,
        close,
        theme,
        show_backdrop.unwrap_or(true),
        desktop,
    )
}

#[derive(Clone, Props)]
struct BottomSheetPanelProps {
    title: String,
    show_header: bool,
    show_handle: bool,
    desktop: bool,
    pc_width: f32,
    native_ref: Option<arkit_arkui::NativeElementRef>,
    on_close: EventHandler<()>,
    children: Element,
}

impl PartialEq for BottomSheetPanelProps {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[allow(non_snake_case)]
fn BottomSheetPanel(props: BottomSheetPanelProps) -> Element {
    let theme = use_theme();
    let safe_area = arkit_hooks::use_safe_area();
    let mut drag_start = use_signal(|| None::<f32>);
    let mut drag_offset = use_signal(|| 0.0_f32);
    let on_close = props.on_close;
    let desktop = props.desktop;
    let radius = if desktop {
        format!("{0},{0},{0},{0}", theme.radii.lg)
    } else {
        format!("{0},{0},0,0", theme.radii.xl)
    };
    let border_width = if desktop { "1" } else { "1,1,0,1" };
    let body_bottom_padding = if desktop {
        spacing::LG
    } else {
        safe_area.bottom + spacing::LG
    };
    let body_top_padding = if props.show_header {
        if desktop {
            spacing::LG
        } else {
            spacing::XXL
        }
    } else {
        if desktop {
            spacing::LG
        } else {
            spacing::SM
        }
    };

    rsx! {
        column {
            accessibility_role: "dialog",
            accessibility_text: props.title.clone(),
            native_ref: props.native_ref,
            width: if desktop { format!("{}", props.pc_width) } else { "100%".to_string() },
            max_width: if desktop { props.pc_width },
            align_self: if desktop { "center" },
            constraint_size: format!(
                "0,100000,{},100000",
                if desktop { 0.0 } else { BOTTOM_SHEET_MIN_HEIGHT },
            ),
            border_radius: radius,
            border_width: border_width,
            border_color: theme.colors.border,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            background_color: if desktop { theme.colors.popover } else { theme.colors.card },
            shadow: if desktop { "lg" } else { "sm" },
            clip: true,
            ontouch: move |evt| {
                if desktop {
                    return;
                }
                let Some(pointer) = evt.data().pointer else {
                    return;
                };
                let pointer_y = if pointer.has_window_position() {
                    pointer.window_y
                } else {
                    pointer.y
                };
                let ratio = display_vp_ratio();
                match pointer.action {
                    dioxus_elements::event::PointerAction::Down => {
                        drag_start.set(Some(pointer_y));
                        drag_offset.set(0.0);
                    }
                    dioxus_elements::event::PointerAction::Move => {
                        if let Some(start) = drag_start() {
                            let next_offset = ((pointer_y - start) / ratio).max(0.0);
                            if next_offset >= BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD {
                                drag_start.set(None);
                                drag_offset.set(0.0);
                                on_close.call(());
                            } else {
                                drag_offset.set(next_offset);
                            }
                        }
                    }
                    dioxus_elements::event::PointerAction::Up => {
                        let should_dismiss = drag_offset() >= BOTTOM_SHEET_DRAG_DISMISS_THRESHOLD;
                        drag_start.set(None);
                        drag_offset.set(0.0);
                        if should_dismiss {
                            on_close.call(());
                        }
                    }
                    dioxus_elements::event::PointerAction::Cancel => {
                        drag_start.set(None);
                        drag_offset.set(0.0);
                    }
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            if props.show_handle && !desktop {
                row {
                    width: "100%",
                    height: BOTTOM_SHEET_HANDLE_HEIGHT,
                    align_items: "center",
                    justify_content: "center",
                    row {
                        width: 32.0,
                        height: 4.0,
                        border_radius: theme.radii.full,
                        background_color: theme.colors.foreground,
                        opacity: 0.4_f32,
                    }
                }
            }
            if props.show_header {
                row {
                    width: "100%",
                    height: BOTTOM_SHEET_HEADER_HEIGHT,
                    align_items: "center",
                    padding_left: spacing::LG,
                    padding_right: if desktop { spacing::SM } else { 0.0 },
                    border_width: "0,0,1,0",
                    border_color: theme.colors.border,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    row {
                        layout_weight: 1.0,
                        text {
                            width: "100%",
                            font_size: typography::LG,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 24.0,
                            text_align: if desktop { "start" } else { "center" },
                            "{props.title}"
                        }
                    }
                    button {
                        button_type: "normal",
                        accessibility_text: "Close bottom sheet",
                        width: control::ICON_SM,
                        height: control::ICON_SM,
                        padding: 0.0,
                        background_color: "#00000000",
                        border_width: 0.0,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_radius: theme.radii.md,
                        clip: true,
                        focusable: true,
                        focus_on_touch: true,
                        alignment: "center",
                        onclick: move |_| on_close.call(()),
                        {icon_placeholder("x", 16.0, theme.colors.muted_foreground)}
                    }
                }
            }
            column {
                width: "100%",
                padding_top: body_top_padding,
                padding_right: spacing::LG,
                padding_bottom: body_bottom_padding,
                padding_left: spacing::LG,
                {props.children}
            }
        }
    }
}

/// Compact single-line input used inside bottom-sheet forms.
#[component]
pub fn BottomSheetTextInput(
    #[props(default)] accessibility_label: Option<String>,
    placeholder: Option<String>,
    value: Option<String>,
    on_change: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();

    rsx! {
        textinput {
            accessibility_role: "text_input",
            accessibility_text: if let Some(label) = accessibility_label.or_else(|| placeholder.clone()) { label },
            value: if let Some(value) = value { value },
            placeholder: if let Some(placeholder) = placeholder { placeholder },
            placeholder_color: theme.colors.muted_foreground,
            caret_color: theme.colors.primary,
            width: "100%",
            height: control::HEIGHT,
            padding_top: spacing::XXS,
            padding_right: spacing::MD,
            padding_bottom: spacing::XXS,
            padding_left: spacing::MD,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_radius: theme.radii.md,
            background_color: theme.colors.background,
            font_size: typography::SM,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            onchange: move |evt| {
                if let Some(handler) = on_change {
                    handler.call(evt.data().string_value.clone());
                }
            },
        }
    }
}
