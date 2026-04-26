use super::*;

impl<Message, AppTheme> Node<Message, AppTheme> {
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

    pub fn on_grid_scroll_index(
        self,
        handler: impl Fn(GridScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.on_event(NodeEventType::GridOnScrollIndex, move |event| {
            arkit_runtime::dispatch(handler(GridScrollIndexEvent {
                first_index: event.i32_value(0).unwrap_or_default(),
                last_index: event.i32_value(1).unwrap_or_default(),
            }));
        })
    }

    pub fn on_water_flow_scroll_index(
        self,
        handler: impl Fn(WaterFlowScrollIndexEvent) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.on_event(NodeEventType::WaterFlowOnScrollIndex, move |event| {
            arkit_runtime::dispatch(handler(WaterFlowScrollIndexEvent {
                start_index: event.i32_value(0).unwrap_or_default(),
                end_index: event.i32_value(1).unwrap_or_default(),
            }));
        })
    }

    pub fn on_visible_range_change(
        self,
        handler: impl Fn(VirtualVisibleRange) -> Message + 'static,
    ) -> Self
    where
        Message: Send + 'static,
    {
        match self.kind {
            NodeKind::List => self.on_list_scroll_index(move |event| {
                handler(VirtualVisibleRange {
                    first_index: event.first_index,
                    last_index: event.last_index,
                })
            }),
            NodeKind::Grid => self.on_grid_scroll_index(move |event| {
                handler(VirtualVisibleRange {
                    first_index: event.first_index,
                    last_index: event.last_index,
                })
            }),
            NodeKind::WaterFlow => self.on_water_flow_scroll_index(move |event| {
                handler(VirtualVisibleRange {
                    first_index: event.start_index,
                    last_index: event.end_index,
                })
            }),
            _ => self,
        }
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
        if loading {
            return self;
        }

        let trigger_index = total_count.saturating_sub(threshold.max(1)) as i32;

        match self.kind {
            NodeKind::List => {
                let message = message.clone();
                self.on_event(NodeEventType::ListOnScrollIndex, move |event| {
                    let last_index = event.i32_value(1).unwrap_or_default();
                    if total_count > 0 && last_index >= trigger_index {
                        arkit_runtime::dispatch(message.clone());
                    }
                })
            }
            NodeKind::Grid => {
                let message = message.clone();
                self.on_event(NodeEventType::GridOnScrollIndex, move |event| {
                    let last_index = event.i32_value(1).unwrap_or_default();
                    if total_count > 0 && last_index >= trigger_index {
                        arkit_runtime::dispatch(message.clone());
                    }
                })
            }
            NodeKind::WaterFlow => {
                let message = message.clone();
                self.on_event(NodeEventType::WaterFlowOnScrollIndex, move |event| {
                    let last_index = event.i32_value(1).unwrap_or_default();
                    if total_count > 0 && last_index >= trigger_index {
                        arkit_runtime::dispatch(message.clone());
                    }
                })
            }
            _ => self,
        }
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
