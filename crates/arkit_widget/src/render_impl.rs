use std::any::{type_name, Any, TypeId};
use std::cell::{Cell, RefCell};
#[cfg(feature = "webview")]
use std::collections::HashMap;
use std::mem::{align_of, size_of, ManuallyDrop};
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(feature = "webview")]
use std::path::PathBuf;
use std::rc::Rc;
#[cfg(feature = "webview")]
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{Alignment, LayoutFrame, LayoutSize};
use arkit_core::{advanced, Horizontal, Length, Padding, Size, Vertical};
#[cfg(feature = "webview")]
use napi_ohos::{
    bindgen_prelude::{FnArgs, Function, JsObjectValue, Object, ObjectRef},
    Either,
};
use ohos_arkui_binding::api::attribute_option::ProgressLinearStyleOption;
use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent, ArkUIGesture,
};
use ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, DatePicker, Image, List, ListItem, Progress, Radio,
    Refresh, Row, Scroll, Slider, Stack, Swiper, Text, TextArea, TextInput, Toggle,
};
use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use ohos_arkui_binding::gesture::gesture_data::GestureEventData;
use ohos_arkui_binding::gesture::inner_gesture::Gesture;
use ohos_arkui_binding::types::advanced::{
    FontWeight, HorizontalAlignment, NodeCustomEventType, ShadowStyle, VerticalAlignment,
};
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::types::event::NodeEventType;
use ohos_arkui_binding::types::gesture_event::GestureEventAction;
use ohos_arkui_binding::types::text_alignment::TextAlignment;
#[cfg(feature = "webview")]
use openharmony_ability::{get_helper, get_main_thread_env, WebViewInitData};
#[cfg(feature = "webview")]
pub use openharmony_ability::{DownloadStartResult, WebViewStyle, Webview};

#[cfg(feature = "webview")]
fn run_with_webview_dispatcher<R>(
    dispatcher: Option<arkit_runtime::internal::GlobalRuntimeDispatcher>,
    f: impl FnOnce() -> R,
) -> R {
    arkit_runtime::internal::with_global_dispatcher(dispatcher, f)
}

pub use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem as AttributeValue;
pub use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType as Attribute;

pub type Element<Message = (), AppTheme = arkit_core::Theme> =
    arkit_core::Element<'static, Message, AppTheme, Renderer>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderer;

type Cleanup = Box<dyn FnOnce()>;
type MountEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
type AttachEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
type PatchEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static>;
type ExitEffect =
    Box<dyn FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<Option<Cleanup>> + 'static>;
type EventCallback = Rc<dyn Fn(&ArkEvent)>;
type UiStateCallback = Rc<dyn Fn(&mut ArkUINode, UiState)>;

const DEFAULT_LONG_PRESS_DURATION_MS: i32 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl From<FontStyle> for i32 {
    fn from(value: FontStyle) -> Self {
        match value {
            FontStyle::Normal => 0,
            FontStyle::Italic => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

impl From<BorderStyle> for i32 {
    fn from(value: BorderStyle) -> Self {
        match value {
            BorderStyle::Solid => 0,
            BorderStyle::Dashed => 1,
            BorderStyle::Dotted => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressType {
    Linear,
    Ring,
    Eclipse,
    ScaleRing,
    Capsule,
}

impl From<ProgressType> for i32 {
    fn from(value: ProgressType) -> Self {
        match value {
            ProgressType::Linear => 0,
            ProgressType::Ring => 1,
            ProgressType::Eclipse => 2,
            ProgressType::ScaleRing => 3,
            ProgressType::Capsule => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollViewport {
    pub offset: ScrollOffset,
    pub viewport_size: Size<f32>,
    pub content_size: Size<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListScrollIndexEvent {
    pub first_index: i32,
    pub last_index: i32,
    pub center_index: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListVisibleContentChangeEvent {
    pub first_index: i32,
    pub start_area: i32,
    pub start_item_index: i32,
    pub last_index: i32,
    pub end_area: i32,
    pub end_item_index: i32,
}

#[derive(Default)]
struct ScrollState {
    offset: ScrollOffset,
    viewport: Option<ScrollViewport>,
    node: Option<ArkUINode>,
}

#[cfg(feature = "webview")]
static NEXT_WEBVIEW_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "webview")]
type WebViewReadyCallback = Rc<dyn Fn(&Webview)>;
#[cfg(feature = "webview")]
type WebViewLifecycleCallback = Rc<RefCell<Box<dyn FnMut()>>>;
#[cfg(feature = "webview")]
type DragAndDropCallback = Rc<dyn Fn(String)>;
#[cfg(feature = "webview")]
type DownloadStartCallback = Rc<dyn Fn(String, &mut PathBuf) -> bool>;
#[cfg(feature = "webview")]
type DownloadEndCallback = Rc<dyn Fn(String, Option<PathBuf>, bool)>;
#[cfg(feature = "webview")]
type NavigationRequestCallback = Rc<dyn Fn(String) -> bool>;
#[cfg(feature = "webview")]
type TitleChangeCallback = Rc<dyn Fn(String)>;

#[cfg(feature = "webview")]
#[derive(Debug, Clone, Default)]
struct WebViewAppliedState {
    url: Option<String>,
    html: Option<String>,
    headers: Option<HashMap<String, String>>,
    style: Option<WebViewStyle>,
}

#[cfg(feature = "webview")]
struct WebViewControllerState {
    id: String,
    webview: RefCell<Option<Webview>>,
    embedded_node: RefCell<Option<ArkUINode>>,
    ready_callbacks: RefCell<Vec<WebViewReadyCallback>>,
    controller_attach_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    page_begin_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    page_end_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    destroy_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    applied: RefCell<Option<WebViewAppliedState>>,
    active_binding: Cell<bool>,
    attached: Cell<bool>,
}

#[cfg(feature = "webview")]
impl WebViewControllerState {
    fn new(id: String) -> Self {
        Self {
            id,
            webview: RefCell::new(None),
            embedded_node: RefCell::new(None),
            ready_callbacks: RefCell::new(Vec::new()),
            controller_attach_callbacks: RefCell::new(Vec::new()),
            page_begin_callbacks: RefCell::new(Vec::new()),
            page_end_callbacks: RefCell::new(Vec::new()),
            destroy_callbacks: RefCell::new(Vec::new()),
            applied: RefCell::new(None),
            active_binding: Cell::new(false),
            attached: Cell::new(false),
        }
    }
}

#[cfg(feature = "webview")]
#[derive(Clone)]
pub struct WebViewController {
    inner: Rc<WebViewControllerState>,
}

#[cfg(feature = "webview")]
impl Default for WebViewController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "webview")]
impl WebViewController {
    pub fn new() -> Self {
        let id = format!(
            "arkit-webview-{}",
            NEXT_WEBVIEW_ID.fetch_add(1, Ordering::Relaxed)
        );
        Self::with_id(id)
    }

    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(WebViewControllerState::new(id.into())),
        }
    }

    pub fn id(&self) -> &str {
        self.inner.id.as_str()
    }

    pub fn handle(&self) -> Result<Webview, String> {
        self.inner
            .webview
            .borrow()
            .clone()
            .ok_or_else(|| format!("webview '{}' is not mounted", self.id()))
    }

    pub fn on_ready(&self, callback: impl Fn(&Webview) + 'static) {
        let callback = Rc::new(callback) as WebViewReadyCallback;
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            callback(webview);
        }
        self.inner.ready_callbacks.borrow_mut().push(callback);
    }

    pub fn on_controller_attach(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::ControllerAttach)?;
        }
        self.inner
            .controller_attach_callbacks
            .borrow_mut()
            .push(callback);
        Ok(())
    }

    pub fn on_page_begin(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::PageBegin)?;
        }
        self.inner.page_begin_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn on_page_end(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::PageEnd)?;
        }
        self.inner.page_end_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn on_destroy(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::Destroy)?;
        }
        self.inner.destroy_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn url(&self) -> Result<String, String> {
        self.with_handle(|webview| webview.url())
    }

    pub fn cookies_with_url(&self, url: &str) -> Result<String, String> {
        self.with_handle(|webview| webview.cookies_with_url(url))
    }

    pub fn load_url(&self, url: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.load_url(url))
    }

    pub fn load_url_with_headers(
        &self,
        url: &str,
        headers: impl IntoIterator<Item = (String, String)>,
    ) -> Result<(), String> {
        let mut header_map = http::HeaderMap::new();
        for (key, value) in headers {
            let name = key
                .parse::<http::header::HeaderName>()
                .map_err(|error| error.to_string())?;
            let value = value
                .parse::<http::header::HeaderValue>()
                .map_err(|error| error.to_string())?;
            header_map.insert(name, value);
        }
        self.with_handle(|webview| webview.load_url_with_headers(url, header_map))
    }

    pub fn load_html(&self, html: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.load_html(html))
    }

    pub fn reload(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.reload())
    }

    pub fn focus(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.focus())
    }

    pub fn set_zoom(&self, zoom: f64) -> Result<(), String> {
        self.with_handle(|webview| webview.set_zoom(zoom))
    }

    pub fn evaluate_script(&self, js: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.evaluate_script(js))
    }

    pub fn evaluate_script_with_callback(
        &self,
        js: &str,
        callback: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<(), String> {
        let webview = self.handle()?;
        let env_state = get_main_thread_env();
        let env_borrow = env_state.borrow();
        let Some(env) = env_borrow.as_ref() else {
            return Err(String::from("failed to get main thread env"));
        };
        let run_javascript = webview
            .inner()
            .get_value(env)
            .map_err(|error| error.to_string())?
            .get_named_property::<Function<'_, FnArgs<(String, Function<'_, String, ()>)>, ()>>(
                "runJavaScript",
            )
            .map_err(|error| error.to_string())?;
        let callback_dispatcher = arkit_runtime::internal::global_dispatcher();

        let cb = env
            .create_function_from_closure("arkit_evaluate_js_callback", move |ctx| {
                let ret = ctx.try_get::<String>(0)?;
                let ret = match ret {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::from("undefined"),
                };
                if let Some(callback) = callback.as_ref() {
                    run_with_webview_dispatcher(callback_dispatcher.clone(), move || callback(ret));
                }
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        run_javascript
            .call((js.to_string(), cb).into())
            .map_err(|error| error.to_string())
    }

    pub fn clear_all_browsing_data(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.clear_all_browsing_data())
    }

    pub fn set_background_color(&self, color: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.set_background_color(color))
    }

    pub fn set_visible(&self, visible: bool) -> Result<(), String> {
        self.with_handle(|webview| webview.set_visible(visible))
    }

    fn with_handle<R, E: ToString>(
        &self,
        f: impl FnOnce(&Webview) -> Result<R, E>,
    ) -> Result<R, String> {
        let webview = self.handle()?;
        f(&webview).map_err(|error| error.to_string())
    }
}

#[cfg(feature = "webview")]
#[derive(Clone, Default)]
struct WebViewSpec {
    controller: Option<WebViewController>,
    url: Option<String>,
    html: Option<String>,
    style: Option<WebViewStyle>,
    javascript_enabled: Option<bool>,
    devtools: Option<bool>,
    transparent: Option<bool>,
    autoplay: Option<bool>,
    user_agent: Option<String>,
    initialization_scripts: Option<Vec<String>>,
    headers: Option<HashMap<String, String>>,
    on_drag_and_drop: Option<DragAndDropCallback>,
    on_download_start: Option<DownloadStartCallback>,
    on_download_end: Option<DownloadEndCallback>,
    on_navigation_request: Option<NavigationRequestCallback>,
    on_title_change: Option<TitleChangeCallback>,
}

#[cfg(feature = "webview")]
#[derive(Clone, Copy)]
enum WebViewLifecycle {
    ControllerAttach,
    PageBegin,
    PageEnd,
    Destroy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressLinearStyle {
    pub stroke_width: f32,
    pub stroke_radius: f32,
    pub scan_effect_enabled: bool,
    pub smooth_effect_enabled: bool,
}

impl ProgressLinearStyle {
    pub fn new(stroke_width: f32, stroke_radius: f32) -> Self {
        Self {
            stroke_width,
            stroke_radius,
            scan_effect_enabled: false,
            smooth_effect_enabled: true,
        }
    }

    pub fn scan_effect_enabled(mut self, value: bool) -> Self {
        self.scan_effect_enabled = value;
        self
    }

