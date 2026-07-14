//! Typed errors returned by foundational animation primitives.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::{Composition, PropertyId, TweenId, ValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    NonFinite,
    Negative,
    ZeroPlaybackRate,
    Overflow,
}

impl Display for TimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NonFinite => "time value must be finite",
            Self::Negative => "time value must not be negative",
            Self::ZeroPlaybackRate => "playback rate must be greater than zero",
            Self::Overflow => "time value exceeds the supported nanosecond range",
        })
    }
}

impl Error for TimeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    KindMismatch { from: ValueKind, to: ValueKind },
    UnitMismatch,
    CustomInterpolationRequired,
    InvalidColor,
    NonFinite,
    UnitNotSupported,
    ArithmeticUnsupported(ValueKind),
}

impl Display for ValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KindMismatch { from, to } => {
                write!(formatter, "cannot interpolate {from:?} as {to:?}")
            }
            Self::UnitMismatch => {
                formatter.write_str("length units must be resolved before interpolation")
            }
            Self::CustomInterpolationRequired => {
                formatter.write_str("custom values require an adapter-owned interpolator")
            }
            Self::InvalidColor => formatter.write_str("color channels must be finite"),
            Self::NonFinite => formatter.write_str("animation values must be finite"),
            Self::UnitNotSupported => {
                formatter.write_str("length unit is not supported by this property")
            }
            Self::ArithmeticUnsupported(kind) => {
                write!(
                    formatter,
                    "{kind:?} values do not support arithmetic composition"
                )
            }
        }
    }
}

impl Error for ValueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EasingError {
    EmptyLinearPoints,
    NonFinitePoint,
    UnsortedLinearPoints,
    InvalidBezierX,
    InvalidSpring,
    InvalidBuiltin,
    InvalidIrregular,
}

impl Display for EasingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::EmptyLinearPoints => "linear easing requires at least one point",
            Self::NonFinitePoint => "easing points must be finite",
            Self::UnsortedLinearPoints => "linear easing x positions must be sorted",
            Self::InvalidBezierX => "cubic Bezier x control points must be in 0..=1",
            Self::InvalidSpring => "spring parameters must be finite and physically valid",
            Self::InvalidBuiltin => "builtin easing parameters must be finite and valid",
            Self::InvalidIrregular => "irregular easing parameters must be finite and valid",
        })
    }
}

impl Error for EasingError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ModifierError {
    Value(ValueError),
    InvalidRange,
    InvalidStep,
    NonFinite,
    Custom(String),
}

impl Display for ModifierError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(error) => Display::fmt(error, formatter),
            Self::InvalidRange => {
                formatter.write_str("modifier range must have distinct finite bounds")
            }
            Self::InvalidStep => {
                formatter.write_str("modifier step must be finite and greater than zero")
            }
            Self::NonFinite => formatter.write_str("modifier input and parameters must be finite"),
            Self::Custom(message) => formatter.write_str(message),
        }
    }
}

impl Error for ModifierError {}

impl From<ValueError> for ModifierError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationCompileError {
    MissingRootTimeDomain,
    InvalidTimeDomainParent {
        domain: crate::TimeDomainId,
        parent: Option<crate::TimeDomainId>,
    },
    InvalidTimeDomainEasing {
        domain: crate::TimeDomainId,
        source: EasingError,
    },
    UnknownTimeDomain(crate::TimeDomainId),
    UnknownTarget(crate::TargetId),
    UnknownProperty(PropertyId),
    AdapterMismatch {
        target: crate::TargetId,
        property: PropertyId,
    },
    PropertyNotWritable(PropertyId),
    InvalidPropertyPrecision(PropertyId),
    InvalidValue {
        property: PropertyId,
        source: ValueError,
    },
    InvalidEasing {
        tween: TweenId,
        source: EasingError,
    },
    InvalidModifier {
        property: PropertyId,
        source: ModifierError,
    },
    UnsupportedComposition {
        property: PropertyId,
        composition: Composition,
    },
    TimeOverflow(TweenId),
}

