use super::*;

struct BodyOnlyWidget;

impl advanced::Widget<(), arkit_core::Theme, Renderer> for BodyOnlyWidget {
    fn body(
        &self,
        _tree: &mut advanced::widget::Tree,
        _renderer: &Renderer,
    ) -> Option<Element<()>> {
        Some(text("body").into())
    }
}

#[test]
fn composite_widget_body_tree_is_initialized_lazily() {
    let element = Element::new(BodyOnlyWidget);
    let mut tree = arkit_core::advanced::tree_of(&element);
    assert!(tree.children().is_empty());

    let mut state_cache = StateCache::default();
    let _compiled = compile_element(
        element,
        &mut tree,
        &mut state_cache,
        &Renderer::default(),
        false,
    );

    assert_eq!(tree.children().len(), 1);
}

#[test]
fn desired_attrs_preserve_last_set_order() {
    let attrs = desired_attrs(
        vec![
            (ArkUINodeAttributeType::Width, 10.0_f32.into()),
            (ArkUINodeAttributeType::Height, 20.0_f32.into()),
            (ArkUINodeAttributeType::Width, 30.0_f32.into()),
        ],
        vec![
            (
                ArkUINodeAttributeType::BackgroundColor,
                0xFF000000_u32.into(),
            ),
            (ArkUINodeAttributeType::Height, 40.0_f32.into()),
        ],
    );

    assert_eq!(
        attr_types(&attrs),
        vec![
            ArkUINodeAttributeType::Width,
            ArkUINodeAttributeType::BackgroundColor,
            ArkUINodeAttributeType::Height,
        ]
    );
}

#[test]
fn scalar_edges_expand_to_explicit_edges() {
    assert_eq!(6.0_f32.edge_values(), vec![6.0, 6.0, 6.0, 6.0]);
}

#[test]
fn visual_clipping_attrs_are_applied_after_size_and_background() {
    let attrs = ordered_attrs_for_application(vec![
        (ArkUINodeAttributeType::BorderRadius, 6.0_f32.into()),
        (ArkUINodeAttributeType::Clip, true.into()),
        (ArkUINodeAttributeType::Height, 40.0_f32.into()),
        (ArkUINodeAttributeType::Padding, vec![0.0_f32, 8.0].into()),
        (
            ArkUINodeAttributeType::BackgroundColor,
            0xFF000000_u32.into(),
        ),
    ]);

    assert_eq!(
        attr_types(&attrs),
        vec![
            ArkUINodeAttributeType::Height,
            ArkUINodeAttributeType::Padding,
            ArkUINodeAttributeType::BackgroundColor,
            ArkUINodeAttributeType::BorderRadius,
            ArkUINodeAttributeType::Clip,
        ]
    );
}

#[test]
fn button_component_uses_pressable_surface_host() {
    let node = button_component::<(), arkit_core::Theme>();

    assert_eq!(node.kind(), NodeKind::Stack);
    assert_eq!(node.attr_f32(ArkUINodeAttributeType::ButtonType), None);
}

#[test]
fn button_component_keeps_border_radius_as_surface_style() {
    let node = button_component::<(), arkit_core::Theme>().border_radius(8.0);

    assert_eq!(node.kind(), NodeKind::Stack);
    assert_eq!(node.attr_f32(ArkUINodeAttributeType::ButtonType), None);
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::BorderRadius),
        Some(8.0)
    );
}

#[test]
fn label_button_uses_native_button() {
    let node = button::<(), arkit_core::Theme>("OK").border_radius(8.0);

    assert_eq!(node.kind(), NodeKind::Button);
    assert_eq!(
        node.attr_string(ArkUINodeAttributeType::ButtonLabel),
        Some("OK")
    );
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::BorderRadius),
        Some(8.0)
    );
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::ButtonType),
        Some(i32::from(ButtonType::Normal) as f32)
    );
}

#[test]
fn native_button_keeps_default_type_until_custom_radius_is_set() {
    let node = button::<(), arkit_core::Theme>("OK");

    assert_eq!(node.kind(), NodeKind::Button);
    assert_eq!(node.attr_f32(ArkUINodeAttributeType::ButtonType), None);
}

#[test]
fn explicit_button_type_is_not_overwritten_by_border_radius() {
    let node = button::<(), arkit_core::Theme>("OK")
        .button_type(ButtonType::RoundedRectangle)
        .border_radius(8.0);

    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::ButtonType),
        Some(i32::from(ButtonType::RoundedRectangle) as f32)
    );
}

#[test]
fn list_component_uses_native_list() {
    let node = list_component::<(), arkit_core::Theme>();

    assert_eq!(node.kind(), NodeKind::List);
}

#[test]
fn list_item_component_uses_native_list_item() {
    let node = list_item_component::<(), arkit_core::Theme>();

    assert_eq!(node.kind(), NodeKind::ListItem);
}