    pub fn smooth_effect_enabled(mut self, value: bool) -> Self {
        self.smooth_effect_enabled = value;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAlignment {
    Auto,
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

impl From<ItemAlignment> for i32 {
    fn from(value: ItemAlignment) -> Self {
        match value {
            ItemAlignment::Auto => 0,
            ItemAlignment::Start => 1,
            ItemAlignment::Center => 2,
            ItemAlignment::End => 3,
            ItemAlignment::Stretch => 4,
            ItemAlignment::Baseline => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}

impl From<Visibility> for i32 {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Visible => 0,
            Visibility::Hidden => 1,
            Visibility::None => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestBehavior {
    Default,
    Block,
    Transparent,
    None,
    BlockHierarchy,
    BlockDescendants,
}

impl From<HitTestBehavior> for i32 {
    fn from(value: HitTestBehavior) -> Self {
        match value {
            HitTestBehavior::Default => 0,
            HitTestBehavior::Block => 1,
            HitTestBehavior::Transparent => 2,
            HitTestBehavior::None => 3,
            HitTestBehavior::BlockHierarchy => 4,
            HitTestBehavior::BlockDescendants => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    Normal,
    Capsule,
    Circle,
    RoundedRectangle,
}

impl From<ButtonType> for i32 {
    fn from(value: ButtonType) -> Self {
        match value {
            ButtonType::Normal => 0,
            ButtonType::Capsule => 1,
            ButtonType::Circle => 2,
            ButtonType::RoundedRectangle => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState(i32);

impl UiState {
    pub const NORMAL: Self = Self(0);
    pub const PRESSED: Self = Self(1);
    pub const FOCUSED: Self = Self(2);
    pub const DISABLED: Self = Self(4);
    pub const SELECTED: Self = Self(8);

    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> i32 {
        self.0
    }

    pub const fn contains(self, state: Self) -> bool {
        self.0 & state.0 == state.0
    }
}

impl std::ops::BitOr for UiState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl From<UiState> for i32 {
    fn from(value: UiState) -> Self {
        value.bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFit {
    Contain,
    Cover,
    Auto,
    Fill,
    ScaleDown,
    None,
    NoneAndAlignTopStart,
    NoneAndAlignTop,
    NoneAndAlignTopEnd,
    NoneAndAlignStart,
    NoneAndAlignCenter,
    NoneAndAlignEnd,
    NoneAndAlignBottomStart,
    NoneAndAlignBottom,
    NoneAndAlignBottomEnd,
    NoneMatrix,
}

impl From<ObjectFit> for i32 {
    fn from(value: ObjectFit) -> Self {
        match value {
            ObjectFit::Contain => 0,
            ObjectFit::Cover => 1,
            ObjectFit::Auto => 2,
            ObjectFit::Fill => 3,
            ObjectFit::ScaleDown => 4,
            ObjectFit::None => 5,
            ObjectFit::NoneAndAlignTopStart => 6,
            ObjectFit::NoneAndAlignTop => 7,
            ObjectFit::NoneAndAlignTopEnd => 8,
            ObjectFit::NoneAndAlignStart => 9,
            ObjectFit::NoneAndAlignCenter => 10,
            ObjectFit::NoneAndAlignEnd => 11,
            ObjectFit::NoneAndAlignBottomStart => 12,
            ObjectFit::NoneAndAlignBottom => 13,
            ObjectFit::NoneAndAlignBottomEnd => 14,
            ObjectFit::NoneMatrix => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<JustifyContent> for i32 {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::Start => 1,
            JustifyContent::Center => 2,
            JustifyContent::End => 3,
            JustifyContent::SpaceBetween => 6,
            JustifyContent::SpaceAround => 7,
            JustifyContent::SpaceEvenly => 8,
        }
    }
}

fn font_weight_value(value: FontWeight) -> i32 {
    match value {
        FontWeight::W100 => 0,
        FontWeight::W200 => 1,
        FontWeight::W300 => 2,
        FontWeight::W400 => 3,
        FontWeight::W500 => 4,
        FontWeight::W600 => 5,
        FontWeight::W700 => 6,
        FontWeight::W800 => 7,
        FontWeight::W900 => 8,
        FontWeight::Bold => 9,
        FontWeight::Normal => 10,
        FontWeight::Bolder => 11,
        FontWeight::Lighter => 12,
        FontWeight::Medium => 13,
        FontWeight::Regular => 14,
    }
}

fn shadow_style_value(value: ShadowStyle) -> i32 {
    match value {
        ShadowStyle::OuterDefaultXs => 0,
        ShadowStyle::OuterDefaultSm => 1,
        ShadowStyle::OuterDefaultMd => 2,
        ShadowStyle::OuterDefaultLg => 3,
        ShadowStyle::OuterFloatingSm => 4,
        ShadowStyle::OuterFloatingMd => 5,
    }
}

fn apply_progress_linear_style(
    node: &mut ArkUINode,
    style: ProgressLinearStyle,
) -> ArkUIResult<()> {
    let mut option = ProgressLinearStyleOption::new()?;
    option.set_stroke_width(style.stroke_width);
    option.set_stroke_radius(style.stroke_radius);
    option.set_scan_effect_enabled(style.scan_effect_enabled);
    option.set_smooth_effect_enabled(style.smooth_effect_enabled);

    let result = RuntimeNode(node).set_attribute(
        ArkUINodeAttributeType::ProgressLinearStyle,
        (&option).into(),
    );
    option.destroy();
    result
}

fn apply_scroll_offset(node: &mut ArkUINode, offset: ScrollOffset) -> ArkUIResult<()> {
    RuntimeNode(node).set_attribute(
        ArkUINodeAttributeType::ScrollOffset,
        vec![
            ArkUINodeAttributeNumber::Float(offset.x),
            ArkUINodeAttributeNumber::Float(offset.y),
            ArkUINodeAttributeNumber::Int(0),
        ]
        .into(),
    )
}

fn read_scroll_offset(node: &mut ArkUINode) -> Option<ScrollOffset> {
    let value = RuntimeNode(node)
        .get_attribute(ArkUINodeAttributeType::ScrollOffset)
        .ok()?;
    let ArkUINodeAttributeItem::NumberValue(values) = value else {
        return None;
    };

    Some(ScrollOffset {
        x: attribute_number_as_f32(values.first()?)?,
        y: attribute_number_as_f32(values.get(1)?)?,
    })
}

fn attribute_number_as_f32(value: &ArkUINodeAttributeNumber) -> Option<f32> {
    match value {
        ArkUINodeAttributeNumber::Float(value) => Some(*value),
        ArkUINodeAttributeNumber::Int(value) => Some(*value as f32),
        ArkUINodeAttributeNumber::Uint(value) => Some(*value as f32),
    }
}

struct RuntimeNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for RuntimeNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUICommonAttribute for RuntimeNode<'_> {}
impl ArkUIEvent for RuntimeNode<'_> {}
impl ArkUIGesture for RuntimeNode<'_> {}

#[derive(Clone)]
struct EventHandlerSpec {
    event_type: NodeEventType,
    callback: EventCallback,
}

#[derive(Clone)]
struct LongPressHandlerSpec {
    callback: Rc<dyn Fn()>,
}

struct LongPressCallbackContext {
    callback: Rc<RefCell<Rc<dyn Fn()>>>,
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::from("non-string panic payload")
    }
}

fn run_guarded_ui_callback(error_label: &'static str, callback: impl FnOnce()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(callback)) {
        ohos_hilog_binding::error(format!(
            "{error_label}: {}",
            panic_payload_message(payload.as_ref())
        ));
    }
}

fn long_press_gesture_callback(event: GestureEventData) {
    let Some(data) = event.data else {
        return;
    };
    let context = unsafe { &*(data as *const LongPressCallbackContext) };
    let callback = context.callback.borrow().clone();
    run_guarded_ui_callback(
        "gesture error: on_long_press callback panicked",
        move || (callback.as_ref())(),
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeKind {
    Button,
    CalendarPicker,
    Checkbox,
    Column,
    DatePicker,
    Image,
    List,
    ListItem,
    Progress,
    Radio,
    Refresh,
    Row,
    Scroll,
    Slider,
    Stack,
    Swiper,
    Text,
    TextArea,
    TextInput,
    Toggle,
    #[cfg(feature = "webview")]
    WebViewHost,
}

pub struct MountedNode {
    tree: advanced::widget::Tree,
    retained_state: StateCache,
    render: MountedRenderNode,
}

struct MountedRenderNode {
    tag: TypeId,
    key: Option<String>,
    attrs: Vec<ArkUINodeAttributeType>,
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
    long_press_cleanup: Option<Cleanup>,
    long_press_callback: Option<Rc<RefCell<Rc<dyn Fn()>>>>,
    cleanups: Vec<Cleanup>,
    exit_effect: Option<ExitEffect>,
    exiting_children: Rc<RefCell<Vec<PendingExit>>>,
    pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    pending_attach_effects: Vec<AttachEffect>,
    pending_patch_effects: Vec<PatchEffect>,
    children: Vec<MountedRenderNode>,
}

struct PendingExit {
    raw_handle: usize,
    alive: Rc<Cell<bool>>,
    mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    effect_cleanup: Rc<RefCell<Option<Cleanup>>>,
}

#[derive(Default)]
struct StateCache {
    entries: Vec<advanced::widget::Tree>,
}

impl StateCache {
    fn store(&mut self, mut tree: advanced::widget::Tree) {
        let Some(key) = tree.persistent_key().map(str::to_string) else {
            return;
        };
        snapshot_tree_state(&mut tree);
        let tag = tree.tag();
        self.entries
            .retain(|entry| !(entry.tag() == tag && entry.persistent_key() == Some(key.as_str())));
        self.entries.push(tree);
    }

    fn take(
        &mut self,
        tag: advanced::widget::Tag,
        persistent_key: Option<&str>,
    ) -> Option<advanced::widget::Tree> {
        let persistent_key = persistent_key?;
        let index = self
            .entries
            .iter()
            .position(|tree| tree.tag() == tag && tree.persistent_key() == Some(persistent_key))?;
        Some(self.entries.remove(index))
    }
}

fn snapshot_tree_state(tree: &mut advanced::widget::Tree) {
    if let Some(scroll_state) = tree
        .state()
        .downcast_mut::<Rc<RefCell<ScrollState>>>()
        .cloned()
    {
        let offset = scroll_state
            .borrow()
            .node
            .clone()
            .and_then(|mut node| read_scroll_offset(&mut node));
        if let Some(offset) = offset {
            scroll_state.borrow_mut().offset = offset;
        }
    }

    for child in tree.children_mut() {
        snapshot_tree_state(child);
    }
}

impl MountedNode {
    fn new(tree: advanced::widget::Tree, render: MountedRenderNode) -> Self {
        Self {
            tree,
            retained_state: StateCache::default(),
            render,
        }
    }

    fn render_mut(&mut self) -> &mut MountedRenderNode {
        &mut self.render
    }

    pub fn cleanup_recursive(self) {
        self.render.cleanup_recursive();
    }
}

impl MountedRenderNode {
    fn new(
        tag: TypeId,
        key: Option<String>,
        attrs: Vec<ArkUINodeAttributeType>,
        events: Vec<NodeEventType>,
        mount_effect_count: usize,
        attach_effect_count: usize,
        patch_effect_count: usize,
        has_long_press: bool,
        long_press_cleanup: Option<Cleanup>,
        long_press_callback: Option<Rc<RefCell<Rc<dyn Fn()>>>>,
        cleanups: Vec<Cleanup>,
        exit_effect: Option<ExitEffect>,
        pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
        pending_attach_effects: Vec<AttachEffect>,
        pending_patch_effects: Vec<PatchEffect>,
        children: Vec<MountedRenderNode>,
    ) -> Self {
        Self {
            tag,
            key,
            attrs,
            events,
            mount_effect_count,
            attach_effect_count,
            patch_effect_count,
            has_long_press,
            long_press_cleanup,
            long_press_callback,
            cleanups,
            exit_effect,
            exiting_children: Rc::new(RefCell::new(Vec::new())),
            pending_patch_attrs,
            pending_attach_effects,
            pending_patch_effects,
            children,
        }
    }

    fn cleanup_recursive(self) {
        for child in self.children {
            child.cleanup_recursive();
        }
        let pending_exits = self
            .exiting_children
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        for exit in pending_exits {
            exit.alive.set(false);
            if let Some(cleanup) = exit.effect_cleanup.borrow_mut().take() {
                cleanup();
            }
            if let Some(mounted) = exit.mounted.borrow_mut().take() {
                mounted.cleanup_recursive();
            }
        }
        if let Some(cleanup) = self.long_press_cleanup {
            cleanup();
        }
        run_cleanups(self.cleanups);
    }
}

fn padding_edges(value: Padding) -> Vec<f32> {
    vec![value.top, value.right, value.bottom, value.left]
}

pub trait EdgeAttributeValue {
    fn edge_values(self) -> Vec<f32>;
}

impl EdgeAttributeValue for f32 {
    fn edge_values(self) -> Vec<f32> {
        vec![self, self, self, self]
    }
}

impl EdgeAttributeValue for [f32; 2] {
    fn edge_values(self) -> Vec<f32> {
        padding_edges(Padding::from(self))
    }
}

impl EdgeAttributeValue for [f32; 4] {
    fn edge_values(self) -> Vec<f32> {
        self.to_vec()
    }
}

impl EdgeAttributeValue for Padding {
    fn edge_values(self) -> Vec<f32> {
        padding_edges(self)
    }
}

fn clone_attr_value(value: &ArkUINodeAttributeItem) -> ArkUINodeAttributeItem {
    match value {
        ArkUINodeAttributeItem::NumberValue(values) => {
            ArkUINodeAttributeItem::NumberValue(values.clone())
        }
        ArkUINodeAttributeItem::String(value) => ArkUINodeAttributeItem::String(value.clone()),
        ArkUINodeAttributeItem::Object(value) => ArkUINodeAttributeItem::Object(*value),
        ArkUINodeAttributeItem::Composite(value) => {
            ArkUINodeAttributeItem::Composite(value.clone())
        }
    }
}

pub struct Node<Message, AppTheme = arkit_core::Theme> {
    kind: NodeKind,
    key: Option<String>,
    persistent_key: Option<String>,
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    event_handlers: Vec<EventHandlerSpec>,
    long_press_handler: Option<LongPressHandlerSpec>,
    mount_effects: Vec<MountEffect>,
    attach_effects: Vec<AttachEffect>,
    patch_effects: Vec<PatchEffect>,
    exit_effect: Option<ExitEffect>,
    state_bound: bool,
    #[cfg(feature = "webview")]
    webview: Option<WebViewSpec>,
    children: Vec<Element<Message, AppTheme>>,
}

impl<Message, AppTheme> Node<Message, AppTheme> {
    fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            key: None,
            persistent_key: None,
            init_attrs: Vec::new(),
            patch_attrs: Vec::new(),
            event_handlers: Vec::new(),
            long_press_handler: None,
            mount_effects: Vec::new(),
            attach_effects: Vec::new(),
            patch_effects: Vec::new(),
            exit_effect: None,
            state_bound: false,
            #[cfg(feature = "webview")]
            webview: None,
            children: Vec::new(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn persistent_state_key(mut self, key: impl Into<String>) -> Self {
        self.persistent_key = Some(key.into());
        self
    }

    pub fn child(mut self, child: impl Into<Element<Message, AppTheme>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Element<Message, AppTheme>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn map_descendants(self, mut map: impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        self.map_descendants_with(&mut map)
    }

    fn map_descendants_with(self, map: &mut impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        let Self {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            #[cfg(feature = "webview")]
            webview,
            children,
        } = self;

        let children = children
            .into_iter()
            .map(|child| into_node(child).map_descendants_with(map).into())
            .collect();

        map(Self {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            #[cfg(feature = "webview")]
            webview,
            children,
        })
    }

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.init_attrs.push((attr, value.into()));
        self
    }

    pub fn patch_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.patch_attrs.push((attr, value.into()));
        self
    }

    fn builder_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        let value = value.into();
        self.init_attrs.push((attr, clone_attr_value(&value)));
        self.patch_attrs.push((attr, value));
        self
    }

    pub fn attr_string(&self, attr: ArkUINodeAttributeType) -> Option<&str> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::String(value) => Some(value.as_str()),
            ArkUINodeAttributeItem::NumberValue(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    pub fn attr_f32(&self, attr: ArkUINodeAttributeType) -> Option<f32> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::NumberValue(values) => {
                values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value,
                    ArkUINodeAttributeNumber::Int(value) => *value as f32,
                    ArkUINodeAttributeNumber::Uint(value) => *value as f32,
                })
            }
            ArkUINodeAttributeItem::String(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    fn attr_bool(&self, attr: ArkUINodeAttributeType) -> Option<bool> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::NumberValue(values) => {
                values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value != 0.0,
                    ArkUINodeAttributeNumber::Int(value) => *value != 0,
                    ArkUINodeAttributeNumber::Uint(value) => *value != 0,
                })
            }
            ArkUINodeAttributeItem::String(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    fn attr_value(&self, attr: ArkUINodeAttributeType) -> Option<&ArkUINodeAttributeItem> {
        self.patch_attrs
            .iter()
            .rev()
            .chain(self.init_attrs.iter().rev())
            .find_map(|(current_attr, value)| (*current_attr == attr).then_some(value))
    }

    #[cfg(feature = "webview")]
    fn webview_spec_mut(&mut self) -> Option<&mut WebViewSpec> {
        self.webview.as_mut()
    }

    #[cfg(feature = "webview")]
    fn map_webview(mut self, update: impl FnOnce(&mut WebViewSpec)) -> Self {
        if let Some(spec) = self.webview_spec_mut() {
            update(spec);
        }
        self
    }

    pub fn width(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::WidthPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Width, value);
            }
        }
        self
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::HeightPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Height, value);
            }
        }
        self
    }

    pub fn percent_width(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::WidthPercent, value)
    }

    pub fn percent_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HeightPercent, value)
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::ConstraintSize,
            vec![0.0_f32, value, 0.0_f32, 100_000.0_f32],
        )
    }

    pub fn constraint_size(
        self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        let value = vec![min_width, max_width, min_height, max_height];
        self.builder_attr(ArkUINodeAttributeType::ConstraintSize, value)
    }

    pub fn background_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BackgroundColor, value)
    }