impl Display for AnimationCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRootTimeDomain => {
                formatter.write_str("compiled plan requires a root time domain")
            }
            Self::InvalidTimeDomainParent { domain, parent } => write!(
                formatter,
                "time domain {domain:?} has invalid parent {parent:?}"
            ),
            Self::InvalidTimeDomainEasing { domain, source } => {
                write!(
                    formatter,
                    "time domain {domain:?} has invalid playback easing: {source}"
                )
            }
            Self::UnknownTimeDomain(domain) => write!(formatter, "unknown time domain {domain:?}"),
            Self::UnknownTarget(target) => write!(formatter, "unknown target {target:?}"),
            Self::UnknownProperty(property) => write!(formatter, "unknown property {property:?}"),
            Self::AdapterMismatch { target, property } => write!(
                formatter,
                "target {target:?} and property {property:?} belong to different adapters"
            ),
            Self::PropertyNotWritable(property) => {
                write!(formatter, "property {property:?} is not writable")
            }
            Self::InvalidPropertyPrecision(property) => write!(
                formatter,
                "property {property:?} precision must be finite and greater than zero"
            ),
            Self::InvalidValue { property, source } => {
                write!(
                    formatter,
                    "invalid value for property {property:?}: {source}"
                )
            }
            Self::InvalidEasing { tween, source } => {
                write!(formatter, "invalid easing for tween {tween:?}: {source}")
            }
            Self::InvalidModifier { property, source } => {
                write!(
                    formatter,
                    "invalid modifier for property {property:?}: {source}"
                )
            }
            Self::UnsupportedComposition {
                property,
                composition,
            } => write!(
                formatter,
                "property {property:?} does not support {composition:?} composition"
            ),
            Self::TimeOverflow(tween) => {
                write!(formatter, "time range overflows for tween {tween:?}")
            }
        }
    }
}

impl Error for AnimationCompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidValue { source, .. } => Some(source),
            Self::InvalidEasing { source, .. } => Some(source),
            Self::InvalidTimeDomainEasing { source, .. } => Some(source),
            Self::InvalidModifier { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationResolveError {
    Context(Arc<str>),
    EmptyTargetSelection,
    DuplicateTarget {
        adapter: crate::AdapterId,
        target: crate::AdapterTargetId,
    },
    PropertyBindingConflict {
        adapter: crate::AdapterId,
        property: crate::AdapterPropertyId,
    },
    Value {
        property: crate::PropertyName,
        source: ValueError,
    },
    UnknownLabel(crate::LabelName),
    DuplicateLabel(crate::LabelName),
    InvalidPercentage(f32),
    TimeOverflow,
    PositionAfterInfiniteTimeline,
}

impl AnimationResolveError {
    pub fn context(message: impl Into<Arc<str>>) -> Self {
        Self::Context(message.into())
    }
}

impl Display for AnimationResolveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(message) => formatter.write_str(message),
            Self::EmptyTargetSelection => formatter.write_str("target selection resolved empty"),
            Self::DuplicateTarget { adapter, target } => write!(
                formatter,
                "target selection contains duplicate adapter target ({adapter:?}, {target:?})"
            ),
            Self::PropertyBindingConflict { adapter, property } => write!(
                formatter,
                "adapter property ({adapter:?}, {property:?}) resolved with conflicting schemas"
            ),
            Self::Value { property, source } => {
                write!(
                    formatter,
                    "failed to resolve property {property:?}: {source}"
                )
            }
            Self::UnknownLabel(label) => write!(formatter, "unknown timeline label {label:?}"),
            Self::DuplicateLabel(label) => {
                write!(formatter, "duplicate timeline label {label:?}")
            }
            Self::InvalidPercentage(value) => {
                write!(
                    formatter,
                    "timeline percentage must be in 0..=1, got {value}"
                )
            }
            Self::TimeOverflow => formatter.write_str("resolved timeline time exceeds u64 nanos"),
            Self::PositionAfterInfiniteTimeline => formatter.write_str(
                "relative position cannot use an infinite timeline or previous-node end",
            ),
        }
    }
}

