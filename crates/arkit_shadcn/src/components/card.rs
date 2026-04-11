use super::*;

pub fn card<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    card_surface(
        arkit::column_component::<Message, arkit::Theme>()
            .width(arkit::Length::Fill)
            .children(vec![stack(children, spacing::XXL)]),
    )
    .into()
}

pub fn card_header<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .children(vec![arkit::column_component::<Message, arkit::Theme>()
            .width(arkit::Length::Fill)
            .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
            .children(vec![
                card_title(title),
                arkit::row_component::<Message, arkit::Theme>()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![spacing::XS, 0.0, 0.0, 0.0],
                    )
                    .children(vec![card_description(description)])
                    .into(),
            ])
            .into()])
        .into()
}

pub fn card_title<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .style(ArkUINodeAttributeType::FontWeight, 5_i32)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
        .style(ArkUINodeAttributeType::TextLetterSpacing, -0.2_f32)
        .into()
}

pub fn card_description<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    muted_text(content).into()
}

pub fn card_content<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
        .children(children)
        .into()
}

pub fn card_footer<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
        .align_items_center()
        .children(children)
        .into()
}