    pub fn padding(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Padding, padding_edges(value.into()))
    }

    pub fn padding_x(self, value: f32) -> Self {
        self.padding(Padding::symmetric(value, 0.0))
    }

    pub fn padding_y(self, value: f32) -> Self {
        self.padding(Padding::symmetric(0.0, value))
    }

    pub fn margin(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Margin, padding_edges(value.into()))
    }

    pub fn margin_x(self, value: f32) -> Self {
        self.margin(Padding::symmetric(value, 0.0))
    }

    pub fn margin_y(self, value: f32) -> Self {
        self.margin(Padding::symmetric(0.0, value))
    }

    pub fn margin_top(self, value: f32) -> Self {
        self.margin(Padding {
            top: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_right(self, value: f32) -> Self {
        self.margin(Padding {
            right: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_bottom(self, value: f32) -> Self {
        self.margin(Padding {
            bottom: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_left(self, value: f32) -> Self {
        self.margin(Padding {
            left: value,
            ..Padding::ZERO
        })
    }

    pub fn foreground_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ForegroundColor, value)
    }

    pub fn font_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontColor, value)
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontWeight, font_weight_value(value))
    }

    pub fn font_family(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.builder_attr(ArkUINodeAttributeType::FontFamily, value)
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontStyle, i32::from(value))
    }

    pub fn font_size(self, value: f32) -> Self {
        let node = self.builder_attr(ArkUINodeAttributeType::FontSize, value);
        let placeholder_attr = match node.kind {
            NodeKind::TextInput => Some(ArkUINodeAttributeType::TextInputPlaceholderFont),
            NodeKind::TextArea => Some(ArkUINodeAttributeType::TextAreaPlaceholderFont),
            _ => None,
        };

        if let Some(attr) = placeholder_attr {
            node.builder_attr(
                attr,
                ArkUINodeAttributeItem::NumberValue(vec![ArkUINodeAttributeNumber::Float(value)]),
            )
        } else {
            node
        }
    }

    pub fn line_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLineHeight, value)
    }

    pub fn text_align(self, value: TextAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextAlign, i32::from(value))
    }

    pub fn text_letter_spacing(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLetterSpacing, value)
    }

    pub fn text_decoration(self, value: impl Into<ArkUINodeAttributeItem>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextDecoration, value)
    }

    pub fn enabled(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Enabled, value)
    }

    pub fn opacity(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Opacity, value)
    }

    pub fn clip(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Clip, value)
    }

    pub fn focusable(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Focusable, value)
    }

    pub fn focus_on_touch(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FocusOnTouch, value)
    }

    pub fn border_radius(self, value: impl EdgeAttributeValue) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderRadius, value.edge_values())
    }

    pub fn border_width(self, value: impl EdgeAttributeValue) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderWidth, value.edge_values())
    }

    pub fn border_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderColor, vec![value])
    }

    pub fn border_color_all(self, value: u32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::BorderColor,
            vec![value, value, value, value],
        )
    }

    pub fn border_style(self, value: BorderStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderStyle, i32::from(value))
    }

    pub fn shadow(self, value: ShadowStyle) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::Shadow,
            vec![shadow_style_value(value)],
        )
    }

    pub fn custom_shadow(
        self,
        blur_radius: f32,
        offset_x: f32,
        offset_y: f32,
        color: u32,
        fill: bool,
    ) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::CustomShadow,
            vec![
                ArkUINodeAttributeNumber::Float(blur_radius),
                ArkUINodeAttributeNumber::Int(0),
                ArkUINodeAttributeNumber::Float(offset_x),
                ArkUINodeAttributeNumber::Float(offset_y),
                ArkUINodeAttributeNumber::Int(0),
                ArkUINodeAttributeNumber::Uint(color),
                ArkUINodeAttributeNumber::Uint(u32::from(fill)),
            ],
        )
    }

    pub fn clear_shadow(self) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Shadow, vec![0_i32])
    }

    pub fn alignment(self, value: Alignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Alignment, i32::from(value))
    }

    pub fn align_self(self, value: ItemAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AlignSelf, i32::from(value))
    }

    pub fn layout_weight(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::LayoutWeight, value)
    }

    pub fn visibility(self, value: Visibility) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Visibility, i32::from(value))
    }

    pub fn hit_test_behavior(self, value: HitTestBehavior) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HitTestBehavior, i32::from(value))
    }

    pub fn button_type(self, value: ButtonType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ButtonType, i32::from(value))
    }

    pub fn color_blend(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ColorBlend, value)
    }

    pub fn position(self, x: f32, y: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Position, vec![x, y])
    }

    pub fn z_index(self, value: i32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ZIndex, value)
    }

    pub fn aspect_ratio(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AspectRatio, value)
    }

    pub fn image_object_fit(self, value: ObjectFit) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ImageObjectFit, i32::from(value))
    }

    pub fn progress_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressColor, value)
    }

    pub fn progress_type(self, value: ProgressType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressType, i32::from(value))
    }

    pub fn progress_linear_style(mut self, value: ProgressLinearStyle) -> Self {
        self.patch_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)
        }));
        self.mount_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)?;
            Ok(None)
        }));
        self
    }

    pub fn refreshing(self, value: bool) -> Self {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::RefreshRefreshing, value)
    }

    pub fn refresh_offset(self, value: f32) -> Self {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::RefreshOffset, value)
    }

    pub fn refresh_pull_to_refresh(self, value: bool) -> Self {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::RefreshPullToRefresh, value)
    }

    pub fn on_scroll_offset(self, callback: impl Fn(ScrollOffset) + 'static) -> Self {
        if self.kind != NodeKind::Scroll {
            return self;
        }

        self.on_event(NodeEventType::ScrollEventOnScroll, move |event| {
            callback(ScrollOffset {
                x: event.f32_value(0).unwrap_or_default(),
                y: event.f32_value(1).unwrap_or_default(),
            });
        })
    }

    pub fn toggle_selected_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleSelectedColor, value)
    }

    pub fn toggle_unselected_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleUnselectedColor, value)
    }

    pub fn toggle_switch_point_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleSwitchPointColor, value)
    }

    pub fn justify_content(self, value: JustifyContent) -> Self {
        let value = i32::from(value);
        match self.kind {
            NodeKind::Column => self
                .attr(ArkUINodeAttributeType::ColumnJustifyContent, value)
                .patch_attr(ArkUINodeAttributeType::ColumnJustifyContent, value),
            NodeKind::Row => self
                .attr(ArkUINodeAttributeType::RowJustifyContent, value)
                .patch_attr(ArkUINodeAttributeType::RowJustifyContent, value),
            _ => self,
        }
    }

    pub fn justify_content_start(self) -> Self {
        self.justify_content(JustifyContent::Start)
    }

    pub fn justify_content_center(self) -> Self {
        self.justify_content(JustifyContent::Center)
    }

    pub fn justify_content_end(self) -> Self {
        self.justify_content(JustifyContent::End)
    }

    pub fn align_x(self, alignment: Horizontal) -> Self {
        match self.kind {
            NodeKind::Column => match alignment {
                Horizontal::Left => self.align_items_start(),
                Horizontal::Center => self.align_items_center(),
                Horizontal::Right => self.align_items_end(),
            },
            _ => self,
        }
    }

    pub fn align_y(self, alignment: Vertical) -> Self {
        match self.kind {
            NodeKind::Row => match alignment {
                Vertical::Top => self.align_items_top(),
                Vertical::Center => self.align_items_center(),
                Vertical::Bottom => self.align_items_bottom(),
            },
            NodeKind::Column => {
                let justify = match alignment {
                    Vertical::Top => JustifyContent::Start,
                    Vertical::Center => JustifyContent::Center,
                    Vertical::Bottom => JustifyContent::End,
                };
                self.justify_content(justify)
            }
            _ => self,
        }
    }

    pub fn align_items_start(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Start as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Start as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_center(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                ),
            NodeKind::Row => self
                .attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Center as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Center as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_end(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::End as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::End as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_top(self) -> Self {
        match self.kind {
            NodeKind::Row => self
                .attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Top as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Top as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_bottom(self) -> Self {
        match self.kind {
            NodeKind::Row => self
                .attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Bottom as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Bottom as i32,
                ),
            _ => self,
        }
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.attr(ArkUINodeAttributeType::ButtonLabel, label.clone())
            .patch_attr(ArkUINodeAttributeType::ButtonLabel, label)
    }

    pub fn content(self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.attr(ArkUINodeAttributeType::TextContent, content.clone())
            .patch_attr(ArkUINodeAttributeType::TextContent, content)
    }

    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputText, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaText, value),
            _ => self,
        }
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholder, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholder, value),
            _ => self,
        }
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value),
            _ => self,
        }
    }

    pub fn checked(self, value: bool) -> Self {
        match self.kind {
            NodeKind::Checkbox => self
                .attr(ArkUINodeAttributeType::CheckboxSelect, value)
                .patch_attr(ArkUINodeAttributeType::CheckboxSelect, value),
            NodeKind::Toggle => self
                .attr(ArkUINodeAttributeType::ToggleValue, value)
                .patch_attr(ArkUINodeAttributeType::ToggleValue, value),
            NodeKind::Radio => self
                .attr(ArkUINodeAttributeType::RadioChecked, value)
                .patch_attr(ArkUINodeAttributeType::RadioChecked, value),
            _ => self,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        match self.kind {
            NodeKind::Slider => {
                self = self
                    .builder_attr(ArkUINodeAttributeType::SliderMinValue, min)
                    .builder_attr(ArkUINodeAttributeType::SliderMaxValue, max);
            }
            NodeKind::Progress => {
                self = self.builder_attr(ArkUINodeAttributeType::ProgressTotal, max);
            }
            _ => {}
        }
        self
    }

    #[cfg(feature = "webview")]
    pub fn webview_style(self, style: WebViewStyle) -> Self {
        self.map_webview(|spec| spec.style = Some(style))
    }

    #[cfg(feature = "webview")]
    pub fn url(self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.map_webview(|spec| spec.url = Some(url))
    }

    #[cfg(feature = "webview")]
    pub fn html(self, html: impl Into<String>) -> Self {
        let html = html.into();
        self.map_webview(|spec| spec.html = Some(html))
    }

    #[cfg(feature = "webview")]
    pub fn javascript_enabled(self, enabled: bool) -> Self {
        self.map_webview(|spec| spec.javascript_enabled = Some(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn devtools(self, enabled: bool) -> Self {
        self.map_webview(|spec| spec.devtools = Some(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn transparent(self, enabled: bool) -> Self {
        self.map_webview(|spec| spec.transparent = Some(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn autoplay(self, enabled: bool) -> Self {
        self.map_webview(|spec| spec.autoplay = Some(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn user_agent(self, user_agent: impl Into<String>) -> Self {
        let user_agent = user_agent.into();
        self.map_webview(|spec| spec.user_agent = Some(user_agent))
    }

    #[cfg(feature = "webview")]
    pub fn initialization_scripts(self, scripts: Vec<String>) -> Self {
        self.map_webview(|spec| spec.initialization_scripts = Some(scripts))
    }

    #[cfg(feature = "webview")]
    pub fn headers(self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
        let headers = headers.into_iter().collect::<HashMap<_, _>>();
        self.map_webview(|spec| spec.headers = Some(headers))
    }

    #[cfg(feature = "webview")]
    pub fn on_drag_and_drop(self, callback: impl Fn(String) + 'static) -> Self {
        self.map_webview(|spec| spec.on_drag_and_drop = Some(Rc::new(callback)))
    }

    #[cfg(feature = "webview")]
    pub fn on_download_start(
        self,
        callback: impl Fn(String, &mut PathBuf) -> bool + 'static,
    ) -> Self {
        self.map_webview(|spec| spec.on_download_start = Some(Rc::new(callback)))
    }

    #[cfg(feature = "webview")]
    pub fn on_download_end(
        self,
        callback: impl Fn(String, Option<PathBuf>, bool) + 'static,
    ) -> Self {
        self.map_webview(|spec| spec.on_download_end = Some(Rc::new(callback)))
    }

    #[cfg(feature = "webview")]
    pub fn on_navigation_request(self, callback: impl Fn(String) -> bool + 'static) -> Self {
        self.map_webview(|spec| spec.on_navigation_request = Some(Rc::new(callback)))
    }

    #[cfg(feature = "webview")]
    pub fn on_title_change(self, callback: impl Fn(String) + 'static) -> Self {
        self.map_webview(|spec| spec.on_title_change = Some(Rc::new(callback)))
    }

    pub fn with(
        mut self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.mount_effects.push(Box::new(move |node| {
            effect(node)?;
            Ok(None)
        }));
        self
    }

    pub fn with_cleanup<C>(
        mut self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.mount_effects.push(Box::new(move |node| {
            effect(node).map(|cleanup| Some(Box::new(cleanup) as Cleanup))
        }));
        self
    }

    pub fn with_exit(
        mut self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.exit_effect = Some(Box::new(move |node, finish| {
            effect(node, finish)?;
            Ok(None)
        }));
        self
    }

    pub fn with_exit_cleanup<C>(
        mut self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.exit_effect = Some(Box::new(move |node, finish| {
            effect(node, finish).map(|cleanup| Some(Box::new(cleanup) as Cleanup))
        }));
        self
    }

    pub fn native(self, effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.with(effect)
    }

    /// Run a callback after the node is attached to the render tree and on
    /// every subsequent patch. Use this to capture a live reference to the
    /// underlying native node that stays valid across re-renders.
    pub fn with_patch(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.patch_effects.push(Box::new(move |node| shared(node)));
        self
    }

    pub fn with_next_frame(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.attach_effects.push(Box::new(move |node| {
            let alive = Rc::new(Cell::new(true));
            let callback_alive = alive.clone();
            let callback_effect = shared.clone();
            let callback_node = node.clone();
            node.post_frame_callback(move |_timestamp, _frame| {
                if !callback_alive.get() {
                    return;
                }
                let mut node = callback_node.clone();
                let effect = callback_effect.clone();
                run_guarded_ui_callback(
                    "renderer error: next-frame callback panicked",
                    move || {
                        if let Err(error) = effect(&mut node) {
                            ohos_hilog_binding::error(format!(
                                "renderer error: next-frame callback failed: {error}"
                            ));
                        }
                    },
                );
            })?;

            Ok(Some(Box::new(move || alive.set(false)) as Cleanup))
        }));
        self
    }

    pub fn with_next_idle(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.attach_effects.push(Box::new(move |node| {
            let alive = Rc::new(Cell::new(true));
            let callback_alive = alive.clone();
            let callback_effect = shared.clone();
            let callback_node = node.clone();
            node.post_idle_callback(move |_time_left, _frame| {
                if !callback_alive.get() {
                    return;
                }
                let mut node = callback_node.clone();
                let effect = callback_effect.clone();
                run_guarded_ui_callback("renderer error: next-idle callback panicked", move || {
                    if let Err(error) = effect(&mut node) {
                        ohos_hilog_binding::error(format!(
                            "renderer error: next-idle callback failed: {error}"
                        ));
                    }
                });
            })?;

            Ok(Some(Box::new(move || alive.set(false)) as Cleanup))
        }));
        self
    }

    pub fn native_with_cleanup<C>(
        self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.with_cleanup(effect)
    }

    pub fn on_press(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.on_click(move || arkit_runtime::dispatch(message.clone()))
    }

    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.event_handlers.push(EventHandlerSpec {
            event_type: NodeEventType::OnClick,
            callback: Rc::new(move |_| callback()),
        });
        self
    }

    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        self.event_handlers.push(EventHandlerSpec {
            event_type,
            callback: Rc::new(callback),
        });
        self
    }

    pub fn on_event_no_param(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        self.event_handlers.push(EventHandlerSpec {
            event_type,
            callback: Rc::new(move |_| callback()),
        });
        self
    }

    pub fn on_supported_ui_states(
        mut self,
        ui_states: UiState,
        exclude_inner: bool,
        callback: impl Fn(&mut ArkUINode, UiState) + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as UiStateCallback;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        let ui_state_bits = ui_states.bits();

        self.attach_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let callback_node = node.clone();
            node.add_supported_ui_states(
                ui_state_bits,
                move |current_states| {
                    let current = UiState::from_bits(current_states);
                    let handler = callback_state.borrow().clone();
                    let mut node = callback_node.clone();
                    handler(&mut node, current);
                },
                exclude_inner,
            )?;

            let node_handle = node.clone();
            Ok(Some(Box::new(move || {
                let _ = node_handle.remove_supported_ui_states(ui_state_bits);
            }) as Cleanup))
        }));

        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_custom_event(
        mut self,
        event_type: NodeCustomEventType,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn(&NodeCustomEvent)>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let mut node = RuntimeNode(node);
            node.on_custom_event(event_type, move |event| {
                let callback = callback_state.borrow().clone();
                callback(event);
            });
            Ok(None)
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_long_press_message(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.on_long_press(move || arkit_runtime::dispatch(message.clone()))
    }

    pub fn on_long_press(mut self, callback: impl Fn() + 'static) -> Self {
        self.long_press_handler = Some(LongPressHandlerSpec {
            callback: Rc::new(callback),
        });
        self
    }

    pub fn on_input(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        match self.kind {
            NodeKind::TextInput => self.on_event(NodeEventType::TextInputOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.async_string().unwrap_or_default()));
            }),
            NodeKind::TextArea => self.on_event(NodeEventType::TextAreaOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.async_string().unwrap_or_default()));
            }),
            _ => self,
        }
    }

    pub fn on_submit(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        match self.kind {
            NodeKind::TextInput => self.on_event(NodeEventType::TextInputOnSubmit, move |_| {
                arkit_runtime::dispatch(message.clone());
            }),
            NodeKind::TextArea => self.on_event(NodeEventType::TextAreaOnSubmit, move |_| {
                arkit_runtime::dispatch(message.clone());
            }),
            _ => self,
        }
    }

    pub fn on_refresh(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.on_event(NodeEventType::RefreshOnRefresh, move |_| {
            arkit_runtime::dispatch(message.clone());
        })
    }

    pub fn on_refresh_state_change(self, handler: impl Fn(i32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.on_event(NodeEventType::RefreshStateChange, move |event| {
            arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default()));
        })
    }

    pub fn on_refresh_offset_change(self, handler: impl Fn(f32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::Refresh {
            return self;
        }

        self.on_event(NodeEventType::RefreshOnOffsetChange, move |event| {
            arkit_runtime::dispatch(handler(event.f32_value(0).unwrap_or_default()));
        })
    }

    pub fn on_toggle(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        match self.kind {
            NodeKind::Checkbox => {
                self.on_event(NodeEventType::CheckboxEventOnChange, move |event| {
                    arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default() != 0));
                })
            }
            NodeKind::Toggle => {
                let checked = self
                    .attr_bool(ArkUINodeAttributeType::ToggleValue)
                    .unwrap_or(false);
                self.on_event(NodeEventType::OnClick, move |_| {
                    arkit_runtime::dispatch(handler(!checked));
                })
            }
            NodeKind::Radio => self.on_event(NodeEventType::RadioEventOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default() != 0));
            }),
            _ => self,
        }
    }

    pub fn on_toggle_local(self, handler: impl Fn(bool) + 'static) -> Self {
        let handler = Rc::new(handler) as Rc<dyn Fn(bool)>;
        match self.kind {
            NodeKind::Checkbox => {
                self.on_event(NodeEventType::CheckboxEventOnChange, move |event| {
                    handler(event.i32_value(0).unwrap_or_default() != 0);
                })
            }
            NodeKind::Toggle => {
                let checked = self
                    .attr_bool(ArkUINodeAttributeType::ToggleValue)
                    .unwrap_or(false);
                self.on_event(NodeEventType::OnClick, move |_| {
                    handler(!checked);
                })
            }
            NodeKind::Radio => self.on_event(NodeEventType::RadioEventOnChange, move |event| {
                handler(event.i32_value(0).unwrap_or_default() != 0);
            }),
            _ => self,
        }
    }

    pub fn on_list_scroll_index(
        self,
        handler: impl Fn(ListScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::List {
            return self;
        }

        self.on_event(NodeEventType::ListOnScrollIndex, move |event| {
            arkit_runtime::dispatch(handler(ListScrollIndexEvent {
                first_index: event.i32_value(0).unwrap_or_default(),
                last_index: event.i32_value(1).unwrap_or_default(),
                center_index: event.i32_value(2).unwrap_or_default(),
            }));
        })
    }

    pub fn on_list_scroll_index_local(
        self,
        handler: impl Fn(ListScrollIndexEvent) + 'static,
    ) -> Self {
        if self.kind != NodeKind::List {
            return self;
        }

        self.on_event(NodeEventType::ListOnScrollIndex, move |event| {
            handler(ListScrollIndexEvent {
                first_index: event.i32_value(0).unwrap_or_default(),
                last_index: event.i32_value(1).unwrap_or_default(),
                center_index: event.i32_value(2).unwrap_or_default(),
            });
        })
    }

    pub fn on_list_visible_content_change(
        self,
        handler: impl Fn(ListVisibleContentChangeEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::List {
            return self;
        }

        self.on_event(
            NodeEventType::ListOnScrollVisibleContentChange,
            move |event| {
                arkit_runtime::dispatch(handler(ListVisibleContentChangeEvent {
                    first_index: event.i32_value(0).unwrap_or_default(),
                    start_area: event.i32_value(1).unwrap_or_default(),
                    start_item_index: event.i32_value(2).unwrap_or_default(),
                    last_index: event.i32_value(3).unwrap_or_default(),
                    end_area: event.i32_value(4).unwrap_or_default(),
                    end_item_index: event.i32_value(5).unwrap_or_default(),
                }));
            },
        )
    }

    pub fn on_change(self, handler: impl Fn(f32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        let kind = self.kind;
        self.with(move |node| {
            if matches!(kind, NodeKind::Slider) {
                let mut slider: Slider = wrap_component(node.clone());
                slider.on_slider_change(move |value| {
                    arkit_runtime::dispatch(handler(value.value));
                });
            }
            Ok(())
        })
    }
}

impl<Message: 'static, AppTheme: 'static> advanced::Widget<Message, AppTheme, Renderer>
    for Node<Message, AppTheme>
{
    fn tag(&self) -> advanced::widget::Tag {
        node_widget_tag(self.kind)
    }

    fn state(&self) -> advanced::widget::State {
        match self.kind {
            NodeKind::Scroll => advanced::widget::State::new(Box::new(Rc::new(RefCell::new(
                ScrollState::default(),
            )))),
            _ => advanced::widget::State::none(),
        }
    }

    fn persistent_key(&self) -> Option<&str> {
        self.persistent_key.as_deref()
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        self.children
            .iter()
            .map(arkit_core::advanced::tree_of)
            .collect()
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        tree.set_persistent_key(self.persistent_key.clone());
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self) -> arkit_core::layout::Node {
        arkit_core::layout::Node::new(Size::new(0.0, 0.0))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub fn button_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Stack)
}

pub fn button<Message, AppTheme>(label: impl Into<String>) -> Node<Message, AppTheme> {
    Node::new(NodeKind::Button).label(label)
}

pub fn text_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Text)
}

