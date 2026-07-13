//! Owned animation values and platform-independent interpolation.

use std::sync::Arc;

use crate::{ValueError, ValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnit {
    Vp,
    Px,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length {
    value: f32,
    unit: LengthUnit,
}

impl Length {
    pub const fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    pub const fn vp(value: f32) -> Self {
        Self::new(value, LengthUnit::Vp)
    }

    pub const fn px(value: f32) -> Self {
        Self::new(value, LengthUnit::Px)
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(value, LengthUnit::Percent)
    }

    pub const fn value(self) -> f32 {
        self.value
    }

    pub const fn unit(self) -> LengthUnit {
        self.unit
    }

    fn interpolate(self, to: Self, progress: f32) -> Result<Self, ValueError> {
        if self.unit != to.unit {
            return Err(ValueError::UnitMismatch);
        }
        Ok(Self::new(lerp(self.value, to.value, progress), self.unit))
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::vp(0.0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Angle(f32);

impl Angle {
    pub const fn radians(value: f32) -> Self {
        Self(value)
    }

    pub fn degrees(value: f32) -> Self {
        Self(value.to_radians())
    }

    pub const fn as_radians(self) -> f32 {
        self.0
    }

    pub fn as_degrees(self) -> f32 {
        self.0.to_degrees()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgba {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl LinearRgba {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Result<Self, ValueError> {
        if [red, green, blue, alpha]
            .into_iter()
            .any(|channel| !channel.is_finite())
        {
            return Err(ValueError::InvalidColor);
        }
        Ok(Self {
            red,
            green,
            blue,
            alpha,
        })
    }

    pub fn from_argb(argb: u32) -> Self {
        let alpha = ((argb >> 24) & 0xff) as f32 / 255.0;
        let red = srgb_to_linear(((argb >> 16) & 0xff) as f32 / 255.0);
        let green = srgb_to_linear(((argb >> 8) & 0xff) as f32 / 255.0);
        let blue = srgb_to_linear((argb & 0xff) as f32 / 255.0);
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn to_argb(self) -> u32 {
        let alpha = channel_to_u8(self.alpha);
        let red = channel_to_u8(linear_to_srgb(self.red));
        let green = channel_to_u8(linear_to_srgb(self.green));
        let blue = channel_to_u8(linear_to_srgb(self.blue));
        (alpha << 24) | (red << 16) | (green << 8) | blue
    }

    fn interpolate(self, to: Self, progress: f32) -> Self {
        Self {
            red: lerp(self.red, to.red, progress),
            green: lerp(self.green, to.green, progress),
            blue: lerp(self.blue, to.blue, progress),
            alpha: lerp(self.alpha, to.alpha, progress),
        }
    }
}

impl Default for LinearRgba {
    fn default() -> Self {
        Self {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformValue {
    pub translation: [Length; 3],
    pub scale: Vec3,
    pub rotation: [Angle; 3],
    pub skew: [Angle; 2],
    pub perspective: Length,
    pub origin: [Length; 3],
}

impl Default for TransformValue {
    fn default() -> Self {
        Self {
            translation: [Length::default(); 3],
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: [Angle::default(); 3],
            skew: [Angle::default(); 2],
            perspective: Length::default(),
            origin: [
                Length::percent(50.0),
                Length::percent(50.0),
                Length::vp(0.0),
            ],
        }
    }
}

impl TransformValue {
    fn interpolate(&self, to: &Self, progress: f32) -> Result<Self, ValueError> {
        Ok(Self {
            translation: [
                self.translation[0].interpolate(to.translation[0], progress)?,
                self.translation[1].interpolate(to.translation[1], progress)?,
                self.translation[2].interpolate(to.translation[2], progress)?,
            ],
            scale: interpolate_vec3(self.scale, to.scale, progress),
            rotation: [
                interpolate_angle(self.rotation[0], to.rotation[0], progress),
                interpolate_angle(self.rotation[1], to.rotation[1], progress),
                interpolate_angle(self.rotation[2], to.rotation[2], progress),
            ],
            skew: [
                interpolate_angle(self.skew[0], to.skew[0], progress),
                interpolate_angle(self.skew[1], to.skew[1], progress),
            ],
            perspective: self.perspective.interpolate(to.perspective, progress)?,
            origin: [
                self.origin[0].interpolate(to.origin[0], progress)?,
                self.origin[1].interpolate(to.origin[1], progress)?,
                self.origin[2].interpolate(to.origin[2], progress)?,
            ],
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShadowValue {
    pub offset_x: Length,
    pub offset_y: Length,
    pub blur: Length,
    pub spread: Length,
    pub color: LinearRgba,
}

impl ShadowValue {
    fn interpolate(&self, to: &Self, progress: f32) -> Result<Self, ValueError> {
        Ok(Self {
            offset_x: self.offset_x.interpolate(to.offset_x, progress)?,
            offset_y: self.offset_y.interpolate(to.offset_y, progress)?,
            blur: self.blur.interpolate(to.blur, progress)?,
            spread: self.spread.interpolate(to.spread, progress)?,
            color: self.color.interpolate(to.color, progress),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscreteValue(Arc<str>);

impl DiscreteValue {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomValue {
    type_name: Arc<str>,
    payload: Arc<[u8]>,
}

impl CustomValue {
    pub fn new(type_name: impl Into<Arc<str>>, payload: impl Into<Arc<[u8]>>) -> Self {
        Self {
            type_name: type_name.into(),
            payload: payload.into(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationValue {
    Scalar(f32),
    Length(Length),
    Angle(Angle),
    Color(LinearRgba),
    Vec2(Vec2),
    Vec3(Vec3),
    Transform(TransformValue),
    Shadow(ShadowValue),
    Discrete(DiscreteValue),
    Custom(CustomValue),
}

impl AnimationValue {
    pub const fn kind(&self) -> ValueKind {
        match self {
            Self::Scalar(_) => ValueKind::Scalar,
            Self::Length(_) => ValueKind::Length,
            Self::Angle(_) => ValueKind::Angle,
            Self::Color(_) => ValueKind::Color,
            Self::Vec2(_) => ValueKind::Vec2,
            Self::Vec3(_) => ValueKind::Vec3,
            Self::Transform(_) => ValueKind::Transform,
            Self::Shadow(_) => ValueKind::Shadow,
            Self::Discrete(_) => ValueKind::Discrete,
            Self::Custom(_) => ValueKind::Custom,
        }
    }

    pub fn interpolate(&self, to: &Self, progress: f32) -> Result<Self, ValueError> {
        if self.kind() != to.kind() {
            return Err(ValueError::KindMismatch {
                from: self.kind(),
                to: to.kind(),
            });
        }
        match (self, to) {
            (Self::Scalar(from), Self::Scalar(to)) => Ok(Self::Scalar(lerp(*from, *to, progress))),
            (Self::Length(from), Self::Length(to)) => {
                Ok(Self::Length(from.interpolate(*to, progress)?))
            }
            (Self::Angle(from), Self::Angle(to)) => {
                Ok(Self::Angle(interpolate_angle(*from, *to, progress)))
            }
            (Self::Color(from), Self::Color(to)) => {
                Ok(Self::Color(from.interpolate(*to, progress)))
            }
            (Self::Vec2(from), Self::Vec2(to)) => {
                Ok(Self::Vec2(interpolate_vec2(*from, *to, progress)))
            }
            (Self::Vec3(from), Self::Vec3(to)) => {
                Ok(Self::Vec3(interpolate_vec3(*from, *to, progress)))
            }
            (Self::Transform(from), Self::Transform(to)) => {
                Ok(Self::Transform(from.interpolate(to, progress)?))
            }
            (Self::Shadow(from), Self::Shadow(to)) => {
                Ok(Self::Shadow(from.interpolate(to, progress)?))
            }
            (Self::Discrete(from), Self::Discrete(to)) => Ok(Self::Discrete(if progress < 1.0 {
                from.clone()
            } else {
                to.clone()
            })),
            (Self::Custom(_), Self::Custom(_)) => Err(ValueError::CustomInterpolationRequired),
            _ => unreachable!("matching value kinds were checked above"),
        }
    }

    pub fn validate_finite(&self) -> Result<(), ValueError> {
        let finite = match self {
            Self::Scalar(value) => value.is_finite(),
            Self::Length(value) => value.value().is_finite(),
            Self::Angle(value) => value.as_radians().is_finite(),
            Self::Color(value) => [value.red, value.green, value.blue, value.alpha]
                .into_iter()
                .all(f32::is_finite),
            Self::Vec2(value) => [value.x, value.y].into_iter().all(f32::is_finite),
            Self::Vec3(value) => [value.x, value.y, value.z].into_iter().all(f32::is_finite),
            Self::Transform(value) => {
                value
                    .translation
                    .iter()
                    .chain(value.origin.iter())
                    .all(|length| length.value().is_finite())
                    && value.perspective.value().is_finite()
                    && [value.scale.x, value.scale.y, value.scale.z]
                        .into_iter()
                        .all(f32::is_finite)
                    && value
                        .rotation
                        .iter()
                        .chain(value.skew.iter())
                        .all(|angle| angle.as_radians().is_finite())
            }
            Self::Shadow(value) => [
                value.offset_x.value(),
                value.offset_y.value(),
                value.blur.value(),
                value.spread.value(),
                value.color.red,
                value.color.green,
                value.color.blue,
                value.color.alpha,
            ]
            .into_iter()
            .all(f32::is_finite),
            Self::Discrete(_) | Self::Custom(_) => true,
        };
        finite.then_some(()).ok_or(ValueError::NonFinite)
    }

    pub fn compose_add(&self, contribution: &Self) -> Result<Self, ValueError> {
        binary_arithmetic(self, contribution, |left, right| left + right)
    }

    pub fn delta_from(&self, baseline: &Self) -> Result<Self, ValueError> {
        binary_arithmetic(self, baseline, |left, right| left - right)
    }

    pub fn scale(&self, factor: f32) -> Result<Self, ValueError> {
        if !factor.is_finite() {
            return Err(ValueError::NonFinite);
        }
        match self {
            Self::Scalar(value) => Ok(Self::Scalar(value * factor)),
            Self::Length(value) => Ok(Self::Length(Length::new(
                value.value() * factor,
                value.unit(),
            ))),
            Self::Angle(value) => Ok(Self::Angle(Angle::radians(value.as_radians() * factor))),
            Self::Color(value) => Ok(Self::Color(LinearRgba::new(
                value.red * factor,
                value.green * factor,
                value.blue * factor,
                value.alpha * factor,
            )?)),
            Self::Vec2(value) => Ok(Self::Vec2(Vec2::new(value.x * factor, value.y * factor))),
            Self::Vec3(value) => Ok(Self::Vec3(Vec3::new(
                value.x * factor,
                value.y * factor,
                value.z * factor,
            ))),
            value => Err(ValueError::ArithmeticUnsupported(value.kind())),
        }
    }

    pub fn approximately_eq(&self, other: &Self, precision: f32) -> bool {
        if self.kind() != other.kind() || !precision.is_finite() || precision < 0.0 {
            return false;
        }
        let close = |left: f32, right: f32| (left - right).abs() <= precision;
        match (self, other) {
            (Self::Scalar(left), Self::Scalar(right)) => close(*left, *right),
            (Self::Length(left), Self::Length(right)) => {
                left.unit() == right.unit() && close(left.value(), right.value())
            }
            (Self::Angle(left), Self::Angle(right)) => close(left.as_radians(), right.as_radians()),
            (Self::Color(left), Self::Color(right)) => [
                (left.red, right.red),
                (left.green, right.green),
                (left.blue, right.blue),
                (left.alpha, right.alpha),
            ]
            .into_iter()
            .all(|(left, right)| close(left, right)),
            (Self::Vec2(left), Self::Vec2(right)) => {
                close(left.x, right.x) && close(left.y, right.y)
            }
            (Self::Vec3(left), Self::Vec3(right)) => {
                close(left.x, right.x) && close(left.y, right.y) && close(left.z, right.z)
            }
            (Self::Transform(left), Self::Transform(right)) => {
                left.translation
                    .iter()
                    .zip(right.translation.iter())
                    .chain(left.origin.iter().zip(right.origin.iter()))
                    .all(|(left, right)| {
                        left.unit() == right.unit() && close(left.value(), right.value())
                    })
                    && [
                        (left.scale.x, right.scale.x),
                        (left.scale.y, right.scale.y),
                        (left.scale.z, right.scale.z),
                    ]
                    .into_iter()
                    .all(|(left, right)| close(left, right))
                    && left
                        .rotation
                        .iter()
                        .zip(right.rotation.iter())
                        .chain(left.skew.iter().zip(right.skew.iter()))
                        .all(|(left, right)| close(left.as_radians(), right.as_radians()))
                    && left.perspective.unit() == right.perspective.unit()
                    && close(left.perspective.value(), right.perspective.value())
            }
            (Self::Shadow(left), Self::Shadow(right)) => {
                [
                    (left.offset_x, right.offset_x),
                    (left.offset_y, right.offset_y),
                    (left.blur, right.blur),
                    (left.spread, right.spread),
                ]
                .into_iter()
                .all(|(left, right)| {
                    left.unit() == right.unit() && close(left.value(), right.value())
                }) && Self::Color(left.color).approximately_eq(&Self::Color(right.color), precision)
            }
            (Self::Discrete(left), Self::Discrete(right)) => left == right,
            (Self::Custom(left), Self::Custom(right)) => left == right,
            _ => false,
        }
    }
}

fn binary_arithmetic(
    left: &AnimationValue,
    right: &AnimationValue,
    operation: impl Fn(f32, f32) -> f32 + Copy,
) -> Result<AnimationValue, ValueError> {
    if left.kind() != right.kind() {
        return Err(ValueError::KindMismatch {
            from: right.kind(),
            to: left.kind(),
        });
    }
    match (left, right) {
        (AnimationValue::Scalar(left), AnimationValue::Scalar(right)) => {
            Ok(AnimationValue::Scalar(operation(*left, *right)))
        }
        (AnimationValue::Length(left), AnimationValue::Length(right)) => {
            if left.unit() != right.unit() {
                return Err(ValueError::UnitMismatch);
            }
            Ok(AnimationValue::Length(Length::new(
                operation(left.value(), right.value()),
                left.unit(),
            )))
        }
        (AnimationValue::Angle(left), AnimationValue::Angle(right)) => Ok(AnimationValue::Angle(
            Angle::radians(operation(left.as_radians(), right.as_radians())),
        )),
        (AnimationValue::Color(left), AnimationValue::Color(right)) => {
            Ok(AnimationValue::Color(LinearRgba::new(
                operation(left.red, right.red),
                operation(left.green, right.green),
                operation(left.blue, right.blue),
                operation(left.alpha, right.alpha),
            )?))
        }
        (AnimationValue::Vec2(left), AnimationValue::Vec2(right)) => Ok(AnimationValue::Vec2(
            Vec2::new(operation(left.x, right.x), operation(left.y, right.y)),
        )),
        (AnimationValue::Vec3(left), AnimationValue::Vec3(right)) => {
            Ok(AnimationValue::Vec3(Vec3::new(
                operation(left.x, right.x),
                operation(left.y, right.y),
                operation(left.z, right.z),
            )))
        }
        (left, _) => Err(ValueError::ArithmeticUnsupported(left.kind())),
    }
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn interpolate_angle(from: Angle, to: Angle, progress: f32) -> Angle {
    Angle::radians(lerp(from.as_radians(), to.as_radians(), progress))
}

fn interpolate_vec2(from: Vec2, to: Vec2, progress: f32) -> Vec2 {
    Vec2::new(lerp(from.x, to.x, progress), lerp(from.y, to.y, progress))
}

fn interpolate_vec3(from: Vec3, to: Vec3, progress: f32) -> Vec3 {
    Vec3::new(
        lerp(from.x, to.x, progress),
        lerp(from.y, to.y, progress),
        lerp(from.z, to.z, progress),
    )
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    let channel = channel.clamp(0.0, 1.0);
    if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

fn channel_to_u8(channel: f32) -> u32 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_interpolation_requires_resolved_units() {
        let from = AnimationValue::Length(Length::vp(0.0));
        let to = AnimationValue::Length(Length::vp(10.0));
        assert_eq!(
            from.interpolate(&to, 0.25).unwrap(),
            AnimationValue::Length(Length::vp(2.5))
        );
        assert_eq!(
            from.interpolate(&AnimationValue::Length(Length::px(10.0)), 0.5),
            Err(ValueError::UnitMismatch)
        );
    }

    #[test]
    fn colors_round_trip_argb_and_interpolate_in_linear_space() {
        let black = LinearRgba::from_argb(0xff000000);
        let white = LinearRgba::from_argb(0xffffffff);
        assert_eq!(black.to_argb(), 0xff000000);
        assert_eq!(white.to_argb(), 0xffffffff);

        let midpoint = black.interpolate(white, 0.5).to_argb();
        assert_eq!(midpoint, 0xffbcbcbc);
    }

    #[test]
    fn sparse_discrete_values_switch_at_the_terminal_boundary() {
        let from = AnimationValue::Discrete(DiscreteValue::new("hidden"));
        let to = AnimationValue::Discrete(DiscreteValue::new("visible"));
        assert_eq!(from.interpolate(&to, 0.999).unwrap(), from);
        assert_eq!(
            from.interpolate(&to, 1.0).unwrap(),
            AnimationValue::Discrete(DiscreteValue::new("visible"))
        );
    }
}
