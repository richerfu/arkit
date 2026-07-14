//! Type-safe public property handles and adapter property descriptors.

use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::{
    Angle, AnimationValue, CustomValue, DiscreteValue, Length, LengthUnit, LinearRgba, ShadowValue,
    TransformValue, ValueError, Vec2, Vec3,
};

#[derive(Clone)]
pub enum SymbolName {
    Static(&'static str),
    Owned(Arc<str>),
}

impl SymbolName {
    pub const fn static_name(value: &'static str) -> Self {
        Self::Static(value)
    }

    pub fn owned(value: impl Into<Arc<str>>) -> Self {
        Self::Owned(value.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(value) => value,
            Self::Owned(value) => value,
        }
    }
}

impl Debug for SymbolName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), formatter)
    }
}

impl Hash for SymbolName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq for SymbolName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SymbolName {}

impl PartialOrd for SymbolName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SymbolName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PropertyName(SymbolName);

impl PropertyName {
    pub const fn static_name(value: &'static str) -> Self {
        Self(SymbolName::static_name(value))
    }

    pub fn owned(value: impl Into<Arc<str>>) -> Self {
        Self(SymbolName::owned(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Scalar,
    Length,
    Angle,
    Color,
    Vec2,
    Vec3,
    Transform,
    Shadow,
    Discrete,
    Custom,
}

pub trait AnimatableValue: Clone + Debug + 'static {
    const KIND: ValueKind;

    fn into_animation_value(self) -> AnimationValue;

    fn try_from_animation_value(value: AnimationValue) -> Result<Self, ValueError>;
}

macro_rules! impl_animatable_value {
    ($type:ty, $kind:ident, $variant:ident) => {
        impl AnimatableValue for $type {
            const KIND: ValueKind = ValueKind::$kind;

            fn into_animation_value(self) -> AnimationValue {
                AnimationValue::$variant(self)
            }

            fn try_from_animation_value(value: AnimationValue) -> Result<Self, ValueError> {
                let from = value.kind();
                match value {
                    AnimationValue::$variant(value) => Ok(value),
                    _ => Err(ValueError::KindMismatch {
                        from,
                        to: Self::KIND,
                    }),
                }
            }
        }
    };
}

impl_animatable_value!(f32, Scalar, Scalar);
impl_animatable_value!(Length, Length, Length);
impl_animatable_value!(Angle, Angle, Angle);
impl_animatable_value!(LinearRgba, Color, Color);
impl_animatable_value!(Vec2, Vec2, Vec2);
impl_animatable_value!(Vec3, Vec3, Vec3);
impl_animatable_value!(TransformValue, Transform, Transform);
impl_animatable_value!(ShadowValue, Shadow, Shadow);
impl_animatable_value!(DiscreteValue, Discrete, Discrete);
impl_animatable_value!(CustomValue, Custom, Custom);

pub struct Property<T: AnimatableValue> {
    name: PropertyName,
    marker: PhantomData<fn() -> T>,
}

impl<T: AnimatableValue> Property<T> {
    pub const fn static_name(name: &'static str) -> Self {
        Self {
            name: PropertyName::static_name(name),
            marker: PhantomData,
        }
    }

    pub fn owned(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: PropertyName::owned(name),
            marker: PhantomData,
        }
    }

    pub fn name(&self) -> &PropertyName {
        &self.name
    }

    pub const fn value_kind(&self) -> ValueKind {
        T::KIND
    }
}

impl<T: AnimatableValue> Clone for Property<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: AnimatableValue> Debug for Property<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Property")
            .field("name", &self.name)
            .field("kind", &T::KIND)
            .finish()
    }
}

impl<T: AnimatableValue> PartialEq for Property<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T: AnimatableValue> Eq for Property<T> {}

