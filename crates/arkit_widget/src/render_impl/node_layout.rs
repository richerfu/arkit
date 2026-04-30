use super::*;

impl<Message, AppTheme> Node<Message, AppTheme> {
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
        let encoded = i32::from(value);
        match self.kind {
            NodeKind::Column => self
                .attr(ArkUINodeAttributeType::ColumnJustifyContent, encoded)
                .patch_attr(ArkUINodeAttributeType::ColumnJustifyContent, encoded),
            NodeKind::Row => self
                .attr(ArkUINodeAttributeType::RowJustifyContent, encoded)
                .patch_attr(ArkUINodeAttributeType::RowJustifyContent, encoded),
            NodeKind::Flex => self.flex_justify_content(value),
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
            NodeKind::Flex => self.flex_align_items(ItemAlignment::Start),
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
            NodeKind::Flex => self.flex_align_items(ItemAlignment::Center),
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
            NodeKind::Flex => self.flex_align_items(ItemAlignment::End),
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

    pub fn flex_options(self, value: FlexOptions) -> Self {
        if self.kind != NodeKind::Flex {
            return self;
        }

        self.builder_attr(
            ArkUINodeAttributeType::FlexOption,
            value.attribute_numbers(),
        )
    }

    pub fn flex_direction(self, value: FlexDirection) -> Self {
        self.with_flex_options_update(|options| options.direction = value)
    }

    pub fn flex_wrap(self, value: FlexWrap) -> Self {
        self.with_flex_options_update(|options| options.wrap = value)
    }

    pub fn flex_justify_content(self, value: JustifyContent) -> Self {
        self.with_flex_options_update(|options| options.justify_content = value)
    }

    pub fn flex_align_items(self, value: ItemAlignment) -> Self {
        self.with_flex_options_update(|options| options.align_items = value)
    }

    pub fn flex_align_content(self, value: JustifyContent) -> Self {
        self.with_flex_options_update(|options| options.align_content = value)
    }

    fn with_flex_options_update(mut self, update: impl FnOnce(&mut FlexOptions)) -> Self {
        if self.kind != NodeKind::Flex {
            return self;
        }

        let mut options = self
            .attr_value(ArkUINodeAttributeType::FlexOption)
            .and_then(FlexOptions::from_attribute_value)
            .unwrap_or_default();
        update(&mut options);
        self = self.builder_attr(
            ArkUINodeAttributeType::FlexOption,
            options.attribute_numbers(),
        );
        self
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

    pub fn caret_color(self, value: u32) -> Self {
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputCaretColor, value)
                .patch_attr(ArkUINodeAttributeType::TextInputCaretColor, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaCaretColor, value)
                .patch_attr(ArkUINodeAttributeType::TextAreaCaretColor, value),
            _ => self,
        }
    }

    pub fn caret_style(self, width: f32) -> Self {
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputCaretStyle, width)
                .patch_attr(ArkUINodeAttributeType::TextInputCaretStyle, width),
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
}
