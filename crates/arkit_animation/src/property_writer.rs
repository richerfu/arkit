use arkit_animation_core::{AdapterPropertyId, AnimationValue, LengthUnit};
use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

use crate::diagnostic::AnimationAdapterError;
use crate::target::AnimationTargetBinding;

/// Canonical native slot written by an animation property.
///
/// Ownership and writes share this mapping so aliases and compound x/y
/// properties cannot silently acquire different renderer locks.
pub(crate) fn native_attribute(property_name: &str) -> Option<ArkUINodeAttributeType> {
    Some(match property_name {
        "opacity" => ArkUINodeAttributeType::Opacity,
        "translate_x" | "translate_y" => ArkUINodeAttributeType::Translate,
        "position_x" | "position_y" => ArkUINodeAttributeType::Position,
        "scale_x" | "scale_y" => ArkUINodeAttributeType::Scale,
        "rotation" => ArkUINodeAttributeType::Rotate,
        "background_color" => ArkUINodeAttributeType::BackgroundColor,
        "font_color" => ArkUINodeAttributeType::FontColor,
        "border_color" => ArkUINodeAttributeType::BorderColor,
        "foreground_color" => ArkUINodeAttributeType::ForegroundColor,
        "border_radius" => ArkUINodeAttributeType::BorderRadius,
        "border_width" => ArkUINodeAttributeType::BorderWidth,
        "blur" => ArkUINodeAttributeType::Blur,
        "width" => ArkUINodeAttributeType::Width,
        "height" => ArkUINodeAttributeType::Height,
        "font_size" => ArkUINodeAttributeType::FontSize,
        "line_height" => ArkUINodeAttributeType::TextLineHeight,
        "letter_spacing" => ArkUINodeAttributeType::TextLetterSpacing,
        "brightness" => ArkUINodeAttributeType::Brightness,
        "saturation" => ArkUINodeAttributeType::Saturation,
        "grayscale" => ArkUINodeAttributeType::GrayScale,
        "invert" => ArkUINodeAttributeType::Invert,
        "sepia" => ArkUINodeAttributeType::Sepia,
        "contrast" => ArkUINodeAttributeType::Contrast,
        "aspect_ratio" => ArkUINodeAttributeType::AspectRatio,
        _ => return None,
    })
}

pub(crate) fn ownership_attributes(property_name: &str) -> Option<Vec<ArkUINodeAttributeType>> {
    let attribute = native_attribute(property_name)?;
    Some(match attribute {
        ArkUINodeAttributeType::Width => vec![
            ArkUINodeAttributeType::Width,
            ArkUINodeAttributeType::WidthPercent,
        ],
        ArkUINodeAttributeType::Height => vec![
            ArkUINodeAttributeType::Height,
            ArkUINodeAttributeType::HeightPercent,
        ],
        _ => vec![attribute],
    })
}

pub(crate) fn write(
    binding: &mut AnimationTargetBinding,
    property: AdapterPropertyId,
    property_name: &str,
    value: &AnimationValue,
) -> Result<(), AnimationAdapterError> {
    let item = match (property_name, value) {
        ("opacity", AnimationValue::Scalar(value)) => {
            // The Engine intentionally permits arithmetic composition to
            // overshoot. ArkUI's opacity attribute does not: its native domain
            // is closed over [0, 1], matching the final rendering semantics of
            // CSS opacity. Lower at the adapter boundary so a valid additive
            // animation cannot reject an otherwise atomic frame.
            ArkUINodeAttributeItem::from(value.clamp(0.0, 1.0))
        }
        (
            "scale_x" | "scale_y" | "brightness" | "saturation" | "grayscale" | "invert" | "sepia"
            | "contrast" | "aspect_ratio",
            AnimationValue::Scalar(value),
        ) => ArkUINodeAttributeItem::from(*value),
        ("translate_x", AnimationValue::Length(value)) if value.unit() == LengthUnit::Vp => {
            binding.visual.translate[0] = value.value();
            binding.visual.translate.to_vec().into()
        }
        ("translate_y", AnimationValue::Length(value)) if value.unit() == LengthUnit::Vp => {
            binding.visual.translate[1] = value.value();
            binding.visual.translate.to_vec().into()
        }
        ("position_x", AnimationValue::Length(value)) if value.unit() == LengthUnit::Vp => {
            binding.visual.position[0] = value.value();
            binding.visual.position.to_vec().into()
        }
        ("position_y", AnimationValue::Length(value)) if value.unit() == LengthUnit::Vp => {
            binding.visual.position[1] = value.value();
            binding.visual.position.to_vec().into()
        }
        ("rotation", AnimationValue::Angle(value)) => {
            binding.visual.rotation_degrees = value.as_degrees();
            vec![0.0_f32, 0.0, 1.0, binding.visual.rotation_degrees, 0.0].into()
        }
        (
            "background_color" | "font_color" | "border_color" | "foreground_color",
            AnimationValue::Color(value),
        ) => value.to_argb().into(),
        ("border_radius" | "border_width", AnimationValue::Length(value))
            if value.unit() == LengthUnit::Vp =>
        {
            vec![value.value(); 4].into()
        }
        (
            "blur" | "width" | "height" | "font_size" | "line_height" | "letter_spacing",
            AnimationValue::Length(value),
        ) if value.unit() == LengthUnit::Vp => value.value().into(),
        _ => {
            return Err(AnimationAdapterError::UnsupportedValue { property });
        }
    };
    if property_name == "scale_x" {
        let AnimationValue::Scalar(value) = value else {
            unreachable!();
        };
        binding.visual.scale[0] = *value;
    }
    if property_name == "scale_y" {
        let AnimationValue::Scalar(value) = value else {
            unreachable!();
        };
        binding.visual.scale[1] = *value;
    }
    let attribute = native_attribute(property_name)
        .ok_or(AnimationAdapterError::UnsupportedValue { property })?;
    let item = match attribute {
        ArkUINodeAttributeType::Translate => binding.visual.translate.to_vec().into(),
        ArkUINodeAttributeType::Position => binding.visual.position.to_vec().into(),
        ArkUINodeAttributeType::Scale => binding.visual.scale.to_vec().into(),
        _ => item,
    };
    let node = binding.node.clone();
    // SAFETY: this writes one renderer-compatible visual attribute to the
    // live target and neither retains nor reparents the native node.
    let result = unsafe { node.with_native(|node| node.set_attribute(attribute, item)) }
        .ok_or_else(|| AnimationAdapterError::NativeWrite {
            target: binding.id,
            property,
            reason: "target is no longer mounted".into(),
        })?;
    result.map_err(|error| AnimationAdapterError::NativeWrite {
        target: binding.id,
        property,
        reason: error.to_string().into_boxed_str(),
    })
}