impl<T: AnimatableValue> Hash for Property<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interpolation {
    Linear,
    Discrete,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaselineStrategy {
    Required,
    Default(AnimationValue),
    ExplicitOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompositionSupport {
    pub replace: bool,
    pub add: bool,
    pub accumulate: bool,
}

impl CompositionSupport {
    pub const REPLACE_ONLY: Self = Self {
        replace: true,
        add: false,
        accumulate: false,
    };

    pub const NUMERIC: Self = Self {
        replace: true,
        add: true,
        accumulate: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitDomain {
    Unitless,
    Length { vp: bool, px: bool, percent: bool },
    Angle,
    Color,
    Composite,
    Discrete,
    Custom,
}

impl UnitDomain {
    pub const ALL_LENGTHS: Self = Self::Length {
        vp: true,
        px: true,
        percent: true,
    };

    pub const fn accepts_length(self, unit: LengthUnit) -> bool {
        match (self, unit) {
            (Self::Length { vp, .. }, LengthUnit::Vp) => vp,
            (Self::Length { px, .. }, LengthUnit::Px) => px,
            (Self::Length { percent, .. }, LengthUnit::Percent) => percent,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InvalidationClass {
    Transform,
    Paint,
    Layout,
    Measure,
    Discrete,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NativeSupport {
    pub implicit: bool,
    pub keyframe: bool,
    pub animator: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDescriptor {
    pub name: PropertyName,
    pub value_kind: ValueKind,
    pub baseline: BaselineStrategy,
    pub interpolation: Interpolation,
    pub composition: CompositionSupport,
    pub unit_domain: UnitDomain,
    pub precision: f32,
    pub invalidation: InvalidationClass,
    pub readable: bool,
    pub writable: bool,
    pub native: NativeSupport,
}

impl PropertyDescriptor {
    pub fn new<T: AnimatableValue>(property: &Property<T>) -> Self {
        let (interpolation, unit_domain) = match T::KIND {
            ValueKind::Scalar => (Interpolation::Linear, UnitDomain::Unitless),
            ValueKind::Length => (Interpolation::Linear, UnitDomain::ALL_LENGTHS),
            ValueKind::Angle => (Interpolation::Linear, UnitDomain::Angle),
            ValueKind::Color => (Interpolation::Linear, UnitDomain::Color),
            ValueKind::Vec2 | ValueKind::Vec3 | ValueKind::Transform | ValueKind::Shadow => {
                (Interpolation::Linear, UnitDomain::Composite)
            }
            ValueKind::Discrete => (Interpolation::Discrete, UnitDomain::Discrete),
            ValueKind::Custom => (Interpolation::Custom, UnitDomain::Custom),
        };
        Self {
            name: property.name.clone(),
            value_kind: T::KIND,
            baseline: BaselineStrategy::Required,
            interpolation,
            composition: CompositionSupport::REPLACE_ONLY,
            unit_domain,
            precision: 0.000_1,
            invalidation: InvalidationClass::Paint,
            readable: true,
            writable: true,
            native: NativeSupport::default(),
        }
    }

    pub fn validate_value(&self, value: &AnimationValue) -> Result<(), ValueError> {
        if value.kind() != self.value_kind {
            return Err(ValueError::KindMismatch {
                from: value.kind(),
                to: self.value_kind,
            });
        }
        value.validate_finite()?;
        if let AnimationValue::Length(length) = value {
            if !self.unit_domain.accepts_length(length.unit()) {
                return Err(ValueError::UnitNotSupported);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPACITY: Property<f32> = Property::static_name("opacity");

    #[test]
    fn typed_properties_keep_their_value_kind() {
        assert_eq!(OPACITY.value_kind(), ValueKind::Scalar);
        let descriptor = PropertyDescriptor::new(&OPACITY);
        assert!(descriptor
            .validate_value(&AnimationValue::Scalar(0.5))
            .is_ok());
        assert!(descriptor
            .validate_value(&AnimationValue::Length(Length::vp(1.0)))
            .is_err());
    }

    #[test]
    fn symbol_hash_is_independent_of_storage() {
        use std::collections::hash_map::DefaultHasher;

        let static_name = SymbolName::static_name("opacity");
        let owned_name = SymbolName::owned("opacity");
        let mut static_hash = DefaultHasher::new();
        let mut owned_hash = DefaultHasher::new();
        static_name.hash(&mut static_hash);
        owned_name.hash(&mut owned_hash);
        assert_eq!(static_hash.finish(), owned_hash.finish());
        assert_eq!(static_name, owned_name);
    }
}
