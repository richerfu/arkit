//! Unified, root-owned animation engine and ArkUI/Drawing adapters.
//!
//! The crate is layered, and the import path should say which layer you are
//! writing against:
//!
//! - [`prelude`] — application-facing API for Dioxus components. Recommended
//!   for anything that renders UI.
//! - crate root — application-adjacent builders, hooks and value types.
//! - [`engine`] — the machinery behind them: the root-owned host, frame
//!   driver, native backend negotiation, adapters and lowering reports.
//!
//! The three engine types that appear in application signatures
//! ([`engine::ExecutionPolicy`] and [`engine::CapabilityRequirements`] as
//! builder parameters, [`engine::LoweringReport`] as a diagnostic result) are
//! re-exported by [`prelude`], so applications rarely need to open [`engine`]
//! directly.

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

// --- Application-facing API ---

pub use animatable::{
    use_animatable, use_animatable_with_defaults, Animatable, AnimatableDefaults,
};
pub use api::{Animation, PropertyKeyframe, Timeline};
pub use controls::{AnimationControls, AnimationFinished, AnimationSubscription};
pub use diagnostic::AnimationBuildError;
pub use draggable::{
    use_draggable, AutoScroll, DragAxis, DragConstraints, DragMapping, DragPhase, DragSnap,
    DragUpdate, Draggable, DraggableCallbacks, DraggableConfig, DraggableHandle, VelocityTracker,
};
pub use hooks::{use_animation, use_animation_snapshot, use_animation_target, AnimationTarget};
pub use layout::{
    use_animation_layout, use_layout_snapshot, LayoutAnimation, LayoutAnimationMode,
    LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutMountState, LayoutNode, LayoutSnapshot,
    SharedElementProjection,
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
pub use transition::{
    use_presence_visibility, MountTransition, PresenceTransition, PresenceVisibility,
    TransitionPreset, VisibleTransition,
};

// --- Shared animation value types ---

pub use arkit_animation_core::{
    Angle, AnimatableValue, AnimationInstanceSnapshot, AnimationOutcome, AnimationValue,
    BuiltinEase, CallPolicy, Composition, CustomValue, DiscreteValue, EaseDirection, Easing,
    EasingError, EasingFunction, InvalidationClass, IrregularEase, IterationCount, JumpMode,
    LabelName, LayoutId, LayoutNodeId, Length, LengthUnit, LinearPoint, LinearRgba, Modifier,
    PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState, Property, PropertyName,
    ScopeMethodName, ShadowValue, SpringSpec, TargetName, TargetSetName, TimeError, TimeOffset,
    TimePoint, TimeSpan, TimelinePosition, TransformValue, ValueError, ValueKind, Vec2, Vec3,
    WindowMetrics,
};

/// Engine-level animation machinery.
///
/// This is the integrator surface: plugging in a backend or adapter, driving the
/// host and frame loop directly, and reading capability negotiation results.
/// Rendering UI does not require anything from here — use [`crate::prelude`].
pub mod engine {
    pub use crate::adapter::{TargetAdapter, TargetLifecycle};
    pub use crate::adapter_registry::AdapterRegistry;
    pub use crate::arkui_adapter::ArkUiAdapter;
    pub use crate::diagnostic::AnimationAdapterError;
    pub use crate::drawing_adapter::DrawingAdapter;
    pub use crate::frame_driver::FrameDriver;
    pub use crate::hooks::use_animation_host_provider;
    pub use crate::host::{AnimationHost, AnimationHostError, AnimationPerformanceCounters};
    pub use crate::native_capability::{
        AnimationBackend, CapabilityRequirements, ExecutionPolicy, NativeCapability,
    };
    pub use crate::native_instance::{
        ArkUiAnimatorInstance, ArkUiImplicitInstance, ArkUiKeyframeInstance,
        NativeAnimationInstance, NativeAnimatorSpec, NativeInstanceError, NativeKeyframe,
    };
    pub use crate::native_lowerer::{
        BackendRejection, LoweringReport, NativeLowerer, NativeLoweringError, UnsupportedFeature,
    };
    pub use crate::property_schema::PropertySchema;
    pub use crate::resolver::AdapterResolutionSnapshot;
    pub use crate::target::{AnimationTargetBinding, TargetVisualState};
    pub use crate::target_store::TargetStore;
    pub use arkit_animation_core::{EngineDiagnostics, PropertyDescriptor};
}

/// Application-facing animation API.
///
/// This is the recommended import surface for Dioxus components. Engine-level
/// types such as the host, frame driver, lowering reports and native backend
/// negotiation stay in [`crate::engine`]; the few that appear in application
/// signatures are re-exported below so ordinary code never has to open it.
pub mod prelude {
    pub use crate::{
        stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
        use_animation_layout, use_animation_scope, use_animation_snapshot, use_animation_target,
        use_draggable, use_layout_snapshot, use_presence_visibility, use_scoped_animation,
        use_scroll_observer, Angle, Animatable, AnimatableDefaults, AnimatableValue,
        AnimatePresence, Animation, AnimationBuildError, AnimationControls, AnimationFinished,
        AnimationInstanceSnapshot, AnimationOutcome, AnimationScope, AnimationScopeDefaults,
        AnimationSelector, AnimationSubscription, AnimationTarget, AnimationValue, AutoScroll,
        BuiltinEase, CallPolicy, Composition, DiscreteValue, DragAxis, DragConstraints,
        DragMapping, DragPhase, DragSnap, DragUpdate, Draggable, DraggableCallbacks,
        DraggableConfig, DraggableHandle, EaseDirection, Easing, EasingError, ExitCancelPolicy,
        InvalidationClass, IrregularEase, IterationCount, JumpMode, LabelName, LayoutAnimation,
        LayoutAnimationMode, LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId,
        LayoutMountState, LayoutNode, LayoutNodeId, LayoutSnapshot, Length, LengthUnit,
        LinearPoint, LinearRgba, Modifier, MountTransition, PlaybackDirection, PlaybackRate,
        PlaybackSettings, PlaybackState, PresenceEntry, PresenceHandle, PresenceKey, PresenceMode,
        PresencePhase, PresenceTransition, PresenceVisibility, Property, PropertyKeyframe,
        PropertyName, ScopeCleanupPolicy, ScopeMethodName, ScrollAxis, ScrollCallbacks,
        ScrollDirection, ScrollObserver, ScrollRange, ScrollSample, ScrollSync, ScrollThreshold,
        ShadowValue, SharedElementProjection, SpringSpec, Stagger, StaggerAxis, StaggerDirection,
        StaggerFrom, StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint, TimeSpan, Timeline,
        TimelinePosition, TransformValue, TransitionPreset, ValueError, ValueKind, Vec2, Vec3,
        VelocityTracker, VisibleTransition, WindowCondition,
    };
    pub use crate::{
        ASPECT_RATIO, BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH,
        BRIGHTNESS, CONTRAST, FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT,
        LETTER_SPACING, LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION,
        SCALE_X, SCALE_Y, SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
    };

    /// Engine types that appear directly in application signatures.
    pub use crate::engine::{CapabilityRequirements, ExecutionPolicy, LoweringReport};

    pub use crate::WindowMetrics as AnimationWindowMetrics;
}
