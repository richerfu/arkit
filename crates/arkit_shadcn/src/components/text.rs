//! Text — shadcn typography scale (`text-sm` / `text-lg` / `text-4xl` mapped).

use arkit_component::components::TextVariant;
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Text(content: String, #[props(default)] variant: TextVariant) -> Element {
    let theme = use_theme();
    match variant {
        TextVariant::Default => rsx! {
            text {
                content,
                font_size: spec::TEXT_BASE,
                font_color: theme.colors.foreground,
                line_height: 24.0,
            }
        },
        TextVariant::P => rsx! {
            text {
                content,
                font_size: spec::TEXT_BASE,
                font_color: theme.colors.foreground,
                line_height: 28.0,
            }
        },
        TextVariant::H1 => rsx! {
            text {
                content,
                font_size: 36.0,
                font_weight: 700,
                font_color: theme.colors.foreground,
                line_height: 40.0,
                text_align: "center",
            }
        },
        TextVariant::H2 => rsx! {
            text {
                content,
                font_size: 30.0,
                font_weight: spec::FONT_SEMIBOLD,
                font_color: theme.colors.foreground,
                line_height: 36.0,
            }
        },
        TextVariant::H3 => rsx! {
            text {
                content,
                font_size: spec::TEXT_XL,
                font_weight: spec::FONT_SEMIBOLD,
                font_color: theme.colors.foreground,
                line_height: 32.0,
            }
        },
        TextVariant::Small => rsx! {
            text {
                content,
                font_size: spec::TEXT_SM,
                font_weight: spec::FONT_MEDIUM,
                font_color: theme.colors.foreground,
                line_height: 14.0,
            }
        },
        TextVariant::Muted => rsx! {
            text {
                content,
                font_size: spec::TEXT_SM,
                font_color: theme.colors.muted_foreground,
                line_height: 20.0,
            }
        },
        TextVariant::Large => rsx! {
            text {
                content,
                font_size: spec::TEXT_LG,
                font_weight: spec::FONT_SEMIBOLD,
                font_color: theme.colors.foreground,
                line_height: 28.0,
            }
        },
        TextVariant::Lead => rsx! {
            text {
                content,
                font_size: spec::TEXT_XL,
                font_color: theme.colors.muted_foreground,
                line_height: 28.0,
            }
        },
        TextVariant::Code => rsx! {
            row {
                background_color: theme.colors.muted,
                border_radius: spec::RADIUS_MD,
                padding_left: 6.0,
                padding_right: 6.0,
                text {
                    content,
                    font_size: spec::TEXT_SM,
                    font_weight: spec::FONT_SEMIBOLD,
                    font_color: theme.colors.foreground,
                }
            }
        },
        TextVariant::Blockquote => rsx! {
            text {
                content,
                font_size: spec::TEXT_BASE,
                font_style: "italic",
                font_color: theme.colors.foreground,
                line_height: 24.0,
            }
        },
    }
}
