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

pub fn alert(title: impl Into<String>, description: impl Into<String>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![14.0, spacing::LG, spacing::SM, spacing::LG],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::LG, radius::LG, radius::LG, radius::LG],
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .background_color(color::CARD)
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .children(vec![
                arkit::row_component()
                    .width(14.0)
                    .children(vec![muted_text("i").into()])
                    .into(),
                arkit::column_component()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![0.0, 0.0, 0.0, spacing::SM],
                    )
                    .children(vec![
                        body_text(title)
                            .style(ArkUINodeAttributeType::FontColor, color::CARD_FOREGROUND)
                            .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                            .into(),
                        margin_top(muted_text(description), spacing::XS).into(),
                    ])
                    .into(),
            ])
            .into()])
        .into()
}

pub fn alert_destructive(title: impl Into<String>, description: impl Into<String>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![14.0, spacing::LG, spacing::SM, spacing::LG],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::LG, radius::LG, radius::LG, radius::LG],
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .background_color(color::CARD)
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .children(vec![
                arkit::row_component()
                    .width(14.0)
                    .children(vec![arkit::text("!")
                        .font_size(typography::SM)
                        .style(ArkUINodeAttributeType::FontColor, color::DESTRUCTIVE)
                        .into()])
                    .into(),
                arkit::column_component()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![0.0, 0.0, 0.0, spacing::SM],
                    )
                    .children(vec![
                        body_text(title)
                            .style(ArkUINodeAttributeType::FontColor, color::DESTRUCTIVE)
                            .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                            .into(),
                        margin_top(
                            muted_text(description)
                                .style(ArkUINodeAttributeType::FontColor, 0xE6EF4444_u32),
                            spacing::XS,
                        )
                        .into(),
                    ])
                    .into(),
            ])
            .into()])
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
