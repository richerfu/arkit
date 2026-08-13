use arkit_prelude::*;

use super::select::Select;

#[component]
pub fn Combobox(
    options: Vec<String>,
    placeholder: Option<String>,
    label: Option<String>,
    selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        Select {
            options,
            placeholder,
            label,
            selected: Some(selected),
            default_selected: String::new(),
            open,
            default_open,
            on_open_change,
            on_select,
        }
    }
}
