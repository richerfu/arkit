use super::*;

pub fn combobox(options: Vec<String>, value: Signal<String>) -> Element {
    super::select::select(options, value)
}
