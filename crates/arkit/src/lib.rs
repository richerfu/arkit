//! arkit — Dioxus 0.7 + ArkUI framework for OpenHarmony.
//!
//! The default facade exports Dioxus core (`rsx!`, `use_signal`, `Element`),
//! the ArkUI element registry, per-root runtime, and exact-element hooks. Domain
//! libraries are opt-in through the `animation`, `barcode`, `camera`, `canvas`,
//! `chart`, `code`, `i18n`, `icon`, `lottie`, `markdown`, `router`, `shadcn`,
//! `terminal`, `video`, and `webview` features (or `full`). Barcode/QR generation is the
//! `barcode` feature (no camera). Code highlighting uses `code`; Markdown
//! fences need `markdown` + `code`. Terminal uses `terminal` (rio-vt).
//! `webview` enables the pluginized WebView capability (`ohos.webview` bridge
//! plugin): the framework registers the plugin facade and injects its
//! initialization automatically, so integrators only enable the feature and
//! use [`RuntimeHandle::webview`]. The `#[entry]` macro mounts a
//! `fn() -> Element` root component (or a one-argument form that receives an
//! [`openharmony_ability::OpenHarmonyApp`] handle for registering application
//! `BridgePlugin`s) into a NodeContent slot.

// --- Entry macro ---
pub use arkit_derive::entry;

// --- Runtime: VirtualDom host ---
pub use arkit_runtime::{
    use_runtime_handle, ApplicationLifecycleEvent, ApplicationLifecycleHandle,
    ApplicationLifecyclePhase, ApplicationLifecycleState, ApplicationLifecycleSubscription,
    ArkRuntime, BackPressRegistration, EdgeInsets, PhysicalRect, RuntimeHandle, RuntimeId,
    SafeAreaPolicy, VirtualDom, WindowMetrics, WindowMetricsHandle, WindowMetricsSubscription,
};
// The runtime crate is part of the public facade: the `#[entry]` expansion
// reaches `arkit_runtime::inject_plugins` through it, and advanced users can
// mount runtimes directly.
pub use arkit_runtime;

// --- Pluginized WebView capability (feature `webview`) ---
#[cfg(feature = "webview")]
pub use arkit_runtime::{
    inject_webview_plugins, NodeExt, NodeSurface, WebviewCallbacksBuilder, WebviewClient,
    WebviewCreateRequest, WebviewHandle, WebviewJavascriptProxyBuilder, WebviewProtocol,
    WebviewProtocolOptions, WebviewProtocolRequest, WebviewProtocolResponse, WebviewStyle,
};

// --- Renderer-owned handles safe for application code ---
pub use arkit_arkui::{
    ArkImageSource, LayoutFramePx, MountedNodeLease, NativeElementEvent, NativeElementRef,
    NativeElementSubscription, NativeVisibility, VirtualKind, VirtualSource,
};

/// Explicit advanced-native construction APIs.
///
/// Nodes built here have unique Rust ownership until they are transferred to a
/// renderer or virtual source. Renderer-owned mounted nodes are only available
/// through generation-checked [`MountedNodeLease`] values.
pub mod native {
    pub use arkit_arkui::{
        NativeNodeEvent, NodeBuilder, NodeEventType, OwnedNativeNode, PreDragStatus,
    };
}

