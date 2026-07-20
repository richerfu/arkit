//! arkit — Dioxus 0.7 + ArkUI framework for OpenHarmony.
//!
//! The default facade exports Dioxus core (`rsx!`, `use_signal`, `Element`),
//! the ArkUI element registry, renderer, runtime, and host hooks. Domain
//! libraries are opt-in through the `animation`, `camera`, `chart`, `i18n`,
//! `canvas`, `icon`, `lottie`, `markdown`, `router`, and `shadcn` features (or `full`). The
//! `#[entry]` macro mounts a `fn() -> Element` root component into a NodeContent
//! slot.

// --- Entry macro ---
pub use arkit_derive::entry;

// --- Runtime: VirtualDom host ---
pub use arkit_runtime::{
    queue_ui_loop, register_back_press_handler, register_scope_resolver, tokio_handle,
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecyclePhase,
    ApplicationLifecycleState, ApplicationLifecycleSubscription, ArkRuntime, BackPressRegistration,
    EdgeInsets, EmbeddedWebViewController, EmbeddedWebViewInit, PhysicalRect, SafeAreaPolicy,
    ScopeNodeResolver, ScopeResolverRegistration, VirtualDom, WebViewFrame, WebViewStyle,
    WindowMetrics, WindowMetricsHandle, WindowMetricsSubscription,
};

// --- Renderer + native node primitives ---
pub use arkit_arkui::{
    canonical_tag, create_node, create_node_by_tag, kind_from_tag, ArkUIRenderer, EventSink,
    NodeBuilder, NodeKind, VirtualKind, VirtualNodeAdapter,
};

// --- Hooks (escape hatches: overlay / layout / virtual range / ark node) ---
pub use arkit_hooks as hooks;
pub use arkit_hooks::{
    use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
    use_ark_host_provider, use_ark_node, use_component_lifecycle, use_component_visibility,
    use_layout_frame, use_layout_frame_node, use_layout_size, use_overlay, use_safe_area,
    use_safe_area_policy, use_virtual_node_adapter, use_virtual_range, use_window_metrics, ArkHost,
    ArkNodeRef, ComponentLifecycleState, HitTestMode, LayoutFrame, LayoutSize, OverlayLayer,
    OverlayRoot, OverlayViewport, SafeArea, SafeAreaEdges, SafeAreaProps, VirtualVisibleRange,
};

// --- i18n ---
#[cfg(feature = "i18n")]
pub use arkit_i18n as i18n;
#[cfg(feature = "i18n")]
pub use arkit_i18n::i18n;
/// Translate a message. Re-export of [`arkit_i18n::t!`].
#[cfg(feature = "i18n")]
pub use arkit_i18n::t;
#[cfg(feature = "i18n")]
pub use arkit_i18n::{use_i18n, use_i18n_provider, I18nContext};

// --- Router ---
#[cfg(feature = "router")]
pub use arkit_router as router;
#[cfg(feature = "router")]
pub use arkit_router::{
    use_back_handler, AnimatedOutlet, Link, LinkProps, Routable, RouteTransition, Router,
};

// --- Animation ---
#[cfg(feature = "animation")]
pub use arkit_animation as animation;
#[cfg(feature = "animation")]
pub use arkit_animation::WindowMetrics as AnimationWindowMetrics;
#[cfg(feature = "animation")]
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
    DragConstraints, DragMapping, DragPhase, DragSnap, DragUpdate, Draggable, DraggableCallbacks,
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

// --- Native CameraKit preview and capture ---
#[cfg(feature = "camera")]
pub use arkit_camera as camera;
#[cfg(feature = "camera")]
pub use arkit_camera::{
    CameraCapabilities, CameraCaptureOptions, CameraController, CameraControls, CameraError,
    CameraErrorKind, CameraExposureMode, CameraFlashMode, CameraFloatRange, CameraFocusMode,
    CameraFocusState, CameraFrameRateRange, CameraImageRotation, CameraLocation, CameraMode,
    CameraPhotoModeConfiguration, CameraPhotoPreviewInteractions, CameraPhotoQuality,
    CameraPhotoToolbarConfiguration, CameraPoint, CameraPosition, CameraPreview,
    CameraPreviewProps, CameraProfileSelection, CameraQualityPriority, CameraResult,
    CameraSessionInfo, CameraSize, CameraStabilizationMode, CameraStatus, CameraTorchMode,
    CameraView, CameraViewProps, CameraWhiteBalanceMode, CapturedPhoto,
};

