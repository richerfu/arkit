use super::*;

impl<Message, AppTheme, Kind> Component<Message, AppTheme, Kind> {
    pub fn refreshing(self, value: bool) -> Self {
        self.map_node(|node| node.refreshing(value))
    }

    pub fn refresh_offset(self, value: f32) -> Self {
        self.map_node(|node| node.refresh_offset(value))
    }

    pub fn refresh_pull_to_refresh(self, value: bool) -> Self {
        self.map_node(|node| node.refresh_pull_to_refresh(value))
    }

    pub fn list_sticky(self, value: ListStickyStyle) -> Self {
        self.map_node(|node| node.list_sticky(value))
    }

    pub fn list_cached_count(self, value: u32) -> Self {
        self.map_node(|node| node.list_cached_count(value))
    }

    pub fn grid_column_template(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.grid_column_template(value))
    }

    pub fn grid_row_template(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.grid_row_template(value))
    }

    pub fn grid_column_gap(self, value: f32) -> Self {
        self.map_node(|node| node.grid_column_gap(value))
    }

    pub fn grid_row_gap(self, value: f32) -> Self {
        self.map_node(|node| node.grid_row_gap(value))
    }

    pub fn grid_cached_count(self, value: u32) -> Self {
        self.map_node(|node| node.grid_cached_count(value))
    }

    pub fn water_flow_column_template(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.water_flow_column_template(value))
    }

    pub fn water_flow_row_template(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.water_flow_row_template(value))
    }

    pub fn water_flow_column_gap(self, value: f32) -> Self {
        self.map_node(|node| node.water_flow_column_gap(value))
    }

    pub fn water_flow_row_gap(self, value: f32) -> Self {
        self.map_node(|node| node.water_flow_row_gap(value))
    }

    pub fn water_flow_cached_count(self, value: u32) -> Self {
        self.map_node(|node| node.water_flow_cached_count(value))
    }

    pub fn list_item_group_header(self, header: impl Into<Element<Message, AppTheme>>) -> Self
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        self.map_node(|node| node.list_item_group_header(header))
    }

    pub fn list_item_group_footer(self, footer: impl Into<Element<Message, AppTheme>>) -> Self
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        self.map_node(|node| node.list_item_group_footer(footer))
    }

    pub fn on_scroll_offset(self, callback: impl Fn(ScrollOffset) + 'static) -> Self {
        self.map_node(|node| node.on_scroll_offset(callback))
    }

    pub fn toggle_selected_color(self, value: u32) -> Self {
        self.map_node(|node| node.toggle_selected_color(value))
    }

    pub fn toggle_unselected_color(self, value: u32) -> Self {
        self.map_node(|node| node.toggle_unselected_color(value))
    }

    pub fn toggle_switch_point_color(self, value: u32) -> Self {
        self.map_node(|node| node.toggle_switch_point_color(value))
    }

    pub fn justify_content(self, value: JustifyContent) -> Self {
        self.map_node(|node| node.justify_content(value))
    }

    pub fn justify_content_start(self) -> Self {
        self.map_node(|node| node.justify_content_start())
    }

    pub fn justify_content_center(self) -> Self {
        self.map_node(|node| node.justify_content_center())
    }

    pub fn justify_content_end(self) -> Self {
        self.map_node(|node| node.justify_content_end())
    }

    pub fn flex_options(self, value: FlexOptions) -> Self {
        self.map_node(|node| node.flex_options(value))
    }

    pub fn flex_direction(self, value: FlexDirection) -> Self {
        self.map_node(|node| node.flex_direction(value))
    }

    pub fn flex_wrap(self, value: FlexWrap) -> Self {
        self.map_node(|node| node.flex_wrap(value))
    }

    pub fn flex_justify_content(self, value: JustifyContent) -> Self {
        self.map_node(|node| node.flex_justify_content(value))
    }

    pub fn flex_align_items(self, value: ItemAlignment) -> Self {
        self.map_node(|node| node.flex_align_items(value))
    }

    pub fn flex_align_content(self, value: JustifyContent) -> Self {
        self.map_node(|node| node.flex_align_content(value))
    }

    pub fn align_x(self, alignment: Horizontal) -> Self {
        self.map_node(|node| node.align_x(alignment))
    }

    pub fn align_y(self, alignment: Vertical) -> Self {
        self.map_node(|node| node.align_y(alignment))
    }

    pub fn align_items_start(self) -> Self {
        self.map_node(|node| node.align_items_start())
    }

    pub fn align_items_center(self) -> Self {
        self.map_node(|node| node.align_items_center())
    }

    pub fn align_items_end(self) -> Self {
        self.map_node(|node| node.align_items_end())
    }

    pub fn align_items_top(self) -> Self {
        self.map_node(|node| node.align_items_top())
    }

    pub fn align_items_bottom(self) -> Self {
        self.map_node(|node| node.align_items_bottom())
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        self.map_node(|node| node.label(label))
    }

    pub fn content(self, content: impl Into<String>) -> Self {
        self.map_node(|node| node.content(content))
    }

    pub fn value(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.value(value))
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.placeholder(value))
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        self.map_node(|node| node.placeholder_color(value))
    }

    pub fn checked(self, value: bool) -> Self {
        self.map_node(|node| node.checked(value))
    }

    pub fn range(self, min: f32, max: f32) -> Self {
        self.map_node(|node| node.range(min, max))
    }

    #[cfg(feature = "webview")]
    pub fn webview_style(self, style: WebViewStyle) -> Self {
        self.map_node(|node| node.webview_style(style))
    }

    #[cfg(feature = "webview")]
    pub fn url(self, url: impl Into<String>) -> Self {
        self.map_node(|node| node.url(url))
    }

    #[cfg(feature = "webview")]
    pub fn html(self, html: impl Into<String>) -> Self {
        self.map_node(|node| node.html(html))
    }

    #[cfg(feature = "webview")]
    pub fn javascript_enabled(self, enabled: bool) -> Self {
        self.map_node(|node| node.javascript_enabled(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn devtools(self, enabled: bool) -> Self {
        self.map_node(|node| node.devtools(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn transparent(self, enabled: bool) -> Self {
        self.map_node(|node| node.transparent(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn autoplay(self, enabled: bool) -> Self {
        self.map_node(|node| node.autoplay(enabled))
    }

    #[cfg(feature = "webview")]
    pub fn user_agent(self, user_agent: impl Into<String>) -> Self {
        self.map_node(|node| node.user_agent(user_agent))
    }

    #[cfg(feature = "webview")]
    pub fn initialization_scripts(self, scripts: Vec<String>) -> Self {
        self.map_node(|node| node.initialization_scripts(scripts))
    }

    #[cfg(feature = "webview")]
    pub fn headers(self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.map_node(|node| node.headers(headers))
    }

    #[cfg(feature = "webview")]
    pub fn on_drag_and_drop(self, callback: impl Fn(String) + 'static) -> Self {
        self.map_node(|node| node.on_drag_and_drop(callback))
    }

    #[cfg(feature = "webview")]
    pub fn on_download_start(
        self,
        callback: impl Fn(String, &mut std::path::PathBuf) -> bool + 'static,
    ) -> Self {
        self.map_node(|node| node.on_download_start(callback))
    }

    #[cfg(feature = "webview")]
    pub fn on_download_end(
        self,
        callback: impl Fn(String, Option<std::path::PathBuf>, bool) + 'static,
    ) -> Self {
        self.map_node(|node| node.on_download_end(callback))
    }

    #[cfg(feature = "webview")]
    pub fn on_navigation_request(self, callback: impl Fn(String) -> bool + 'static) -> Self {
        self.map_node(|node| node.on_navigation_request(callback))
    }

    #[cfg(feature = "webview")]
    pub fn on_title_change(self, callback: impl Fn(String) + 'static) -> Self {
        self.map_node(|node| node.on_title_change(callback))
    }
}
