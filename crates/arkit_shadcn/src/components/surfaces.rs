use super::*;

pub fn card(children: Vec<Element>) -> Element {
    card_surface(
        arkit::column_component()
            .percent_width(1.0)
            .children(vec![stack(children, spacing::XXL)]),
    )
    .into()
}

pub fn card_header(title: impl Into<String>, description: impl Into<String>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, spacing::XXL, 0.0, spacing::XXL],
        )
        .children(vec![
            card_title(title),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::XS, 0.0, 0.0, 0.0],
                )
                .children(vec![card_description(description)])
                .into(),
        ])
        .into()
}

pub fn card_title(content: impl Into<String>) -> Element {
    title_text(content)
        .style(ArkUINodeAttributeType::FontSize, typography::MD)
        .style(ArkUINodeAttributeType::FontWeight, 5_i32)
        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
        .into()
}

pub fn card_description(content: impl Into<String>) -> Element {
    muted_text(content).into()
}

pub fn card_content(children: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, spacing::XXL, 0.0, spacing::XXL],
        )
        .children(vec![stack(children, spacing::SM)])
        .into()
}

pub fn card_footer(children: Vec<Element>) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, spacing::XXL, 0.0, spacing::XXL],
        )
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

pub fn toast(message: impl Into<String>) -> Element {
    panel_surface(
        arkit::row_component()
            .percent_width(1.0)
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::SM, spacing::LG, spacing::SM, spacing::LG],
            )
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .children(vec![body_text_regular(message)
                .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                .into()]),
    )
    .into()
}

pub fn toast_destructive(message: impl Into<String>) -> Element {
    border_color(
        panel_surface(arkit::row_component().percent_width(1.0).children(vec![
            body_text_regular(message)
                .style(ArkUINodeAttributeType::FontColor, color::DESTRUCTIVE)
                .into(),
        ]))
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::SM, spacing::LG, spacing::SM, spacing::LG],
        ),
        color::DESTRUCTIVE,
    )
    .into()
}

pub fn sonner(messages: Vec<String>) -> Element {
    card(messages.into_iter().map(toast).collect::<Vec<Element>>())
}