#[cfg(feature = "camera-scan")]
pub use arkit_camera::{
    CameraScanConfiguration, CameraScanFormat, CameraScanModeConfiguration,
    CameraScanPreviewInteractions, CameraScanRegion, CameraScanResult,
    CameraScanToolbarConfiguration,
};

// --- W3C-aligned Canvas 2D ---
#[cfg(feature = "canvas")]
pub use arkit_canvas as canvas;
#[cfg(feature = "canvas")]
pub use arkit_canvas::{
    Canvas, CanvasColor, CanvasColorSpace, CanvasColorType, CanvasController, CanvasError,
    CanvasFont, CanvasFontFace, CanvasFontKerning, CanvasFontRegistry, CanvasFontStretch,
    CanvasFontStyle, CanvasFontVariantCaps, CanvasGradient, CanvasImage, CanvasImageDecodeOptions,
    CanvasImageEncodeOptions, CanvasImageFormat, CanvasImageSmoothingQuality, CanvasLineCap,
    CanvasLineJoin, CanvasPattern, CanvasPatternRepetition, CanvasRadius, CanvasRenderer,
    CanvasRenderingContext2D, CanvasRenderingContext2DSettings, CanvasResult, CanvasStyle,
    CanvasTextAlign, CanvasTextBaseline, CanvasTextDirection, CanvasTextMetrics,
    CanvasTextRendering, DomMatrix2D, FillRule, Float16, GlobalCompositeOperation, ImageData,
    ImageDataArray, ImageDataPixelFormat, ImageDataSettings, IntoCanvasFont, IntoCanvasRadii,
    IntoCanvasStyle, OffscreenCanvas, Path2D,
};

// --- Icon ---
#[cfg(feature = "icon")]
pub use arkit_icon::{has_icon, icon, icon_names};

// --- High-performance native Lottie ---
#[cfg(feature = "lottie")]
pub use arkit_lottie as lottie;
#[cfg(feature = "lottie")]
pub use arkit_lottie::{
    LottieAlignment, LottieComposition, LottieController, LottieError, LottieErrorKind, LottieFit,
    LottieFrame, LottieFrameRenderOptions, LottieFrameRenderer, LottieNetworkSource, LottiePlayer,
    LottiePlayerProps, LottieRenderedFrame, LottieRepeatMode, LottieResult, LottieSource,
    LottieStatus,
};

