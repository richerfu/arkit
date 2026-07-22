//! Declarative barcode preview component (encode off the UI thread).

use arkit_prelude::*;

use crate::async_job::JobEpoch;
use crate::format::BarcodeFormat;
use crate::hook::{schedule_encode, BarcodePhase};
use crate::request::BarcodeOptions;

/// Props for [`Barcode`].
#[derive(Props, Clone, PartialEq)]
pub struct BarcodeProps {
    /// Payload to encode.
    pub contents: String,
    /// Symbology (default QR).
    #[props(default)]
    pub format: BarcodeFormat,
    /// Layout / encode edge for matrix codes; width for linear codes.
    #[props(default = 200.0)]
    pub size: f32,
    /// Optional height override (linear barcodes). Defaults to `size` for matrix
    /// codes and `size * 0.35` for linear codes when omitted.
    #[props(default)]
    pub height: Option<f32>,
    /// Fine-grained options. When set, overrides format/size/height for encode.
    #[props(default)]
    pub options: Option<BarcodeOptions>,
    #[props(default)]
    pub on_error: Option<EventHandler<crate::error::BarcodeError>>,
}

/// Render a barcode as a native image (SVG → PixelMap).
///
/// Encoding is scheduled on a background worker; the UI shows a lightweight
/// placeholder while [`BarcodePhase::Encoding`].
#[component]
pub fn Barcode(props: BarcodeProps) -> Element {
    let contents = props.contents;
    let options = props.options.unwrap_or_else(|| {
        let width = props.size.max(1.0).round() as u32;
        let height = props
            .height
            .map(|value| value.max(1.0).round() as u32)
            .unwrap_or_else(|| {
                if props.format.is_matrix() {
                    width
                } else {
                    (props.size.max(1.0) * 0.35).round().max(32.0) as u32
                }
            });
        BarcodeOptions {
            format: props.format,
            width,
            height,
            ..BarcodeOptions::default()
        }
        .with_format(props.format)
    });
    let layout_w = options.width as f32;
    let layout_h = options.height as f32;

    let phase = use_signal(|| BarcodePhase::Empty);
    let epoch = use_hook(JobEpoch::default);

    use_effect(use_reactive((&contents, &options), {
        let epoch = epoch.clone();
        move |(contents, options)| {
            schedule_encode(contents, options, phase, epoch.clone());
        }
    }));

    let on_error = props.on_error;
    use_effect(move || {
        if let BarcodePhase::Error(error) = phase.cloned() {
            if let Some(handler) = on_error.as_ref() {
                handler.call(error);
            }
        }
    });

    match phase.cloned() {
        BarcodePhase::Ready(artifact) => {
            let source = artifact.image;
            rsx! {
                image {
                    src: dioxus_core::AttributeValue::any_value(source),
                    object_fit: 1_i32,
                    width: layout_w,
                    height: layout_h,
                }
            }
        }
        BarcodePhase::Encoding => rsx! {
            column {
                width: layout_w,
                height: layout_h,
                background_color: 0xFFF4F4F5_u32,
                border_radius: 8.0,
                align_items: "center",
                justify_content: "center",
                text {
                    content: "Encoding…".to_string(),
                    font_size: 12.0,
                    font_color: 0xFF71717A_u32,
                }
            }
        },
        BarcodePhase::Empty => rsx! {
            column {
                width: layout_w,
                height: layout_h,
                background_color: 0xFFF4F4F5_u32,
                border_radius: 8.0,
                align_items: "center",
                justify_content: "center",
                text {
                    content: "Enter content".to_string(),
                    font_size: 12.0,
                    font_color: 0xFF71717A_u32,
                }
            }
        },
        BarcodePhase::Error(error) => rsx! {
            column {
                width: layout_w,
                height: layout_h,
                background_color: 0xFFFEF2F2_u32,
                border_radius: 8.0,
                align_items: "center",
                justify_content: "center",
                padding_left: 8.0,
                padding_right: 8.0,
                text {
                    content: error.message().to_string(),
                    font_size: 11.0,
                    font_color: 0xFFB91C1C_u32,
                    text_align: 1_i32,
                }
            }
        },
    }
}
