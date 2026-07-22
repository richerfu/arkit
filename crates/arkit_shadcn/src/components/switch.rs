//! Switch — shadcn-style toggle switch.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original styling: 32x18.4 native `Toggle` with
//! `primary` selected color, `input` unselected color, `background` switch
//! point, transparent 1px border, `full` radius, `shadow-sm`. Supports
//! controlled (`checked`) and uncontrolled (`default_checked`) usage.

use crate::theme::*;
use arkit_prelude::*;

const SWITCH_WIDTH: f32 = 32.0;
const SWITCH_HEIGHT: f32 = 18.4;

/// Props for [`Switch`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    pub checked: Option<bool>,
    pub default_checked: Option<bool>,
    pub on_change: Option<EventHandler<bool>>,
}

/// A toggle switch.
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| props.default_checked.unwrap_or(false));
    let controlled = props.checked.is_some();
    let current = props.checked.unwrap_or_else(|| *internal.read());
    let on_change = props.on_change;

    rsx! {
        toggle {
            checked: current,
            toggle_selected_color: theme.colors.primary,
            toggle_unselected_color: theme.colors.input,
            toggle_switch_point_color: theme.colors.background,
            border_width: 1.0,
            border_color: 0x0000_0000,
            border_radius: theme.radii.full,
            clip: true,
            width: SWITCH_WIDTH,
            height: SWITCH_HEIGHT,
            onclick: move |_| {
                let next = !current;
                if !controlled {
                    internal.set(next);
                }
                if let Some(handler) = on_change {
                    handler.call(next);
                }
            },
        }
    }
}
