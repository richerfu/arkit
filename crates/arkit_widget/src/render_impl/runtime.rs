use super::*;

pub(super) fn font_weight_value(value: FontWeight) -> i32 {
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

pub(super) fn shadow_style_value(value: ShadowStyle) -> i32 {
    match value {
        ShadowStyle::OuterDefaultXs => 0,
        ShadowStyle::OuterDefaultSm => 1,
        ShadowStyle::OuterDefaultMd => 2,
        ShadowStyle::OuterDefaultLg => 3,
        ShadowStyle::OuterFloatingSm => 4,
        ShadowStyle::OuterFloatingMd => 5,
    }
}

pub(super) fn apply_progress_linear_style(
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

pub(super) fn apply_scroll_offset(node: &mut ArkUINode, offset: ScrollOffset) -> ArkUIResult<()> {
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

pub(super) fn read_scroll_offset(node: &mut ArkUINode) -> Option<ScrollOffset> {
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

pub(super) fn attribute_number_as_f32(value: &ArkUINodeAttributeNumber) -> Option<f32> {
    match value {
        ArkUINodeAttributeNumber::Float(value) => Some(*value),
        ArkUINodeAttributeNumber::Int(value) => Some(*value as f32),
        ArkUINodeAttributeNumber::Uint(value) => Some(*value as f32),
    }
}

pub(super) struct RuntimeNode<'a>(pub(super) &'a mut ArkUINode);

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
pub(super) struct EventHandlerSpec {
    pub(super) event_type: NodeEventType,
    pub(super) callback: EventCallback,
}

#[derive(Clone)]
pub(super) struct LongPressHandlerSpec {
    pub(super) callback: Rc<dyn Fn()>,
}

pub(super) struct LongPressCallbackContext {
    pub(super) callback: Rc<RefCell<Rc<dyn Fn()>>>,
}

pub(super) fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::from("non-string panic payload")
    }
}

pub(super) fn run_guarded_ui_callback(error_label: &'static str, callback: impl FnOnce()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(callback)) {
        ohos_hilog_binding::error(format!(
            "{error_label}: {}",
            panic_payload_message(payload.as_ref())
        ));
    }
}

pub(super) fn long_press_gesture_callback(event: GestureEventData) {
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
pub(super) enum NodeKind {
    Button,
    CalendarPicker,
    Checkbox,
    Column,
    DatePicker,
    FlowItem,
    Grid,
    GridItem,
    Image,
    List,
    ListItem,
    ListItemGroup,
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
    WaterFlow,
    #[cfg(feature = "webview")]
    WebViewHost,
}
