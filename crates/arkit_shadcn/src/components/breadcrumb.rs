use super::*;

pub fn breadcrumb(items: Vec<String>) -> Element {
    let mut children = Vec::new();
    let total = items.len();
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            children.push(
                arkit::text("/")
                    .font_size(typography::SM)
                    .font_color(color::MUTED_FOREGROUND)
                    .into(),
            );
        }
        if index + 1 == total {
            children.push(body_text_regular(item).font_color(color::FOREGROUND).into());
        } else {
            children.push(muted_text(item).into());
        }
    }
    arkit::row_component()
        .percent_width(1.0)
        .align_items_center()
        .children(children)
        .into()
}

pub fn breadcrumb_item(title: impl Into<String>) -> Element {
    muted_text(title).into()
}
