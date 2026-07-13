//! arkit — Dioxus 0.7 + ArkUI framework for OpenHarmony.
//!
//! This facade re-exports the full stack: dioxus core (`rsx!`, `use_signal`,
//! `Element`), the `dioxus_elements` registry (ArkUI element/attribute/event
//! descriptors), the ArkUI renderer + runtime host, and the component/hooks/
//! i18n/router/animation/icon libraries. The `#[entry]` macro mounts a
//! `fn() -> Element` root component into a NodeContent slot.

// --- Entry macro ---
pub use arkit_derive::entry;

// --- Runtime: VirtualDom host ---
pub use arkit_runtime::{
    set_back_press_handler, tokio_handle, ArkRuntime, EdgeInsets, EmbeddedWebViewController,
    EmbeddedWebViewInit, PhysicalRect, SafeAreaPolicy, ScopeNodeResolver, VirtualDom, WebViewFrame,
    WebViewStyle, WindowMetrics, WindowMetricsHandle, WindowMetricsSubscription,
};

// --- Renderer + native node primitives ---
pub use arkit_arkui::{
    canonical_tag, create_node, create_node_by_tag, kind_from_tag, ArkUIRenderer, EventSink,
    NodeBuilder, NodeKind, VirtualKind, VirtualListAdapter,
};

// --- Hooks (escape hatches: overlay / layout / virtual range / ark node) ---
pub use arkit_hooks as hooks;
pub use arkit_hooks::{
    use_ark_host_provider, use_ark_node, use_layout_frame, use_layout_frame_node, use_layout_size,
    use_overlay, use_safe_area, use_safe_area_policy, use_virtual_list, use_virtual_range,
    use_window_metrics, ArkHost, ArkNodeRef, LayoutFrame, LayoutSize, OverlayRoot, OverlayViewport,
    SafeArea, SafeAreaEdges, SafeAreaProps, VirtualListHandle, VirtualVisibleRange,
};

// --- i18n ---
pub use arkit_i18n as i18n;
/// Translate a message. Re-export of [`arkit_i18n::t!`].
pub use arkit_i18n::t;
pub use arkit_i18n::{use_i18n, use_i18n_provider, I18nContext};
pub use arkit_i18n_macros::i18n;

// --- Router ---
pub use arkit_router as router;
pub use arkit_router::{
    use_back_handler, AnimatedOutlet, Link, LinkProps, Routable, RouteTransition, Router,
};

// --- Animation ---
pub use arkit_animation as animation;
pub use arkit_animation::WindowMetrics as AnimationWindowMetrics;
pub use arkit_animation::{
    stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
    use_animation_host_provider, use_animation_layout, use_animation_scope, use_animation_snapshot,
    use_animation_target, use_draggable, use_layout_snapshot, use_scoped_animation,
    use_scroll_observer, Angle, Animatable, AnimatableDefaults, AnimatableValue, AnimatePresence,
    Animation, AnimationAdapterError, AnimationBackend, AnimationBuildError, AnimationControls,
    AnimationFinished, AnimationHostError, AnimationInstanceSnapshot, AnimationOutcome,
    AnimationPerformanceCounters, AnimationScope, AnimationScopeDefaults, AnimationSelector,
    AnimationSubscription, AnimationTarget, AnimationValue, AutoScroll, BackendRejection,
    BuiltinEase, CallPolicy, CapabilityRequirements, Composition, DiscreteValue, DragAxis,
    DragConstraints, DragPhase, DragSnap, DragUpdate, Draggable, DraggableCallbacks,
    DraggableConfig, DraggableHandle, EaseDirection, Easing, EasingError, ExecutionPolicy,
    ExitCancelPolicy, InvalidationClass, IrregularEase, IterationCount, JumpMode, LabelName,
    LayoutAnimation, LayoutAnimationMode, LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId,
    LayoutMountState, LayoutNode, LayoutNodeId, LayoutSnapshot, Length, LengthUnit, LinearPoint,
    LinearRgba, LoweringReport, Modifier, MountTransition, NativeCapability, NativeLoweringError,
    PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState, PresenceEntry,
    PresenceHandle, PresenceKey, PresenceMode, PresencePhase, Property, PropertyKeyframe,
    PropertyName, ScopeCleanupPolicy, ScopeMethodName, ScrollAxis, ScrollCallbacks,
    ScrollDirection, ScrollObserver, ScrollRange, ScrollSample, ScrollSync, ScrollThreshold,
    ShadowValue, SharedElementProjection, SpringSpec, Stagger, StaggerAxis, StaggerDirection,
    StaggerFrom, StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint, TimeSpan, Timeline,
    TimelinePosition, TransformValue, TransitionPreset, UnsupportedFeature, ValueError, ValueKind,
    Vec2, Vec3, VelocityTracker, WindowCondition, ASPECT_RATIO, BACKGROUND_COLOR, BLUR,
    BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS, CONTRAST, FONT_COLOR, FONT_SIZE,
    FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING, LINE_HEIGHT, OPACITY, POSITION_X,
    POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y, SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
};

