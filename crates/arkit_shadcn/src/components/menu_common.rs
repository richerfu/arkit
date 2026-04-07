use super::*;
use super::floating_layer::FloatingSurfaceRegistry;
use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit::component;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) const TRANSPARENT: u32 = 0x00000000;

#[derive(Clone)]
pub(crate) struct MenuContext {
    pub(crate) dismiss: Rc<dyn Fn()>,
    pub(crate) root_open: bool,
    pub(crate) root_surfaces: FloatingSurfaceRegistry,
    pub(crate) current_surfaces: FloatingSurfaceRegistry,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MenuInteractionVariant {
    Default,
    Destructive,
}

struct RuntimeMenuRowNode(ArkUINode);

impl ArkUIAttributeBasic for RuntimeMenuRowNode {
    fn raw(&self) -> &ArkUINode {
        &self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        &mut self.0
    }
}

impl ArkUICommonAttribute for RuntimeMenuRowNode {}

pub(crate) fn menu_content_with_width(width: f32, items: Vec<Element>) -> Element {
    shadow_sm(
        arkit::column_component()
            .width(width)
            .align_items_start()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
            )
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
            .style(ArkUINodeAttributeType::Clip, true)
            .background_color(color::POPOVER)
            .children(items),
    )
    .into()
}

#[component]
pub(crate) fn provided_menu_content(
    width: f32,
    items: Vec<Element>,
    context: MenuContext,
) -> Element {
    arkit::provide_context(context);
    menu_content_with_width(width, items)
}

pub(crate) fn dismiss_menu_row(row: RowElement) -> RowElement {
    menu_action_row(row, || {})
}

pub(crate) fn menu_action_row(mut row: RowElement, on_select: impl Fn() + 'static) -> RowElement {
    if let Some(menu) = arkit::use_context::<MenuContext>() {
        let dismiss = menu.dismiss.clone();
        row = row.on_click(move || {
            on_select();
            dismiss();
        });
    } else {
        row = row.on_click(on_select);
    }

    row
}

pub(crate) fn menu_dismiss_context() -> Option<MenuContext> {
    // Menu context is intentionally inherited so nested submenus participate in
    // the same dismiss tree and root-surface registry.
    arkit::use_context::<MenuContext>()
}

pub(crate) fn root_menu_context(dismiss: Rc<dyn Fn()>, root_open: bool) -> MenuContext {
    let root_surfaces = FloatingSurfaceRegistry::new();
    MenuContext {
        dismiss,
        root_open,
        root_surfaces: root_surfaces.clone(),
        current_surfaces: root_surfaces,
    }
}

pub(crate) fn submenu_menu_context(
    parent: &MenuContext,
    current_surfaces: FloatingSurfaceRegistry,
) -> MenuContext {
    MenuContext {
        dismiss: parent.dismiss.clone(),
        root_open: parent.root_open.clone(),
        root_surfaces: parent.root_surfaces.clone(),
        current_surfaces,
    }
}

pub(crate) fn root_menu_surfaces(context: &MenuContext) -> Vec<FloatingSurfaceRegistry> {
    vec![context.root_surfaces.clone()]
}

pub(crate) fn current_menu_surface(context: &MenuContext) -> FloatingSurfaceRegistry {
    context.current_surfaces.clone()
}

pub(crate) fn submenu_menu_surfaces(
    parent: &MenuContext,
    current: &FloatingSurfaceRegistry,
) -> Vec<FloatingSurfaceRegistry> {
    let mut registries = vec![parent.root_surfaces.clone()];
    if !parent.current_surfaces.same_instance(&parent.root_surfaces) {
        registries.push(parent.current_surfaces.clone());
    }
    if !current.same_instance(&parent.root_surfaces)
        && !current.same_instance(&parent.current_surfaces)
    {
        registries.push(current.clone());
    }
    registries
}

pub(crate) fn item_text(content: impl Into<String>, color_value: u32, weight: i32) -> Element {
    arkit::text(content)
        .font_size(typography::SM)
        .style(ArkUINodeAttributeType::FontColor, color_value)
        .style(ArkUINodeAttributeType::FontWeight, weight)
        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
        .into()
}

pub(crate) fn shortcut_text(content: impl Into<String>) -> Element {
    arkit::text(content)
        .font_size(typography::XS)
        .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
        .style(ArkUINodeAttributeType::TextLetterSpacing, 1.2_f32)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Start),
        )
        .into()
}

pub(crate) fn leading_slot(child: Option<Element>) -> Element {
    let mut slot = arkit::row_component()
        .width(16.0)
        .height(16.0)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER);

    if let Some(child) = child {
        slot = slot.children(vec![child]);
    }

    arkit::row_component()
        .style(ArkUINodeAttributeType::Margin, vec![0.0, 8.0, 0.0, 0.0])
        .children(vec![slot.into()])
        .into()
}

pub(crate) fn fill_slot(child: Element) -> Element {
    arkit::row_component()
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .children(vec![child])
        .into()
}

pub(crate) fn menu_row(children: Vec<Element>, disabled: bool) -> RowElement {
    let mut row = arkit::row_component()
        .percent_width(1.0)
        .height(36.0)
        .align_items_center()
        .style(ArkUINodeAttributeType::Padding, vec![8.0, 8.0, 8.0, 8.0])
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::Clip, true)
        .background_color(TRANSPARENT)
        .children(children);

    if disabled {
        row = row.style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    }

    row
}

fn menu_row_pressed_background(variant: MenuInteractionVariant) -> u32 {
    match variant {
        MenuInteractionVariant::Default => color::ACCENT,
        MenuInteractionVariant::Destructive => 0x1AEF4444,
    }
}

pub(crate) fn interactive_menu_row(
    children: Vec<Element>,
    disabled: bool,
    variant: MenuInteractionVariant,
    active: Option<bool>,
) -> RowElement {
    let runtime_node = Rc::new(RefCell::new(None::<RuntimeMenuRowNode>));
    let capture_node = runtime_node.clone();
    let mut row = menu_row(children, disabled)
        .background_color(if active.unwrap_or(false) {
            menu_row_pressed_background(variant)
        } else {
            TRANSPARENT
        })
        .native(move |node| {
            capture_node.replace(Some(RuntimeMenuRowNode(node.borrow_mut().clone())));
            Ok(())
        });

    if disabled {
        return row;
    }

    let detach_node = runtime_node.clone();
    row = row.on_event_no_param(arkit::prelude::NodeEventType::EventOnDetach, move || {
        detach_node.borrow_mut().take();
    });

    row.on_event(arkit::prelude::NodeEventType::TouchEvent, move |event| {
        let Some(input_event) = event.input_event() else {
            return;
        };
        let row_binding = runtime_node.borrow();
        let Some(node) = row_binding.as_ref() else {
            return;
        };

        match input_event.action {
            UIInputAction::Down => {
                let _ = node.background_color(menu_row_pressed_background(variant));
            }
            UIInputAction::Up | UIInputAction::Cancel => {
                let keep_active = active.unwrap_or(false);
                let _ = node.background_color(if keep_active {
                    menu_row_pressed_background(variant)
                } else {
                    TRANSPARENT
                });
            }
            UIInputAction::Move => {}
        }
    })
}
