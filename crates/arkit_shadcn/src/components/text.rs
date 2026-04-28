use super::*;
use arkit::TextAlignment;

const TRACKING_TIGHT: f32 = -0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVariant {
    Default,
    H1,
    H2,
    H3,
    P,
    Blockquote,
    Code,
    Lead,
    Large,
    Small,
    Muted,
}

fn base_text<Message: 'static>(content: impl Into<String>) -> TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .font_color(colors().foreground)
        .line_height(24.0)
        .text_align(TextAlignment::Start)
}

fn text<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    text_with_variant(content, TextVariant::Default)
}

/// Equivalent to Tailwind `text-sm` (no extra weight / leading utilities).
fn text_sm<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_color(colors().foreground)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
        .into()
}

/// Equivalent to Tailwind `text-sm font-medium` (default `text-sm` line height).
pub(super) fn text_sm_medium<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_weight(FontWeight::W500)
        .font_color(colors().foreground)
        .line_height(20.0)
        .text_align(TextAlignment::Start)
        .into()
}

fn text_with_variant<Message: 'static>(
    content: impl Into<String>,
    variant: TextVariant,
) -> Element<Message> {
    let content = content.into();
    match variant {
        TextVariant::Default => base_text::<Message>(content).into(),
        TextVariant::H1 => base_text::<Message>(content)
            .font_size(36.0)
            .font_weight(FontWeight::W700)
            .line_height(40.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .text_align(TextAlignment::Center)
            .into(),
        TextVariant::H2 => arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .children(vec![
                base_text::<Message>(content)
                    .font_size(30.0)
                    .font_weight(FontWeight::W600)
                    .line_height(36.0)
                    .text_letter_spacing(TRACKING_TIGHT)
                    .into(),
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .height(1.0)
                    .margin_top(8.0)
                    .background_color(colors().border)
                    .into(),
            ])
            .into(),
        TextVariant::H3 => base_text::<Message>(content)
            .font_size(24.0)
            .font_weight(FontWeight::W600)
            .line_height(32.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .into(),
        TextVariant::P => base_text::<Message>(content).line_height(28.0).into(),
        TextVariant::Blockquote => arkit::row_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .border_width([0.0, 0.0, 0.0, 2.0])
            .border_color(colors().border)
            .padding([0.0, 0.0, 0.0, 12.0])
            .children(vec![base_text::<Message>(content)
                .font_style(FontStyle::Italic)
                .line_height(24.0)
                .into()])
            .into(),
        TextVariant::Code => crate::styles::rounded(
            arkit::row_component::<Message, arkit::Theme>()
                .background_color(colors().muted)
                .padding([5.0, 3.0])
                .children(vec![arkit::text::<Message, arkit::Theme>(content)
                    .font_size(typography::SM)
                    .font_family("monospace")
                    .font_weight(FontWeight::W600)
                    .font_color(colors().foreground)
                    .line_height(18.0)
                    .into()]),
            radii().sm,
        )
        .into(),
        TextVariant::Lead => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::XL)
            .font_color(colors().muted_foreground)
            .line_height(28.0)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Large => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::LG)
            .font_weight(FontWeight::W600)
            .font_color(colors().foreground)
            .line_height(28.0)
            .text_letter_spacing(TRACKING_TIGHT)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Small => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_weight(FontWeight::W500)
            .font_color(colors().foreground)
            .line_height(14.0)
            .text_align(TextAlignment::Start)
            .into(),
        TextVariant::Muted => arkit::text::<Message, arkit::Theme>(content)
            .font_size(typography::SM)
            .font_color(colors().muted_foreground)
            .line_height(20.0)
            .text_align(TextAlignment::Start)
            .into(),
    }
}

fn text_variant<Message: 'static>(content: impl Into<String>, muted: bool) -> Element<Message> {
    text_with_variant(
        content,
        if muted {
            TextVariant::Muted
        } else {
            TextVariant::Default
        },
    )
}

// Struct component API
pub struct Text<Message = ()> {
    content: String,
    variant: TextVariant,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Text<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            variant: TextVariant::P,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn small(content: impl Into<String>) -> Self {
        Self::new(content).variant(TextVariant::Small)
    }

    pub fn small_medium(content: impl Into<String>) -> Self {
        Self::new(content).variant(TextVariant::Small)
    }

    pub fn with_variant(content: impl Into<String>, variant: TextVariant) -> Self {
        Self::new(content).variant(variant)
    }

    pub fn muted(content: impl Into<String>) -> Self {
        Self::new(content).variant(TextVariant::Muted)
    }

    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl_component_widget!(Text<Message>, Message, |value: &Text<Message>| {
    text_with_variant(value.content.clone(), value.variant)
});
