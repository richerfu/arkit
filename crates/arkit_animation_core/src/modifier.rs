//! Value modifiers applied after easing and interpolation.

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::{AnimationValue, ModifierError, ValueError, ValueKind};

pub trait ModifierFunction: Send + Sync + 'static {
    fn apply(&self, value: AnimationValue) -> Result<AnimationValue, ModifierError>;
}

#[derive(Clone, Default)]
pub enum Modifier {
    #[default]
    Identity,
    Clamp {
        min: f32,
        max: f32,
    },
    Round {
        decimal_places: u8,
    },
    Snap {
        step: f32,
    },
    Wrap {
        min: f32,
        max: f32,
    },
    MapRange {
        input_min: f32,
        input_max: f32,
        output_min: f32,
        output_max: f32,
    },
    Chain(Arc<[Modifier]>),
    Custom {
        name: Arc<str>,
        function: Arc<dyn ModifierFunction>,
    },
}

impl Modifier {
    pub fn clamp(min: f32, max: f32) -> Result<Self, ModifierError> {
        validate_range(min, max)?;
        Ok(Self::Clamp { min, max })
    }

    pub fn snap(step: f32) -> Result<Self, ModifierError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(ModifierError::InvalidStep);
        }
        Ok(Self::Snap { step })
    }

    pub fn wrap(min: f32, max: f32) -> Result<Self, ModifierError> {
        validate_range(min, max)?;
        Ok(Self::Wrap { min, max })
    }

    pub fn map_range(
        input_min: f32,
        input_max: f32,
        output_min: f32,
        output_max: f32,
    ) -> Result<Self, ModifierError> {
        validate_range(input_min, input_max)?;
        if !output_min.is_finite() || !output_max.is_finite() {
            return Err(ModifierError::NonFinite);
        }
        Ok(Self::MapRange {
            input_min,
            input_max,
            output_min,
            output_max,
        })
    }

    pub fn chain(modifiers: impl Into<Arc<[Modifier]>>) -> Self {
        Self::Chain(modifiers.into())
    }

    pub fn custom(name: impl Into<Arc<str>>, function: impl ModifierFunction) -> Self {
        Self::Custom {
            name: name.into(),
            function: Arc::new(function),
        }
    }

    pub fn apply(&self, value: AnimationValue) -> Result<AnimationValue, ModifierError> {
        match self {
            Self::Identity => Ok(value),
            Self::Clamp { min, max } => map_scalar(value, |value| value.clamp(*min, *max)),
            Self::Round { decimal_places } => {
                let factor = 10.0_f32.powi(i32::from(*decimal_places));
                map_scalar(value, |value| (value * factor).round() / factor)
            }
            Self::Snap { step } => map_scalar(value, |value| (value / step).round() * step),
            Self::Wrap { min, max } => map_scalar(value, |value| {
                let span = max - min;
                (value - min).rem_euclid(span) + min
            }),
            Self::MapRange {
                input_min,
                input_max,
                output_min,
                output_max,
            } => map_scalar(value, |value| {
                let progress = (value - input_min) / (input_max - input_min);
                output_min + (output_max - output_min) * progress
            }),
            Self::Chain(modifiers) => modifiers
                .iter()
                .try_fold(value, |value, modifier| modifier.apply(value)),
            Self::Custom { function, .. } => function.apply(value),
        }
    }

    pub fn validate_for_kind(&self, kind: ValueKind) -> Result<(), ModifierError> {
        match self {
            Self::Identity | Self::Custom { .. } => Ok(()),
            Self::Clamp { min, max } | Self::Wrap { min, max } => {
                require_scalar(kind)?;
                validate_range(*min, *max)
            }
            Self::Round { .. } => require_scalar(kind),
            Self::Snap { step } => {
                require_scalar(kind)?;
                if !step.is_finite() || *step <= 0.0 {
                    return Err(ModifierError::InvalidStep);
                }
                Ok(())
            }
            Self::MapRange {
                input_min,
                input_max,
                output_min,
                output_max,
            } => {
                require_scalar(kind)?;
                validate_range(*input_min, *input_max)?;
                if !output_min.is_finite() || !output_max.is_finite() {
                    return Err(ModifierError::NonFinite);
                }
                Ok(())
            }
            Self::Chain(modifiers) => modifiers
                .iter()
                .try_for_each(|modifier| modifier.validate_for_kind(kind)),
        }
    }
}

impl Debug for Modifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identity => formatter.write_str("Identity"),
            Self::Clamp { min, max } => formatter
                .debug_struct("Clamp")
                .field("min", min)
                .field("max", max)
                .finish(),
            Self::Round { decimal_places } => formatter
                .debug_struct("Round")
                .field("decimal_places", decimal_places)
                .finish(),
            Self::Snap { step } => formatter.debug_struct("Snap").field("step", step).finish(),
            Self::Wrap { min, max } => formatter
                .debug_struct("Wrap")
                .field("min", min)
                .field("max", max)
                .finish(),
            Self::MapRange {
                input_min,
                input_max,
                output_min,
                output_max,
            } => formatter
                .debug_struct("MapRange")
                .field("input_min", input_min)
                .field("input_max", input_max)
                .field("output_min", output_min)
                .field("output_max", output_max)
                .finish(),
            Self::Chain(modifiers) => formatter.debug_tuple("Chain").field(modifiers).finish(),
            Self::Custom { name, .. } => formatter.debug_tuple("Custom").field(name).finish(),
        }
    }
}

fn validate_range(min: f32, max: f32) -> Result<(), ModifierError> {
    if !min.is_finite() || !max.is_finite() {
        return Err(ModifierError::NonFinite);
    }
    if min >= max {
        return Err(ModifierError::InvalidRange);
    }
    Ok(())
}

fn require_scalar(kind: ValueKind) -> Result<(), ModifierError> {
    if kind == ValueKind::Scalar {
        Ok(())
    } else {
        Err(ValueError::KindMismatch {
            from: kind,
            to: ValueKind::Scalar,
        }
        .into())
    }
}

fn map_scalar(
    value: AnimationValue,
    map: impl FnOnce(f32) -> f32,
) -> Result<AnimationValue, ModifierError> {
    match value {
        AnimationValue::Scalar(value) if value.is_finite() => {
            Ok(AnimationValue::Scalar(map(value)))
        }
        AnimationValue::Scalar(_) => Err(ModifierError::NonFinite),
        value => Err(ValueError::KindMismatch {
            from: value.kind(),
            to: ValueKind::Scalar,
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_chains_are_applied_in_order() {
        let modifiers: Arc<[Modifier]> = Arc::from([
            Modifier::map_range(0.0, 1.0, 0.0, 10.0).unwrap(),
            Modifier::snap(2.0).unwrap(),
            Modifier::clamp(0.0, 8.0).unwrap(),
        ]);
        let result = Modifier::chain(modifiers)
            .apply(AnimationValue::Scalar(0.95))
            .unwrap();
        assert_eq!(result, AnimationValue::Scalar(8.0));
    }

    #[test]
    fn scalar_modifiers_reject_other_value_kinds() {
        assert!(Modifier::clamp(0.0, 1.0)
            .unwrap()
            .apply(AnimationValue::Length(crate::Length::vp(1.0)))
            .is_err());
    }
}
