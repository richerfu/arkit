use super::*;

impl<Message, AppTheme, Kind> Component<Message, AppTheme, Kind> {
    pub fn key(self, key: impl Into<String>) -> Self {
        self.map_node(|node| node.key(key))
    }

    pub fn persistent_state_key(self, key: impl Into<String>) -> Self {
        self.map_node(|node| node.persistent_state_key(key))
    }

    pub fn child(self, child: impl Into<Element<Message, AppTheme>>) -> Self {
        self.map_node(|node| node.child(child))
    }

    pub fn children(self, children: Vec<Element<Message, AppTheme>>) -> Self {
        self.map_node(|node| node.children(children))
    }

    pub fn map_descendants(
        self,
        map: impl FnMut(Node<Message, AppTheme>) -> Node<Message, AppTheme>,
    ) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        self.map_node(|node| node.map_descendants(map))
    }

    pub fn attr(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.map_node(|node| node.attr(attr, value))
    }

    pub fn patch_attr(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.map_node(|node| node.patch_attr(attr, value))
    }

    pub fn attr_string(&self, attr: ArkUINodeAttributeType) -> Option<&str> {
        self.node.attr_string(attr)
    }

    pub fn attr_f32(&self, attr: ArkUINodeAttributeType) -> Option<f32> {
        self.node.attr_f32(attr)
    }

    pub fn attr_bool(&self, attr: ArkUINodeAttributeType) -> Option<bool> {
        self.node.attr_bool(attr)
    }

    pub fn width(self, value: impl Into<Length>) -> Self {
        self.map_node(|node| node.width(value))
    }

    pub fn height(self, value: impl Into<Length>) -> Self {
        self.map_node(|node| node.height(value))
    }

    pub fn percent_width(self, value: f32) -> Self {
        self.map_node(|node| node.percent_width(value))
    }

    pub fn percent_height(self, value: f32) -> Self {
        self.map_node(|node| node.percent_height(value))
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.map_node(|node| node.max_width_constraint(value))
    }

    pub fn constraint_size(
        self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        self.map_node(|node| node.constraint_size(min_width, max_width, min_height, max_height))
    }

    pub fn background_color(self, value: u32) -> Self {
        self.map_node(|node| node.background_color(value))
    }

    pub fn padding(self, value: impl Into<Padding>) -> Self {
        self.map_node(|node| node.padding(value))
    }

    pub fn padding_x(self, value: f32) -> Self {
        self.map_node(|node| node.padding_x(value))
    }

    pub fn padding_y(self, value: f32) -> Self {
        self.map_node(|node| node.padding_y(value))
    }

    pub fn margin(self, value: impl Into<Padding>) -> Self {
        self.map_node(|node| node.margin(value))
    }

    pub fn margin_x(self, value: f32) -> Self {
        self.map_node(|node| node.margin_x(value))
    }

    pub fn margin_y(self, value: f32) -> Self {
        self.map_node(|node| node.margin_y(value))
    }

    pub fn margin_top(self, value: f32) -> Self {
        self.map_node(|node| node.margin_top(value))
    }

    pub fn margin_right(self, value: f32) -> Self {
        self.map_node(|node| node.margin_right(value))
    }

    pub fn margin_bottom(self, value: f32) -> Self {
        self.map_node(|node| node.margin_bottom(value))
    }

    pub fn margin_left(self, value: f32) -> Self {
        self.map_node(|node| node.margin_left(value))
    }

    pub fn foreground_color(self, value: u32) -> Self {
        self.map_node(|node| node.foreground_color(value))
    }

    pub fn font_color(self, value: u32) -> Self {
        self.map_node(|node| node.font_color(value))
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.map_node(|node| node.font_weight(value))
    }

    pub fn font_family(self, value: impl Into<String>) -> Self {
        self.map_node(|node| node.font_family(value))
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.map_node(|node| node.font_style(value))
    }

    pub fn font_size(self, value: f32) -> Self {
        self.map_node(|node| node.font_size(value))
    }

    pub fn line_height(self, value: f32) -> Self {
        self.map_node(|node| node.line_height(value))
    }

    pub fn caret_color(self, value: u32) -> Self {
        self.map_node(|node| node.caret_color(value))
    }

    pub fn caret_style(self, width: f32) -> Self {
        self.map_node(|node| node.caret_style(width))
    }

    pub fn text_align(self, value: TextAlignment) -> Self {
        self.map_node(|node| node.text_align(value))
    }

    pub fn text_letter_spacing(self, value: f32) -> Self {
        self.map_node(|node| node.text_letter_spacing(value))
    }

    pub fn text_decoration(self, value: impl Into<ArkUINodeAttributeItem>) -> Self {
        self.map_node(|node| node.text_decoration(value))
    }

    pub fn enabled(self, value: bool) -> Self {
        self.map_node(|node| node.enabled(value))
    }

    pub fn opacity(self, value: f32) -> Self {
        self.map_node(|node| node.opacity(value))
    }

    pub fn clip(self, value: bool) -> Self {
        self.map_node(|node| node.clip(value))
    }

    pub fn focusable(self, value: bool) -> Self {
        self.map_node(|node| node.focusable(value))
    }

    pub fn focus_on_touch(self, value: bool) -> Self {
        self.map_node(|node| node.focus_on_touch(value))
    }

    pub fn border_radius(self, value: impl EdgeAttributeValue) -> Self {
        self.map_node(|node| node.border_radius(value))
    }

    pub fn border_width(self, value: impl EdgeAttributeValue) -> Self {
        self.map_node(|node| node.border_width(value))
    }

    pub fn border_color(self, value: u32) -> Self {
        self.map_node(|node| node.border_color(value))
    }

    pub fn border_color_all(self, value: u32) -> Self {
        self.map_node(|node| node.border_color_all(value))
    }

    pub fn border_style(self, value: BorderStyle) -> Self {
        self.map_node(|node| node.border_style(value))
    }

    pub fn shadow(self, value: ShadowStyle) -> Self {
        self.map_node(|node| node.shadow(value))
    }

    pub fn custom_shadow(
        self,
        blur_radius: f32,
        offset_x: f32,
        offset_y: f32,
        color: u32,
        fill: bool,
    ) -> Self {
        self.map_node(|node| node.custom_shadow(blur_radius, offset_x, offset_y, color, fill))
    }

    pub fn clear_shadow(self) -> Self {
        self.map_node(|node| node.clear_shadow())
    }

    pub fn alignment(self, value: Alignment) -> Self {
        self.map_node(|node| node.alignment(value))
    }

    pub fn align_self(self, value: ItemAlignment) -> Self {
        self.map_node(|node| node.align_self(value))
    }

    pub fn layout_weight(self, value: f32) -> Self {
        self.map_node(|node| node.layout_weight(value))
    }

    pub fn visibility(self, value: Visibility) -> Self {
        self.map_node(|node| node.visibility(value))
    }

    pub fn hit_test_behavior(self, value: HitTestBehavior) -> Self {
        self.map_node(|node| node.hit_test_behavior(value))
    }

    pub fn button_type(self, value: ButtonType) -> Self {
        self.map_node(|node| node.button_type(value))
    }

    pub fn color_blend(self, value: u32) -> Self {
        self.map_node(|node| node.color_blend(value))
    }

    pub fn position(self, x: f32, y: f32) -> Self {
        self.map_node(|node| node.position(x, y))
    }

    pub fn z_index(self, value: i32) -> Self {
        self.map_node(|node| node.z_index(value))
    }

    pub fn aspect_ratio(self, value: f32) -> Self {
        self.map_node(|node| node.aspect_ratio(value))
    }

    pub fn image_object_fit(self, value: ObjectFit) -> Self {
        self.map_node(|node| node.image_object_fit(value))
    }

    pub fn progress_color(self, value: u32) -> Self {
        self.map_node(|node| node.progress_color(value))
    }

    pub fn progress_type(self, value: ProgressType) -> Self {
        self.map_node(|node| node.progress_type(value))
    }

    pub fn progress_linear_style(self, value: ProgressLinearStyle) -> Self {
        self.map_node(|node| node.progress_linear_style(value))
    }
}