impl Error for AnimationResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Value { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRuntimeError {
    UnknownInstance(crate::InstanceKey),
    InstanceGenerationExhausted(crate::InstanceId),
    InfiniteAnimationCannotComplete(crate::InstanceKey),
    BaselineCountMismatch {
        expected: usize,
        actual: usize,
    },
    BaselineKindMismatch(crate::OutputId),
    GlobalPropertyContractMismatch {
        adapter: crate::AdapterId,
        target: crate::AdapterTargetId,
        property: crate::AdapterPropertyId,
    },
    UnknownOutput {
        instance: crate::InstanceKey,
        adapter: crate::AdapterId,
        target: crate::AdapterTargetId,
        property: crate::AdapterPropertyId,
    },
    TrackSamplingFailed(crate::TrackId),
    OutputCompositionFailed(crate::EngineOutputId),
    FrameNotAcknowledged(crate::FrameId),
    FrameSequenceExhausted,
    NoFramePending(crate::FrameId),
    UnexpectedFrameAcknowledgement {
        expected: crate::FrameId,
        actual: crate::FrameId,
    },
}

impl Display for AnimationRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownInstance(instance) => {
                write!(formatter, "unknown animation instance {instance:?}")
            }
            Self::InstanceGenerationExhausted(instance) => write!(
                formatter,
                "animation instance slot {instance:?} exhausted its generation counter"
            ),
            Self::InfiniteAnimationCannotComplete(instance) => write!(
                formatter,
                "infinite animation instance {instance:?} cannot be completed"
            ),
            Self::BaselineCountMismatch { expected, actual } => write!(
                formatter,
                "baseline snapshot contains {actual} outputs, expected {expected}"
            ),
            Self::BaselineKindMismatch(output) => {
                write!(
                    formatter,
                    "baseline value kind does not match output {output:?}"
                )
            }
            Self::GlobalPropertyContractMismatch {
                adapter,
                target,
                property,
            } => write!(
                formatter,
                "global property contract mismatch for {adapter:?}/{target:?}/{property:?}"
            ),
            Self::UnknownOutput {
                instance,
                adapter,
                target,
                property,
            } => write!(
                formatter,
                "animation instance {instance:?} has no output for {adapter:?}/{target:?}/{property:?}"
            ),
            Self::TrackSamplingFailed(track) => {
                write!(formatter, "failed to sample track {track:?}")
            }
            Self::OutputCompositionFailed(output) => {
                write!(formatter, "failed to compose engine output {output:?}")
            }
            Self::FrameNotAcknowledged(frame) => {
                write!(
                    formatter,
                    "frame {frame:?} must be acknowledged before the next tick"
                )
            }
            Self::FrameSequenceExhausted => formatter.write_str("frame sequence exhausted"),
            Self::NoFramePending(frame) => {
                write!(
                    formatter,
                    "cannot acknowledge {frame:?}; no frame is pending"
                )
            }
            Self::UnexpectedFrameAcknowledgement { expected, actual } => write!(
                formatter,
                "expected acknowledgement for {expected:?}, received {actual:?}"
            ),
        }
    }
}

impl Error for AnimationRuntimeError {}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationSampleError {
    Value {
        tween: crate::TweenId,
        source: ValueError,
    },
    Modifier {
        tween: crate::TweenId,
        source: ModifierError,
    },
}

impl Display for AnimationSampleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value { tween, source } => {
                write!(formatter, "failed to sample tween {tween:?}: {source}")
            }
            Self::Modifier { tween, source } => {
                write!(formatter, "failed to modify tween {tween:?}: {source}")
            }
        }
    }
}

impl Error for AnimationSampleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Value { source, .. } => Some(source),
            Self::Modifier { source, .. } => Some(source),
        }
    }
}
