//! Per-property tween source specifications.

use crate::{
    AnimationValue, Composition, Easing, Modifier, PropertyName, TimeSpan, ValueFunctionName,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FromValue {
    Explicit(AnimationValue),
    Current,
    Previous,
    RelativeBaseline(AnimationValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueSource {
    Fixed(AnimationValue),
    Relative(AnimationValue),
    Function(ValueFunctionName),
}

#[derive(Debug, Clone)]
pub struct TweenSpec {
    pub property: PropertyName,
    pub from: FromValue,
    pub to: ValueSource,
    pub delay: TimeSpan,
    pub duration: TimeSpan,
    pub priority: i32,
    pub easing: Easing,
    pub composition: Composition,
    pub modifier: Modifier,
}

impl TweenSpec {
    pub fn new(
        property: PropertyName,
        from: FromValue,
        to: ValueSource,
        duration: TimeSpan,
    ) -> Self {
        Self {
            property,
            from,
            to,
            delay: TimeSpan::ZERO,
            duration,
            priority: 0,
            easing: Easing::default(),
            composition: Composition::default(),
            modifier: Modifier::default(),
        }
    }
}
