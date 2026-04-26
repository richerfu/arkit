use super::button::button;
use super::card::card;
use super::label::label;
use super::*;

fn form(fields: Vec<Element>, submit_label: impl Into<String>) -> Element {
    card(
        fields
            .into_iter()
            .chain(std::iter::once(
                button(submit_label)
                    .theme(ButtonVariant::Default)
                    .margin_top(spacing::SM)
                    .into(),
            ))
            .collect(),
    )
}

fn form_item(label_text: impl Into<String>, field: Element) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            label(label_text).into(),
            arkit::row_component()
                .margin_top(spacing::XXS)
                .children(vec![field])
                .into(),
        ])
        .into()
}
