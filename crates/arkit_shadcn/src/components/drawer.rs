use super::*;

pub fn drawer(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    content: Vec<Element>,
) -> Element {
    dialog(title, open, on_open_change, content)
}
