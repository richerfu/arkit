use super::*;

pub fn button_component<Message, AppTheme>() -> ButtonElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Stack))
}

pub fn button<Message, AppTheme>(label: impl Into<String>) -> ButtonElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Button).label(label))
}

pub fn text_component<Message, AppTheme>() -> TextElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Text))
}

pub fn text<Message, AppTheme>(content: impl Into<String>) -> TextElement<Message, AppTheme> {
    text_component().content(content)
}

pub fn text_input_component<Message, AppTheme>() -> TextInputElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::TextInput))
}

pub fn text_input<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> TextInputElement<Message, AppTheme> {
    text_input_component().placeholder(placeholder).value(value)
}

pub fn text_area_component<Message, AppTheme>() -> TextAreaElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::TextArea))
}

pub fn text_area<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> TextAreaElement<Message, AppTheme> {
    text_area_component().placeholder(placeholder).value(value)
}

pub fn checkbox_component<Message, AppTheme>() -> CheckboxElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Checkbox))
}

pub fn checkbox<Message, AppTheme>(checked: bool) -> CheckboxElement<Message, AppTheme> {
    checkbox_component().checked(checked)
}

pub fn toggle_component<Message, AppTheme>() -> ToggleElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Toggle))
}

pub fn toggle<Message, AppTheme>(checked: bool) -> ToggleElement<Message, AppTheme> {
    toggle_component().checked(checked)
}

pub fn radio_component<Message, AppTheme>() -> RadioElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Radio))
}

pub fn radio<Message, AppTheme>(checked: bool) -> RadioElement<Message, AppTheme> {
    radio_component().checked(checked)
}

pub fn slider_component<Message, AppTheme>() -> SliderElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Slider))
}

pub fn slider<Message, AppTheme>(
    value: f32,
    min: f32,
    max: f32,
) -> SliderElement<Message, AppTheme> {
    slider_component()
        .attr(ArkUINodeAttributeType::SliderValue, value)
        .patch_attr(ArkUINodeAttributeType::SliderValue, value)
        .range(min, max)
}

pub fn progress_component<Message, AppTheme>() -> ProgressElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Progress))
}

pub fn progress<Message, AppTheme>(value: f32, total: f32) -> ProgressElement<Message, AppTheme> {
    progress_component()
        .attr(ArkUINodeAttributeType::ProgressValue, value)
        .patch_attr(ArkUINodeAttributeType::ProgressValue, value)
        .range(0.0, total)
}

pub fn image_component<Message, AppTheme>() -> ImageElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Image))
}

pub fn image<Message, AppTheme>(src: impl Into<String>) -> ImageElement<Message, AppTheme> {
    let src = src.into();
    image_component()
        .attr(ArkUINodeAttributeType::ImageSrc, src.clone())
        .patch_attr(ArkUINodeAttributeType::ImageSrc, src)
}

pub fn list_component<Message, AppTheme>() -> ListElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::List))
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

pub fn list_item_component<Message, AppTheme>() -> ListItemElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::ListItem))
}

pub fn list_item<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    list_item_component().child(child.into()).into()
}

pub fn list_item_group_component<Message, AppTheme>() -> ListItemGroupElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::ListItemGroup))
}

pub fn grid_component<Message, AppTheme>() -> GridElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Grid))
}

pub fn grid<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    grid_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub fn grid_item_component<Message, AppTheme>() -> GridItemElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::GridItem))
}

pub fn grid_item<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    grid_item_component().child(child.into()).into()
}

pub fn water_flow_component<Message, AppTheme>() -> WaterFlowElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::WaterFlow))
}

pub fn flow_item_component<Message, AppTheme>() -> FlowItemElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::FlowItem))
}

pub fn flow_item<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    flow_item_component().child(child.into()).into()
}

