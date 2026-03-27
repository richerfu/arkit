use super::floating_layer::{floating_panel_aligned, FloatingAlign, FloatingSide};
use super::*;
use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit::{component, create_signal};
use arkit_icon as lucide;
use std::cell::RefCell;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;
const MENU_PANEL_WIDTH: f32 = 208.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

#[derive(Debug, Clone, Copy)]
enum MenuInteractionVariant {
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

fn menu_content_with_width(width: f32, items: Vec<Element>) -> Element {
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

fn menu_content(items: Vec<Element>) -> Element {
    menu_content_with_width(MENU_PANEL_WIDTH, items)
}

fn item_text(content: impl Into<String>, color_value: u32, weight: i32) -> Element {
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

fn shortcut_text(content: impl Into<String>) -> Element {
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

fn leading_slot(child: Option<Element>) -> Element {
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

fn fill_slot(child: Element) -> Element {
    arkit::row_component()
        .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
        .children(vec![child])
        .into()
}

fn menu_row(children: Vec<Element>, disabled: bool) -> RowElement {
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
        .background_color(0x00000000)
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

fn apply_menu_row_background(node: &RuntimeMenuRowNode, color_value: u32) {
    let _ = node.background_color(color_value);
}

fn interactive_menu_row(
    children: Vec<Element>,
    disabled: bool,
    variant: MenuInteractionVariant,
    rest_background: u32,
) -> RowElement {
    let runtime_node = Rc::new(RefCell::new(None::<RuntimeMenuRowNode>));
    let capture_node = runtime_node.clone();
    let mut row = menu_row(children, disabled)
        .background_color(rest_background)
        .native(move |node| {
            capture_node.replace(Some(RuntimeMenuRowNode(node.borrow_mut().clone())));
            Ok(())
        });

    let detach_node = runtime_node.clone();
    row = row.on_event_no_param(arkit::prelude::NodeEventType::EventOnDetach, move || {
        detach_node.borrow_mut().take();
    });

    if disabled {
        return row;
    }

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
                apply_menu_row_background(node, menu_row_pressed_background(variant))
            }
            UIInputAction::Up | UIInputAction::Cancel => {
                apply_menu_row_background(node, rest_background)
            }
            UIInputAction::Move => {}
        }
    })
}

pub fn context_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
    let dismiss = {
        let open = open.clone();
        Rc::new(move || open.set(false))
    };
    floating_panel_aligned(
        trigger,
        menu_content(items),
        open.get(),
        FloatingSide::Bottom,
        FloatingAlign::Start,
        Some(dismiss),
    )
}

pub fn context_menu_item(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![fill_slot(item_text(
            title,
            color::POPOVER_FOREGROUND,
            3_i32,
        ))],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_item_destructive(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32))],
        false,
        MenuInteractionVariant::Destructive,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_item_inset(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_item_inset_destructive(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Destructive,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    interactive_menu_row(
        vec![
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn disabled_context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        true,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_sub_trigger_inset(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            lucide::icon("chevron-right")
                .size(16.0)
                .color(color::FOREGROUND)
                .render(),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_subcontent(items: Vec<Element>) -> Element {
    menu_content_with_width(SUBMENU_PANEL_WIDTH, items)
}

pub fn context_menu_submenu_inset_with_state(
    title: impl Into<String>,
    items: Vec<Element>,
    open: Signal<bool>,
) -> Element {
    let title = title.into();
    let is_open = open.get();
    let toggle = open.clone();
    let mut trigger = interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            lucide::icon(if is_open {
                "chevron-up"
            } else {
                "chevron-down"
            })
            .size(16.0)
            .color(color::FOREGROUND)
            .render(),
        ],
        false,
        MenuInteractionVariant::Default,
        if is_open { color::ACCENT } else { TRANSPARENT },
    );

    if is_open {
        trigger = trigger.style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 4.0, 0.0]);
    }

    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .children(vec![
            trigger
                .on_click(move || toggle.update(|value| *value = !*value))
                .into(),
            if is_open {
                arkit::column_component()
                    .percent_width(1.0)
                    .align_items_start()
                    .children(vec![context_menu_subcontent(items)])
                    .into()
            } else {
                arkit::row_component().height(0.0).into()
            },
        ])
        .into()
}

#[component]
pub fn context_menu_submenu_inset(title: impl Into<String>, items: Vec<Element>) -> Element {
    let open = create_signal(false);
    context_menu_submenu_inset_with_state(title, items, open)
}

pub fn context_menu_checkbox_item(title: impl Into<String>, checked: Signal<bool>) -> Element {
    let is_checked = checked.get();
    let toggle = checked.clone();

    interactive_menu_row(
        vec![
            leading_slot(if is_checked {
                Some(
                    lucide::icon("check")
                        .size(16.0)
                        .stroke_width(3.0)
                        .color(color::FOREGROUND)
                        .render(),
                )
            } else {
                None
            }),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .on_click(move || toggle.update(|value| *value = !*value))
    .into()
}

pub fn context_menu_label(title: impl Into<String>) -> Element {
    menu_row(
        vec![fill_slot(item_text(title, color::FOREGROUND, 4_i32))],
        false,
    )
    .into()
}

pub fn context_menu_label_inset(title: impl Into<String>) -> Element {
    menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::FOREGROUND, 4_i32)),
        ],
        false,
    )
    .into()
}

pub fn context_menu_separator() -> Element {
    arkit::row_component()
        .height(1.0)
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::Margin, vec![4.0, 0.0, 4.0, 0.0])
        .background_color(color::BORDER)
        .into()
}

pub fn context_menu_radio_item(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: Signal<String>,
) -> Element {
    let value = value.into();
    let is_selected = selected.get() == value;
    let set_selected = selected.clone();
    let click_value = value.clone();

    interactive_menu_row(
        vec![
            leading_slot(if is_selected {
                Some(
                    arkit::row_component()
                        .width(8.0)
                        .height(8.0)
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
                        )
                        .background_color(color::FOREGROUND)
                        .into(),
                )
            } else {
                None
            }),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .on_click(move || set_selected.set(click_value.clone()))
    .into()
}
