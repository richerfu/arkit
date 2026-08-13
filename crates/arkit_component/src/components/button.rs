//! Button — headless pressable control.
//!
//! Owns press behavior and native structure. Paint comes from an explicit
//! [`ButtonAppearance`] (style crates wrap this) or, if omitted, the shadcn
//! kit when one is mounted.

use crate::appearance::{ButtonAppearance, ButtonStyleInput};
use crate::style::{use_style_kit, PaletteColor};
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

pub use crate::appearance::{ButtonSize, ButtonVariant};

/// Props for [`Button`].
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    /// Optional exact height override for compact host-app surfaces.
    #[props(default)]
    pub height: Option<f32>,
    /// Optional exact corner radius override.
    #[props(default)]
    pub border_radius: Option<f32>,
    pub disabled: Option<bool>,
    /// CSS width (`"100%"`, `"48%"`, `"120"`). When unset, size defaults apply.
    pub width: Option<String>,
    /// Override the kit's default elevation.
    #[props(default)]
    pub shadow: Option<bool>,
    /// Named palette hint for kits that expose a color system.
    #[props(default)]
    pub color: Option<PaletteColor>,
    /// Ask the kit for a pill / fully-round treatment.
    #[props(default)]
    pub round: Option<bool>,
    /// Ask the kit for a block (full-width) treatment.
    #[props(default)]
    pub block: Option<bool>,
    /// Concrete paint from a style crate. When set, the mounted kit is ignored.
    #[props(default)]
    pub appearance: Option<ButtonAppearance>,
    /// Exact reference forwarded to the button's native root.
    #[props(default)]
    pub native_ref: Option<arkit_arkui::NativeElementRef>,
    pub onclick: Option<EventHandler<()>>,
    pub children: Element,
}

/// A headless button. Style crates pass [`ButtonAppearance`]; shadcn can omit
/// it and let the mounted theme kit resolve paint.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let kit = use_style_kit();
    let disabled = props.disabled.unwrap_or(false);
    let appearance: ButtonAppearance = props.appearance.unwrap_or_else(|| {
        kit.button(&ButtonStyleInput {
            variant: props.variant,
            size: props.size,
            disabled,
            color: props.color,
            round: props.round.unwrap_or(false),
            block: props.block.unwrap_or(false),
            height: props.height,
            border_radius: props.border_radius,
            width: props.width.clone(),
            shadow: props.shadow,
        })
    });
    let onclick = props.onclick;

    rsx! {
        button {
            native_ref: props.native_ref,
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            height: appearance.height,
            width: if let Some(w) = appearance.width { w },
            padding_top: appearance.padding[0],
            padding_right: appearance.padding[1],
            padding_bottom: appearance.padding[2],
            padding_left: appearance.padding[3],
            font_size: appearance.text_size,
            font_weight: appearance.font_weight,
            font_color: appearance.foreground,
            foreground_color: appearance.foreground,
            background_color: appearance.background,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            border_radius: appearance.border_radius,
            clip: true,
            alignment: "center",
            shadow: if appearance.shadow { "sm" },
            opacity: appearance.opacity,
            enabled: !disabled,
            onclick: move |_| {
                if !disabled {
                    if let Some(handler) = onclick {
                        handler.call(());
                    }
                }
            },
            row {
                align_items: "center",
                justify_content: "center",
                {props.children}
            }
        }
    }
}
