use super::*;

pub fn sheet(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    dialog(title, open, content)
}