pub fn text<Message, AppTheme>(content: impl Into<String>) -> Node<Message, AppTheme> {
    text_component().content(content)
}

pub fn text_input_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::TextInput)
}

pub fn text_input<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> Node<Message, AppTheme> {
    text_input_component().placeholder(placeholder).value(value)
}

pub fn text_area_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::TextArea)
}

pub fn text_area<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> Node<Message, AppTheme> {
    text_area_component().placeholder(placeholder).value(value)
}

pub fn checkbox_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Checkbox)
}

pub fn checkbox<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    checkbox_component().checked(checked)
}

pub fn toggle_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Toggle)
}

pub fn toggle<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    toggle_component().checked(checked)
}

pub fn radio_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Radio)
}

pub fn radio<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    radio_component().checked(checked)
}

pub fn slider_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Slider)
}

pub fn slider<Message, AppTheme>(value: f32, min: f32, max: f32) -> Node<Message, AppTheme> {
    slider_component()
        .attr(ArkUINodeAttributeType::SliderValue, value)
        .patch_attr(ArkUINodeAttributeType::SliderValue, value)
        .range(min, max)
}

pub fn progress_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Progress)
}

pub fn progress<Message, AppTheme>(value: f32, total: f32) -> Node<Message, AppTheme> {
    progress_component()
        .attr(ArkUINodeAttributeType::ProgressValue, value)
        .patch_attr(ArkUINodeAttributeType::ProgressValue, value)
        .range(0.0, total)
}

pub fn image_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Image)
}

pub fn image<Message, AppTheme>(src: impl Into<String>) -> Node<Message, AppTheme> {
    let src = src.into();
    image_component()
        .attr(ArkUINodeAttributeType::ImageSrc, src.clone())
        .patch_attr(ArkUINodeAttributeType::ImageSrc, src)
}

pub fn list_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::List)
}

pub fn list<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    list_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub fn list_item_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::ListItem)
}

pub fn list_item<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    list_item_component().child(child.into()).into()
}

pub fn column_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Column)
}

pub fn column<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub fn row_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Row)
}

pub fn row<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    row_component().percent_width(1.0).children(children).into()
}

pub fn stack_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Stack)
}

pub fn stack<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub type Container<Message, AppTheme = arkit_core::Theme> = Node<Message, AppTheme>;

pub fn container<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Container<Message, AppTheme> {
    column_component().child(child.into())
}

pub fn scroll_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Scroll)
}

pub fn scroll<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    scroll_component().child(child.into()).into()
}

pub fn refresh_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Refresh)
}

pub fn refresh<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    refresh_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .child(child.into())
        .into()
}

pub(crate) fn read_layout_size(node: &ArkUINode) -> Option<LayoutSize> {
    let size = node.layout_size().ok()?;
    Some(LayoutSize {
        width: size.width as f32,
        height: size.height as f32,
    })
}

pub(crate) fn read_layout_frame(node: &ArkUINode) -> Option<LayoutFrame> {
    let size = read_layout_size(node)?;
    let position = node
        .position_with_translate_in_window()
        .or_else(|_| node.layout_position_in_window())
        .ok()?;
    Some(LayoutFrame {
        x: position.x as f32,
        y: position.y as f32,
        width: size.width,
        height: size.height,
    })
}

pub fn observe_layout_size<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    on_change: impl Fn(LayoutSize) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutSize)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(size) = read_layout_size(&runtime) {
                    on_change(size);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(size) = read_layout_size(node) {
                    on_change(size);
                }
            }
        })
        .into()
}

pub fn observe_layout_frame<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    enabled: bool,
    on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    if !enabled {
        return element;
    }

    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutFrame)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(frame) = read_layout_frame(&runtime) {
                    on_change(frame);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(frame) = read_layout_frame(node) {
                    on_change(frame);
                }
            }
        })
        .into()
}

pub fn swiper_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Swiper)
}

pub fn swiper<Message, AppTheme>(
    children: Vec<Element<Message, AppTheme>>,
) -> Node<Message, AppTheme> {
    swiper_component().children(children)
}

pub fn calendar_picker_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::CalendarPicker)
}

pub fn calendar_picker<Message, AppTheme>() -> Node<Message, AppTheme> {
    calendar_picker_component()
}

