//! Platform-independent animation data model and execution primitives.
//!
//! This crate deliberately contains no Dioxus, ArkUI, NAPI, logging, or
//! platform-node types. Public UI builders and target adapters live in
//! `arkit_animation`; this crate owns the value, timing, interpolation, plan,
//! and engine semantics shared by every backend.

mod command;
mod compiler;
mod composition;
mod cursor;
mod easing;
mod engine;
mod error;
mod event;
mod frame;
mod id;
mod modifier;
mod plan;
mod player;
mod property;
mod resolved;
mod resolver;
mod sampler;
mod source;
mod time;
mod time_domain;
mod timeline;
mod tween;
mod value;

pub use command::{EngineCommand, SeekMode};
pub use compiler::AnimationCompiler;
pub use composition::Composition;
pub use cursor::TrackCursor;
pub use easing::{
    BuiltinEase, EaseDirection, Easing, EasingFunction, IrregularEase, JumpMode, LinearPoint,
    SpringSpec,
};
pub use engine::{
    AnimationBaselineSnapshot, AnimationEngine, AnimationInstanceSnapshot, EngineDiagnostics,
};
pub use error::{
    AnimationCompileError, AnimationResolveError, AnimationRuntimeError, AnimationSampleError,
    EasingError, ModifierError, TimeError, ValueError,
};
pub use event::{AnimationOutcome, EngineEvent};
pub use frame::{FrameBatch, FrameId, PropertyUpdate};
pub use id::{
    AdapterId, AdapterPropertyId, AdapterTargetId, CallId, EngineOutputId, InstanceId, LabelId,
    LayoutNodeId, OutputId, PropertyId, TargetId, TargetSetId, TimeDomainId, TimelineNodeId,
    TrackId, TrackSegmentId, TweenId, ValueFunctionId,
};
pub use modifier::{Modifier, ModifierFunction};
pub use plan::{
    CompiledAnimation, CompiledEvent, CompiledOutput, CompiledProperty, CompiledTarget,
    CompiledTimeDomain, CompiledTrack, CompiledTrackSegment, CompiledTween,
};
pub use player::{PlaybackDirection, PlaybackState};
pub use property::{
    AnimatableValue, BaselineStrategy, CompositionSupport, Interpolation, InvalidationClass,
    NativeSupport, Property, PropertyDescriptor, PropertyName, SymbolName, UnitDomain, ValueKind,
};
pub use resolved::{
    ResolvedAnimation, ResolvedEvent, ResolvedProperty, ResolvedTarget, ResolvedTimeDomain,
    ResolvedTween,
};
pub use resolver::{
    AnimationResolver, ResolutionContext, ResolutionTarget, TargetContext, TargetLayoutSnapshot,
    WindowMetrics,
};
pub use sampler::{AnimationSampler, SampledReplace, SampledTrack, TrackSampleContext};
pub use source::{
    LabelName, LayoutId, ScopeMethodName, SourceAnimation, SourceSet, SourceTarget, TargetName,
    TargetSetName, ValueFunctionName,
};
pub use time::{
    IterationCount, PlaybackRate, TimeExtent, TimeOffset, TimePoint, TimeSpan,
    NANOS_PER_MILLISECOND, NANOS_PER_SECOND,
};
pub use time_domain::{TimeDomainMapper, TimeDomainOptions, TimeDomainPhase, TimeDomainSample};
pub use timeline::{CallPolicy, PlaybackSettings, TimelineNode, TimelinePosition, TimelineSource};
pub use tween::{FromValue, TweenSpec, ValueSource};
pub use value::{
    Angle, AnimationValue, CustomValue, DiscreteValue, Length, LengthUnit, LinearRgba, ShadowValue,
    TransformValue, Vec2, Vec3,
};
