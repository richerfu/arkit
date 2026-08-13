//! Toggle group — ColorUI `.cu-capsule` (line vs fill).

use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn ToggleGroup(
    options: Vec<String>,
    #[props(default)] selected: Option<Vec<String>>,
    #[props(default)] default_selected: Vec<String>,
    #[props(default)] icons: bool,
    #[props(default)] multi: bool,
    #[props(default)] width: Option<String>,
    #[props(default)] height: Option<f32>,
    #[props(default)] shadow: Option<bool>,
    #[props(default)] on_change: EventHandler<Vec<String>>,
) -> Element {
    let _ = (shadow, width);
    let theme = use_colorui_theme();
    let fill = swatch(theme.primary).fill;
    let controlled = selected.is_some();
    let local = use_signal(|| default_selected.clone());
    let current: Vec<String> = selected.unwrap_or_else(|| local.read().clone());
    let item_h = height.unwrap_or(spec::BTN_HEIGHT);

    let items: Vec<Element> = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let active = current.contains(option);
            let value = option.clone();
            let selected_now = current.clone();
            let mut local = local;
            rsx! {
                button {
                    button_type: "normal",
                    height: item_h,
                    padding_left: spec::BTN_PAD_SM,
                    padding_right: spec::BTN_PAD_SM,
                    background_color: if active { fill } else { 0x00000000u32 },
                    border_width: 1.0,
                    border_color: fill,
                    border_radius: if index == 0 {
                        format!("{},0,0,{}", spec::RADIUS, spec::RADIUS)
                    } else if index + 1 == options.len() {
                        format!("0,{},{},0", spec::RADIUS, spec::RADIUS)
                    } else {
                        "0,0,0,0".into()
                    },
                    focusable: false,
                    focus_on_touch: false,
                    onclick: move |_| {
                        let mut next = selected_now.clone();
                        if multi {
                            if let Some(i) = next.iter().position(|item| item == &value) {
                                next.remove(i);
                            } else {
                                next.push(value.clone());
                            }
                        } else {
                            next = vec![value.clone()];
                        }
                        if !controlled {
                            local.set(next.clone());
                        }
                        on_change.call(next);
                    },
                    if icons {
                        {arkit_icon::icon(option.clone(), 14.0, if active { spec::INK_ON_FILL } else { fill })}
                    } else {
                        text {
                            content: option.clone(),
                            font_size: spec::TEXT_SM,
                            font_color: if active { spec::INK_ON_FILL } else { fill },
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        row {
            align_items: "center",
            {items.into_iter()}
        }
    }
}
