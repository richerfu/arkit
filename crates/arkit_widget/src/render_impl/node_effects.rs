use super::*;

impl<Message, AppTheme> Node<Message, AppTheme> {
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
}