// --- Native ECharts-compatible charts ---
#[cfg(feature = "chart")]
pub use arkit_chart as echarts;
#[cfg(feature = "chart")]
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
#[cfg(feature = "shadcn")]
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
    #[cfg(feature = "animation")]
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
    //! element/event descriptors, the entry macro, and escape-hatch hooks.
    //! Domain APIs appear only when their facade feature is enabled.

    // Dioxus primitives, hooks, signals, and ArkUI element descriptors.
    pub use arkit_prelude::*;

    // Entry + runtime + renderer.
    pub use crate::{
        entry, mount_entry, mount_entry_with_policy, ApplicationLifecycleEvent,
        ApplicationLifecycleHandle, ApplicationLifecyclePhase, ApplicationLifecycleState,
        ApplicationLifecycleSubscription, ArkRuntime, ArkUIRenderer, EdgeInsets,
        EmbeddedWebViewController, EmbeddedWebViewInit, EventSink, PhysicalRect, SafeAreaPolicy,
        ScopeNodeResolver, VirtualDom, WebViewFrame, WebViewStyle, WindowMetrics,
        WindowMetricsHandle, WindowMetricsSubscription,
    };

    // UI-loop handoff for native callbacks that must update Dioxus state
    // without re-entering the current render or native callback.
    pub use crate::queue_ui_loop;

    // Native node primitives + virtual container builder.
    pub use crate::{
        canonical_tag, create_node, create_node_by_tag, kind_from_tag, NodeBuilder, NodeKind,
        VirtualKind, VirtualNodeAdapter,
    };

    // Escape-hatch hooks.
    pub use crate::{
        use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
        use_ark_host_provider, use_ark_node, use_component_lifecycle, use_component_visibility,
        use_layout_frame, use_layout_frame_node, use_layout_size, use_overlay, use_safe_area,
        use_safe_area_policy, use_virtual_node_adapter, use_virtual_range, use_window_metrics,
        ArkHost, ArkNodeRef, ComponentLifecycleState, HitTestMode, LayoutFrame, LayoutSize,
        OverlayLayer, OverlayRoot, OverlayViewport, SafeArea, SafeAreaEdges, SafeAreaProps,
        VirtualVisibleRange,
    };

    #[cfg(feature = "i18n")]
    pub use crate::t;
    #[cfg(feature = "i18n")]
    pub use crate::{i18n, use_i18n, use_i18n_provider, I18nContext};

    #[cfg(feature = "icon")]
    pub use crate::{has_icon, icon, icon_names};

    #[cfg(feature = "lottie")]
    pub use crate::{
        lottie, LottieAlignment, LottieComposition, LottieController, LottieError, LottieErrorKind,
        LottieFit, LottieFrame, LottieFrameRenderOptions, LottieFrameRenderer, LottieNetworkSource,
        LottiePlayer, LottiePlayerProps, LottieRenderedFrame, LottieRepeatMode, LottieResult,
        LottieSource, LottieStatus,
    };

    #[cfg(feature = "router")]
    pub use crate::{
        use_back_handler, AnimatedOutlet, Link, LinkProps, Routable, RouteTransition, Router,
    };

    #[cfg(feature = "animation")]
    pub use crate::{
        stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
        use_animation_layout, use_animation_scope, use_animation_snapshot, use_animation_target,
        use_draggable, use_layout_snapshot, use_scoped_animation, use_scroll_observer, Angle,
        Animatable, AnimatableDefaults, AnimatableValue, AnimatePresence, Animation,
        AnimationAdapterError, AnimationBackend, AnimationBuildError, AnimationControls,
        AnimationFinished, AnimationHostError, AnimationInstanceSnapshot, AnimationOutcome,
        AnimationPerformanceCounters, AnimationScope, AnimationScopeDefaults, AnimationSelector,
        AnimationSubscription, AnimationTarget, AnimationValue, AnimationWindowMetrics, AutoScroll,
        BackendRejection, BuiltinEase, CallPolicy, CapabilityRequirements, Composition,
        DiscreteValue, DragAxis, DragConstraints, DragMapping, DragPhase, DragSnap, DragUpdate,
        Draggable, DraggableCallbacks, DraggableConfig, DraggableHandle, EaseDirection, Easing,
        EasingError, ExecutionPolicy, ExitCancelPolicy, InvalidationClass, IrregularEase,
        IterationCount, JumpMode, LabelName, LayoutAnimation, LayoutAnimationMode,
        LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId, LayoutMountState, LayoutNode,
        LayoutNodeId, LayoutSnapshot, Length, LengthUnit, LinearPoint, LinearRgba, LoweringReport,
        Modifier, MountTransition, NativeCapability, NativeLoweringError, PlaybackDirection,
        PlaybackRate, PlaybackSettings, PlaybackState, PresenceEntry, PresenceHandle, PresenceKey,
        PresenceMode, PresencePhase, Property, PropertyKeyframe, PropertyName, ScopeCleanupPolicy,
        ScopeMethodName, ScrollAxis, ScrollCallbacks, ScrollDirection, ScrollObserver, ScrollRange,
        ScrollSample, ScrollSync, ScrollThreshold, ShadowValue, SharedElementProjection,
        SpringSpec, Stagger, StaggerAxis, StaggerDirection, StaggerFrom, StaggerGrid, TargetName,
        TimeError, TimeOffset, TimePoint, TimeSpan, Timeline, TimelinePosition, TransformValue,
        TransitionPreset, UnsupportedFeature, ValueError, ValueKind, Vec2, Vec3, VelocityTracker,
        WindowCondition, ASPECT_RATIO, BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS,
        BORDER_WIDTH, BRIGHTNESS, CONTRAST, FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE,
        HEIGHT, INVERT, LETTER_SPACING, LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION,
        SATURATION, SCALE_X, SCALE_Y, SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
    };

    #[cfg(feature = "chart")]
    pub use crate::{
        Axis, AxisLabelStyle, AxisLine, AxisOrientation, AxisTick, AxisType, BasicSeries,
        ChartAction, ChartActionKind, ChartActionTarget, ChartAppendData, ChartController,
        ChartCoordinateFinder, ChartCoordinatePoint, ChartEvent, ChartOption, ChartParseError,
        ChartRuntimeEvent, ChartRuntimeEventBatchItem, ChartSelectedItems, DataPoint, DataValue,
        Dataset, Diagnostic, ECharts, EChartsProps, GraphSeries, Grid, ItemStyle,
        LabelLayoutCallback, LabelLayoutCallbackParams, LabelLayoutCallbackResult,
        LabelLayoutOptions, LabelStyle, Legend, LineStyle, LinkData, MapFeature, MapOptions,
        MapPolygon, MapSeries, NodeData, SankeySeries, Series, SeriesOptions, Title, Tooltip,
        VisualStyle,
    };

    #[cfg(feature = "canvas")]
    pub use crate::{
        canvas, Canvas, CanvasColor, CanvasColorSpace, CanvasColorType, CanvasController,
        CanvasError, CanvasFont, CanvasFontFace, CanvasFontKerning, CanvasFontRegistry,
        CanvasFontStretch, CanvasFontStyle, CanvasFontVariantCaps, CanvasGradient, CanvasImage,
        CanvasImageDecodeOptions, CanvasImageEncodeOptions, CanvasImageFormat,
        CanvasImageSmoothingQuality, CanvasLineCap, CanvasLineJoin, CanvasPattern,
        CanvasPatternRepetition, CanvasRadius, CanvasRenderer, CanvasRenderingContext2D,
        CanvasRenderingContext2DSettings, CanvasResult, CanvasStyle, CanvasTextAlign,
        CanvasTextBaseline, CanvasTextDirection, CanvasTextMetrics, CanvasTextRendering,
        DomMatrix2D, FillRule, Float16, GlobalCompositeOperation, ImageData, ImageDataArray,
        ImageDataPixelFormat, ImageDataSettings, IntoCanvasFont, IntoCanvasRadii, IntoCanvasStyle,
        OffscreenCanvas, Path2D,
    };

    #[cfg(feature = "camera")]
    pub use crate::{
        CameraCapabilities, CameraCaptureOptions, CameraController, CameraControls, CameraError,
        CameraErrorKind, CameraFlashMode, CameraFocusMode, CameraMode,
        CameraPhotoModeConfiguration, CameraPhotoPreviewInteractions,
        CameraPhotoToolbarConfiguration, CameraPoint, CameraPosition, CameraPreview,
        CameraPreviewProps, CameraProfileSelection, CameraResult, CameraSessionInfo, CameraSize,
        CameraStatus, CameraTorchMode, CameraView, CameraViewProps, CapturedPhoto,
    };

    #[cfg(feature = "camera-scan")]
    pub use crate::{
        CameraScanConfiguration, CameraScanFormat, CameraScanModeConfiguration,
        CameraScanPreviewInteractions, CameraScanRegion, CameraScanResult,
        CameraScanToolbarConfiguration,
    };

    #[cfg(feature = "shadcn")]
    pub use crate::shadcn;
}
