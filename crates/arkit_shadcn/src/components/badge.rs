use super::*;
use arkit_icon as lucide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

fn badge_style(variant: BadgeVariant) -> (u32, u32, Vec<f32>, Vec<u32>) {
    match variant {
        BadgeVariant::Default => (
            color::PRIMARY,
            color::PRIMARY_FOREGROUND,
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0x00000000],
        ),
        BadgeVariant::Secondary => (
            color::SECONDARY,
            color::SECONDARY_FOREGROUND,
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0x00000000],
        ),
        BadgeVariant::Destructive => (
            color::DESTRUCTIVE,
            color::DESTRUCTIVE_FOREGROUND,
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0x00000000],
        ),
        BadgeVariant::Outline => (
            color::BACKGROUND,
            color::FOREGROUND,
            vec![1.0, 1.0, 1.0, 1.0],
            vec![color::BORDER],
        ),
    }
}

fn badge_label_text(content: impl Into<String>, foreground: u32) -> Element {
    arkit::text(content)
        .font_size(typography::XS)
        .style(ArkUINodeAttributeType::FontWeight, 4_i32)
        .style(ArkUINodeAttributeType::FontColor, foreground)
        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
        .into()
}

fn badge_shell(
    background: u32,
    border_width: Vec<f32>,
    border_color: Vec<u32>,
    horizontal_padding: f32,
    children: Vec<Element>,
) -> Element {
    crate::styles::rounded(
        arkit::row_component()
            .constraint_size(0.0, 100000.0, 20.0, 100000.0)
            .style(ArkUINodeAttributeType::BackgroundColor, background)
            .style(ArkUINodeAttributeType::BorderWidth, border_width)
            .style(ArkUINodeAttributeType::BorderColor, border_color)
            .style(ArkUINodeAttributeType::Clip, true)
            .align_items_center()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![2.0, horizontal_padding, 2.0, horizontal_padding],
            )
            .children(children),
        radius::FULL,
    )
    .into()
}

pub fn badge(label: impl Into<String>) -> Element {
    badge_with_variant(label, BadgeVariant::Default)
}

pub fn badge_with_variant(label: impl Into<String>, variant: BadgeVariant) -> Element {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    badge_shell(
        background,
        border_width,
        border_color,
        spacing::SM,
        vec![badge_label_text(label, foreground)],
    )
}

pub fn badge_with_icon(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: BadgeVariant,
) -> Element {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    badge_shell(
        background,
        border_width,
        border_color,
        spacing::SM,
        vec![
            lucide::icon(icon_name.into())
                .size(14.0)
                .color(foreground)
                .render(),
            arkit::row_component()
                .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 4.0])
                .children(vec![badge_label_text(label, foreground)])
                .into(),
        ],
    )
}

pub fn badge_with_icon_colors(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    background: u32,
    foreground: u32,
) -> Element {
    badge_shell(
        background,
        vec![1.0, 1.0, 1.0, 1.0],
        vec![0x00000000],
        spacing::SM,
        vec![
            lucide::icon(icon_name.into())
                .size(14.0)
                .color(foreground)
                .render(),
            arkit::row_component()
                .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 4.0])
                .children(vec![badge_label_text(label, foreground)])
                .into(),
        ],
    )
}

pub fn pill_badge_with_variant(label: impl Into<String>, variant: BadgeVariant) -> Element {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    crate::styles::rounded(
        arkit::row_component()
            .constraint_size(20.0, 100000.0, 20.0, 100000.0)
            .style(ArkUINodeAttributeType::BackgroundColor, background)
            .style(ArkUINodeAttributeType::BorderWidth, border_width)
            .style(ArkUINodeAttributeType::BorderColor, border_color)
            .style(ArkUINodeAttributeType::Clip, true)
            .align_items_center()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![2.0, spacing::XXS, 2.0, spacing::XXS],
            )
            .children(vec![badge_label_text(label, foreground)]),
        radius::FULL,
    )
    .into()
}