// --- Icon ---
pub use arkit_icon::{has_icon, icon, icon_names};

// --- Native ECharts-compatible charts ---
pub use arkit_chart as echarts;
pub use arkit_chart::{
    Axis, AxisLabelStyle, AxisLine, AxisOrientation, AxisTick, AxisType, BasicSeries, ChartAction,
    ChartActionKind, ChartActionTarget, ChartAppendData, ChartController, ChartCoordinateFinder,
    ChartCoordinatePoint, ChartEvent, ChartOption, ChartParseError, ChartRuntimeEvent,
    ChartRuntimeEventBatchItem, ChartSelectedItems, DataPoint, DataValue, Dataset, Diagnostic,
    ECharts, EChartsProps, GraphSeries, Grid, ItemStyle, LabelLayoutCallback,
    LabelLayoutCallbackParams, LabelLayoutCallbackResult, LabelLayoutOptions, LabelStyle, Legend,
    LineStyle, LinkData, MapFeature, MapOptions, MapPolygon, MapSeries, NodeData, SankeySeries,
    Series, SeriesOptions, Title, Tooltip, VisualStyle,
};

// --- shadcn component library ---
pub use arkit_shadcn as shadcn;

// --- Dioxus core pieces — re-exported so `rsx!`-emitted paths
// (`dioxus_core::...`, `dioxus_elements::...`) resolve at the call site after
// `use arkit::prelude::*`. ---
pub use arkit_prelude::{
    component, dioxus_core, dioxus_core_macro, dioxus_elements, dioxus_hooks, dioxus_signals, rsx,
    use_coroutine, use_future, use_resource, use_signal, Element, Props,
};

// --- OpenHarmony / napi re-exports used by the `#[entry]` macro expansion. ---
pub use napi_derive_ohos;
pub use napi_ohos;
pub use ohos_arkui_binding;
pub use openharmony_ability;

#[derive(Clone, Props)]
struct EntryRootProps {
    root: fn() -> Element,
}

impl PartialEq for EntryRootProps {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.root, other.root)
    }
}

/// Mount an arkit entry component into an ArkUI [`NodeContent`] slot.
///
/// The public facade wraps every app root with the framework host context used
/// by `use_ark_node`, layout observers, virtual adapters, WebView embedding,
/// and overlay rendering. Business entry components should not call
/// [`use_ark_host_provider`] themselves.
pub fn mount_entry(
    slot: ohos_arkui_binding::common::handle::ArkUIHandle,
    app: openharmony_ability::OpenHarmonyApp,
    root: fn() -> Element,
) -> napi_ohos::Result<ArkRuntime> {
    mount_entry_with_policy(slot, app, root, SafeAreaPolicy::Safe)
}

/// Mount an Arkit entry component with an explicit root safe-area policy.
pub fn mount_entry_with_policy(
    slot: ohos_arkui_binding::common::handle::ArkUIHandle,
    app: openharmony_ability::OpenHarmonyApp,
    root: fn() -> Element,
    safe_area_policy: SafeAreaPolicy,
) -> napi_ohos::Result<ArkRuntime> {
    let dom = VirtualDom::new_with_props(arkit_entry_root, EntryRootProps { root });
    arkit_runtime::mount_virtual_dom_with_policy(slot, app, dom, safe_area_policy)
}

fn arkit_entry_root(props: EntryRootProps) -> Element {
    let _host = use_ark_host_provider();
    arkit_animation::use_animation_host_provider();
    let policy = use_safe_area_policy();
    let measured_safe_area = use_safe_area();
    let safe_area = if policy == SafeAreaPolicy::Safe {
        measured_safe_area
    } else {
        EdgeInsets::ZERO
    };
    // Keep the business root as a real Dioxus component boundary. Calling the
    // function directly would merge its hooks into this framework wrapper's
    // scope and break normal component identity/memoization semantics.
    let content = dioxus_core::DynamicNode::Component(dioxus_core::VComponent::new(
        props.root,
        (),
        "ArkitApp",
    ));

    rsx! {
        stack {
            percent_width: 1.0,
            percent_height: 1.0,
            alignment: 0,
            clip: false,
            stack {
                percent_width: 1.0,
                percent_height: 1.0,
                alignment: 0,
                padding_top: safe_area.top,
                padding_right: safe_area.right,
                padding_bottom: safe_area.bottom,
                padding_left: safe_area.left,
                {content}
            }
            OverlayRoot {}
        }
    }
}

pub mod prelude {
    //! Everything an app needs in one glob.
    //!
    //! `use arkit::prelude::*` brings in `rsx!`, signals/hooks, all ArkUI
    //! element/event descriptors, the entry macro, escape-hatch hooks, and the
    //! shadcn component + theme prelude.

    // Dioxus primitives, hooks, signals, and ArkUI element descriptors.
    pub use arkit_prelude::*;

