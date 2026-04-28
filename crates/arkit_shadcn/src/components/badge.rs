use super::*;
use arkit_icon as lucide;

const BADGE_ICON_SIZE: f32 = 12.0;
const BADGE_VERTICAL_PADDING: f32 = 2.0;
const BADGE_ICON_GAP: f32 = 4.0;
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
            colors().primary,
            colors().primary_foreground,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Secondary => (
            colors().secondary,
            colors().secondary_foreground,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Destructive => (
            colors().destructive,
            colors().destructive_foreground,
            [1.0, 1.0, 1.0, 1.0],
            0x00000000,
        ),
        BadgeVariant::Outline => (
            colors().background,
            colors().foreground,
            [1.0, 1.0, 1.0, 1.0],
            colors().border,
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
        .border_radius([radii().md, radii().md, radii().md, radii().md])
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

fn badge<Message: 'static>(label: impl Into<String>) -> Element<Message> {
    badge_with_variant(label, BadgeVariant::Default)
}

fn badge_with_variant<Message: 'static>(
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

fn badge_with_icon<Message: 'static>(
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

fn badge_with_icon_colors<Message: 'static>(
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

fn pill_badge_with_variant<Message: 'static>(
    label: impl Into<String>,
    variant: BadgeVariant,
) -> Element<Message> {
    let (background, foreground, border_width, border_color) = badge_style(variant);
    arkit::row_component::<Message, arkit::Theme>()
        .constraint_size(20.0, 100000.0, BADGE_MIN_HEIGHT, 100000.0)
        .border_radius([radii().full, radii().full, radii().full, radii().full])
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

// Struct component API
pub struct Badge<Message = ()> {
    label: String,
    variant: BadgeVariant,
    icon_name: Option<String>,
    colors: Option<(u32, u32)>,
    pill: bool,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Badge<Message> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Default,
            icon_name: None,
            colors: None,
            pill: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(icon_name.into());
        self
    }

    pub fn icon_colors(
        mut self,
        icon_name: impl Into<String>,
        background: u32,
        foreground: u32,
    ) -> Self {
        self.icon_name = Some(icon_name.into());
        self.colors = Some((background, foreground));
        self
    }

    pub fn pill(mut self, pill: bool) -> Self {
        self.pill = pill;
        self
    }
}

impl_component_widget!(Badge<Message>, Message, |value: &Badge<Message>| {
    if value.pill {
        pill_badge_with_variant(value.label.clone(), value.variant)
    } else if let Some((background, foreground)) = value.colors {
        badge_with_icon_colors(
            value.label.clone(),
            value.icon_name.clone().unwrap_or_default(),
            background,
            foreground,
        )
    } else if let Some(icon_name) = value.icon_name.clone() {
        badge_with_icon(value.label.clone(), icon_name, value.variant)
    } else {
        badge_with_variant(value.label.clone(), value.variant)
    }
});
