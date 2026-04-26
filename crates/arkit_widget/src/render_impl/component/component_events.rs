use super::*;

impl<Message, AppTheme, Kind> Component<Message, AppTheme, Kind> {
    pub fn on_press(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.map_node(|node| node.on_press(message))
    }

    pub fn on_click(self, callback: impl Fn() + 'static) -> Self {
        self.map_node(|node| node.on_click(callback))
    }

    pub fn on_event(
        self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        self.map_node(|node| node.on_event(event_type, callback))
    }

    pub fn on_event_no_param(
        self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        self.map_node(|node| node.on_event_no_param(event_type, callback))
    }

    pub fn on_supported_ui_states(
        self,
        ui_states: UiState,
        exclude_inner: bool,
        callback: impl Fn(&mut ArkUINode, UiState) + 'static,
    ) -> Self {
        self.map_node(|node| node.on_supported_ui_states(ui_states, exclude_inner, callback))
    }

    pub fn on_custom_event(
        self,
        event_type: NodeCustomEventType,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.map_node(|node| node.on_custom_event(event_type, callback))
    }

    pub fn on_long_press_message(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.map_node(|node| node.on_long_press_message(message))
    }

    pub fn on_long_press(self, callback: impl Fn() + 'static) -> Self {
        self.map_node(|node| node.on_long_press(callback))
    }

    pub fn on_input(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_input(handler))
    }

    pub fn on_submit(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.map_node(|node| node.on_submit(message))
    }

    pub fn on_refresh(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.map_node(|node| node.on_refresh(message))
    }

    pub fn on_refresh_state_change(self, handler: impl Fn(i32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_refresh_state_change(handler))
    }

    pub fn on_refresh_offset_change(self, handler: impl Fn(f32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_refresh_offset_change(handler))
    }

    pub fn on_toggle(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_toggle(handler))
    }

    pub fn on_toggle_local(self, handler: impl Fn(bool) + 'static) -> Self {
        self.map_node(|node| node.on_toggle_local(handler))
    }

    pub fn on_list_scroll_index(
        self,
        handler: impl Fn(ListScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_list_scroll_index(handler))
    }

    pub fn on_list_scroll_index_local(
        self,
        handler: impl Fn(ListScrollIndexEvent) + 'static,
    ) -> Self {
        self.map_node(|node| node.on_list_scroll_index_local(handler))
    }

    pub fn on_list_visible_content_change(
        self,
        handler: impl Fn(ListVisibleContentChangeEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_list_visible_content_change(handler))
    }

    pub fn on_grid_scroll_index(
        self,
        handler: impl Fn(GridScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_grid_scroll_index(handler))
    }

    pub fn on_water_flow_scroll_index(
        self,
        handler: impl Fn(WaterFlowScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_water_flow_scroll_index(handler))
    }

    pub fn on_visible_range_change(
        self,
        handler: impl Fn(VirtualVisibleRange) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_visible_range_change(handler))
    }

    pub fn on_load_more(
        self,
        total_count: u32,
        threshold: u32,
        loading: bool,
        message: Message,
    ) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.map_node(|node| node.on_load_more(total_count, threshold, loading, message))
    }

    pub fn on_change(self, handler: impl Fn(f32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        self.map_node(|node| node.on_change(handler))
    }
}
