//! Dioxus host component for the native chart engine.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::rc::Rc;

use arkit_hooks::use_ark_node;
use arkit_prelude::*;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUIEvent};
use ohos_arkui_binding::types::advanced::NodeDirtyFlag;
use ohos_drawing_binding::Canvas;

use crate::model::{ChartEvent, ChartOption};
use crate::render::{
    drag_window_at, draw_option, hit_test_with_hidden, initial_windows, inside_zoom_at,
    nearest_axis_event, ZoomDrag, ZoomHandle, ZoomWindow,
};

/// Dioxus props for [`ECharts`].
#[derive(Props, Clone, PartialEq)]
pub struct EChartsProps {
    /// Complete chart option. Replacing this value triggers a native redraw.
    pub option: ChartOption,
    /// Fixed width in vp. Omit to use `percent_width`.
    #[props(default)]
    pub width: Option<f32>,
    /// Fixed height in vp. Defaults to 320 when neither height is specified.
    #[props(default)]
    pub height: Option<f32>,
    /// Relative width (`1.0` means 100%).
    #[props(default = 1.0)]
    pub percent_width: f32,
    /// Relative height. When set, it suppresses the default fixed height.
    #[props(default)]
    pub percent_height: Option<f32>,
    /// Called after a point, bar, sector, node, or region is selected.
    #[props(default)]
    pub on_select: Option<EventHandler<ChartEvent>>,
}

#[derive(Clone)]
struct ChartRenderState {
    option: Rc<RefCell<ChartOption>>,
    selected: Rc<RefCell<Option<ChartEvent>>>,
    hidden_series: Rc<RefCell<BTreeSet<usize>>>,
    zoom_windows: Rc<RefCell<Vec<ZoomWindow>>>,
    zoom_drag: Rc<RefCell<Option<ZoomDrag>>>,
}

impl ChartRenderState {
    fn new(option: ChartOption) -> Self {
        let zoom_windows = initial_windows(&option);
        Self {
            option: Rc::new(RefCell::new(option)),
            selected: Rc::new(RefCell::new(None)),
            hidden_series: Rc::new(RefCell::new(BTreeSet::new())),
            zoom_windows: Rc::new(RefCell::new(zoom_windows)),
            zoom_drag: Rc::new(RefCell::new(None)),
        }
    }

    fn update_option(&self, option: &ChartOption) {
        if *self.option.borrow() != *option {
            let reset_zoom = self.option.borrow().data_zoom != option.data_zoom;
            self.option.replace(option.clone());
            self.selected.replace(None);
            self.hidden_series
                .borrow_mut()
                .retain(|index| *index < option.series.len());
            if reset_zoom {
                self.zoom_windows.replace(initial_windows(option));
                self.zoom_drag.replace(None);
            }
        }
    }

    fn begin_zoom_drag(&self, hit: &ChartEvent, x: f32, y: f32) -> bool {
        if hit.component_type != "dataZoom" {
            return false;
        }
        self.begin_zoom_index(
            hit.series_index,
            match hit.data_index {
                0 => ZoomHandle::Start,
                1 => ZoomHandle::End,
                _ => ZoomHandle::Window,
            },
            x,
            y,
        )
    }

    fn begin_zoom_index(&self, data_zoom_index: usize, handle: ZoomHandle, x: f32, y: f32) -> bool {
        let Some(data_zoom) = self.option.borrow().data_zoom.get(data_zoom_index).cloned() else {
            return false;
        };
        let Some(window) = self.zoom_windows.borrow().get(data_zoom_index).copied() else {
            return false;
        };
        self.zoom_drag.replace(Some(ZoomDrag {
            data_zoom_index,
            handle,
            pointer_start: if data_zoom.orient == "vertical" { y } else { x },
            window_start: window,
        }));
        true
    }