pub fn date_picker_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::DatePicker)
}

pub fn date_picker<Message, AppTheme>() -> Node<Message, AppTheme> {
    date_picker_component()
}

#[cfg(feature = "webview")]
pub fn web_view_component<Message, AppTheme>(
    controller: WebViewController,
) -> Node<Message, AppTheme> {
    let mut node = Node::new(NodeKind::WebViewHost)
        .key(controller.id().to_string())
        .clip(true)
        .hit_test_behavior(HitTestBehavior::Transparent);
    node.webview = Some(WebViewSpec {
        controller: Some(controller),
        ..WebViewSpec::default()
    });
    node
}

#[cfg(feature = "webview")]
pub fn web_view<Message, AppTheme>(
    controller: WebViewController,
    url: impl Into<String>,
) -> Node<Message, AppTheme> {
    web_view_component(controller)
        .percent_width(1.0)
        .percent_height(1.0)
        .url(url)
}

#[cfg(feature = "webview")]
fn register_lifecycle_callback(
    webview: &Webview,
    callback: &WebViewLifecycleCallback,
    lifecycle: WebViewLifecycle,
) -> Result<(), String> {
    let callback = callback.clone();
    let callback_dispatcher = arkit_runtime::internal::global_dispatcher();
    match lifecycle {
        WebViewLifecycle::ControllerAttach => webview.on_controller_attach({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::PageBegin => webview.on_page_begin({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::PageEnd => webview.on_page_end({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
        WebViewLifecycle::Destroy => webview.on_destroy({
            let callback = callback.clone();
            let callback_dispatcher = callback_dispatcher.clone();
            move || {
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    let mut callback = callback.borrow_mut();
                    (callback.as_mut())();
                });
            }
        }),
    }
    .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
fn register_webview_lifecycle_callbacks(
    controller: &WebViewController,
    webview: &Webview,
) -> Result<(), String> {
    for callback in controller.inner.controller_attach_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::ControllerAttach)?;
    }
    for callback in controller.inner.page_begin_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::PageBegin)?;
    }
    for callback in controller.inner.page_end_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::PageEnd)?;
    }
    for callback in controller.inner.destroy_callbacks.borrow().iter() {
        register_lifecycle_callback(webview, callback, WebViewLifecycle::Destroy)?;
    }
    Ok(())
}

#[cfg(feature = "webview")]
fn register_internal_webview_callbacks(
    controller: &WebViewController,
    webview: &Webview,
) -> Result<(), String> {
    let attach_controller = controller.clone();
    webview
        .on_controller_attach(move || {
            attach_controller.inner.attached.set(true);
        })
        .map_err(|error| error.to_string())?;

    let state = controller.inner.clone();
    webview
        .on_destroy(move || {
            state.attached.set(false);
        })
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
fn build_initial_webview_style(
    spec: &WebViewSpec,
    _frame: Option<LayoutFrame>,
) -> Option<WebViewStyle> {
    let style = spec.style.clone().unwrap_or_default();
    let has_style = style.x.is_some()
        || style.y.is_some()
        || style.visible.is_some()
        || style.background_color.is_some();
    has_style.then_some(style)
}

#[cfg(feature = "webview")]
fn webview_frame_is_valid(frame: LayoutFrame) -> bool {
    frame.width.is_finite() && frame.height.is_finite() && frame.width > 0.0 && frame.height > 0.0
}

#[cfg(feature = "webview")]
fn sync_embedded_webview_node_bounds(
    controller: &WebViewController,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let Some(frame) = frame.filter(|frame| webview_frame_is_valid(*frame)) else {
        return Ok(());
    };
    let Some(mut node) = controller.inner.embedded_node.borrow().clone() else {
        return Ok(());
    };

    let runtime = RuntimeNode(&mut node);
    runtime
        .set_position(vec![0.0_f32, 0.0_f32])
        .map_err(|error| error.to_string())?;
    runtime
        .set_size(vec![frame.width, frame.height])
        .map_err(|error| error.to_string())?;
    runtime
        .set_layout_rect(vec![0.0_f32, 0.0_f32, frame.width, frame.height])
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
fn same_style_value(
    left: &Option<Either<f64, String>>,
    right: &Option<Either<f64, String>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(Either::A(left)), Some(Either::A(right))) => left == right,
        (Some(Either::B(left)), Some(Either::B(right))) => left == right,
        _ => false,
    }
}

#[cfg(feature = "webview")]
fn same_webview_style(left: Option<&WebViewStyle>, right: Option<&WebViewStyle>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            same_style_value(&left.x, &right.x)
                && same_style_value(&left.y, &right.y)
                && left.visible == right.visible
                && left.background_color == right.background_color
        }
        _ => false,
    }
}

#[cfg(feature = "webview")]
fn current_applied_state(spec: &WebViewSpec, frame: Option<LayoutFrame>) -> WebViewAppliedState {
    WebViewAppliedState {
        url: spec.url.clone(),
        html: spec.html.clone(),
        headers: spec.headers.clone(),
        style: build_initial_webview_style(spec, frame),
    }
}

#[cfg(feature = "webview")]
struct NativeWebViewMount {
    webview: Webview,
    node: ArkUINode,
}

#[cfg(feature = "webview")]
fn create_native_webview(
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<NativeWebViewMount, String> {
    let controller = spec
        .controller
        .as_ref()
        .ok_or_else(|| String::from("webview controller is missing"))?;
    let helper = unsafe { get_helper() };
    let helper_borrow = helper.borrow();
    let helper_ref = helper_borrow
        .as_ref()
        .ok_or_else(|| String::from("arkts helper is not available"))?;
    let env = get_main_thread_env();
    let env_borrow = env.borrow();
    let env_ref = env_borrow
        .as_ref()
        .ok_or_else(|| String::from("main thread env is not available"))?;
    let helper_object = helper_ref
        .get_value(env_ref)
        .map_err(|error| error.to_string())?;
    let create_webview = helper_object
        .get_named_property::<Function<'_, WebViewInitData<'_>, ObjectRef>>("createEmbeddedWebview")
        .map_err(|error| error.to_string())?;
    let callback_dispatcher = arkit_runtime::internal::global_dispatcher();

    let on_drag_and_drop = spec.on_drag_and_drop.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_drag_and_drop", move |ctx| {
                let event = ctx.try_get::<String>(0)?;
                let event = match event {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || handler(event));
                Ok(())
            })
            .ok()
    });

    let on_download_start = spec.on_download_start.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_download_start", move |ctx| {
                let origin_url = ctx.try_get::<String>(0)?;
                let temp_path = ctx.try_get::<String>(1)?;
                let origin_url = match origin_url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let temp_path = match temp_path {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let mut path = PathBuf::from(temp_path);
                let handler = handler.clone();
                let allow = run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(origin_url, &mut path)
                });
                Ok(DownloadStartResult {
                    allow,
                    temp_path: Some(path.to_string_lossy().to_string()),
                })
            })
            .ok()
    });

    let on_download_end = spec.on_download_end.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_download_end", move |ctx| {
                let origin_url = ctx.try_get::<String>(0)?;
                let temp_path = ctx.try_get::<String>(1)?;
                let success = ctx.try_get::<bool>(2)?;
                let origin_url = match origin_url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let temp_path = match temp_path {
                    Either::A(value) => Some(PathBuf::from(value)),
                    Either::B(_undefined) => None,
                };
                let success = match success {
                    Either::A(value) => value,
                    Either::B(_undefined) => false,
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(origin_url, temp_path, success);
                });
                Ok(())
            })
            .ok()
    });

    let on_navigation_request = spec.on_navigation_request.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_navigation_request", move |ctx| {
                let url = ctx.try_get::<String>(0)?;
                let url = match url {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                Ok(run_with_webview_dispatcher(
                    callback_dispatcher.clone(),
                    || handler(url),
                ))
            })
            .ok()
    });

    let on_title_change = spec.on_title_change.as_ref().and_then(|handler| {
        let handler = handler.clone();
        let callback_dispatcher = callback_dispatcher.clone();
        env_ref
            .create_function_from_closure("arkit_on_title_change", move |ctx| {
                let title = ctx.try_get::<String>(0)?;
                let title = match title {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::new(),
                };
                let handler = handler.clone();
                run_with_webview_dispatcher(callback_dispatcher.clone(), || {
                    handler(title);
                });
                Ok(())
            })
            .ok()
    });

    let embedded_webview = create_webview
        .call(WebViewInitData {
            url: spec.url.clone(),
            id: Some(controller.id().to_string()),
            style: build_initial_webview_style(spec, frame),
            javascript_enabled: spec.javascript_enabled,
            devtools: spec.devtools,
            user_agent: spec.user_agent.clone(),
            autoplay: spec.autoplay,
            initialization_scripts: spec.initialization_scripts.clone(),
            headers: spec.headers.clone(),
            html: spec.html.clone(),
            transparent: spec.transparent,
            on_drag_and_drop,
            on_download_start,
            on_download_end,
            on_navigation_request,
            on_title_change,
        })
        .map_err(|error| error.to_string())?;

    let embedded_value = embedded_webview
        .get_value(env_ref)
        .map_err(|error| error.to_string())?;
    let controller_object = embedded_value
        .get::<Object>("controller")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| String::from("embedded webview controller is missing"))?;
    let node_raw = embedded_value
        .get::<ArkUINodeRaw>("content")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| String::from("embedded webview content is missing"))?;
    let controller_ref = controller_object
        .create_ref::<true>()
        .map_err(|error| error.to_string())?;
    let node = ArkUINode::from_raw_handle(node_raw.raw)
        .ok_or_else(|| String::from("embedded webview content handle is null"))?;
    let webview = Webview::new(controller.id().to_string(), controller_ref)
        .map_err(|error| error.to_string())?;

    Ok(NativeWebViewMount { webview, node })
}

#[cfg(feature = "webview")]
fn attach_embedded_webview_node(
    host: &mut ArkUINode,
    controller: &WebViewController,
) -> Result<(), String> {
    let Some(node) = controller.inner.embedded_node.borrow().clone() else {
        return Ok(());
    };
    let raw_handle = node.raw_handle() as usize;
    if host
        .children()
        .iter()
        .any(|child| child.borrow().raw_handle() as usize == raw_handle)
    {
        return Ok(());
    }

    let mut runtime = RuntimeNode(host);
    runtime
        .add_existing_child(node)
        .map_err(|error| error.to_string())
}

#[cfg(feature = "webview")]
fn detach_embedded_webview_node(host: &mut ArkUINode, controller: &WebViewController) {
    let Some(raw_handle) = controller
        .inner
        .embedded_node
        .borrow()
        .as_ref()
        .map(|node| node.raw_handle() as usize)
    else {
        return;
    };

    if let Err(error) = remove_child_by_raw(host, raw_handle) {
        ohos_hilog_binding::error(format!(
            "webview error: failed to detach embedded webview '{}': {error}",
            controller.id()
        ));
    }
}

#[cfg(feature = "webview")]
fn mount_or_show_webview(
    host: &mut ArkUINode,
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let created = if controller.inner.webview.borrow().is_none() {
        controller.inner.attached.set(false);
        let mount = match create_native_webview(spec, frame) {
            Ok(mount) => mount,
            Err(error) => {
                controller.inner.active_binding.set(false);
                return Err(error);
            }
        };
        let mut runtime = RuntimeNode(host);
        if let Err(error) = runtime.add_existing_child(mount.node.clone()) {
            let _ = mount.webview.dispose();
            return Err(error.to_string());
        }
        controller
            .inner
            .embedded_node
            .replace(Some(mount.node.clone()));
        let webview = mount.webview;
        controller.inner.webview.replace(Some(webview.clone()));
        register_internal_webview_callbacks(controller, &webview)?;
        register_webview_lifecycle_callbacks(controller, &webview)?;
        for callback in controller.inner.ready_callbacks.borrow().iter() {
            callback(&webview);
        }
        controller
            .inner
            .applied
            .replace(Some(current_applied_state(spec, frame)));
        true
    } else {
        false
    };

    if controller.inner.active_binding.replace(true) && created {
        ohos_hilog_binding::error(format!(
            "webview error: controller '{}' is already bound to an active host",
            controller.id()
        ));
    }

    attach_embedded_webview_node(host, controller)?;
    sync_embedded_webview_node_bounds(controller, frame)?;

    if !created {
        sync_webview_config(controller, spec, frame)?;
    }
    Ok(())
}

#[cfg(feature = "webview")]
fn ensure_webview_mounted(
    host: &mut ArkUINode,
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    if controller.inner.webview.borrow().is_none() {
        mount_or_show_webview(host, controller, spec, frame)
    } else {
        attach_embedded_webview_node(host, controller)?;
        sync_embedded_webview_node_bounds(controller, frame)?;
        sync_webview_config(controller, spec, frame)
    }
}

