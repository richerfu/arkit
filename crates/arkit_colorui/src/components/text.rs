//! Text — ColorUI `.text-df` / `.text-lg` / `.text-xl` scale, `#333` / `#888`.

use arkit_component::components::TextVariant;
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_colorui_theme;

#[component]
pub fn Text(content: String, #[props(default)] variant: TextVariant) -> Element {
    let tokens = use_colorui_theme().tokens();
    let (size, weight, color, line, italic, code) = match variant {
        TextVariant::Default | TextVariant::P => {
            (spec::TEXT_DF, 400, spec::TEXT, 22.0, false, false)
        }
        TextVariant::Small => (spec::TEXT_SM, 400, spec::TEXT, 18.0, false, false),
        TextVariant::Muted => (spec::TEXT_SM, 400, spec::TEXT_MUTED, 18.0, false, false),
        TextVariant::Large => (spec::TEXT_LG, 700, spec::TEXT, 24.0, false, false),
        TextVariant::Lead => (spec::TEXT_LG, 400, spec::TEXT_MUTED, 24.0, false, false),
        TextVariant::H3 => (spec::TEXT_XL, 700, spec::TEXT, 26.0, false, false),
        TextVariant::H2 => (spec::TEXT_XXL, 700, spec::TEXT, 30.0, false, false),
        TextVariant::H1 => (spec::TEXT_XXL, 700, spec::TEXT, 32.0, false, false),
        TextVariant::Code => (spec::TEXT_SM, 600, spec::TEXT, 18.0, false, true),
        TextVariant::Blockquote => (spec::TEXT_DF, 400, spec::TEXT, 22.0, true, false),
    };
    if code {
        return rsx! {
            row {
                background_color: spec::PAGE_BG,
                border_radius: spec::RADIUS,
                padding_left: 6.0,
                padding_right: 6.0,
                padding_top: 2.0,
                padding_bottom: 2.0,
                text {
                    content,
                    font_size: size,
                    font_weight: weight,
                    font_color: color,
                    line_height: line,
                }
            }
        };
    }
    if matches!(variant, TextVariant::Blockquote) {
        return rsx! {
            row {
                border_width: 0.0,
                padding_left: spec::PADDING,
                text {
                    content,
                    font_size: size,
                    font_style: "italic",
                    font_color: color,
                    line_height: line,
                }
            }
        };
    }
    let _ = tokens;
    rsx! {
        text {
            content,
            font_size: size,
            font_weight: weight,
            font_color: color,
            line_height: line,
            font_style: if italic { "italic" } else { "normal" },
        }
    }
}
