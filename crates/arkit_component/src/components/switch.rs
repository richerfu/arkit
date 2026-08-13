//! Switch — unstyled toggle.

use crate::appearance::{SwitchAppearance, SwitchStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Switch`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    pub checked: Option<bool>,
    pub default_checked: Option<bool>,
    pub on_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub appearance: Option<SwitchAppearance>,
}

/// A toggle switch.
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let kit = use_style_kit();
    let mut internal = use_signal(|| props.default_checked.unwrap_or(false));
    let controlled = props.checked.is_some();
    let current = props.checked.unwrap_or_else(|| *internal.read());
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.switch(&SwitchStyleInput { checked: current }));
    let on_change = props.on_change;

    rsx! {
        toggle {
            checked: current,
            toggle_selected_color: appearance.selected,
            toggle_unselected_color: appearance.unselected,
            toggle_switch_point_color: appearance.knob,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            border_radius: appearance.radius,
            clip: true,
            width: appearance.width,
            height: appearance.height,
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