#[cfg(feature = "webview")]
fn sync_webview_config(
    controller: &WebViewController,
    spec: &WebViewSpec,
    frame: Option<LayoutFrame>,
) -> Result<(), String> {
    let current = current_applied_state(spec, frame);
    let previous = controller.inner.applied.borrow().clone();
    controller.inner.applied.replace(Some(current.clone()));

    let Some(webview) = controller.inner.webview.borrow().clone() else {
        return Ok(());
    };

    if force_webview_style_sync(previous.as_ref(), &current) {
        let previous_style = previous.as_ref().and_then(|state| state.style.as_ref());
        let current_style = current.style.as_ref();

        let previous_visible = previous_style.and_then(|style| style.visible);
        let current_visible = current_style.and_then(|style| style.visible);
        if previous_visible != current_visible {
            if let Some(visible) = current_visible {
                webview
                    .set_visible(visible)
                    .map_err(|error| error.to_string())?;
            }
        }

        let previous_background =
            previous_style.and_then(|style| style.background_color.as_deref());
        let current_background = current_style.and_then(|style| style.background_color.as_deref());
        if previous_background != current_background {
            if let Some(background) = current_background {
                webview
                    .set_background_color(background)
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    if !controller.inner.attached.get() {
        return Ok(());
    }

    let page_source_changed = match previous.as_ref() {
        Some(state) => state.url != current.url || state.headers != current.headers,
        None => current.url.is_some() || current.headers.is_some(),
    };
    let html_changed = match previous.as_ref() {
        Some(state) => state.html != current.html,
        None => current.html.is_some(),
    };

    if page_source_changed {
        if let Some(url) = current.url.as_deref() {
            if let Some(headers) = current.headers.clone() {
                let headers = headers
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key.parse::<http::header::HeaderName>()
                                .expect("webview header name should be valid"),
                            value
                                .parse::<http::header::HeaderValue>()
                                .expect("webview header value should be valid"),
                        )
                    })
                    .collect();
                webview
                    .load_url_with_headers(url, headers)
                    .map_err(|error| error.to_string())?;
            } else {
                webview.load_url(url).map_err(|error| error.to_string())?;
            }
        } else if let Some(html) = current.html.as_deref() {
            webview.load_html(html).map_err(|error| error.to_string())?;
        }
    } else if html_changed {
        if let Some(html) = current.html.as_deref() {
            webview.load_html(html).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

#[cfg(feature = "webview")]
fn force_webview_style_sync(
    previous: Option<&WebViewAppliedState>,
    current: &WebViewAppliedState,
) -> bool {
    !same_webview_style(
        previous.and_then(|state| state.style.as_ref()),
        current.style.as_ref(),
    )
}

#[cfg(feature = "webview")]
fn unmount_webview(controller: &WebViewController) {
    controller.inner.active_binding.set(false);
    controller.inner.attached.set(false);
    controller.inner.applied.replace(None);
    controller.inner.embedded_node.replace(None);
    if let Some(webview) = controller.inner.webview.borrow_mut().take() {
        let _ = webview.dispose();
    }
}

#[cfg(feature = "webview")]
fn enrich_webview_host<Message, AppTheme>(node: &mut Node<Message, AppTheme>) {
    let Some(spec) = node.webview.clone() else {
        return;
    };
    let Some(controller) = spec.controller.clone() else {
        return;
    };
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));

    node.mount_effects.push(Box::new({
        let controller = controller.clone();
        let node_ref = node_ref.clone();
        move |_node| {
            Ok(Some(Box::new(move || {
                if let Some(mut host) = node_ref.borrow().as_ref().cloned() {
                    detach_embedded_webview_node(&mut host, &controller);
                }
                unmount_webview(&controller);
            }) as Cleanup))
        }
    }));

    node.attach_effects.push(Box::new({
        let controller = controller.clone();
        let spec = spec.clone();
        let node_ref = node_ref.clone();
        move |node| {
            let frame = read_layout_frame(node);
            if let Err(error) = ensure_webview_mounted(node, &controller, &spec, frame) {
                ohos_hilog_binding::error(format!(
                    "webview error: failed to mount webview '{}': {error}",
                    controller.id()
                ));
            }
            node_ref.replace(Some(node.clone()));
            Ok(None)
        }
    }));

    node.event_handlers.push(EventHandlerSpec {
        event_type: NodeEventType::EventOnAreaChange,
        callback: Rc::new({
            let controller = controller.clone();
            let spec = spec.clone();
            let node_ref = node_ref.clone();
            move |_| {
                let Some(node) = node_ref.borrow().as_ref().cloned() else {
                    return;
                };
                if controller.inner.webview.borrow().is_none() {
                    return;
                }
                if let Err(error) =
                    sync_webview_config(&controller, &spec, read_layout_frame(&node))
                {
                    ohos_hilog_binding::error(format!(
                        "webview error: failed to sync webview '{}' after area change: {error}",
                        controller.id()
                    ));
                }
            }
        }),
    });

    node.patch_effects.push(Box::new({
        let controller = controller.clone();
        let spec = spec.clone();
        let node_ref = node_ref.clone();
        move |node| {
            let frame = read_layout_frame(node);
            if let Err(error) = ensure_webview_mounted(node, &controller, &spec, frame) {
                ohos_hilog_binding::error(format!(
                    "webview error: failed to sync webview '{}': {error}",
                    controller.id()
                ));
            }
            node_ref.replace(Some(node.clone()));
            Ok(())
        }
    }));
}

fn apply_attr_list(
    node: &mut ArkUINode,
    attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) {
    let runtime = RuntimeNode(node);
    for (attr, value) in ordered_attrs_for_application(attrs) {
        if let Err(error) = runtime.set_attribute(attr, value) {
            ohos_hilog_binding::error(format!(
                "renderer error: failed to set attribute {attr:?}: {error}"
            ));
        }
    }
}

fn ordered_attrs_for_application(
    attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) -> Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)> {
    let mut ordered = Vec::with_capacity(attrs.len());
    let mut deferred = Vec::new();

    for (attr, value) in attrs {
        match attr {
            ArkUINodeAttributeType::BorderRadius
            | ArkUINodeAttributeType::BorderRadiusPercent
            | ArkUINodeAttributeType::Clip
            | ArkUINodeAttributeType::ClipShape => deferred.push((attr, value)),
            _ => ordered.push((attr, value)),
        }
    }

    ordered.extend(deferred);
    ordered
}

fn run_cleanups(mut cleanups: Vec<Cleanup>) {
    while let Some(cleanup) = cleanups.pop() {
        cleanup();
    }
}

fn wrap_component<T>(node: ArkUINode) -> T {
    assert_eq!(size_of::<T>(), size_of::<ArkUINode>());
    assert_eq!(align_of::<T>(), align_of::<ArkUINode>());
    let node = ManuallyDrop::new(node);
    unsafe { std::ptr::read((&*node as *const ArkUINode).cast::<T>()) }
}

fn create_node(kind: NodeKind) -> ArkUIResult<ArkUINode> {
    Ok(match kind {
        NodeKind::Button => Button::new()?.into(),
        NodeKind::CalendarPicker => CalendarPicker::new()?.into(),
        NodeKind::Checkbox => Checkbox::new()?.into(),
        NodeKind::Column => Column::new()?.into(),
        NodeKind::DatePicker => DatePicker::new()?.into(),
        NodeKind::Image => Image::new()?.into(),
        NodeKind::List => List::new()?.into(),
        NodeKind::ListItem => ListItem::new()?.into(),
        NodeKind::Progress => Progress::new()?.into(),
        NodeKind::Radio => Radio::new()?.into(),
        NodeKind::Refresh => Refresh::new()?.into(),
        NodeKind::Row => Row::new()?.into(),
        NodeKind::Scroll => Scroll::new()?.into(),
        NodeKind::Slider => Slider::new()?.into(),
        NodeKind::Stack => Stack::new()?.into(),
        NodeKind::Swiper => Swiper::new()?.into(),
        NodeKind::Text => Text::new()?.into(),
        NodeKind::TextArea => TextArea::new()?.into(),
        NodeKind::TextInput => TextInput::new()?.into(),
        NodeKind::Toggle => Toggle::new()?.into(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => Stack::new()?.into(),
    })
}

fn node_type_id(kind: NodeKind) -> TypeId {
    match kind {
        NodeKind::Button => TypeId::of::<Button>(),
        NodeKind::CalendarPicker => TypeId::of::<CalendarPicker>(),
        NodeKind::Checkbox => TypeId::of::<Checkbox>(),
        NodeKind::Column => TypeId::of::<Column>(),
        NodeKind::DatePicker => TypeId::of::<DatePicker>(),
        NodeKind::Image => TypeId::of::<Image>(),
        NodeKind::List => TypeId::of::<List>(),
        NodeKind::ListItem => TypeId::of::<ListItem>(),
        NodeKind::Progress => TypeId::of::<Progress>(),
        NodeKind::Radio => TypeId::of::<Radio>(),
        NodeKind::Refresh => TypeId::of::<Refresh>(),
        NodeKind::Row => TypeId::of::<Row>(),
        NodeKind::Scroll => TypeId::of::<Scroll>(),
        NodeKind::Slider => TypeId::of::<Slider>(),
        NodeKind::Stack => TypeId::of::<Stack>(),
        NodeKind::Swiper => TypeId::of::<Swiper>(),
        NodeKind::Text => TypeId::of::<Text>(),
        NodeKind::TextArea => TypeId::of::<TextArea>(),
        NodeKind::TextInput => TypeId::of::<TextInput>(),
        NodeKind::Toggle => TypeId::of::<Toggle>(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => TypeId::of::<WebViewHostNodeTag>(),
    }
}

struct ButtonNodeTag;
struct CalendarPickerNodeTag;
struct CheckboxNodeTag;
struct ColumnNodeTag;
struct DatePickerNodeTag;
struct ImageNodeTag;
struct ListNodeTag;
struct ListItemNodeTag;
struct ProgressNodeTag;
struct RadioNodeTag;
struct RefreshNodeTag;
struct RowNodeTag;
struct ScrollNodeTag;
struct SliderNodeTag;
struct StackNodeTag;
struct SwiperNodeTag;
struct TextNodeTag;
struct TextAreaNodeTag;
struct TextInputNodeTag;
struct ToggleNodeTag;
#[cfg(feature = "webview")]
struct WebViewHostNodeTag;

fn node_widget_tag(kind: NodeKind) -> advanced::widget::Tag {
    match kind {
        NodeKind::Button => advanced::widget::Tag::of::<ButtonNodeTag>(),
        NodeKind::CalendarPicker => advanced::widget::Tag::of::<CalendarPickerNodeTag>(),
        NodeKind::Checkbox => advanced::widget::Tag::of::<CheckboxNodeTag>(),
        NodeKind::Column => advanced::widget::Tag::of::<ColumnNodeTag>(),
        NodeKind::DatePicker => advanced::widget::Tag::of::<DatePickerNodeTag>(),
        NodeKind::Image => advanced::widget::Tag::of::<ImageNodeTag>(),
        NodeKind::List => advanced::widget::Tag::of::<ListNodeTag>(),
        NodeKind::ListItem => advanced::widget::Tag::of::<ListItemNodeTag>(),
        NodeKind::Progress => advanced::widget::Tag::of::<ProgressNodeTag>(),
        NodeKind::Radio => advanced::widget::Tag::of::<RadioNodeTag>(),
        NodeKind::Refresh => advanced::widget::Tag::of::<RefreshNodeTag>(),
        NodeKind::Row => advanced::widget::Tag::of::<RowNodeTag>(),
        NodeKind::Scroll => advanced::widget::Tag::of::<ScrollNodeTag>(),
        NodeKind::Slider => advanced::widget::Tag::of::<SliderNodeTag>(),
        NodeKind::Stack => advanced::widget::Tag::of::<StackNodeTag>(),
        NodeKind::Swiper => advanced::widget::Tag::of::<SwiperNodeTag>(),
        NodeKind::Text => advanced::widget::Tag::of::<TextNodeTag>(),
        NodeKind::TextArea => advanced::widget::Tag::of::<TextAreaNodeTag>(),
        NodeKind::TextInput => advanced::widget::Tag::of::<TextInputNodeTag>(),
        NodeKind::Toggle => advanced::widget::Tag::of::<ToggleNodeTag>(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => advanced::widget::Tag::of::<WebViewHostNodeTag>(),
    }
}

fn sync_element_tree<Message, AppTheme>(
    element: &Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let widget = element.as_widget();
    let next_tag = widget.tag();
    let next_persistent_key = widget.persistent_key();
    if tree.tag() != next_tag || tree.persistent_key() != next_persistent_key {
        let next_tree = state_cache
            .take(next_tag, next_persistent_key)
            .unwrap_or_else(|| arkit_core::advanced::tree_of(element));
        let previous_tree = std::mem::replace(tree, next_tree);
        state_cache.store(previous_tree);
    }

    widget.diff(tree);
    tree.set_persistent_key(next_persistent_key.map(str::to_string));
}

fn sync_child_trees<Message, AppTheme>(
    children: &[Element<Message, AppTheme>],
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let mut next_trees = Vec::with_capacity(children.len());
    let mut existing = std::mem::take(tree.children_mut());

    for child in children {
        let widget = child.as_widget();
        let next_tag = widget.tag();
        let next_persistent_key = widget.persistent_key();
        let mut child_tree = if let Some(persistent_key) = next_persistent_key {
            if let Some(index) = existing.iter().position(|tree| {
                tree.tag() == next_tag && tree.persistent_key() == Some(persistent_key)
            }) {
                existing.remove(index)
            } else {
                state_cache
                    .take(next_tag, Some(persistent_key))
                    .unwrap_or_else(|| arkit_core::advanced::tree_of(child))
            }
        } else if existing.is_empty() {
            arkit_core::advanced::tree_of(child)
        } else {
            existing.remove(0)
        };
        sync_element_tree(child, &mut child_tree, state_cache);
        next_trees.push(child_tree);
    }

    for child_tree in existing {
        state_cache.store(child_tree);
    }

    tree.replace_children(next_trees);
}

struct CompiledElement<Message, AppTheme = arkit_core::Theme> {
    body: Element<Message, AppTheme>,
    overlays: Vec<Element<Message, AppTheme>>,
}

fn bind_node_state(
    kind: NodeKind,
    event_handlers: &mut Vec<EventHandlerSpec>,
    attach_effects: &mut Vec<AttachEffect>,
    state_bound: &mut bool,
    tree: &mut advanced::widget::Tree,
) {
    if *state_bound || !matches!(kind, NodeKind::Scroll) {
        return;
    }

    let Some(scroll_state) = tree
        .state()
        .downcast_mut::<Rc<RefCell<ScrollState>>>()
        .cloned()
    else {
        return;
    };

    event_handlers.push(EventHandlerSpec {
        event_type: NodeEventType::ScrollEventOnScroll,
        callback: Rc::new({
            let scroll_state = scroll_state.clone();
            move |event| {
                let event_offset = ScrollOffset {
                    x: event.f32_value(0).unwrap_or_default(),
                    y: event.f32_value(1).unwrap_or_default(),
                };
                let offset = scroll_state
                    .borrow()
                    .node
                    .clone()
                    .and_then(|mut node| read_scroll_offset(&mut node))
                    .unwrap_or(event_offset);
                let mut state = scroll_state.borrow_mut();
                state.offset = offset;
                if let Some(viewport) = state.viewport.as_mut() {
                    viewport.offset = offset;
                }
            }
        }),
    });

    attach_effects.push(Box::new(move |node| {
        let alive = Rc::new(Cell::new(true));
        let scroll_node = node.clone();
        let scroll_state = scroll_state.clone();
        scroll_state.borrow_mut().node = Some(scroll_node.clone());

        let restore_state = scroll_state.clone();
        let restore = Rc::new(move || {
            let offset = restore_state.borrow().offset;
            if offset == ScrollOffset::default() {
                return;
            }
            let mut scroll_node = scroll_node.clone();
            if let Err(error) = apply_scroll_offset(&mut scroll_node, offset) {
                ohos_hilog_binding::error(format!(
                    "renderer error: failed to restore scroll offset: {error}"
                ));
            }
        });

        let frame_alive = alive.clone();
        let frame_restore = restore.clone();
        node.post_frame_callback(move |_timestamp, _frame| {
            if !frame_alive.get() {
                return;
            }
            frame_restore();
        })?;

        let idle_alive = alive.clone();
        let idle_restore = restore;
        node.post_idle_callback(move |_time_left, _frame| {
            if !idle_alive.get() {
                return;
            }
            idle_restore();
        })?;

        Ok(Some(Box::new(move || {
            alive.set(false);
            scroll_state.borrow_mut().node = None;
        }) as Cleanup))
    }));

    *state_bound = true;
}

#[cfg(feature = "webview")]
fn prepare_node<Message, AppTheme>(mut node: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    if node.kind == NodeKind::WebViewHost {
        enrich_webview_host(&mut node);
    }
    node
}

#[cfg(not(feature = "webview"))]
fn prepare_node<Message, AppTheme>(node: Node<Message, AppTheme>) -> Node<Message, AppTheme> {
    node
}

fn compile_node<Message, AppTheme>(
    node: Node<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
    renderer: &Renderer,
    bind_state: bool,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        persistent_key,
        init_attrs,
        patch_attrs,
        mut event_handlers,
        long_press_handler,
        mount_effects,
        mut attach_effects,
        patch_effects,
        exit_effect,
        mut state_bound,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = prepare_node(node);

    if bind_state {
        bind_node_state(
            kind,
            &mut event_handlers,
            &mut attach_effects,
            &mut state_bound,
            tree,
        );
    }
    sync_child_trees(&children, tree, state_cache);

    let mut compiled_children = Vec::with_capacity(children.len());
    let mut overlays = Vec::new();

    for (child, child_tree) in children.into_iter().zip(tree.children_mut().iter_mut()) {
        let compiled = compile_element(child, child_tree, state_cache, renderer, bind_state);
        compiled_children.push(compiled.body);
        overlays.extend(compiled.overlays);
    }

    CompiledElement {
        body: Node {
            kind,
            key,
            persistent_key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            state_bound,
            #[cfg(feature = "webview")]
            webview: None,
            children: compiled_children,
        }
        .into(),
        overlays,
    }
}

fn compile_element<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    state_cache: &mut StateCache,
    renderer: &Renderer,
    bind_state: bool,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    sync_element_tree(&element, tree, state_cache);

    let widget = element.into_widget();
    if widget.as_any().is::<Node<Message, AppTheme>>() {
        let any = widget.into_any();
        let node = any
            .downcast::<Node<Message, AppTheme>>()
            .unwrap_or_else(|_| {
                panic!(
                    "renderer node downcast failed for {}",
                    type_name::<Node<Message, AppTheme>>()
                )
            });
        return compile_node(*node, tree, state_cache, renderer, bind_state);
    }

    let body = widget
        .body(tree, renderer)
        .unwrap_or_else(|| panic!("composite widget did not provide a body element"));
    let compiled_body = {
        let body_tree = tree
            .child_mut(0)
            .unwrap_or_else(|| panic!("composite widget body child was not initialized"));
        sync_element_tree(&body, body_tree, state_cache);
        compile_element(body, body_tree, state_cache, renderer, bind_state)
    };

    let overlay = widget.overlay(tree, renderer);
    let mut overlays = compiled_body.overlays;
    if let Some(overlay) = overlay {
        let overlay_tree = tree
            .child_mut(1)
            .unwrap_or_else(|| panic!("composite widget overlay child was not initialized"));
        sync_element_tree(&overlay, overlay_tree, state_cache);
        let compiled_overlay =
            compile_element(overlay, overlay_tree, state_cache, renderer, bind_state);
        overlays.push(compiled_overlay.body);
        overlays.extend(compiled_overlay.overlays);
    }

    CompiledElement {
        body: compiled_body.body,
        overlays,
    }
}

fn compose_compiled_overlays<Message, AppTheme>(
    compiled: CompiledElement<Message, AppTheme>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    if compiled.overlays.is_empty() {
        return compiled.body;
    }

    let mut children = vec![compiled.body];

    children.push(
        stack_component::<Message, AppTheme>()
            .percent_width(1.0)
            .percent_height(1.0)
            .attr(ArkUINodeAttributeType::Clip, false)
            .hit_test_behavior(HitTestBehavior::Default)
            .attr(
                ArkUINodeAttributeType::Alignment,
                i32::from(Alignment::TopStart),
            )
            .attr(ArkUINodeAttributeType::ZIndex, 10_000_i32)
            .children(compiled.overlays)
            .into(),
    );

    stack_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .attr(ArkUINodeAttributeType::Clip, false)
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::TopStart),
        )
        .children(children)
        .into()
}

