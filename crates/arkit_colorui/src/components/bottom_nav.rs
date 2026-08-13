//! Bottom navigation — `.cu-bar.tabbar`.

use arkit_component::components::BottomNavigationItem;
use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn BottomNavigation(
    items: Vec<BottomNavigationItem>,
    #[props(default)] selected: Option<usize>,
    #[props(default)] default_selected: usize,
    #[props(default)] on_select: EventHandler<usize>,
) -> Element {
    let theme = use_colorui_theme();
    let fill = swatch(theme.primary).fill;
    let item_count = items.len();
    let mut local = use_signal(move || default_selected.min(item_count.saturating_sub(1)));
    let controlled = selected.is_some();
    let current = selected
        .unwrap_or_else(|| *local.read())
        .min(item_count.saturating_sub(1));

    let destinations: Vec<Element> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let active = index == current;
            let color = if active { fill } else { spec::TEXT_GREY };
            let label = item.label.clone();
            let icon = item.icon.clone();
            rsx! {
                column {
                    layout_weight: 1.0,
                    align_items: "center",
                    justify_content: "center",
                    padding_top: 6.0,
                    padding_bottom: 6.0,
                    onclick: move |_| {
                        if !controlled {
                            local.set(index);
                        }
                        on_select.call(index);
                    },
                    {arkit_icon::icon(icon, 20.0, color)}
                    text {
                        content: label,
                        font_size: spec::TEXT_XS,
                        font_color: color,
                        margin_top: 4.0,
                    }
                }
            }
        })
        .collect();

    rsx! {
        row {
            width: "100%",
            height: spec::BAR_HEIGHT,
            background_color: spec::BG_WHITE,
            align_items: "center",
            {destinations.into_iter()}
        }
    }
}