#[test]
fn grid_and_water_flow_components_use_native_nodes() {
    assert_eq!(
        grid_component::<(), arkit_core::Theme>().kind(),
        NodeKind::Grid
    );
    assert_eq!(
        grid_item_component::<(), arkit_core::Theme>().kind(),
        NodeKind::GridItem
    );
    assert_eq!(
        water_flow_component::<(), arkit_core::Theme>().kind(),
        NodeKind::WaterFlow
    );
    assert_eq!(
        flow_item_component::<(), arkit_core::Theme>().kind(),
        NodeKind::FlowItem
    );
    assert_eq!(
        list_item_group_component::<(), arkit_core::Theme>().kind(),
        NodeKind::ListItemGroup
    );
}

#[test]
fn flex_component_uses_native_flex_node() {
    let node = flex_component::<(), arkit_core::Theme>();

    assert_eq!(node.kind(), NodeKind::Flex);
}

#[test]
fn flex_options_encode_native_flex_option_attribute() {
    let node: Node<(), arkit_core::Theme> = flex_component::<(), arkit_core::Theme>()
        .flex_direction(FlexDirection::Column)
        .flex_wrap(FlexWrap::Wrap)
        .flex_justify_content(JustifyContent::SpaceBetween)
        .flex_align_items(ItemAlignment::Stretch)
        .flex_align_content(JustifyContent::Center)
        .into();

    assert_eq!(
        attr_i32_values(&node, ArkUINodeAttributeType::FlexOption),
        Some(vec![1, 1, 6, 4, 2])
    );
}

#[test]
fn virtual_components_attach_native_adapter_specs() {
    let list = virtual_list_component::<(), arkit_core::Theme, _>(10, |_| text("row").into());
    let grid = virtual_grid_component::<(), arkit_core::Theme, _>(20, |_| text("cell").into());
    let flow =
        virtual_water_flow_component::<(), arkit_core::Theme, _>(30, |_| text("flow").into());

    assert_eq!(list.kind(), NodeKind::List);
    assert_eq!(
        list.virtual_adapter_kind(),
        Some(VirtualContainerKind::List)
    );
    assert_eq!(grid.kind(), NodeKind::Grid);
    assert_eq!(
        grid.virtual_adapter_kind(),
        Some(VirtualContainerKind::Grid)
    );
    assert_eq!(flow.kind(), NodeKind::WaterFlow);
    assert_eq!(
        flow.virtual_adapter_kind(),
        Some(VirtualContainerKind::WaterFlow)
    );
}

fn attr_i32_values<Message, AppTheme>(
    node: &Node<Message, AppTheme>,
    attr: ArkUINodeAttributeType,
) -> Option<Vec<i32>> {
    let ArkUINodeAttributeItem::NumberValue(values) = node.attr_value(attr)? else {
        return None;
    };

    values
        .iter()
        .map(|value| match value {
            ArkUINodeAttributeNumber::Float(value) => Some(*value as i32),
            ArkUINodeAttributeNumber::Int(value) => Some(*value),
            ArkUINodeAttributeNumber::Uint(value) => i32::try_from(*value).ok(),
        })
        .collect()
}

#[test]
fn detached_virtual_items_preserve_native_item_roots() {
    let cases = [
        (VirtualContainerKind::List, NodeKind::ListItem),
        (VirtualContainerKind::Grid, NodeKind::GridItem),
        (VirtualContainerKind::WaterFlow, NodeKind::FlowItem),
        (VirtualContainerKind::ListItemGroup, NodeKind::ListItem),
    ];

    for (container, expected) in cases {
        let item = wrap_virtual_item(container, 7, text::<(), arkit_core::Theme>("item").into());
        let (_, root) = compile_detached_element_root(item);
        let node = into_node(root);

        assert_eq!(node.kind(), expected);
    }
}

#[test]
fn virtual_adapter_count_change_uses_incremental_updates() {
    assert_eq!(
        virtual_adapter_count_change(10, 10),
        VirtualAdapterCountChange::Unchanged
    );
    assert_eq!(
        virtual_adapter_count_change(10, 14),
        VirtualAdapterCountChange::Insert {
            start: 10,
            count: 4
        }
    );
    assert_eq!(
        virtual_adapter_count_change(14, 10),
        VirtualAdapterCountChange::Remove {
            start: 10,
            count: 4
        }
    );
}

#[test]
fn grouped_virtual_list_uses_sticky_native_groups() {
    let groups = vec![VirtualListGroup::new("a", 3), VirtualListGroup::new("b", 4)];
    let node = into_node(grouped_virtual_list::<(), arkit_core::Theme, _, _>(
        groups,
        |_| text("header").into(),
        |_, _| text("item").into(),
    ));

    assert_eq!(node.kind(), NodeKind::List);
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::ListSticky),
        Some(i32::from(ListStickyStyle::Header) as f32)
    );
    let children = node.children;
    assert_eq!(children.len(), 2);
    for child in children {
        let child = into_node(child);
        assert_eq!(child.kind(), NodeKind::ListItemGroup);
        assert_eq!(
            child.virtual_adapter.as_ref().map(|spec| spec.kind),
            Some(VirtualContainerKind::ListItemGroup)
        );
    }
}

