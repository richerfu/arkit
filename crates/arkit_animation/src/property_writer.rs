use arkit_animation_core::{AdapterPropertyId, AnimationValue, LengthUnit};
use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

use crate::{AnimationAdapterError, AnimationTargetBinding};

pub(crate) fn write(
    binding: &mut AnimationTargetBinding,
    property: AdapterPropertyId,
    property_name: &str,
    value: &AnimationValue,
) -> Result<(), AnimationAdapterError> {
    let item = match (property_name, value) {
        (
            "opacity" | "scale_x" | "scale_y" | "brightness" | "saturation" | "grayscale"
            | "invert" | "sepia" | "contrast" | "aspect_ratio",
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
    let (attribute, item) = match property_name {
        "opacity" => (ArkUINodeAttributeType::Opacity, item),
        "translate_x" | "translate_y" => (
            ArkUINodeAttributeType::Translate,
            binding.visual.translate.to_vec().into(),
        ),
        "position_x" | "position_y" => (
            ArkUINodeAttributeType::Position,
            binding.visual.position.to_vec().into(),
        ),
        "scale_x" | "scale_y" => (
            ArkUINodeAttributeType::Scale,
            binding.visual.scale.to_vec().into(),
        ),
        "rotation" => (ArkUINodeAttributeType::Rotate, item),
        "background_color" => (ArkUINodeAttributeType::BackgroundColor, item),
        "font_color" => (ArkUINodeAttributeType::FontColor, item),
        "border_color" => (ArkUINodeAttributeType::BorderColor, item),
        "foreground_color" => (ArkUINodeAttributeType::ForegroundColor, item),
        "border_radius" => (ArkUINodeAttributeType::BorderRadius, item),
        "border_width" => (ArkUINodeAttributeType::BorderWidth, item),
        "blur" => (ArkUINodeAttributeType::Blur, item),
        "width" => (ArkUINodeAttributeType::Width, item),
        "height" => (ArkUINodeAttributeType::Height, item),
        "font_size" => (ArkUINodeAttributeType::FontSize, item),
        "line_height" => (ArkUINodeAttributeType::TextLineHeight, item),
        "letter_spacing" => (ArkUINodeAttributeType::TextLetterSpacing, item),
        "brightness" => (ArkUINodeAttributeType::Brightness, item),
        "saturation" => (ArkUINodeAttributeType::Saturation, item),
        "grayscale" => (ArkUINodeAttributeType::GrayScale, item),
        "invert" => (ArkUINodeAttributeType::Invert, item),
        "sepia" => (ArkUINodeAttributeType::Sepia, item),
        "contrast" => (ArkUINodeAttributeType::Contrast, item),
        "aspect_ratio" => (ArkUINodeAttributeType::AspectRatio, item),
        _ => {
            return Err(AnimationAdapterError::UnsupportedValue { property });
        }
    };
    binding
        .node
        .borrow()
        .set_attribute(attribute, item)
        .map_err(|_| AnimationAdapterError::NativeWrite {
            target: binding.id,
            property,
        })
}