pub fn virtual_list_component<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> ListElement<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    Component::from_node(
        Node::new(NodeKind::List)
            .percent_width(1.0)
            .percent_height(1.0)
            .virtual_adapter(
                VirtualContainerKind::List,
                total_count,
                Rc::new(render_item),
            ),
    )
}

pub fn virtual_list<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> Element<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    virtual_list_component(total_count, render_item).into()
}

pub fn virtual_grid_component<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> GridElement<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    Component::from_node(
        Node::new(NodeKind::Grid)
            .percent_width(1.0)
            .percent_height(1.0)
            .virtual_adapter(
                VirtualContainerKind::Grid,
                total_count,
                Rc::new(render_item),
            ),
    )
}

pub fn virtual_grid<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> Element<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    virtual_grid_component(total_count, render_item).into()
}

pub fn virtual_water_flow_component<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> WaterFlowElement<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    Component::from_node(
        Node::new(NodeKind::WaterFlow)
            .percent_width(1.0)
            .percent_height(1.0)
            .virtual_adapter(
                VirtualContainerKind::WaterFlow,
                total_count,
                Rc::new(render_item),
            ),
    )
}

pub fn virtual_water_flow<Message, AppTheme, F>(
    total_count: u32,
    render_item: F,
) -> Element<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    F: Fn(u32) -> Element<Message, AppTheme> + 'static,
{
    virtual_water_flow_component(total_count, render_item).into()
}

pub fn grouped_virtual_list<Message, AppTheme, Header, Item>(
    groups: Vec<VirtualListGroup>,
    render_header: Header,
    render_item: Item,
) -> Element<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
    Header: Fn(u32) -> Element<Message, AppTheme> + 'static,
    Item: Fn(u32, u32) -> Element<Message, AppTheme> + 'static,
{
    let render_header = Rc::new(render_header);
    let render_item = Rc::new(render_item);
    let children = groups
        .into_iter()
        .enumerate()
        .map(|(group_index, group)| {
            let group_index = group_index as u32;
            let header = render_header(group_index);
            let render_item = render_item.clone();
            list_item_group_component()
                .key(group.key)
                .list_item_group_header(header)
                .virtual_adapter(
                    VirtualContainerKind::ListItemGroup,
                    group.item_count,
                    Rc::new(move |item_index| render_item(group_index, item_index)),
                )
                .into()
        })
        .collect();

    list_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .list_sticky(ListStickyStyle::Header)
        .children(children)
        .into()
}

pub fn column_component<Message, AppTheme>() -> ColumnElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Column))
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

pub fn row_component<Message, AppTheme>() -> RowElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Row))
}

pub fn row<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    row_component().percent_width(1.0).children(children).into()
}

pub fn stack_component<Message, AppTheme>() -> StackElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Stack))
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

pub type Container<Message = (), AppTheme = arkit_core::Theme> =
    ContainerElement<Message, AppTheme>;

pub fn container<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Container<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Column).child(child.into()))
}

pub fn scroll_component<Message, AppTheme>() -> ScrollElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Scroll))
}

pub fn scroll<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    scroll_component().child(child.into()).into()
}

pub fn refresh_component<Message, AppTheme>() -> RefreshElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Refresh))
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

pub fn swiper_component<Message, AppTheme>() -> SwiperElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::Swiper))
}

pub fn swiper<Message, AppTheme>(
    children: Vec<Element<Message, AppTheme>>,
) -> SwiperElement<Message, AppTheme> {
    swiper_component().children(children)
}

pub fn calendar_picker_component<Message, AppTheme>() -> CalendarPickerElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::CalendarPicker))
}

pub fn calendar_picker<Message, AppTheme>() -> CalendarPickerElement<Message, AppTheme> {
    calendar_picker_component()
}

pub fn date_picker_component<Message, AppTheme>() -> DatePickerElement<Message, AppTheme> {
    Component::from_node(Node::new(NodeKind::DatePicker))
}

pub fn date_picker<Message, AppTheme>() -> DatePickerElement<Message, AppTheme> {
    date_picker_component()
}
