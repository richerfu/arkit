//! Unified, root-owned animation engine and ArkUI/Drawing adapters.

mod adapter;
mod adapter_registry;
mod animatable;
mod api;
mod arkui_adapter;
mod callbacks;
mod controls;
mod diagnostic;
mod draggable;
mod drawing_adapter;
mod frame_driver;
mod hooks;
mod host;
mod layout;
mod native_capability;
mod native_instance;
mod native_lowerer;
mod presence;
mod properties;
mod property_reader;
mod property_schema;
mod property_writer;
mod resolver;
mod scope;
mod scroll;
mod selector;
mod stagger;
mod target;
mod target_store;
mod transition;

pub use adapter::{TargetAdapter, TargetLifecycle};
pub use adapter_registry::AdapterRegistry;
pub use animatable::{
    use_animatable, use_animatable_with_defaults, Animatable, AnimatableDefaults,
};
pub use api::{Animation, PropertyKeyframe, Timeline};
pub use arkui_adapter::ArkUiAdapter;
pub use controls::{AnimationControls, AnimationFinished, AnimationSubscription};
pub use diagnostic::{AnimationAdapterError, AnimationBuildError};
pub use draggable::{
    use_draggable, AutoScroll, DragAxis, DragConstraints, DragMapping, DragPhase, DragSnap,
    DragUpdate, Draggable, DraggableCallbacks, DraggableConfig, DraggableHandle, VelocityTracker,
};
pub use drawing_adapter::DrawingAdapter;
pub use frame_driver::FrameDriver;
pub use hooks::{
    use_animation, use_animation_host_provider, use_animation_snapshot, use_animation_target,
    AnimationTarget,
};
pub use host::{AnimationHost, AnimationHostError, AnimationPerformanceCounters};
pub use layout::{
    use_animation_layout, use_layout_snapshot, LayoutAnimation, LayoutAnimationMode,
    LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutMountState, LayoutNode, LayoutSnapshot,
    SharedElementProjection,
};
pub use native_capability::{
    AnimationBackend, CapabilityRequirements, ExecutionPolicy, NativeCapability,
};
pub use native_instance::{
    ArkUiAnimatorInstance, ArkUiImplicitInstance, ArkUiKeyframeInstance, NativeAnimationInstance,
    NativeAnimatorSpec, NativeInstanceError, NativeKeyframe,
};
pub use native_lowerer::{
    BackendRejection, LoweringReport, NativeLowerer, NativeLoweringError, UnsupportedFeature,
};
pub use presence::{
    use_animate_presence, AnimatePresence, ExitCancelPolicy, PresenceEntry, PresenceHandle,
    PresenceKey, PresenceMode, PresencePhase, PresenceStore,
};
pub use properties::{
    ASPECT_RATIO, BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS,
    CONTRAST, FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING,
    LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y, SEPIA,
    TRANSLATE_X, TRANSLATE_Y, WIDTH,
};
pub use property_schema::PropertySchema;
pub use resolver::AdapterResolutionSnapshot;
pub use scope::{
    use_animation_scope, use_scoped_animation, AnimationScope, AnimationScopeDefaults,
    ScopeCleanupPolicy, WindowCondition,
};
pub use scroll::{
    use_scroll_observer, ScrollAxis, ScrollCallbacks, ScrollDirection, ScrollObserver, ScrollRange,
    ScrollSample, ScrollSync, ScrollThreshold,
};
pub use selector::AnimationSelector;
pub use stagger::{stagger, Stagger, StaggerAxis, StaggerDirection, StaggerFrom, StaggerGrid};
pub use target::{AnimationTargetBinding, TargetVisualState};
pub use target_store::TargetStore;
pub use transition::{MountTransition, TransitionPreset};

pub use arkit_animation_core::{
    Angle, AnimatableValue, AnimationInstanceSnapshot, AnimationOutcome, AnimationValue,
    BuiltinEase, CallPolicy, Composition, CustomValue, DiscreteValue, EaseDirection, Easing,
    EasingError, EasingFunction, EngineDiagnostics, InvalidationClass, IrregularEase,
    IterationCount, JumpMode, LabelName, LayoutId, LayoutNodeId, Length, LengthUnit, LinearPoint,
    LinearRgba, Modifier, PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState,
    Property, PropertyDescriptor, PropertyName, ScopeMethodName, ShadowValue, SpringSpec,
    TargetName, TargetSetName, TimeError, TimeOffset, TimePoint, TimeSpan, TimelinePosition,
    TransformValue, ValueError, ValueKind, Vec2, Vec3, WindowMetrics,
};

/// Animation symbols intended for glob import (`use arkit::prelude::*`).
///
/// This is the domain-owned curation consumed by the `arkit` facade prelude,
/// so the list cannot drift from the crate's actual API. It deliberately
/// excludes [`WindowMetrics`]: that name collides with the runtime's window
/// metrics and must be referenced through the `animation::` namespace.
pub mod prelude {
    pub use crate::{
        stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
        use_animation_host_provider, use_animation_layout, use_animation_scope,
        use_animation_snapshot, use_animation_target, use_draggable, use_layout_snapshot,
        use_scoped_animation, use_scroll_observer, Angle, Animatable, AnimatableDefaults,
        AnimatableValue, AnimatePresence, Animation, AnimationAdapterError, AnimationBackend,
        AnimationBuildError, AnimationControls, AnimationFinished, AnimationHostError,
        AnimationInstanceSnapshot, AnimationOutcome, AnimationPerformanceCounters, AnimationScope,
        AnimationScopeDefaults, AnimationSelector, AnimationSubscription, AnimationTarget,
        AnimationValue, AutoScroll, BackendRejection, BuiltinEase, CallPolicy,
        CapabilityRequirements, Composition, DiscreteValue, DragAxis, DragConstraints, DragMapping,
        DragPhase, DragSnap, DragUpdate, Draggable, DraggableCallbacks, DraggableConfig,
        DraggableHandle, EaseDirection, Easing, EasingError, ExecutionPolicy, ExitCancelPolicy,
        InvalidationClass, IrregularEase, IterationCount, JumpMode, LabelName, LayoutAnimation,
        LayoutAnimationMode, LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId,
        LayoutMountState, LayoutNode, LayoutNodeId, LayoutSnapshot, Length, LengthUnit,
        LinearPoint, LinearRgba, LoweringReport, Modifier, MountTransition, NativeCapability,
        NativeLoweringError, PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState,
        PresenceEntry, PresenceHandle, PresenceKey, PresenceMode, PresencePhase, Property,
        PropertyKeyframe, PropertyName, ScopeCleanupPolicy, ScopeMethodName, ScrollAxis,
        ScrollCallbacks, ScrollDirection, ScrollObserver, ScrollRange, ScrollSample, ScrollSync,
        ScrollThreshold, ShadowValue, SharedElementProjection, SpringSpec, Stagger, StaggerAxis,
        StaggerDirection, StaggerFrom, StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint,
        TimeSpan, Timeline, TimelinePosition, TransformValue, TransitionPreset, UnsupportedFeature,
        ValueError, ValueKind, Vec2, Vec3, VelocityTracker, WindowCondition, ASPECT_RATIO,
        BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS, CONTRAST,
        FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING,
        LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y,
        SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
    };
}