    fn update_zoom_drag(
        &self,
        drag: ZoomDrag,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<ChartEvent> {
        let option = self.option.borrow();
        let window = drag_window_at(&option, drag, x, y, width, height)?;
        let mut windows = self.zoom_windows.borrow_mut();
        let source = option.data_zoom.get(drag.data_zoom_index)?;
        for (index, data_zoom) in option.data_zoom.iter().enumerate() {
            let shares_x_axis = data_zoom
                .x_axis_index
                .iter()
                .any(|axis| source.x_axis_index.contains(axis));
            let shares_y_axis = data_zoom
                .y_axis_index
                .iter()
                .any(|axis| source.y_axis_index.contains(axis));
            if shares_x_axis || shares_y_axis {
                if let Some(target) = windows.get_mut(index) {
                    *target = window;
                }
            }
        }
        Some(ChartEvent {
            series_index: drag.data_zoom_index,
            data_index: match drag.handle {
                ZoomHandle::Start => 0,
                ZoomHandle::End => 1,
                ZoomHandle::Window => 2,
            },
            series_name: None,
            name: Some(String::from("dataZoom")),
            value: vec![window.start, window.end],
            x,
            y,
            component_type: String::from("dataZoom"),
        })
    }
}

struct CustomEventNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for CustomEventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIEvent for CustomEventNode<'_> {}

/// Render an ECharts-compatible option through an ArkUI native canvas.
///
/// The component is controlled: read a Dioxus signal while constructing
/// `option`, pass the resulting value here, and every signal change redraws
/// the existing native node without remounting it.
#[component]
pub fn ECharts(props: EChartsProps) -> Element {
    let state = use_hook(|| ChartRenderState::new(props.option.clone()));
    state.update_option(&props.option);

    let node_ref = use_ark_node();
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    let select_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<ChartEvent>>)));
    select_handler.set(props.on_select);
    let draw_state = state.clone();
    let registered_for_effect = registered_node.clone();

    use_effect(move || {
        let Some(node) = node_ref.get() else {
            return;
        };
        let native_key = node.borrow().raw_handle() as usize;
        if registered_for_effect.get() != Some(native_key) {
            let draw_state = draw_state.clone();
            CustomEventNode(&mut node.borrow_mut()).on_custom_draw(move |event| {
                let Some(draw_context) = event.draw_context_in_draw() else {
                    return;
                };
                let Some(raw_canvas) = draw_context.canvas() else {
                    return;
                };
                let canvas = unsafe { Canvas::from_raw_borrowed(raw_canvas.as_ptr().cast()) };
                let size = draw_context.size();
                let pixel_ratio = pixel_ratio();
                let option = draw_state.option.borrow();
                let selected = draw_state.selected.borrow();
                canvas.save();
                canvas.scale(pixel_ratio, pixel_ratio);
                draw_option(
                    &option,
                    selected.as_ref(),
                    &draw_state.hidden_series.borrow(),
                    &draw_state.zoom_windows.borrow(),
                    Some(&canvas),
                    size.width as f32 / pixel_ratio,
                    size.height as f32 / pixel_ratio,
                );
                canvas.restore();
            });
            registered_for_effect.set(Some(native_key));
        }
        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
    });

    let fixed_height = if props.height.is_none() && props.percent_height.is_none() {
        Some(320.0)
    } else {
        props.height
    };
    let click_state = state.clone();
    let click_handler = select_handler.clone();
    rsx! {
        custom {
            width: if let Some(width) = props.width { width },
            height: if let Some(height) = fixed_height { height },
            percent_width: props.percent_width,
            percent_height: if let Some(height) = props.percent_height { height },
            hit_test_behavior: 0,
            ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                let Some(pointer) = event.data().pointer else {
                    return;
                };
                let Some(node) = node_ref.peek() else {
                    return;
                };
                let size = node
                    .borrow()
                    .layout_size()
                    .map(|size| (size.width.max(1) as f32, size.height.max(1) as f32))
                    .unwrap_or((1.0, 1.0));
                let ratio = pixel_ratio();
                let logical_size = (size.0 / ratio, size.1 / ratio);
                let target_is_logical = pointer.target_width > 0.0
                    && (pointer.target_width - logical_size.0).abs()
                        <= (pointer.target_width - size.0).abs();
                let (x, y) = if target_is_logical || pointer.target_width <= 0.0 {
                    (pointer.x, pointer.y)
                } else {
                    (pointer.x / ratio, pointer.y / ratio)
                };
                let action = pointer.action;
                if action == dioxus_elements::event::PointerAction::Cancel {
                    click_state.zoom_drag.replace(None);
                    return;
                }
                if action == dioxus_elements::event::PointerAction::Move {
                    let Some(drag) = *click_state.zoom_drag.borrow() else {
                        if click_state.option.borrow().tooltip.trigger == "axis" {
                            let selected = nearest_axis_event(
                                &click_state.option.borrow(),
                                x,
                                y,
                                logical_size.0,
                                logical_size.1,
                                &click_state.hidden_series.borrow(),
                                &click_state.zoom_windows.borrow(),
                            );
                            click_state.selected.replace(selected);
                            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        }
                        return;
                    };
                    click_state.update_zoom_drag(
                        drag,
                        x,
                        y,
                        logical_size.0,
                        logical_size.1,
                    );
                    let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                    return;
                }
                let hit = hit_test_with_hidden(
                    &click_state.option.borrow(),
                    x,
                    y,
                    logical_size.0,
                    logical_size.1,
                    &click_state.hidden_series.borrow(),
                    &click_state.zoom_windows.borrow(),
                );
                if action == dioxus_elements::event::PointerAction::Down {
                    let began_slider = hit
                        .as_ref()
                        .is_some_and(|hit| click_state.begin_zoom_drag(hit, x, y));
                    if !began_slider {
                        if let Some(index) = inside_zoom_at(
                            &click_state.option.borrow(),
                            x,
                            y,
                            logical_size.0,
                            logical_size.1,
                        ) {
                            click_state.begin_zoom_index(index, ZoomHandle::Window, x, y);
                        }
                    }
                    return;
                }
                if action != dioxus_elements::event::PointerAction::Up {
                    return;
                }
                if let Some(drag) = click_state.zoom_drag.take() {
                    let is_inside_tap = click_state
                        .option
                        .borrow()
                        .data_zoom
                        .get(drag.data_zoom_index)
                        .is_some_and(|data_zoom| {
                            let pointer = if data_zoom.orient == "vertical" { y } else { x };
                            data_zoom.kind == "inside"
                                && (pointer - drag.pointer_start).abs() < 3.0
                        });
                    if !is_inside_tap {
                        let event = click_state.update_zoom_drag(
                            drag,
                            x,
                            y,
                            logical_size.0,
                            logical_size.1,
                        );
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        if let (Some(event), Some(handler)) = (event, click_handler.get()) {
                            handler.call(event);
                        }
                        return;
                    }
                }
                if let Some(hit) = hit.as_ref().filter(|hit| hit.component_type == "legend") {
                    let mut hidden = click_state.hidden_series.borrow_mut();
                    if !hidden.insert(hit.series_index) {
                        hidden.remove(&hit.series_index);
                    }
                    click_state.selected.replace(None);
                } else {
                    click_state.selected.replace(hit.clone());
                }
                let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                if let (Some(hit), Some(handler)) = (hit, click_handler.get()) {
                    handler.call(hit);
                }
            },
        }
    }
}

fn pixel_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Series;

    #[test]
    fn render_state_replaces_option_and_clears_stale_selection() {
        let initial = ChartOption::new()
            .title("initial")
            .push_series(Series::line("line", [1.0]));
        let state = ChartRenderState::new(initial);
        state.selected.replace(Some(ChartEvent {
            series_index: 0,
            data_index: 0,
            series_name: Some(String::from("line")),
            name: None,
            value: vec![1.0],
            x: 1.0,
            y: 1.0,
            component_type: String::from("line"),
        }));

        let next = ChartOption::new()
            .title("next")
            .push_series(Series::bar("bar", [2.0]));
        state.update_option(&next);

        assert_eq!(state.option.borrow().title.as_ref().unwrap().text, "next");
        assert!(state.selected.borrow().is_none());
    }
}
