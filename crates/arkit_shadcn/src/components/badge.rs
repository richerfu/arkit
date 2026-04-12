use super::*;
use arkit_icon as lucide;

const BADGE_ICON_SIZE: f32 = 12.0;
const BADGE_VERTICAL_PADDING: f32 = 2.0;
const BADGE_ICON_GAP: f32 = 4.0;
const BADGE_RADIUS: f32 = radius::MD;
const BADGE_TEXT_LINE_HEIGHT: f32 = 16.0;
const BADGE_MIN_HEIGHT: f32 = 22.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

fn badge_style(variant: BadgeVariant) -> (u32, u32, [f32; 4], u32) {
    match variant {
        BadgeVariant::Default => (
            color::PRIMARY,
            color::PRIMARY_FOREGROUND,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Secondary => (
            color::SECONDARY,
            color::SECONDARY_FOREGROUND,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Destructive => (
            color::DESTRUCTIVE,
            color::DESTRUCTIVE_FOREGROUND,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Outline => (
            color::BACKGROUND,
            color::FOREGROUND,
            [1.0, 1.0, 1.0, 1.0],
            color::BORDER,
        ),
    }
}

fn badge_label_text<Message: 'static>(
    content: impl Into<String>,
    foreground: u32,
) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::XS)
        .font_weight(FontWeight::W500)
        .font_color(foreground)
        .line_height(BADGE_TEXT_LINE_HEIGHT)
        .into()
}

fn badge_shell<Message: 'static>(
    background: u32,
    border_width: [f32; 4],
    border_color: u32,
    horizontal_padding: f32,
    children: Vec<Element<Message>>,
) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .constraint_size(0.0, 100000.0, BADGE_MIN_HEIGHT, 100000.0)
        .border_radius([BADGE_RADIUS, BADGE_RADIUS, BADGE_RADIUS, BADGE_RADIUS])
        .background_color(background)
        .border_width(border_width)
        .border_color(border_color)
        .clip(true)
        .align_items_center()
        .justify_content_center()
        .padding([
            BADGE_VERTICAL_PADDING,
            horizontal_padding,
            BADGE_VERTICAL_PADDING,
            horizontal_padding,
        ])
        .children(children)
        .into()
}

pub fn badge<Message: 'static>(label: impl Into<String>) -> Element<Message> {
    badge_with_variant(label, BadgeVariant::Default)
}

pub fn badge_with_variant<Message: 'static>(
    label: impl Into<String>,
    variant: BadgeVariant,
) -> Element<Message> {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    badge_shell(
        background,
        border_width,
        border_color,
        spacing::SM,
        vec![badge_label_text(label, foreground)],
    )
}

pub fn badge_with_icon<Message: 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: BadgeVariant,
) -> Element<Message> {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    badge_shell(
        background,
        border_width,
        border_color,
        spacing::SM,
        vec![
            lucide::icon(icon_name.into())
                .size(BADGE_ICON_SIZE)
                .color(foreground)
                .render::<Message, arkit::Theme>(),
            arkit::row_component::<Message, arkit::Theme>()
                .margin([0.0, 0.0, 0.0, BADGE_ICON_GAP])
                .children(vec![badge_label_text(label, foreground)])
                .into(),
        ],
    )
}

pub fn badge_with_icon_colors<Message: 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    background: u32,
    foreground: u32,
) -> Element<Message> {
    badge_shell(
        background,
        [1.0, 1.0, 1.0, 1.0],
        0x00000000,
        spacing::SM,
        vec![
            lucide::icon(icon_name.into())
                .size(BADGE_ICON_SIZE)
                .color(foreground)
                .render::<Message, arkit::Theme>(),
            arkit::row_component::<Message, arkit::Theme>()
                .margin([0.0, 0.0, 0.0, BADGE_ICON_GAP])
                .children(vec![badge_label_text(label, foreground)])
                .into(),
        ],
    )
}

pub fn pill_badge_with_variant<Message: 'static>(
    label: impl Into<String>,
    variant: BadgeVariant,
) -> Element<Message> {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    arkit::row_component::<Message, arkit::Theme>()
        .constraint_size(20.0, 100000.0, BADGE_MIN_HEIGHT, 100000.0)
        .border_radius([radius::FULL, radius::FULL, radius::FULL, radius::FULL])
        .background_color(background)
        .border_width(border_width)
        .border_color(border_color)
        .clip(true)
        .align_items_center()
        .justify_content_center()
        .padding([
            BADGE_VERTICAL_PADDING,
            spacing::XXS,
            BADGE_VERTICAL_PADDING,
            spacing::XXS,
        ])
        .children(vec![badge_label_text(label, foreground)])
        .into()
}
