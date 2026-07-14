use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use arkit_animation_core::{
    AdapterPropertyId, CompositionSupport, InvalidationClass, NativeSupport, PropertyDescriptor,
    PropertyName,
};

use crate::properties::{
    ASPECT_RATIO, BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS,
    CONTRAST, FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING,
    LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y, SEPIA,
    TRANSLATE_X, TRANSLATE_Y, WIDTH,
};
use crate::AnimationAdapterError;

#[derive(Default)]
pub struct PropertySchema {
    descriptors: IndexVec<AdapterPropertyId, PropertyDescriptor>,
    names: FxHashMap<PropertyName, AdapterPropertyId>,
}

impl PropertySchema {
    pub fn arkui() -> Self {
        let mut schema = Self::default();
        for mut descriptor in [
            PropertyDescriptor::new(&OPACITY),
            PropertyDescriptor::new(&TRANSLATE_X),
            PropertyDescriptor::new(&TRANSLATE_Y),
            PropertyDescriptor::new(&SCALE_X),
            PropertyDescriptor::new(&SCALE_Y),
            PropertyDescriptor::new(&ROTATION),
            PropertyDescriptor::new(&BACKGROUND_COLOR),
            PropertyDescriptor::new(&FONT_COLOR),
            PropertyDescriptor::new(&BORDER_RADIUS),
            PropertyDescriptor::new(&BLUR),
            PropertyDescriptor::new(&WIDTH),
            PropertyDescriptor::new(&HEIGHT),
            PropertyDescriptor::new(&POSITION_X),
            PropertyDescriptor::new(&POSITION_Y),
            PropertyDescriptor::new(&BORDER_WIDTH),
            PropertyDescriptor::new(&BORDER_COLOR),
            PropertyDescriptor::new(&FOREGROUND_COLOR),
            PropertyDescriptor::new(&FONT_SIZE),
            PropertyDescriptor::new(&LINE_HEIGHT),
            PropertyDescriptor::new(&LETTER_SPACING),
            PropertyDescriptor::new(&BRIGHTNESS),
            PropertyDescriptor::new(&SATURATION),
            PropertyDescriptor::new(&GRAYSCALE),
            PropertyDescriptor::new(&INVERT),
            PropertyDescriptor::new(&SEPIA),
            PropertyDescriptor::new(&CONTRAST),
            PropertyDescriptor::new(&ASPECT_RATIO),
        ] {
            descriptor.composition = CompositionSupport::NUMERIC;
            descriptor.invalidation = invalidation_for(descriptor.name.as_str());
            descriptor.native = NativeSupport {
                implicit: true,
                keyframe: true,
                animator: true,
            };
            schema.insert(descriptor);
        }
        schema
    }

    pub fn insert(&mut self, descriptor: PropertyDescriptor) -> AdapterPropertyId {
        if let Some(id) = self.names.get(&descriptor.name).copied() {
            self.descriptors[id] = descriptor;
            return id;
        }
        let name = descriptor.name.clone();
        let id = self.descriptors.push(descriptor);
        self.names.insert(name, id);
        id
    }

    pub fn resolve(
        &self,
        name: &PropertyName,
    ) -> Result<(AdapterPropertyId, PropertyDescriptor), AnimationAdapterError> {
        let id = self
            .names
            .get(name)
            .copied()
            .ok_or_else(|| AnimationAdapterError::UnknownProperty(name.clone()))?;
        Ok((id, self.descriptors[id].clone()))
    }

    pub fn get(&self, id: AdapterPropertyId) -> Option<&PropertyDescriptor> {
        self.descriptors.raw.get(id.index())
    }
}

fn invalidation_for(name: &str) -> InvalidationClass {
    match name {
        "translate_x" | "translate_y" | "scale_x" | "scale_y" | "rotation" => {
            InvalidationClass::Transform
        }
        "width" | "height" | "position_x" | "position_y" | "font_size" | "line_height"
        | "letter_spacing" | "border_width" | "aspect_ratio" => InvalidationClass::Layout,
        _ => InvalidationClass::Paint,
    }
}