fn into_node<Message, AppTheme>(element: Element<Message, AppTheme>) -> Node<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = compile_element(
        element,
        &mut tree,
        &mut state_cache,
        &Renderer::default(),
        false,
    );
    let widget = compiled.body.into_widget();
    let any = widget.into_any();
    *any.downcast::<Node<Message, AppTheme>>()
        .unwrap_or_else(|_| {
            panic!(
                "arkit renderer only supports renderer::Node widget bodies in this build; got {}",
                type_name::<Node<Message, AppTheme>>()
            )
        })
}

fn desired_attrs(
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) -> Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)> {
    let mut attrs = Vec::new();
    for (attr, value) in init_attrs.into_iter().chain(patch_attrs) {
        if let Some(index) = attrs
            .iter()
            .position(|(current_attr, _)| *current_attr == attr)
        {
            attrs.remove(index);
        }
        attrs.push((attr, value));
    }
    attrs
}

fn attr_types(
    attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    attrs.iter().map(|(attr, _)| *attr).collect()
}

fn desired_attr_types(
    init_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
    patch_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    let mut attrs = Vec::new();
    for (attr, _) in init_attrs.iter().chain(patch_attrs.iter()) {
        if !attrs.contains(attr) {
            attrs.push(*attr);
        }
    }
    attrs
}

fn reset_stale_attrs(
    node: &mut ArkUINode,
    previous: &[ArkUINodeAttributeType],
    next: &[ArkUINodeAttributeType],
) {
    let runtime = RuntimeNode(node);
    for attr in previous {
        if !next.contains(attr) {
            let _ = runtime.reset_attribute(*attr);
        }
    }
}

fn apply_event_handlers(node: &mut ArkUINode, handlers: &[EventHandlerSpec]) {
    let mut runtime = RuntimeNode(node);
    for (event_type, callbacks) in grouped_event_handlers(handlers) {
        runtime.on_event(event_type, move |event| {
            for callback in &callbacks {
                callback(event);
            }
        });
    }
}

fn grouped_event_handlers(
    handlers: &[EventHandlerSpec],
) -> Vec<(NodeEventType, Vec<EventCallback>)> {
    let mut groups = Vec::<(NodeEventType, Vec<EventCallback>)>::new();
    for handler in handlers {
        if let Some((_, callbacks)) = groups
            .iter_mut()
            .find(|(event_type, _)| *event_type == handler.event_type)
        {
            callbacks.push(handler.callback.clone());
        } else {
            groups.push((handler.event_type, vec![handler.callback.clone()]));
        }
    }
    groups
}

fn event_types(handlers: &[EventHandlerSpec]) -> Vec<NodeEventType> {
    let mut events = Vec::new();
    for handler in handlers {
        if !events.contains(&handler.event_type) {
            events.push(handler.event_type);
        }
    }
    events
}

#[derive(Clone, PartialEq, Eq)]
struct NodeSignature {
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
}

fn node_signature<Message, AppTheme>(node: &Node<Message, AppTheme>) -> NodeSignature {
    NodeSignature {
        events: event_types(&node.event_handlers),
        mount_effect_count: node.mount_effects.len(),
        attach_effect_count: node.attach_effects.len(),
        patch_effect_count: node.patch_effects.len(),
        has_long_press: node.long_press_handler.is_some(),
    }
}

fn mounted_signature(mounted: &MountedRenderNode) -> NodeSignature {
    NodeSignature {
        events: mounted.events.clone(),
        mount_effect_count: mounted.mount_effect_count,
        attach_effect_count: mounted.attach_effect_count,
        patch_effect_count: mounted.patch_effect_count,
        has_long_press: mounted.has_long_press,
    }
}

fn clear_removed_events(node: &mut ArkUINode, previous: &[NodeEventType], next: &[NodeEventType]) {
    let mut runtime = RuntimeNode(node);
    for event_type in previous {
        if !next.contains(event_type) {
            runtime.on_event(*event_type, |_| {});
        }
    }
}

fn mount_long_press(
    node: &mut ArkUINode,
    handler: &LongPressHandlerSpec,
) -> ArkUIResult<(Option<Cleanup>, Rc<RefCell<Rc<dyn Fn()>>>)> {
    let gesture = Gesture::create_long_gesture(1, true, DEFAULT_LONG_PRESS_DURATION_MS)?;
    let callback_state = Rc::new(RefCell::new(handler.callback.clone()));
    let callback_data = Box::into_raw(Box::new(LongPressCallbackContext {
        callback: callback_state.clone(),
    }));

    if let Err(error) = gesture.on_gesture_with_data(
        GestureEventAction::Accept | GestureEventAction::Update | GestureEventAction::End,
        callback_data.cast(),
        long_press_gesture_callback,
    ) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let runtime = RuntimeNode(node);
    if let Err(error) = runtime.add_gesture_ref(&gesture, None, None) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let mut cleanup_node = node.clone();
    let cleanup = Box::new(move || {
        let runtime = RuntimeNode(&mut cleanup_node);
        let _ = runtime.remove_gesture(&gesture);
        let _ = gesture.dispose();
        unsafe {
            drop(Box::from_raw(callback_data));
        }
    }) as Cleanup;
    Ok((Some(cleanup), callback_state))
}

fn attach_child(parent: &mut ArkUINode, child: ArkUINode) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.add_child(child)
}

fn insert_child(parent: &mut ArkUINode, child: ArkUINode, index: usize) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.insert_child(child, index)
}

fn attach_child_at(parent: &mut ArkUINode, child: ArkUINode, index: usize) -> ArkUIResult<()> {
    if index == parent.children().len() {
        attach_child(parent, child)
    } else {
        insert_child(parent, child, index)
    }
}

fn remove_child_by_raw(
    parent: &mut ArkUINode,
    raw_handle: usize,
) -> ArkUIResult<Option<Rc<RefCell<ArkUINode>>>> {
    let index = parent
        .children()
        .iter()
        .position(|child| child.borrow().raw_handle() as usize == raw_handle);

    match index {
        Some(index) => parent.remove_child(index),
        None => Ok(None),
    }
}

fn complete_exiting_child(
    mut parent: ArkUINode,
    raw_handle: usize,
    mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
    alive: Rc<Cell<bool>>,
) {
    if !alive.replace(false) {
        return;
    }

    pending_exits
        .borrow_mut()
        .retain(|exit| exit.raw_handle != raw_handle);

    match remove_child_by_raw(&mut parent, raw_handle) {
        Ok(Some(removed)) => {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        Ok(None) => {}
        Err(error) => {
            ohos_hilog_binding::error(format!(
                "renderer error: failed to remove exiting child: {error}"
            ));
        }
    }

    if let Some(mounted) = mounted.borrow_mut().take() {
        mounted.cleanup_recursive();
    }
}

fn remove_or_exit_child(
    parent: &mut ArkUINode,
    index: usize,
    mut mounted: MountedRenderNode,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
) -> ArkUIResult<()> {
    let Some(exit_effect) = mounted.exit_effect.take() else {
        let removed = parent.remove_child(index)?;
        if let Some(removed) = removed {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        mounted.cleanup_recursive();
        return Ok(());
    };

    let Some(child_handle) = parent.children().get(index).cloned() else {
        mounted.cleanup_recursive();
        return Ok(());
    };

    let raw_handle = child_handle.borrow().raw_handle() as usize;
    let alive = Rc::new(Cell::new(true));
    let mounted_slot = Rc::new(RefCell::new(Some(mounted)));
    let effect_cleanup = Rc::new(RefCell::new(None::<Cleanup>));

    pending_exits.borrow_mut().push(PendingExit {
        raw_handle,
        alive: alive.clone(),
        mounted: mounted_slot.clone(),
        effect_cleanup: effect_cleanup.clone(),
    });

    let finish_parent = parent.clone();
    let finish_mounted = mounted_slot.clone();
    let finish_pending = pending_exits.clone();
    let finish_alive = alive.clone();
    let finish = Box::new(move || {
        complete_exiting_child(
            finish_parent,
            raw_handle,
            finish_mounted,
            finish_pending,
            finish_alive,
        );
    }) as Cleanup;

    let mut child_node = child_handle.borrow_mut();
    if let Err(error) = child_node.set_attribute(
        ArkUINodeAttributeType::HitTestBehavior,
        i32::from(HitTestBehavior::None).into(),
    ) {
        ohos_hilog_binding::error(format!(
            "renderer error: failed to disable exiting child hit test: {error}"
        ));
    }

    match exit_effect(&mut child_node, finish) {
        Ok(cleanup) => {
            effect_cleanup.replace(cleanup);
        }
        Err(error) => {
            ohos_hilog_binding::error(format!("renderer error: exit effect failed: {error}"));
            drop(child_node);
            complete_exiting_child(
                parent.clone(),
                raw_handle,
                mounted_slot,
                pending_exits,
                alive,
            );
        }
    }

    Ok(())
}

fn realize_attached_node(node: &mut ArkUINode, mounted: &mut MountedRenderNode) -> ArkUIResult<()> {
    if !mounted.pending_patch_attrs.is_empty() {
        apply_attr_list(node, std::mem::take(&mut mounted.pending_patch_attrs));
    }

    for effect in std::mem::take(&mut mounted.pending_attach_effects) {
        match effect(node)? {
            Some(cleanup) => mounted.cleanups.push(cleanup),
            None => {}
        }
    }

    for effect in std::mem::take(&mut mounted.pending_patch_effects) {
        effect(node)?;
    }

    for (child_handle, child_mounted) in node.children().iter().zip(mounted.children.iter_mut()) {
        let mut child_node = child_handle.borrow_mut();
        realize_attached_node(&mut child_node, child_mounted)?;
    }

    Ok(())
}

fn mount_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedRenderNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        persistent_key: _,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects,
        attach_effects,
        patch_effects,
        exit_effect,
        state_bound: _,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = prepare_node(into_node(element));

    let mut node = create_node(kind)?;
    let init_attr_keys = attr_types(&init_attrs);
    let pending_patch_attrs = desired_attrs(Vec::new(), patch_attrs);
    let final_attr_keys = desired_attr_types(&init_attrs, &pending_patch_attrs);
    let mount_effect_count = mount_effects.len();
    let attach_effect_count = attach_effects.len();
    let patch_effect_count = patch_effects.len();
    let has_long_press = long_press_handler.is_some();
    apply_attr_list(&mut node, init_attrs);

    let mut cleanups = Vec::new();
    for effect in mount_effects {
        match effect(&mut node) {
            Ok(Some(cleanup)) => cleanups.push(cleanup),
            Ok(None) => {}
            Err(error) => {
                run_cleanups(cleanups);
                let _ = node.dispose();
                return Err(error);
            }
        }
    }

    apply_event_handlers(&mut node, &event_handlers);
    let events = event_types(&event_handlers);
    let (long_press_cleanup, long_press_callback) = match long_press_handler.as_ref() {
        Some(handler) => {
            let (cleanup, callback) = mount_long_press(&mut node, handler)?;
            (cleanup, Some(callback))
        }
        None => (None, None),
    };

    let mut mounted_children = Vec::with_capacity(children.len());
    for child in children {
        let (child_node, child_mounted) = mount_node(child)?;
        attach_child(&mut node, child_node)?;
        mounted_children.push(child_mounted);
    }

    Ok((
        node,
        MountedRenderNode::new(
            node_type_id(kind),
            key,
            if pending_patch_attrs.is_empty() && patch_effect_count == 0 {
                init_attr_keys
            } else {
                final_attr_keys
            },
            events,
            mount_effect_count,
            attach_effect_count,
            patch_effect_count,
            has_long_press,
            long_press_cleanup,
            long_press_callback,
            cleanups,
            exit_effect,
            pending_patch_attrs,
            attach_effects,
            patch_effects,
            mounted_children,
        ),
    ))
}

fn patch_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        persistent_key: _,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects: _,
        attach_effects: _,
        patch_effects,
        exit_effect,
        state_bound: _,
        #[cfg(feature = "webview")]
            webview: _,
        children,
    } = prepare_node(into_node(element));

    mounted.tag = node_type_id(kind);
    mounted.key = key;
    mounted.exit_effect = exit_effect;
    let attrs = desired_attrs(init_attrs, patch_attrs);
    let next_attr_types = attr_types(&attrs);
    reset_stale_attrs(node, &mounted.attrs, &next_attr_types);
    apply_attr_list(node, attrs);
    mounted.attrs = next_attr_types;
    for effect in patch_effects {
        effect(node)?;
    }

    let next_events = event_types(&event_handlers);
    clear_removed_events(node, &mounted.events, &next_events);
    apply_event_handlers(node, &event_handlers);
    mounted.events = next_events;

    match (
        long_press_handler.as_ref(),
        mounted.long_press_callback.as_ref(),
    ) {
        (Some(handler), Some(callback)) => {
            callback.replace(handler.callback.clone());
        }
        (Some(handler), None) => {
            let (cleanup, callback) = mount_long_press(node, handler)?;
            mounted.long_press_cleanup = cleanup;
            mounted.long_press_callback = Some(callback);
        }
        (None, Some(_)) => {
            if let Some(cleanup) = mounted.long_press_cleanup.take() {
                cleanup();
            }
            mounted.long_press_callback = None;
        }
        (None, None) => {}
    }
    mounted.has_long_press = long_press_handler.is_some();

    reconcile_children(node, mounted, children)
}

