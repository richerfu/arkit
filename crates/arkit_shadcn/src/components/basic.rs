use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

pub fn button(label: impl Into<String>, variant: ButtonVariant) -> ButtonElement {
    let label = label.into();
    let base = rounded_button_surface(arkit::button_component())
        .label(label)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontWeight, 4_i32)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .height(40.0);

    match variant {
        ButtonVariant::Default => base
            .background_color(color::PRIMARY)
            .style(ArkUINodeAttributeType::FontColor, color::PRIMARY_FOREGROUND)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 0.0],
            ),
        ButtonVariant::Secondary => base
            .background_color(color::SECONDARY)
            .style(
                ArkUINodeAttributeType::FontColor,
                color::SECONDARY_FOREGROUND,
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 0.0],
            ),
        ButtonVariant::Outline => base
            .background_color(color::BACKGROUND)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT]),
        ButtonVariant::Ghost => base
            .background_color(0x00000000)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 0.0],
            ),
        ButtonVariant::Destructive => base
            .background_color(color::DESTRUCTIVE)
            .style(
                ArkUINodeAttributeType::FontColor,
                color::DESTRUCTIVE_FOREGROUND,
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 0.0],
            ),
        ButtonVariant::Link => base
            .background_color(0x00000000)
            .style(ArkUINodeAttributeType::FontColor, color::PRIMARY)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![0.0, 0.0, 0.0, 0.0],
            ),
    }
}

pub fn icon_button(icon: impl Into<String>) -> ButtonElement {
    button(icon, ButtonVariant::Ghost)
        .width(40.0)
        .height(40.0)
        .style(ArkUINodeAttributeType::Padding, vec![0.0])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

pub fn badge(label: impl Into<String>) -> Element {
    badge_with_variant(label, BadgeVariant::Default)
}

pub fn badge_with_variant(label: impl Into<String>, variant: BadgeVariant) -> Element {
    let (bg, fg, border_width, border_color) = match variant {
        BadgeVariant::Default => (
            color::PRIMARY,
            color::PRIMARY_FOREGROUND,
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0x00000000],
        ),
        BadgeVariant::Secondary => (
            color::SECONDARY,
            color::SECONDARY_FOREGROUND,
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0x00000000],
        ),
        BadgeVariant::Destructive => (
            color::DESTRUCTIVE,
            color::DESTRUCTIVE_FOREGROUND,
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0x00000000],
        ),
        BadgeVariant::Outline => (
            color::BACKGROUND,
            color::FOREGROUND,
            vec![1.0, 1.0, 1.0, 1.0],
            vec![color::BORDER],
        ),
    };

    chip_surface(
        arkit::row_component()
            .style(ArkUINodeAttributeType::BackgroundColor, bg)
            .style(ArkUINodeAttributeType::BorderWidth, border_width)
            .style(ArkUINodeAttributeType::BorderColor, border_color)
            .children(vec![arkit::text(label)
                .font_size(typography::XS)
                .style(ArkUINodeAttributeType::FontWeight, 4_i32)
                .style(ArkUINodeAttributeType::FontColor, fg)
                .into()]),
    )
    .into()
}

pub fn label(content: impl Into<String>) -> TextElement {
    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontWeight, 4_i32)
        .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
}

pub fn input(placeholder: impl Into<String>) -> TextInputElement {
    input_surface(
        arkit::text_input_component()
            .style(
                ArkUINodeAttributeType::TextInputPlaceholderColor,
                color::MUTED_FOREGROUND,
            )
            .font_size(typography::SM)
            .height(40.0),
    )
    .placeholder(placeholder)
}

pub fn textarea(placeholder: impl Into<String>) -> TextAreaElement {
    input_surface(arkit::text_area_component())
        .style(
            ArkUINodeAttributeType::TextAreaPlaceholder,
            placeholder.into(),
        )
        .style(
            ArkUINodeAttributeType::TextAreaPlaceholderColor,
            color::MUTED_FOREGROUND,
        )
        .font_size(typography::SM)
        .height(80.0)
}

pub fn switch(state: Signal<bool>) -> ToggleElement {
    let next = state.clone();
    arkit::toggle_component()
        .style(ArkUINodeAttributeType::ToggleValue, state.get())
        .style(ArkUINodeAttributeType::ToggleSelectedColor, color::PRIMARY)
        .style(ArkUINodeAttributeType::ToggleUnselectedColor, color::INPUT)
        .style(
            ArkUINodeAttributeType::ToggleSwitchPointColor,
            color::PRIMARY_FOREGROUND,
        )
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 0.0, 0.0],
        )
        .width(32.0)
        .height(18.0)
        .on_click(move || next.update(|v| *v = !*v))
}

pub fn toggle(label: impl Into<String>, state: Signal<bool>) -> ButtonElement {
    let next = state.clone();
    let variant = if state.get() {
        ButtonVariant::Default
    } else {
        ButtonVariant::Outline
    };
    button(label, variant).on_click(move || next.update(|v| *v = !*v))
}

pub fn toggle_group(options: Vec<String>, selected: Signal<String>) -> Element {
    let children = options
        .into_iter()
        .map(|item| {
            let text = item.clone();
            let value = selected.clone();
            let variant = if selected.get() == text {
                ButtonVariant::Default
            } else {
                ButtonVariant::Outline
            };
            button(item, variant)
                .on_click(move || value.set(text.clone()))
                .into()
        })
        .collect::<Vec<_>>();

    arkit::row_component()
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

pub fn checkbox(label: impl Into<String>, checked: Signal<bool>) -> Element {
    let click = checked.clone();
    arkit::row_component()
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(vec![
            arkit::checkbox_component()
                .style(ArkUINodeAttributeType::CheckboxSelect, checked.get())
                .style(ArkUINodeAttributeType::CheckboxSelectColor, color::PRIMARY)
                .style(
                    ArkUINodeAttributeType::CheckboxUnselectColor,
                    color::BACKGROUND,
                )
                .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
                .style(
                    ArkUINodeAttributeType::BorderWidth,
                    vec![1.0, 1.0, 1.0, 1.0],
                )
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![4.0, 4.0, 4.0, 4.0],
                )
                .width(16.0)
                .height(16.0)
                .on_click(move || click.update(|v| *v = !*v))
                .into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, spacing::SM],
                )
                .children(vec![muted_text(label).into()])
                .into(),
        ])
        .into()
}

pub fn slider(value: f32, min: f32, max: f32) -> SliderElement {
    input_surface(
        arkit::slider_component()
            .style(ArkUINodeAttributeType::SliderValue, value)
            .style(ArkUINodeAttributeType::SliderMinValue, min)
            .style(ArkUINodeAttributeType::SliderMaxValue, max),
    )
}

pub fn progress(value: f32, total: f32) -> ProgressElement {
    rounded_progress(
        arkit::progress_component()
            .style(ArkUINodeAttributeType::ProgressValue, value)
            .style(ArkUINodeAttributeType::ProgressTotal, total)
            .style(ArkUINodeAttributeType::ProgressColor, color::PRIMARY)
            .height(8.0),
    )
}

pub fn skeleton(width: f32, height: f32) -> Element {
    arkit::row_component()
        .width(width)
        .height(height)
        .background_color(color::ACCENT)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .into()
}

pub fn separator() -> Element {
    arkit::row_component()
        .height(1.0)
        .percent_width(1.0)
        .background_color(color::BORDER)
        .into()
}

pub fn separator_vertical(height: f32) -> Element {
    arkit::column_component()
        .width(1.0)
        .height(height)
        .background_color(color::BORDER)
        .into()
}