    // Entry + runtime + renderer.
    pub use crate::{
        entry, mount_entry, mount_entry_with_policy, ArkRuntime, ArkUIRenderer, EdgeInsets,
        EmbeddedWebViewController, EmbeddedWebViewInit, EventSink, PhysicalRect, SafeAreaPolicy,
        ScopeNodeResolver, VirtualDom, WebViewFrame, WebViewStyle, WindowMetrics,
        WindowMetricsHandle, WindowMetricsSubscription,
    };

    // Native node primitives + virtual-list builder.
    pub use crate::{
        canonical_tag, create_node, create_node_by_tag, kind_from_tag, NodeBuilder, NodeKind,
        VirtualKind, VirtualListAdapter,
    };

    // Escape-hatch hooks.
    pub use crate::{
        use_ark_host_provider, use_ark_node, use_layout_frame, use_layout_frame_node,
        use_layout_size, use_overlay, use_safe_area, use_safe_area_policy, use_virtual_list,
        use_virtual_range, use_window_metrics, ArkHost, ArkNodeRef, LayoutFrame, LayoutSize,
        OverlayRoot, OverlayViewport, SafeArea, SafeAreaEdges, SafeAreaProps, VirtualListHandle,
        VirtualVisibleRange,
    };

    // i18n + router + animation + icon + charts.
    pub use crate::t;
    pub use crate::{has_icon, icon, icon_names};
    pub use crate::{
        stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
        use_animation_layout, use_animation_scope, use_animation_snapshot, use_animation_target,
        use_back_handler, use_draggable, use_i18n, use_i18n_provider, use_layout_snapshot,
        use_scoped_animation, use_scroll_observer, Angle, Animatable, AnimatableDefaults,
        AnimatableValue, AnimatePresence, AnimatedOutlet, Animation, AnimationAdapterError,
        AnimationBackend, AnimationBuildError, AnimationControls, AnimationFinished,
        AnimationHostError, AnimationInstanceSnapshot, AnimationOutcome,
        AnimationPerformanceCounters, AnimationScope, AnimationScopeDefaults, AnimationSelector,
        AnimationSubscription, AnimationTarget, AnimationValue, AnimationWindowMetrics, AutoScroll,
        Axis, AxisLabelStyle, AxisLine, AxisOrientation, AxisTick, AxisType, BackendRejection,
        BasicSeries, BuiltinEase, CallPolicy, CapabilityRequirements, ChartAction, ChartActionKind,
        ChartActionTarget, ChartAppendData, ChartController, ChartCoordinateFinder,
        ChartCoordinatePoint, ChartEvent, ChartOption, ChartParseError, ChartRuntimeEvent,
        ChartRuntimeEventBatchItem, ChartSelectedItems, Composition, DataPoint, DataValue, Dataset,
        Diagnostic, DiscreteValue, DragAxis, DragConstraints, DragPhase, DragSnap, DragUpdate,
        Draggable, DraggableCallbacks, DraggableConfig, DraggableHandle, ECharts, EChartsProps,
        EaseDirection, Easing, EasingError, ExecutionPolicy, ExitCancelPolicy, GraphSeries, Grid,
        I18nContext, InvalidationClass, IrregularEase, ItemStyle, IterationCount, JumpMode,
        LabelLayoutCallback, LabelLayoutCallbackParams, LabelLayoutCallbackResult,
        LabelLayoutOptions, LabelName, LabelStyle, LayoutAnimation, LayoutAnimationMode,
        LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId, LayoutMountState, LayoutNode,
        LayoutNodeId, LayoutSnapshot, Legend, Length, LengthUnit, LineStyle, LinearPoint,
        LinearRgba, LinkData, LoweringReport, MapFeature, MapOptions, MapPolygon, MapSeries,
        Modifier, MountTransition, NativeCapability, NativeLoweringError, NodeData,
        PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState, PresenceEntry,
        PresenceHandle, PresenceKey, PresenceMode, PresencePhase, Property, PropertyKeyframe,
        PropertyName, Routable, RouteTransition, Router, SankeySeries, ScopeCleanupPolicy,
        ScopeMethodName, ScrollAxis, ScrollCallbacks, ScrollDirection, ScrollObserver, ScrollRange,
        ScrollSample, ScrollSync, ScrollThreshold, Series, SeriesOptions, ShadowValue,
        SharedElementProjection, SpringSpec, Stagger, StaggerAxis, StaggerDirection, StaggerFrom,
        StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint, TimeSpan, Timeline,
        TimelinePosition, Title, Tooltip, TransformValue, TransitionPreset, UnsupportedFeature,
        ValueError, ValueKind, Vec2, Vec3, VelocityTracker, VisualStyle, WindowCondition,
        ASPECT_RATIO, BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH,
        BRIGHTNESS, CONTRAST, FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT,
        LETTER_SPACING, LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION,
        SCALE_X, SCALE_Y, SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
    };

    // shadcn components + theme.
    pub use arkit_shadcn::prelude::*;
}
