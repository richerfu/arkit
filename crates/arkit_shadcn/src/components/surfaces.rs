use super::*;

pub fn toast(message: impl Into<String>) -> Element {
    panel_surface(
        arkit::row_component()
            .percent_width(1.0)
            .padding([spacing::SM, spacing::LG, spacing::SM, spacing::LG])
            .align_items_center()
            .children(vec![body_text_regular(message)
                .font_color(color::FOREGROUND)
                .into()]),
    )
    .into()
}

pub fn toast_destructive(message: impl Into<String>) -> Element {
    border_color(
        panel_surface(arkit::row_component().percent_width(1.0).children(vec![
            body_text_regular(message)
                .font_color(color::DESTRUCTIVE)
                .into(),
        ]))
        .padding([spacing::SM, spacing::LG, spacing::SM, spacing::LG]),
        color::DESTRUCTIVE,
    )
    .into()
}

pub fn sonner(messages: Vec<String>) -> Element {
    super::card::card(messages.into_iter().map(toast).collect::<Vec<Element>>())
}
