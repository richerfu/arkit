use std::error::Error;
use std::fmt::{Display, Formatter};

use arkit_animation_core::{
    AdapterId, AdapterPropertyId, AdapterTargetId, PropertyName, SourceTarget, TargetName,
    ValueError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationAdapterError {
    DuplicateAdapter(AdapterId),
    UnknownAdapter(AdapterId),
    DuplicateTarget(TargetName),
    UnknownTarget(SourceTarget),
    UnknownTargetId(AdapterTargetId),
    DisposedTarget(AdapterTargetId),
    UnknownProperty(PropertyName),
    UnknownPropertyId(AdapterPropertyId),
    Value(ValueError),
    NativeWrite {
        target: AdapterTargetId,
        property: AdapterPropertyId,
        reason: Box<str>,
    },
    NativeRead {
        target: AdapterTargetId,
        property: AdapterPropertyId,
    },
    UnsupportedValue {
        property: AdapterPropertyId,
    },
}

impl Display for AnimationAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for AnimationAdapterError {}

impl From<ValueError> for AnimationAdapterError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationBuildError {
    EmptyKeyframes,
    InvalidKeyframeOffset(f32),
    NonIncreasingKeyframeOffset,
    TimeOverflow,
}

impl Display for AnimationBuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for AnimationBuildError {}