#[test]
fn refresh_component_sets_refresh_attributes() {
    let node = refresh_component::<(), arkit_core::Theme>()
        .refreshing(true)
        .refresh_offset(72.0)
        .refresh_pull_to_refresh(false);

    assert_eq!(
        node.attr_bool(ArkUINodeAttributeType::RefreshRefreshing),
        Some(true)
    );
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::RefreshOffset),
        Some(72.0)
    );
    assert_eq!(
        node.attr_bool(ArkUINodeAttributeType::RefreshPullToRefresh),
        Some(false)
    );
}

#[test]
fn text_input_font_size_sets_placeholder_font_size() {
    let node = text_input_component::<(), arkit_core::Theme>().font_size(14.0);

    assert_eq!(node.attr_f32(ArkUINodeAttributeType::FontSize), Some(14.0));
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::TextInputPlaceholderFont),
        Some(14.0)
    );
}

#[test]
fn text_area_font_size_sets_placeholder_font_size() {
    let node = text_area_component::<(), arkit_core::Theme>().font_size(13.0);

    assert_eq!(node.attr_f32(ArkUINodeAttributeType::FontSize), Some(13.0));
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::TextAreaPlaceholderFont),
        Some(13.0)
    );
}

#[cfg(feature = "webview")]
#[test]
fn web_view_component_uses_dedicated_host_kind() {
    let controller = WebViewController::with_id("webview-test");
    let node: Node<(), arkit_core::Theme> = web_view_component(controller.clone())
        .javascript_enabled(true)
        .devtools(true)
        .into();

    assert_eq!(node.kind(), NodeKind::WebViewHost);
    let spec = node.webview.as_ref().expect("webview spec should exist");
    assert_eq!(
        spec.controller
            .as_ref()
            .expect("controller should be present")
            .id(),
        "webview-test"
    );
    assert_eq!(spec.javascript_enabled, Some(true));
    assert_eq!(spec.devtools, Some(true));
}

#[cfg(feature = "webview")]
#[test]
fn web_view_component_uses_transparent_hit_testing() {
    let controller = WebViewController::with_id("webview-transparent");
    let node: Node<(), arkit_core::Theme> = web_view_component(controller).into();

    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::HitTestBehavior),
        Some(i32::from(HitTestBehavior::Transparent) as f32)
    );
}

#[cfg(feature = "webview")]
#[test]
fn web_view_component_clips_to_host_bounds() {
    let controller = WebViewController::with_id("webview-clipped");
    let node: Node<(), arkit_core::Theme> = web_view_component(controller).into();

    assert_eq!(node.attr_bool(ArkUINodeAttributeType::Clip), Some(true));
}

#[cfg(feature = "webview")]
#[test]
fn build_initial_webview_style_only_preserves_explicit_style_fields() {
    let style = build_initial_webview_style(
        &WebViewSpec::default(),
        Some(LayoutFrame {
            x: 12.0,
            y: 34.0,
            width: 200.0,
            height: 120.0,
        }),
    )
    .expect("style should be created");

    assert!(style.x.is_none());
    assert!(style.y.is_none());
    assert!(style.background_color.is_none());
    assert!(style.visible.is_none());
}

#[cfg(feature = "webview")]
#[test]
fn compose_compiled_overlays_keeps_stable_wrapper_without_overlays() {
    let node = into_node(compose_compiled_overlays(CompiledElement::<
        (),
        arkit_core::Theme,
    > {
        body: column_component::<(), arkit_core::Theme>().into(),
        overlays: Vec::new(),
    }));

    assert_eq!(node.kind(), NodeKind::Stack);
    assert_eq!(node.children.len(), 2);
    assert_eq!(
        into_node(node.children.into_iter().nth(1).expect("overlay layer"))
            .attr_f32(ArkUINodeAttributeType::HitTestBehavior),
        Some(i32::from(HitTestBehavior::Transparent) as f32)
    );
}

#[cfg(feature = "webview")]
#[test]
fn compose_compiled_overlays_keeps_stack_wrapper_with_overlays() {
    let node = into_node(compose_compiled_overlays(CompiledElement::<
        (),
        arkit_core::Theme,
    > {
        body: column_component::<(), arkit_core::Theme>().into(),
        overlays: vec![text::<(), arkit_core::Theme>("overlay").into()],
    }));

    assert_eq!(node.kind(), NodeKind::Stack);
    assert_eq!(node.children.len(), 2);
    assert_eq!(
        into_node(node.children.into_iter().nth(1).expect("overlay layer"))
            .attr_f32(ArkUINodeAttributeType::HitTestBehavior),
        Some(i32::from(HitTestBehavior::Default) as f32)
    );
}

#[cfg(feature = "webview")]
#[test]
fn web_view_constructor_defaults_to_fill_and_url() {
    let controller = WebViewController::with_id("webview-fill");
    let node: Node<(), arkit_core::Theme> = web_view(controller, "https://example.com").into();

    assert_eq!(node.kind(), NodeKind::WebViewHost);
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::WidthPercent),
        Some(1.0)
    );
    assert_eq!(
        node.attr_f32(ArkUINodeAttributeType::HeightPercent),
        Some(1.0)
    );
    assert_eq!(
        node.webview.as_ref().and_then(|spec| spec.url.as_deref()),
        Some("https://example.com")
    );
}
