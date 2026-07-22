//! Mobile bottom navigation for switching between top-level destinations.
//!
//! The bar owns only destination selection and presentation. Applications keep
//! page or router state in the caller and update it through `on_select`.
//! Controlled and uncontrolled selection are both supported.

use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

const BAR_HEIGHT: f32 = 64.0;
const ICON_SIZE: f32 = 22.0;
const ICON_LABEL_GAP: f32 = 3.0;
const TRANSPARENT: u32 = 0x00000000;

/// A top-level destination displayed in [`BottomNavigation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottomNavigationItem {
    pub label: String,
    /// Lucide icon name rendered through `arkit_icon`.
    pub icon: String,
}

impl BottomNavigationItem {
    pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
        }
    }
}

/// Props for [`BottomNavigation`].
#[derive(Props, Clone, PartialEq)]
pub struct BottomNavigationProps {
    pub items: Vec<BottomNavigationItem>,
    /// Controlled selected destination. When `Some`, the caller owns state.
    #[props(default)]
    pub selected: Option<usize>,
    /// Initial destination for uncontrolled usage.
    #[props(default)]
    pub default_selected: usize,
    /// Fires with the selected item index. Use it to update page/router state.
    #[props(default)]
    pub on_select: EventHandler<usize>,
}

fn normalized_index(index: usize, item_count: usize) -> usize {
    index.min(item_count.saturating_sub(1))
}

/// A full-width, theme-aware mobile navigation bar with equal-width targets.
#[component]
pub fn BottomNavigation(props: BottomNavigationProps) -> Element {
    let theme = use_theme();
    let item_count = props.items.len();
    let initial = normalized_index(props.default_selected, item_count);
    let mut local = use_signal(move || initial);
    let controlled = props.selected.is_some();
    let selected = normalized_index(props.selected.unwrap_or_else(|| *local.read()), item_count);
    let on_select = props.on_select;

    let destinations: Vec<Element> = props
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let active = index == selected;
            let foreground = if active {
                theme.colors.primary
            } else {
                theme.colors.muted_foreground
            };
            let label = item.label.clone();
            let icon_name = item.icon.clone();

            rsx! {
                row {
                    key: "{index}",
                    layout_weight: 1.0,
                    height: "100%",
                    align_items: "center",
                    justify_content: "center",
                    focusable: false,
                    focus_on_touch: false,
                    background_color: TRANSPARENT,
                    onclick: move |_| {
                        if !controlled {
                            local.set(index);
                        }
                        on_select.call(index);
                    },
                    column {
                        align_items: "center",
                        justify_content: "center",
                        {icon_placeholder(icon_name.as_str(), ICON_SIZE, foreground)}
                        row { height: ICON_LABEL_GAP }
                        text {
                            content: label,
                            font_size: typography::XS,
                            font_weight: if active { 600_i32 } else { 500_i32 },
                            font_color: foreground,
                            line_height: 14.0,
                            text_align: "center",
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        row {
            width: "100%",
            height: BAR_HEIGHT,
            align_items: "center",
            justify_content: "start",
            background_color: theme.colors.background,
            border_width: "1,0,0,0",
            border_color: theme.colors.border,
            {destinations.into_iter()}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalized_index;

    #[test]
    fn selection_is_clamped_to_available_destinations() {
        assert_eq!(normalized_index(2, 4), 2);
        assert_eq!(normalized_index(9, 4), 3);
        assert_eq!(normalized_index(9, 0), 0);
    }
}