// --- Hooks (exact refs / portals / virtualization) ---
pub use arkit_hooks as hooks;
pub use arkit_hooks::{
    use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
    use_component_lifecycle, use_component_visibility, use_layout_frame, use_layout_size,
    use_load_more, use_mounted_node, use_native_element_ref, use_overlay_viewport, use_safe_area,
    use_safe_area_policy, use_virtual_range, use_virtual_source, use_virtual_source_items_keyed,
    use_window_metrics, ComponentLifecycleState, LayoutFrame, LayoutSize, LoadMoreController,
    LoadMoreState, ModalPortal, ModalPresentation, OverlayLayer, OverlayViewport, Portal, SafeArea,
    SafeAreaEdges, SafeAreaProps, VirtualSourceItem, VirtualVisibleRange,
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
    use_back_handler, AnimatedOutlet, Link, LinkProps, Routable, RouteProvider, RouteProviderProps,
    RouteTransition, Router, RouterProps,
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
    use_animation_target, use_draggable, use_layout_snapshot, use_presence_visibility,
    use_scoped_animation, use_scroll_observer, Angle, Animatable, AnimatableDefaults,
    AnimatableValue, AnimatePresence, Animation, AnimationAdapterError, AnimationBackend,
    AnimationBuildError, AnimationControls, AnimationFinished, AnimationHostError,
    AnimationInstanceSnapshot, AnimationOutcome, AnimationPerformanceCounters, AnimationScope,
    AnimationScopeDefaults, AnimationSelector, AnimationSubscription, AnimationTarget,
    AnimationValue, AutoScroll, BackendRejection, BuiltinEase, CallPolicy, CapabilityRequirements,
    Composition, DiscreteValue, DragAxis, DragConstraints, DragMapping, DragPhase, DragSnap,
    DragUpdate, Draggable, DraggableCallbacks, DraggableConfig, DraggableHandle, EaseDirection,
    Easing, EasingError, ExecutionPolicy, ExitCancelPolicy, InvalidationClass, IrregularEase,
    IterationCount, JumpMode, LabelName, LayoutAnimation, LayoutAnimationMode, LayoutChangeKind,
    LayoutDelta, LayoutEngine, LayoutId, LayoutMountState, LayoutNode, LayoutNodeId,
    LayoutSnapshot, Length, LengthUnit, LinearPoint, LinearRgba, LoweringReport, Modifier,
    MountTransition, NativeCapability, NativeLoweringError, PlaybackDirection, PlaybackRate,
    PlaybackSettings, PlaybackState, PresenceEntry, PresenceHandle, PresenceKey, PresenceMode,
    PresencePhase, PresenceTransition, PresenceVisibility, Property, PropertyKeyframe,
    PropertyName, ScopeCleanupPolicy, ScopeMethodName, ScrollAxis, ScrollCallbacks,
    ScrollDirection, ScrollObserver, ScrollRange, ScrollSample, ScrollSync, ScrollThreshold,
    ShadowValue, SharedElementProjection, SpringSpec, Stagger, StaggerAxis, StaggerDirection,
    StaggerFrom, StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint, TimeSpan, Timeline,
    TimelinePosition, TransformValue, TransitionPreset, UnsupportedFeature, ValueError, ValueKind,
    Vec2, Vec3, VelocityTracker, VisibleTransition, WindowCondition, ASPECT_RATIO,
    BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS, CONTRAST,
    FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING,
    LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y, SEPIA,
    TRANSLATE_X, TRANSLATE_Y, WIDTH,
};