pub(crate) fn write_compound_pair(
    binding: &mut AnimationTargetBinding,
    first_property: AdapterPropertyId,
    first_name: &str,
    first_value: &AnimationValue,
    second_property: AdapterPropertyId,
    second_name: &str,
    second_value: &AnimationValue,
) -> Result<(), AnimationAdapterError> {
    let attribute = match (first_name, second_name) {
        ("translate_x", "translate_y") => {
            let (AnimationValue::Length(x), AnimationValue::Length(y)) =
                (first_value, second_value)
            else {
                return Err(AnimationAdapterError::UnsupportedValue {
                    property: first_property,
                });
            };
            if x.unit() != LengthUnit::Vp || y.unit() != LengthUnit::Vp {
                return Err(AnimationAdapterError::UnsupportedValue {
                    property: first_property,
                });
            }
            binding.visual.translate[0] = x.value();
            binding.visual.translate[1] = y.value();
            ArkUINodeAttributeType::Translate
        }
        ("scale_x", "scale_y") => {
            let (AnimationValue::Scalar(x), AnimationValue::Scalar(y)) =
                (first_value, second_value)
            else {
                return Err(AnimationAdapterError::UnsupportedValue {
                    property: first_property,
                });
            };
            binding.visual.scale[0] = *x;
            binding.visual.scale[1] = *y;
            ArkUINodeAttributeType::Scale
        }
        ("position_x", "position_y") => {
            let (AnimationValue::Length(x), AnimationValue::Length(y)) =
                (first_value, second_value)
            else {
                return Err(AnimationAdapterError::UnsupportedValue {
                    property: first_property,
                });
            };
            if x.unit() != LengthUnit::Vp || y.unit() != LengthUnit::Vp {
                return Err(AnimationAdapterError::UnsupportedValue {
                    property: first_property,
                });
            }
            binding.visual.position[0] = x.value();
            binding.visual.position[1] = y.value();
            ArkUINodeAttributeType::Position
        }
        _ => {
            return Err(AnimationAdapterError::UnsupportedValue {
                property: second_property,
            });
        }
    };
    let item = match attribute {
        ArkUINodeAttributeType::Translate => binding.visual.translate.to_vec().into(),
        ArkUINodeAttributeType::Scale => binding.visual.scale.to_vec().into(),
        ArkUINodeAttributeType::Position => binding.visual.position.to_vec().into(),
        _ => unreachable!("compound writer only emits transform or position attributes"),
    };
    let node = binding.node.clone();
    // SAFETY: this writes one renderer-compatible compound visual attribute
    // to the live target and does not retain the native borrow.
    let result = unsafe { node.with_native(|node| node.set_attribute(attribute, item)) }
        .ok_or_else(|| AnimationAdapterError::NativeWrite {
            target: binding.id,
            property: first_property,
            reason: "target is no longer mounted".into(),
        })?;
    result.map_err(|error| AnimationAdapterError::NativeWrite {
        target: binding.id,
        property: first_property,
        reason: error.to_string().into_boxed_str(),
    })
}
