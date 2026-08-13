//! Calendar — ColorUI selected day uses `.bg-green` instead of shadcn sky-600.

use arkit_component::components::{Calendar as HeadlessCalendar, CalendarProps};
use arkit_prelude::*;

use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let theme = use_colorui_theme();
    let fill = swatch(theme.primary).fill;
    let mut props = props;
    if props.selection_color.is_none() {
        props.selection_color = Some(fill);
    }
    if props.today_color.is_none() {
        props.today_color = Some(fill);
    }
    super::paint::forward(HeadlessCalendar, props, "Calendar")
}
