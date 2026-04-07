use super::*;

pub fn combobox(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    super::select::select(options, selected, on_select)
}