// --- Barcode / QR generation ---
#[cfg(feature = "barcode")]
pub use arkit_barcode as barcode;
#[cfg(feature = "barcode")]
pub use arkit_barcode::{
    encode_barcode, use_barcode, Barcode, BarcodeArtifact, BarcodeBitmap, BarcodeError,
    BarcodeErrorKind, BarcodeFormat, BarcodeHandle, BarcodeOptions, BarcodePhase, BarcodeProps,
    BarcodeRequest, BarcodeResult, QrEcLevel, DEFAULT_MARGIN, MAX_BARCODE_EDGE,
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

// --- Native AVPlayer video ---
#[cfg(feature = "video")]
pub use arkit_video as video;
#[cfg(feature = "video")]
pub use arkit_video::{
    VideoBuffering, VideoControlLabels, VideoController, VideoControls, VideoControlsStyle,
    VideoError, VideoErrorKind, VideoFileSource, VideoMetadata, VideoNetworkSource, VideoPlayer,
    VideoPlayerProps, VideoProgress, VideoResizeMode, VideoResult, VideoSeekMode, VideoSize,
    VideoSnapshot, VideoSource, VideoStatus, VideoSubtitleCue, VideoSubtitleSource, VideoTrack,
    VideoTrackType,
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

// --- Embedded terminal (rio-vt) ---
#[cfg(feature = "terminal")]
pub use arkit_terminal as terminal;
#[cfg(feature = "terminal")]
pub use arkit_terminal::{
    rgb_to_argb, CursorVisualStyle, KeyChord, KeyMods, MouseAction, MouseButton, MouseInput, Rgb,
    Terminal, TerminalCell, TerminalConfig, TerminalController, TerminalCursor, TerminalEffects,
    TerminalEngine, TerminalError, TerminalErrorKind, TerminalFrame, TerminalInbox, TerminalProps,
    TerminalResult, TerminalRun, TerminalScrollbar, TerminalSize,
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
/// The public facade installs root-specific runtime/window contexts. Exact
/// native observation and root portals remain local to their declaring
/// components. Business content fills the surface edge-to-edge; safe-area
/// avoidance is opt-in through [`use_safe_area`] (or `SafeArea`).
pub fn mount_entry(
    slot: ohos_arkui_binding::common::handle::ArkUIHandle,
    app: openharmony_ability::OpenHarmonyApp,
    root: fn() -> Element,
) -> napi_ohos::Result<ArkRuntime> {
    mount_entry_with_policy(slot, app, root, SafeAreaPolicy::EdgeToEdge)
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
    arkit_hooks::use_runtime_context_providers();
    #[cfg(feature = "animation")]
    let root_ref = arkit_animation::use_animation_host_provider();
    #[cfg(not(feature = "animation"))]
    let root_ref = arkit_hooks::use_native_element_ref();
    use_root_content_rect(root_ref.clone());
    // Safe-area avoidance is opt-in: by default business content fills the
    // surface edge-to-edge and integrators apply insets where they need them
    // through `use_safe_area` (or the `SafeArea` component). Only explicit
    // `SafeAreaPolicy::Safe` mount points keep the framework-owned padding.
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
            native_ref: root_ref,
            width: "100%",
            height: "100%",
            alignment: "top-start",
            clip: false,
            padding_top: safe_area.top,
            padding_right: safe_area.right,
            padding_bottom: safe_area.bottom,
            padding_left: safe_area.left,
            {content}
        }
    }
}

fn use_root_content_rect(reference: NativeElementRef) {
    let metrics = dioxus_core::try_consume_context::<WindowMetricsHandle>();
    arkit_hooks::use_layout_frame(reference, move |frame| {
        let Some(metrics) = metrics.as_ref() else {
            return;
        };
        if !frame.is_measured()
            || !frame.x.is_finite()
            || !frame.y.is_finite()
            || !frame.width.is_finite()
            || !frame.height.is_finite()
        {
            return;
        }
        metrics.report_content_rect(PhysicalRect {
            left: frame.x.round() as i32,
            top: frame.y.round() as i32,
            width: frame.width.round() as i32,
            height: frame.height.round() as i32,
        });
    });
}

pub mod prelude {
    //! Everything an app needs in one glob.
    //!
    //! `use arkit::prelude::*` brings in `rsx!`, signals/hooks, all ArkUI
    //! element/event descriptors, the entry macro, and escape-hatch hooks.
    //! Domain APIs appear only when their facade feature is enabled.

    // Dioxus primitives, hooks, signals, and ArkUI element descriptors.
    pub use arkit_prelude::*;

    // Entry + root-local runtime.
    pub use crate::{
        entry, mount_entry, mount_entry_with_policy, use_runtime_handle, ApplicationLifecycleEvent,
        ApplicationLifecycleHandle, ApplicationLifecyclePhase, ApplicationLifecycleState,
        ApplicationLifecycleSubscription, ArkRuntime, EdgeInsets, PhysicalRect, RuntimeHandle,
        RuntimeId, SafeAreaPolicy, VirtualDom, WindowMetrics, WindowMetricsHandle,
        WindowMetricsSubscription,
    };

    // Pluginized WebView capability (feature `webview`).
    #[cfg(feature = "webview")]
    pub use crate::{
        NodeExt, NodeSurface, WebviewCallbacksBuilder, WebviewClient, WebviewCreateRequest,
        WebviewHandle, WebviewJavascriptProxyBuilder, WebviewProtocol, WebviewProtocolOptions,
        WebviewProtocolRequest, WebviewProtocolResponse, WebviewStyle,
    };

    // Renderer-owned safe handles.
    pub use crate::{
        ArkImageSource, LayoutFramePx, MountedNodeLease, NativeElementEvent, NativeElementRef,
        NativeElementSubscription, NativeVisibility, VirtualKind, VirtualSource,
    };

    // Exact-element, portal, and virtual-source hooks.
    pub use crate::{
        use_app_foreground, use_application_lifecycle, use_application_lifecycle_event,
        use_component_lifecycle, use_component_visibility, use_layout_frame, use_layout_size,
        use_load_more, use_mounted_node, use_native_element_ref, use_overlay_viewport,
        use_safe_area, use_safe_area_policy, use_virtual_range, use_virtual_source,
        use_virtual_source_items_keyed, use_window_metrics, ComponentLifecycleState, LayoutFrame,
        LayoutSize, LoadMoreController, LoadMoreState, ModalPortal, ModalPresentation,
        OverlayLayer, OverlayViewport, Portal, SafeArea, SafeAreaEdges, SafeAreaProps,
        VirtualSourceItem, VirtualVisibleRange,
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

    #[cfg(feature = "video")]
    pub use crate::{
        video, VideoBuffering, VideoControlLabels, VideoController, VideoControls,
        VideoControlsStyle, VideoError, VideoErrorKind, VideoFileSource, VideoMetadata,
        VideoNetworkSource, VideoPlayer, VideoPlayerProps, VideoProgress, VideoResizeMode,
        VideoResult, VideoSeekMode, VideoSize, VideoSnapshot, VideoSource, VideoStatus,
        VideoSubtitleCue, VideoSubtitleSource, VideoTrack, VideoTrackType,
    };

    #[cfg(feature = "router")]
    pub use crate::{
        use_back_handler, AnimatedOutlet, Link, LinkProps, Routable, RouteProvider,
        RouteProviderProps, RouteTransition, Router, RouterProps,
    };

    #[cfg(feature = "animation")]
    pub use crate::{
        stagger, use_animatable, use_animatable_with_defaults, use_animate_presence, use_animation,
        use_animation_layout, use_animation_scope, use_animation_snapshot, use_animation_target,
        use_draggable, use_layout_snapshot, use_presence_visibility, use_scoped_animation,
        use_scroll_observer, Angle, Animatable, AnimatableDefaults, AnimatableValue,
        AnimatePresence, Animation, AnimationAdapterError, AnimationBackend, AnimationBuildError,
        AnimationControls, AnimationFinished, AnimationHostError, AnimationInstanceSnapshot,
        AnimationOutcome, AnimationPerformanceCounters, AnimationScope, AnimationScopeDefaults,
        AnimationSelector, AnimationSubscription, AnimationTarget, AnimationValue,
        AnimationWindowMetrics, AutoScroll, BackendRejection, BuiltinEase, CallPolicy,
        CapabilityRequirements, Composition, DiscreteValue, DragAxis, DragConstraints, DragMapping,
        DragPhase, DragSnap, DragUpdate, Draggable, DraggableCallbacks, DraggableConfig,
        DraggableHandle, EaseDirection, Easing, EasingError, ExecutionPolicy, ExitCancelPolicy,
        InvalidationClass, IrregularEase, IterationCount, JumpMode, LabelName, LayoutAnimation,
        LayoutAnimationMode, LayoutChangeKind, LayoutDelta, LayoutEngine, LayoutId,
        LayoutMountState, LayoutNode, LayoutNodeId, LayoutSnapshot, Length, LengthUnit,
        LinearPoint, LinearRgba, LoweringReport, Modifier, MountTransition, NativeCapability,
        NativeLoweringError, PlaybackDirection, PlaybackRate, PlaybackSettings, PlaybackState,
        PresenceEntry, PresenceHandle, PresenceKey, PresenceMode, PresencePhase,
        PresenceTransition, PresenceVisibility, Property, PropertyKeyframe, PropertyName,
        ScopeCleanupPolicy, ScopeMethodName, ScrollAxis, ScrollCallbacks, ScrollDirection,
        ScrollObserver, ScrollRange, ScrollSample, ScrollSync, ScrollThreshold, ShadowValue,
        SharedElementProjection, SpringSpec, Stagger, StaggerAxis, StaggerDirection, StaggerFrom,
        StaggerGrid, TargetName, TimeError, TimeOffset, TimePoint, TimeSpan, Timeline,
        TimelinePosition, TransformValue, TransitionPreset, UnsupportedFeature, ValueError,
        ValueKind, Vec2, Vec3, VelocityTracker, VisibleTransition, WindowCondition, ASPECT_RATIO,
        BACKGROUND_COLOR, BLUR, BORDER_COLOR, BORDER_RADIUS, BORDER_WIDTH, BRIGHTNESS, CONTRAST,
        FONT_COLOR, FONT_SIZE, FOREGROUND_COLOR, GRAYSCALE, HEIGHT, INVERT, LETTER_SPACING,
        LINE_HEIGHT, OPACITY, POSITION_X, POSITION_Y, ROTATION, SATURATION, SCALE_X, SCALE_Y,
        SEPIA, TRANSLATE_X, TRANSLATE_Y, WIDTH,
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

    #[cfg(feature = "barcode")]
    pub use crate::{
        encode_barcode, use_barcode, Barcode, BarcodeArtifact, BarcodeBitmap, BarcodeError,
        BarcodeErrorKind, BarcodeFormat, BarcodeHandle, BarcodeOptions, BarcodePhase, BarcodeProps,
        BarcodeRequest, BarcodeResult, QrEcLevel,
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

    #[cfg(feature = "terminal")]
    pub use crate::{
        rgb_to_argb, CursorVisualStyle, KeyChord, KeyMods, MouseAction, MouseButton, MouseInput,
        Rgb, Terminal, TerminalCell, TerminalConfig, TerminalController, TerminalCursor,
        TerminalEffects, TerminalEngine, TerminalError, TerminalErrorKind, TerminalFrame,
        TerminalProps, TerminalResult, TerminalRun, TerminalScrollbar, TerminalSize,
    };

    #[cfg(feature = "shadcn")]
    pub use crate::shadcn;
}
