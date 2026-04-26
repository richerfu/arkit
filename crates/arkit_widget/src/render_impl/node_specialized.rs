use super::*;

impl<Message, AppTheme> Node<Message, AppTheme> {
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

    pub fn list_sticky(self, value: ListStickyStyle) -> Self {
        if self.kind != NodeKind::List {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::ListSticky, i32::from(value))
    }

    pub fn list_cached_count(self, value: u32) -> Self {
        if self.kind != NodeKind::List {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::ListCachedCount, value)
    }

    pub fn grid_column_template(self, value: impl Into<String>) -> Self {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::GridColumnTemplate, value.into())
    }

    pub fn grid_row_template(self, value: impl Into<String>) -> Self {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::GridRowTemplate, value.into())
    }

    pub fn grid_column_gap(self, value: f32) -> Self {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::GridColumnGap, value)
    }

    pub fn grid_row_gap(self, value: f32) -> Self {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::GridRowGap, value)
    }

    pub fn grid_cached_count(self, value: u32) -> Self {
        if self.kind != NodeKind::Grid {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::GridCachedCount, value)
    }

    pub fn water_flow_column_template(self, value: impl Into<String>) -> Self {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.builder_attr(
            ArkUINodeAttributeType::WaterFlowColumnTemplate,
            value.into(),
        )
    }

    pub fn water_flow_row_template(self, value: impl Into<String>) -> Self {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::WaterFlowRowTemplate, value.into())
    }

    pub fn water_flow_column_gap(self, value: f32) -> Self {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::WaterFlowColumnGap, value)
    }

    pub fn water_flow_row_gap(self, value: f32) -> Self {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::WaterFlowRowGap, value)
    }

    pub fn water_flow_cached_count(self, value: u32) -> Self {
        if self.kind != NodeKind::WaterFlow {
            return self;
        }

        self.builder_attr(ArkUINodeAttributeType::WaterFlowCachedCount, value)
    }

    pub fn list_item_group_header(self, header: impl Into<Element<Message, AppTheme>>) -> Self
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        self.list_item_group_slot(ArkUINodeAttributeType::ListItemGroupSetHeader, header)
    }

    pub fn list_item_group_footer(self, footer: impl Into<Element<Message, AppTheme>>) -> Self
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        self.list_item_group_slot(ArkUINodeAttributeType::ListItemGroupSetFooter, footer)
    }

    fn list_item_group_slot(
        mut self,
        attr: ArkUINodeAttributeType,
        slot: impl Into<Element<Message, AppTheme>>,
    ) -> Self
    where
        Message: Send + 'static,
        AppTheme: 'static,
    {
        if self.kind != NodeKind::ListItemGroup {
            return self;
        }

        let slot = Rc::new(RefCell::new(Some(slot.into())));
        self.mount_effects.push(Box::new(move |node| {
            let Some(slot) = slot.borrow_mut().take() else {
                return Ok(None);
            };
            let (mut slot_node, mut mounted) = mount_detached_element(slot)?;
            set_node_object_attribute(node, attr, &slot_node)?;
            realize_attached_mount(&mut slot_node, &mut mounted)?;
            Ok(Some(Box::new(move || {
                mounted.cleanup_recursive();
                let _ = slot_node.dispose();
            }) as Cleanup))
        }));
        self
    }
}
