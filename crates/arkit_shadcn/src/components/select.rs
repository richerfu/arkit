use super::*;
use arkit::{component, use_signal};
use arkit_icon as lucide;

#[component]
pub fn select(options: Vec<String>, selected: Signal<String>) -> Element {
    let open = use_signal(|| false);
    let current = selected.get();
    let has_value = !current.is_empty();
    let label = if has_value {
        current.clone()
    } else {
        String::from("Select a fruit")
    };
    let toggle_open = open.clone();

    let trigger = shadow_sm(crate::styles::rounded(
        crate::styles::border(
            arkit::row_component()
                .height(40.0)
                .percent_width(1.0)
                .background_color(color::BACKGROUND)
                .style(ArkUINodeAttributeType::ForegroundColor, color::FOREGROUND)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![8.0, spacing::MD, 8.0, spacing::MD],
                )
                .align_items_center()
                .style(
                    ArkUINodeAttributeType::RowJustifyContent,
                    FLEX_ALIGN_SPACE_BETWEEN,
                )
                .children(vec![
                    arkit::text(label)
                        .font_size(typography::SM)
                        .style(
                            ArkUINodeAttributeType::FontColor,
                            if has_value {
                                color::FOREGROUND
                            } else {
                                color::MUTED_FOREGROUND
                            },
                        )
                        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                        .into(),
                    lucide::icon("chevron-down")
                        .size(16.0)
                        .color(color::MUTED_FOREGROUND)
                        .render(),
                ]),
        ),
        radius::MD,
    ))
    .on_click(move || toggle_open.update(|value| *value = !*value))
    .into();

    let mut children = vec![trigger];
    if open.get() {
        let close = open.clone();
        let count = options.len();
        let items = options
            .into_iter()
            .map(|option| {
                let value = selected.clone();
                let close_dropdown = close.clone();
                let active = current == option;
                let option_label = option.clone();
                arkit::row_component()
                    .percent_width(1.0)
                    .height(36.0)
                    .align_items_center()
                    .style(
                        ArkUINodeAttributeType::RowJustifyContent,
                        FLEX_ALIGN_SPACE_BETWEEN,
                    )
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![8.0, spacing::SM, 8.0, spacing::SM],
                    )
                    .style(
                        ArkUINodeAttributeType::BorderRadius,
                        vec![radius::SM, radius::SM, radius::SM, radius::SM],
                    )
                    .background_color(if active { color::ACCENT } else { 0x00000000 })
                    .on_click(move || {
                        value.set(option.clone());
                        close_dropdown.set(false);
                    })
                    .children(vec![
                        arkit::text(option_label)
                            .font_size(typography::SM)
                            .style(
                                ArkUINodeAttributeType::FontColor,
                                if active {
                                    color::ACCENT_FOREGROUND
                                } else {
                                    color::FOREGROUND
                                },
                            )
                            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                            .into(),
                        if active {
                            lucide::icon("check")
                                .size(16.0)
                                .color(color::MUTED_FOREGROUND)
                                .render()
                        } else {
                            arkit::row_component().width(16.0).height(16.0).into()
                        },
                    ])
                    .into()
            })
            .collect::<Vec<_>>();

        let list = if count > 8 {
            arkit::scroll_component()
                .height(208.0)
                .children(vec![arkit::column_component()
                    .percent_width(1.0)
                    .children(items)
                    .into()])
                .into()
        } else {
            arkit::column_component()
                .percent_width(1.0)
                .children(items)
                .into()
        };

        children.push(
            margin_top(
                panel_surface(
                    arkit::column_component()
                        .percent_width(1.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
                        )
                        .children(vec![
                            arkit::row_component()
                                .style(
                                    ArkUINodeAttributeType::Padding,
                                    vec![8.0, spacing::SM, 8.0, spacing::SM],
                                )
                                .children(vec![arkit::text("Fruits")
                                    .font_size(typography::XS)
                                    .style(
                                        ArkUINodeAttributeType::FontColor,
                                        color::MUTED_FOREGROUND,
                                    )
                                    .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                                    .into()])
                                .into(),
                            list,
                        ]),
                ),
                spacing::XXS,
            )
            .into(),
        );
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
