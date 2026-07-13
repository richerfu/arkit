use std::cell::RefCell;

use arkit_animation_core::{
    AdapterId, AdapterPropertyId, AdapterTargetId, Angle, AnimationValue, Length, LengthUnit,
    LinearRgba, PropertyName, PropertyUpdate, ResolutionTarget, ResolvedProperty, ResolvedTarget,
    SourceTarget, TargetLayoutSnapshot, TargetName,
};
use arkit_hooks::HostNode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

use crate::property_reader::{first_f32, first_u32, numbers};
use crate::{
    property_writer, AnimationAdapterError, PropertySchema, TargetAdapter, TargetLifecycle,
    TargetStore,
};

pub struct ArkUiAdapter {
    id: AdapterId,
    targets: RefCell<TargetStore>,
    properties: PropertySchema,
}

impl ArkUiAdapter {
    pub fn new(id: AdapterId) -> Self {
        Self {
            id,
            targets: RefCell::new(TargetStore::default()),
            properties: PropertySchema::arkui(),
        }
    }

    pub fn register_target(
        &self,
        name: TargetName,
        node: HostNode,
        layout: Option<TargetLayoutSnapshot>,
    ) -> Result<AdapterTargetId, AnimationAdapterError> {
        self.targets.borrow_mut().register(name, node, layout)
    }

    pub fn unregister_target(&self, target: AdapterTargetId) -> bool {
        self.targets.borrow_mut().unregister(target)
    }

    pub fn set_members(
        &self,
        set: arkit_animation_core::TargetSetName,
        members: Vec<AdapterTargetId>,
    ) {
        self.targets.borrow_mut().set_members(set, members);
    }

    fn property_name(&self, property: AdapterPropertyId) -> Result<&str, AnimationAdapterError> {
        self.properties
            .get(property)
            .map(|descriptor| descriptor.name.as_str())
            .ok_or(AnimationAdapterError::UnknownPropertyId(property))
    }
}

impl TargetAdapter for ArkUiAdapter {
    fn id(&self) -> AdapterId {
        self.id
    }

    fn diagnostic_name(&self) -> &str {
        "arkui"
    }

    fn target_lifecycle(&self, target: AdapterTargetId) -> Option<TargetLifecycle> {
        self.targets
            .borrow()
            .get(target)
            .map(|binding| TargetLifecycle {
                version: binding.version,
                mounted: binding.mounted,
            })
    }

    fn property_descriptor(
        &self,
        property: AdapterPropertyId,
    ) -> Option<&arkit_animation_core::PropertyDescriptor> {
        self.properties.get(property)
    }

    fn resolve_targets(
        &self,
        target: &SourceTarget,
    ) -> Result<Vec<ResolutionTarget>, AnimationAdapterError> {
        let targets = self.targets.borrow();
        targets
            .resolve(target)?
            .into_iter()
            .map(|target_id| {
                let binding = targets
                    .get(target_id)
                    .ok_or(AnimationAdapterError::UnknownTargetId(target_id))?;
                Ok(ResolutionTarget {
                    name: binding.name.clone(),
                    target: ResolvedTarget {
                        adapter: self.id,
                        adapter_target: target_id,
                    },
                    layout: binding.layout,
                })
            })
            .collect()
    }

    fn resolve_property(
        &self,
        _target: AdapterTargetId,
        property: &PropertyName,
    ) -> Result<ResolvedProperty, AnimationAdapterError> {
        let (adapter_property, descriptor) = self.properties.resolve(property)?;
        Ok(ResolvedProperty {
            adapter: self.id,
            adapter_property,
            descriptor,
        })
    }

