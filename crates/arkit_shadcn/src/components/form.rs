use super::*;

pub fn form(fields: Vec<Element>, submit_label: impl Into<String>) -> Element {
    card(
        fields
            .into_iter()
            .chain(std::iter::once(
                margin_top(button(submit_label, ButtonVariant::Default), spacing::SM).into(),
            ))
            .collect(),
    )
}

pub fn form_item(label_text: impl Into<String>, field: Element) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            label(label_text).into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::XXS, 0.0, 0.0, 0.0],
                )
                .children(vec![field])
                .into(),
        ])
        .into()
}
