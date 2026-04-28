use super::separator::separator_vertical;
use super::*;

fn resizable(left: Element, right: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(inline(
            vec![left, separator_vertical(120.0), right],
            spacing::SM,
        ))
        .into()
}