    fn read_baseline(
        &self,
        target: AdapterTargetId,
        property: AdapterPropertyId,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        let name = self.property_name(property)?;
        let mut targets = self.targets.borrow_mut();
        let binding = targets
            .get_mut(target)
            .ok_or(AnimationAdapterError::UnknownTargetId(target))?;
        let node = binding.node.borrow();
        let result = match name {
            "opacity" => node
                .get_attribute(ArkUINodeAttributeType::Opacity)
                .ok()
                .and_then(first_f32)
                .map(AnimationValue::Scalar),
            "translate_x" | "translate_y" => {
                if let Ok(item) = node.get_attribute(ArkUINodeAttributeType::Translate) {
                    if let Some(values) = numbers(item) {
                        for (index, value) in values.into_iter().take(3).enumerate() {
                            binding.visual.translate[index] = value;
                        }
                    }
                }
                let index = usize::from(name == "translate_y");
                Some(AnimationValue::Length(Length::vp(
                    binding.visual.translate[index],
                )))
            }
            "position_x" | "position_y" => {
                if let Ok(item) = node.get_attribute(ArkUINodeAttributeType::Position) {
                    if let Some(values) = numbers(item) {
                        for (index, value) in values.into_iter().take(2).enumerate() {
                            binding.visual.position[index] = value;
                        }
                    }
                }
                let index = usize::from(name == "position_y");
                Some(AnimationValue::Length(Length::vp(
                    binding.visual.position[index],
                )))
            }
            "scale_x" | "scale_y" => {
                if let Ok(item) = node.get_attribute(ArkUINodeAttributeType::Scale) {
                    if let Some(values) = numbers(item) {
                        for (index, value) in values.into_iter().take(2).enumerate() {
                            binding.visual.scale[index] = value;
                        }
                    }
                }
                let index = usize::from(name == "scale_y");
                Some(AnimationValue::Scalar(binding.visual.scale[index]))
            }
            "rotation" => node
                .get_attribute(ArkUINodeAttributeType::Rotate)
                .ok()
                .and_then(numbers)
                .and_then(|values| values.get(3).copied())
                .map(Angle::degrees)
                .map(AnimationValue::Angle),
            "background_color" | "font_color" | "border_color" | "foreground_color" => {
                let attribute = match name {
                    "background_color" => ArkUINodeAttributeType::BackgroundColor,
                    "font_color" => ArkUINodeAttributeType::FontColor,
                    "border_color" => ArkUINodeAttributeType::BorderColor,
                    _ => ArkUINodeAttributeType::ForegroundColor,
                };
                node.get_attribute(attribute)
                    .ok()
                    .and_then(first_u32)
                    .map(LinearRgba::from_argb)
                    .map(AnimationValue::Color)
            }
            "border_radius" | "border_width" => node
                .get_attribute(if name == "border_radius" {
                    ArkUINodeAttributeType::BorderRadius
                } else {
                    ArkUINodeAttributeType::BorderWidth
                })
                .ok()
                .and_then(first_f32)
                .map(Length::vp)
                .map(AnimationValue::Length),
            "blur" | "width" | "height" | "font_size" | "line_height" | "letter_spacing" => {
                let attribute = match name {
                    "blur" => ArkUINodeAttributeType::Blur,
                    "width" => ArkUINodeAttributeType::Width,
                    "height" => ArkUINodeAttributeType::Height,
                    "font_size" => ArkUINodeAttributeType::FontSize,
                    "line_height" => ArkUINodeAttributeType::TextLineHeight,
                    _ => ArkUINodeAttributeType::TextLetterSpacing,
                };
                node.get_attribute(attribute)
                    .ok()
                    .and_then(first_f32)
                    .map(Length::vp)
                    .map(AnimationValue::Length)
            }
            "brightness" | "saturation" | "grayscale" | "invert" | "sepia" | "contrast"
            | "aspect_ratio" => {
                let attribute = match name {
                    "brightness" => ArkUINodeAttributeType::Brightness,
                    "saturation" => ArkUINodeAttributeType::Saturation,
                    "grayscale" => ArkUINodeAttributeType::GrayScale,
                    "invert" => ArkUINodeAttributeType::Invert,
                    "sepia" => ArkUINodeAttributeType::Sepia,
                    "contrast" => ArkUINodeAttributeType::Contrast,
                    _ => ArkUINodeAttributeType::AspectRatio,
                };
                node.get_attribute(attribute)
                    .ok()
                    .and_then(first_f32)
                    .map(AnimationValue::Scalar)
            }
            _ => None,
        };
        result.ok_or(AnimationAdapterError::NativeRead { target, property })
    }

    fn resolve_value(
        &self,
        _target: AdapterTargetId,
        property: AdapterPropertyId,
        value: &AnimationValue,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        if let AnimationValue::Length(length) = value {
            if length.unit() != LengthUnit::Vp {
                return Err(AnimationAdapterError::UnsupportedValue { property });
            }
        }
        Ok(value.clone())
    }

    fn apply(&self, update: &PropertyUpdate) -> Result<(), AnimationAdapterError> {
        let name = self.property_name(update.property)?;
        let mut targets = self.targets.borrow_mut();
        let binding = targets
            .get_mut(update.target)
            .ok_or(AnimationAdapterError::UnknownTargetId(update.target))?;
        property_writer::write(binding, update.property, name, &update.value)
    }
}