fn reconcile_children<Message, AppTheme>(
    parent: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
    next_children: Vec<Element<Message, AppTheme>>,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    type ChildKey = (TypeId, String);

    fn child_key(mounted: &MountedRenderNode) -> Option<ChildKey> {
        mounted.key.clone().map(|key| (mounted.tag, key))
    }

    fn can_reuse<Message, AppTheme>(
        next: &Node<Message, AppTheme>,
        mounted: &MountedRenderNode,
    ) -> bool {
        node_type_id(next.kind) == mounted.tag && node_signature(next) == mounted_signature(mounted)
    }

    let mut next_nodes = Vec::with_capacity(next_children.len());
    for child in next_children {
        let node = into_node(child);
        next_nodes.push(node);
    }

    let next_len = next_nodes.len();
    let pending_exits = mounted.exiting_children.clone();
    let mounted_children = &mut mounted.children;
    let old_len = mounted_children.len();
    let mut prefix = 0;

    while prefix < next_len && prefix < old_len {
        let next_key = match next_nodes[prefix].key.clone() {
            Some(key) => Some((node_type_id(next_nodes[prefix].kind), key)),
            None => None,
        };
        let current_key = child_key(&mounted_children[prefix]);
        let matches = if next_key.is_none() && current_key.is_none() {
            can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        } else {
            next_key == current_key && can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        };
        if !matches {
            break;
        }
        prefix += 1;
    }

    for i in 0..prefix {
        let child_handle = parent.children()[i].clone();
        let mut child_node = child_handle.borrow_mut();
        patch_node(
            next_nodes.remove(0).into(),
            &mut child_node,
            &mut mounted_children[i],
        )?;
    }

    if prefix == old_len && prefix == next_len {
        return Ok(());
    }

    for (offset, child) in next_nodes.into_iter().enumerate() {
        let index = prefix + offset;
        let (child_node, mut child_meta) = mount_node(child.into())?;
        attach_child_at(parent, child_node, index)?;
        if let Some(child_handle) = parent.children().get(index) {
            let mut child_node = child_handle.borrow_mut();
            realize_attached_node(&mut child_node, &mut child_meta)?;
        }
        mounted_children.insert(index, child_meta);
    }

    while mounted_children.len() > next_len {
        let index = mounted_children.len() - 1;
        let mounted = mounted_children.remove(index);
        remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
    }

    Ok(())
}

pub fn mount<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let mut state_cache = StateCache::default();
    let compiled = compile_element(
        element,
        &mut tree,
        &mut state_cache,
        &Renderer::default(),
        true,
    );
    let root = compose_compiled_overlays(compiled);
    let (node, render) = mount_node(root)?;
    Ok((node, MountedNode::new(tree, render)))
}

pub fn realize_attached_mount(node: &mut ArkUINode, mounted: &mut MountedNode) -> ArkUIResult<()> {
    realize_attached_node(node, mounted.render_mut())
}

pub fn patch<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedNode,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let MountedNode {
        tree,
        retained_state,
        render,
    } = mounted;
    sync_element_tree(&element, tree, retained_state);
    let compiled = compile_element(element, tree, retained_state, &Renderer::default(), true);
    let root = compose_compiled_overlays(compiled);
    patch_node(root, node, render)
}

impl<Message, AppTheme> From<Node<Message, AppTheme>> for Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    fn from(value: Node<Message, AppTheme>) -> Self {
        arkit_core::Element::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desired_attrs_preserve_last_set_order() {
        let attrs = desired_attrs(
            vec![
                (ArkUINodeAttributeType::Width, 10.0_f32.into()),
                (ArkUINodeAttributeType::Height, 20.0_f32.into()),
                (ArkUINodeAttributeType::Width, 30.0_f32.into()),
            ],
            vec![
                (
                    ArkUINodeAttributeType::BackgroundColor,
                    0xFF000000_u32.into(),
                ),
                (ArkUINodeAttributeType::Height, 40.0_f32.into()),
            ],
        );

        assert_eq!(
            attr_types(&attrs),
            vec![
                ArkUINodeAttributeType::Width,
                ArkUINodeAttributeType::BackgroundColor,
                ArkUINodeAttributeType::Height,
            ]
        );
    }

    #[test]
    fn scalar_edges_expand_to_explicit_edges() {
        assert_eq!(6.0_f32.edge_values(), vec![6.0, 6.0, 6.0, 6.0]);
    }

    #[test]
    fn visual_clipping_attrs_are_applied_after_size_and_background() {
        let attrs = ordered_attrs_for_application(vec![
            (ArkUINodeAttributeType::BorderRadius, 6.0_f32.into()),
            (ArkUINodeAttributeType::Clip, true.into()),
            (ArkUINodeAttributeType::Height, 40.0_f32.into()),
            (ArkUINodeAttributeType::Padding, vec![0.0_f32, 8.0].into()),
            (
                ArkUINodeAttributeType::BackgroundColor,
                0xFF000000_u32.into(),
            ),
        ]);

        assert_eq!(
            attr_types(&attrs),
            vec![
                ArkUINodeAttributeType::Height,
                ArkUINodeAttributeType::Padding,
                ArkUINodeAttributeType::BackgroundColor,
                ArkUINodeAttributeType::BorderRadius,
                ArkUINodeAttributeType::Clip,
            ]
        );
    }

    #[test]
    fn button_component_uses_pressable_surface_host() {
        let node = button_component::<(), arkit_core::Theme>();

        assert_eq!(node.kind, NodeKind::Stack);
        assert_eq!(node.attr_f32(ArkUINodeAttributeType::ButtonType), None);
    }

    #[test]
    fn button_component_keeps_border_radius_as_surface_style() {
        let node = button_component::<(), arkit_core::Theme>().border_radius(8.0);

        assert_eq!(node.kind, NodeKind::Stack);
        assert_eq!(node.attr_f32(ArkUINodeAttributeType::ButtonType), None);
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::BorderRadius),
            Some(8.0)
        );
    }

    #[test]
    fn label_button_uses_native_button() {
        let node = button::<(), arkit_core::Theme>("OK").border_radius(8.0);

        assert_eq!(node.kind, NodeKind::Button);
        assert_eq!(
            node.attr_string(ArkUINodeAttributeType::ButtonLabel),
            Some("OK")
        );
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::BorderRadius),
            Some(8.0)
        );
    }

    #[test]
    fn list_component_uses_native_list() {
        let node = list_component::<(), arkit_core::Theme>();

        assert_eq!(node.kind, NodeKind::List);
    }

    #[test]
    fn list_item_component_uses_native_list_item() {
        let node = list_item_component::<(), arkit_core::Theme>();

        assert_eq!(node.kind, NodeKind::ListItem);
    }

    #[test]
    fn refresh_component_sets_refresh_attributes() {
        let node = refresh_component::<(), arkit_core::Theme>()
            .refreshing(true)
            .refresh_offset(72.0)
            .refresh_pull_to_refresh(false);

        assert_eq!(
            node.attr_bool(ArkUINodeAttributeType::RefreshRefreshing),
            Some(true)
        );
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::RefreshOffset),
            Some(72.0)
        );
        assert_eq!(
            node.attr_bool(ArkUINodeAttributeType::RefreshPullToRefresh),
            Some(false)
        );
    }

    #[test]
    fn text_input_font_size_sets_placeholder_font_size() {
        let node = text_input_component::<(), arkit_core::Theme>().font_size(14.0);

        assert_eq!(node.attr_f32(ArkUINodeAttributeType::FontSize), Some(14.0));
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::TextInputPlaceholderFont),
            Some(14.0)
        );
    }

    #[test]
    fn text_area_font_size_sets_placeholder_font_size() {
        let node = text_area_component::<(), arkit_core::Theme>().font_size(13.0);

        assert_eq!(node.attr_f32(ArkUINodeAttributeType::FontSize), Some(13.0));
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::TextAreaPlaceholderFont),
            Some(13.0)
        );
    }

    #[cfg(feature = "webview")]
    #[test]
    fn web_view_component_uses_dedicated_host_kind() {
        let controller = WebViewController::with_id("webview-test");
        let node = web_view_component::<(), arkit_core::Theme>(controller.clone())
            .javascript_enabled(true)
            .devtools(true);

        assert_eq!(node.kind, NodeKind::WebViewHost);
        let spec = node.webview.as_ref().expect("webview spec should exist");
        assert_eq!(
            spec.controller
                .as_ref()
                .expect("controller should be present")
                .id(),
            "webview-test"
        );
        assert_eq!(spec.javascript_enabled, Some(true));
        assert_eq!(spec.devtools, Some(true));
    }

    #[cfg(feature = "webview")]
    #[test]
    fn web_view_component_uses_transparent_hit_testing() {
        let controller = WebViewController::with_id("webview-transparent");
        let node = web_view_component::<(), arkit_core::Theme>(controller);

        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::HitTestBehavior),
            Some(i32::from(HitTestBehavior::Transparent) as f32)
        );
    }

    #[cfg(feature = "webview")]
    #[test]
    fn web_view_component_clips_to_host_bounds() {
        let controller = WebViewController::with_id("webview-clipped");
        let node = web_view_component::<(), arkit_core::Theme>(controller);

        assert_eq!(node.attr_bool(ArkUINodeAttributeType::Clip), Some(true));
    }

    #[cfg(feature = "webview")]
    #[test]
    fn build_initial_webview_style_only_preserves_explicit_style_fields() {
        let style = build_initial_webview_style(
            &WebViewSpec::default(),
            Some(LayoutFrame {
                x: 12.0,
                y: 34.0,
                width: 200.0,
                height: 120.0,
            }),
        )
        .expect("style should be created");

        assert!(style.x.is_none());
        assert!(style.y.is_none());
        assert!(style.background_color.is_none());
        assert!(style.visible.is_none());
    }

    #[cfg(feature = "webview")]
    #[test]
    fn compose_compiled_overlays_skips_wrapper_without_overlays() {
        let node = into_node(compose_compiled_overlays(CompiledElement::<
            (),
            arkit_core::Theme,
        > {
            body: column_component::<(), arkit_core::Theme>().into(),
            overlays: Vec::new(),
        }));

        assert_eq!(node.kind, NodeKind::Column);
    }

    #[cfg(feature = "webview")]
    #[test]
    fn compose_compiled_overlays_keeps_stack_wrapper_with_overlays() {
        let node = into_node(compose_compiled_overlays(CompiledElement::<
            (),
            arkit_core::Theme,
        > {
            body: column_component::<(), arkit_core::Theme>().into(),
            overlays: vec![text::<(), arkit_core::Theme>("overlay").into()],
        }));

        assert_eq!(node.kind, NodeKind::Stack);
    }

    #[cfg(feature = "webview")]
    #[test]
    fn web_view_constructor_defaults_to_fill_and_url() {
        let controller = WebViewController::with_id("webview-fill");
        let node = web_view::<(), arkit_core::Theme>(controller, "https://example.com");

        assert_eq!(node.kind, NodeKind::WebViewHost);
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::WidthPercent),
            Some(1.0)
        );
        assert_eq!(
            node.attr_f32(ArkUINodeAttributeType::HeightPercent),
            Some(1.0)
        );
        assert_eq!(
            node.webview.as_ref().and_then(|spec| spec.url.as_deref()),
            Some("https://example.com")
        );
    }
}
