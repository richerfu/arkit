use super::*;

pub(super) fn apply_attr_list(
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

pub(super) fn ordered_attrs_for_application(
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

pub(super) fn run_cleanups(mut cleanups: Vec<Cleanup>) {
    while let Some(cleanup) = cleanups.pop() {
        cleanup();
    }
}

pub(super) fn wrap_component<T>(node: ArkUINode) -> T {
    assert_eq!(size_of::<T>(), size_of::<ArkUINode>());
    assert_eq!(align_of::<T>(), align_of::<ArkUINode>());
    let node = ManuallyDrop::new(node);
    unsafe { std::ptr::read((&*node as *const ArkUINode).cast::<T>()) }
}

pub(super) fn create_node(kind: NodeKind) -> ArkUIResult<ArkUINode> {
    Ok(match kind {
        NodeKind::Retained => {
            panic!("retained renderer node cannot be mounted")
        }
        NodeKind::Button => Button::new()?.into(),
        NodeKind::CalendarPicker => CalendarPicker::new()?.into(),
        NodeKind::Checkbox => Checkbox::new()?.into(),
        NodeKind::Column => Column::new()?.into(),
        NodeKind::DatePicker => DatePicker::new()?.into(),
        NodeKind::Flex => Flex::new()?.into(),
        NodeKind::FlowItem => FlowItem::new()?.into(),
        NodeKind::Grid => Grid::new()?.into(),
        NodeKind::GridItem => GridItem::new()?.into(),
        NodeKind::Image => Image::new()?.into(),
        NodeKind::List => List::new()?.into(),
        NodeKind::ListItem => ListItem::new()?.into(),
        NodeKind::ListItemGroup => ListItemGroup::new()?.into(),
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
        NodeKind::WaterFlow => WaterFlow::new()?.into(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => Stack::new()?.into(),
    })
}

pub(super) fn node_type_id(kind: NodeKind) -> TypeId {
    match kind {
        NodeKind::Retained => TypeId::of::<RetainedNodeTag>(),
        NodeKind::Button => TypeId::of::<Button>(),
        NodeKind::CalendarPicker => TypeId::of::<CalendarPicker>(),
        NodeKind::Checkbox => TypeId::of::<Checkbox>(),
        NodeKind::Column => TypeId::of::<Column>(),
        NodeKind::DatePicker => TypeId::of::<DatePicker>(),
        NodeKind::Flex => TypeId::of::<Flex>(),
        NodeKind::FlowItem => TypeId::of::<FlowItem>(),
        NodeKind::Grid => TypeId::of::<Grid>(),
        NodeKind::GridItem => TypeId::of::<GridItem>(),
        NodeKind::Image => TypeId::of::<Image>(),
        NodeKind::List => TypeId::of::<List>(),
        NodeKind::ListItem => TypeId::of::<ListItem>(),
        NodeKind::ListItemGroup => TypeId::of::<ListItemGroup>(),
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
        NodeKind::WaterFlow => TypeId::of::<WaterFlow>(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => TypeId::of::<WebViewHostNodeTag>(),
    }
}

pub(super) struct ButtonNodeTag;
pub(super) struct RetainedNodeTag;
pub(super) struct CalendarPickerNodeTag;
pub(super) struct CheckboxNodeTag;
pub(super) struct ColumnNodeTag;
pub(super) struct DatePickerNodeTag;
pub(super) struct FlexNodeTag;
pub(super) struct FlowItemNodeTag;
pub(super) struct GridNodeTag;
pub(super) struct GridItemNodeTag;
pub(super) struct ImageNodeTag;
pub(super) struct ListNodeTag;
pub(super) struct ListItemNodeTag;
pub(super) struct ListItemGroupNodeTag;
pub(super) struct ProgressNodeTag;
pub(super) struct RadioNodeTag;
pub(super) struct RefreshNodeTag;
pub(super) struct RowNodeTag;
pub(super) struct ScrollNodeTag;
pub(super) struct SliderNodeTag;
pub(super) struct StackNodeTag;
pub(super) struct SwiperNodeTag;
pub(super) struct TextNodeTag;
pub(super) struct TextAreaNodeTag;
pub(super) struct TextInputNodeTag;
pub(super) struct ToggleNodeTag;
pub(super) struct WaterFlowNodeTag;
#[cfg(feature = "webview")]
pub(super) struct WebViewHostNodeTag;

pub(super) fn node_widget_tag(kind: NodeKind) -> advanced::widget::Tag {
    match kind {
        NodeKind::Retained => advanced::widget::Tag::of::<RetainedNodeTag>(),
        NodeKind::Button => advanced::widget::Tag::of::<ButtonNodeTag>(),
        NodeKind::CalendarPicker => advanced::widget::Tag::of::<CalendarPickerNodeTag>(),
        NodeKind::Checkbox => advanced::widget::Tag::of::<CheckboxNodeTag>(),
        NodeKind::Column => advanced::widget::Tag::of::<ColumnNodeTag>(),
        NodeKind::DatePicker => advanced::widget::Tag::of::<DatePickerNodeTag>(),
        NodeKind::Flex => advanced::widget::Tag::of::<FlexNodeTag>(),
        NodeKind::FlowItem => advanced::widget::Tag::of::<FlowItemNodeTag>(),
        NodeKind::Grid => advanced::widget::Tag::of::<GridNodeTag>(),
        NodeKind::GridItem => advanced::widget::Tag::of::<GridItemNodeTag>(),
        NodeKind::Image => advanced::widget::Tag::of::<ImageNodeTag>(),
        NodeKind::List => advanced::widget::Tag::of::<ListNodeTag>(),
        NodeKind::ListItem => advanced::widget::Tag::of::<ListItemNodeTag>(),
        NodeKind::ListItemGroup => advanced::widget::Tag::of::<ListItemGroupNodeTag>(),
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
        NodeKind::WaterFlow => advanced::widget::Tag::of::<WaterFlowNodeTag>(),
        #[cfg(feature = "webview")]
        NodeKind::WebViewHost => advanced::widget::Tag::of::<WebViewHostNodeTag>(),
    }
}
