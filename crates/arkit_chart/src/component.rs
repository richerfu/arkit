//! Dioxus host component for the native chart engine.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::BTreeSet;
use std::ops::Deref;
use std::rc::Rc;

use arkit_hooks::{use_app_foreground, use_ark_node, use_component_visibility};
use arkit_prelude::*;
use dioxus_core::use_drop;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUIEvent};
use ohos_arkui_binding::types::advanced::NodeDirtyFlag;
use ohos_drawing_binding::Canvas;

use crate::animation::{ChartAnimationClock, ChartTransition, ChartTransitionDriver};
use crate::export::{save_chart_image, ExportContext};
use crate::model::{
    BrushArea, ChartAction, ChartActionKind, ChartActionTarget, ChartAppendData,
    ChartCoordinateFinder, ChartCoordinatePoint, ChartEvent, ChartOption, ChartRuntimeEvent,
    ChartRuntimeEventBatchItem, ChartSelectedItems, DataPoint, Series,
};
use crate::render::{
    cartesian_plot_at, coordinate_contains_pixel, coordinate_from_pixel, coordinate_to_pixel,
    drag_window_at, draw_option_with_domain, draw_toolbox_zoom_selection, hit_test_with_hidden,
    initial_windows, inside_zoom_at, nearest_axis_event, nearest_axis_event_from_hits, HitRegion,
    ZoomDrag, ZoomHandle, ZoomWindow,
};

#[derive(Debug, Clone, PartialEq)]
enum ChartCommand {
    Action(ChartAction),
    Actions(Vec<ChartAction>),
    AppendData(ChartAppendData),
    Clear,
}

type ChartCommandDispatcher = Rc<dyn Fn(ChartCommand)>;
type ChartOptionReader = Rc<dyn Fn() -> ChartOption>;
type ChartSizeReader = Rc<dyn Fn() -> [f32; 2]>;

struct ChartControllerBinding {
    id: u64,
    dispatcher: ChartCommandDispatcher,
    option_reader: ChartOptionReader,
    size_reader: ChartSizeReader,
}

#[derive(Default)]
struct ChartControllerState {
    next_binding: u64,
    binding: Option<ChartControllerBinding>,
    pending: Vec<ChartCommand>,
}

/// Imperative ECharts-compatible action handle. Create one with
/// [`ChartController::new`], pass it to [`EChartsProps::controller`], then call
/// [`dispatch_action`](Self::dispatch_action) from any UI callback.
#[derive(Clone, Default)]
pub struct ChartController {
    inner: Rc<RefCell<ChartControllerState>>,
}

impl ChartController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dispatch_action(&self, action: ChartAction) {
        self.dispatch(ChartCommand::Action(action));
    }

    /// Dispatch multiple actions as one ECharts-style batch and emit one
    /// aggregate event after all state mutations have completed.
    pub fn dispatch_actions(&self, actions: impl IntoIterator<Item = ChartAction>) {
        self.dispatch(ChartCommand::Actions(actions.into_iter().collect()));
    }

    /// Append a chunk without replacing the current option. This follows
    /// ECharts' native incremental-series restriction: scatter and lines.
    pub fn append_data(&self, data: ChartAppendData) {
        self.dispatch(ChartCommand::AppendData(data));
    }

    /// Clear the mounted instance. A later controlled prop update can populate
    /// it again, matching the relationship between `clear` and `setOption`.
    pub fn clear(&self) {
        self.dispatch(ChartCommand::Clear);
    }

    /// Return the current resolved option, including runtime legend/dataZoom
    /// and selected-item state. Returns `None` before mount or after unmount.
    pub fn get_option(&self) -> Option<ChartOption> {
        let reader = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.option_reader.clone())?;
        Some(reader())
    }

    /// Current logical canvas width and height in vp.
    pub fn get_size(&self) -> Option<[f32; 2]> {
        let reader = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.size_reader.clone())?;
        Some(reader())
    }

    pub fn get_width(&self) -> Option<f32> {
        self.get_size().map(|size| size[0])
    }

    pub fn get_height(&self) -> Option<f32> {
        self.get_size().map(|size| size[1])
    }

    /// Convert a cartesian data point into logical canvas pixels using the
    /// current responsive layout and dataZoom state.
    pub fn convert_to_pixel(
        &self,
        finder: ChartCoordinateFinder,
        value: ChartCoordinatePoint,
    ) -> Option<[f32; 2]> {
        let option = self.get_option()?;
        let [width, height] = self.get_size()?;
        coordinate_to_pixel(
            &option,
            &finder,
            &value,
            &initial_hidden_series(&option),
            &initial_windows(&option),
            width,
            height,
        )
    }

    /// Convert logical canvas pixels back into cartesian data/category values.
    pub fn convert_from_pixel(
        &self,
        finder: ChartCoordinateFinder,
        pixel: [f32; 2],
    ) -> Option<ChartCoordinatePoint> {
        let option = self.get_option()?;
        let [width, height] = self.get_size()?;
        coordinate_from_pixel(
            &option,
            &finder,
            pixel,
            &initial_hidden_series(&option),
            &initial_windows(&option),
            width,
            height,
        )
    }

    /// Test whether a logical pixel lies inside the selected cartesian grid.
    pub fn contain_pixel(&self, finder: ChartCoordinateFinder, pixel: [f32; 2]) -> Option<bool> {
        let option = self.get_option()?;
        let [width, height] = self.get_size()?;
        coordinate_contains_pixel(
            &option,
            &finder,
            pixel,
            &initial_hidden_series(&option),
            &initial_windows(&option),
            width,
            height,
        )
    }

    fn dispatch(&self, command: ChartCommand) {
        let dispatcher = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.dispatcher.clone());
        if let Some(dispatcher) = dispatcher {
            dispatcher(command);
        } else {
            self.inner.borrow_mut().pending.push(command);
        }
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().binding.is_some()
    }

    fn bind(
        &self,
        dispatcher: ChartCommandDispatcher,
        option_reader: ChartOptionReader,
        size_reader: ChartSizeReader,
    ) -> u64 {
        let (binding, pending) = {
            let mut state = self.inner.borrow_mut();
            state.next_binding = state
                .next_binding
                .checked_add(1)
                .expect("arkit_chart: controller binding id space exhausted");
            let binding = state.next_binding;
            state.binding = Some(ChartControllerBinding {
                id: binding,
                dispatcher: dispatcher.clone(),
                option_reader,
                size_reader,
            });
            (binding, std::mem::take(&mut state.pending))
        };
        for command in pending {
            dispatcher(command);
        }
        binding
    }

    fn unbind(&self, binding: u64) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.binding = None;
        }
    }
}

impl std::fmt::Debug for ChartController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ChartController")
            .field("mounted", &self.is_mounted())
            .finish_non_exhaustive()
    }
}

impl PartialEq for ChartController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

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
    /// Imperative `dispatchAction` bridge.
    #[props(default)]
    pub controller: Option<ChartController>,
    /// Unified ECharts-style pointer and component action events.
    #[props(default)]
    pub on_event: Option<EventHandler<ChartRuntimeEvent>>,
}

struct SharedChartOption(RefCell<Rc<ChartOption>>);

impl SharedChartOption {
    fn new(option: ChartOption) -> Self {
        Self(RefCell::new(Rc::new(option)))
    }

    fn borrow(&self) -> Ref<'_, ChartOption> {
        Ref::map(self.0.borrow(), |option| option.as_ref())
    }

    fn borrow_mut(&self) -> RefMut<'_, ChartOption> {
        RefMut::map(self.0.borrow_mut(), Rc::make_mut)
    }

    fn replace(&self, option: ChartOption) {
        self.replace_shared(Rc::new(option));
    }

    fn replace_shared(&self, option: Rc<ChartOption>) {
        *self.0.borrow_mut() = option;
    }

    fn snapshot(&self) -> Rc<ChartOption> {
        self.0.borrow().clone()
    }
}

#[derive(Clone)]
struct ChartRenderState {
    inner: Rc<ChartRenderStateInner>,
}

impl Deref for ChartRenderState {
    type Target = ChartRenderStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

struct ChartRenderStateInner {
    prop_option: RefCell<ChartOption>,
    source_option: RefCell<ChartOption>,
    option: SharedChartOption,
    selected: RefCell<Option<ChartEvent>>,
    action_tooltip: RefCell<Option<ChartEvent>>,
    highlighted: RefCell<Option<ChartEvent>>,
    draw_hits: RefCell<Vec<HitRegion>>,
    hidden_series: RefCell<BTreeSet<usize>>,
    selected_items: RefCell<BTreeSet<(usize, usize)>>,
    zoom_windows: RefCell<Vec<ZoomWindow>>,
    zoom_drag: RefCell<Option<ZoomDrag>>,
    map_drag: RefCell<Option<MapDrag>>,
    label_drag: RefCell<Option<LabelDrag>>,
    brush_drag: RefCell<Option<BrushDrag>>,
    toolbox_zoom_active: Cell<bool>,
    toolbox_zoom_drag: RefCell<Option<BrushDrag>>,
    toolbox_zoom_history: RefCell<Vec<Vec<ZoomWindow>>>,
    magic_override: Cell<MagicOverride>,
    magic_stack_override: Cell<Option<bool>>,
    data_view_visible: Cell<bool>,
    media_signature: RefCell<Vec<isize>>,
    media_timeline_index: Cell<usize>,
    media_size: Cell<(f32, f32)>,
    transition: RefCell<Option<ChartTransition>>,
    state_transition: RefCell<Option<ChartTransition>>,
    transition_driver: ChartTransitionDriver,
    state_transition_driver: ChartTransitionDriver,
    state_key: RefCell<StateKey>,
    state_target: RefCell<Option<Rc<ChartOption>>>,
    timeline_elapsed_ms: Cell<u64>,
    effect_elapsed_ms: Cell<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StateKey {
    hovered: Option<(usize, usize)>,
    selected: BTreeSet<(usize, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MagicOverride {
    None,
    Line,
    Bar,
}

#[derive(Debug, Clone, Copy)]
struct MapDrag {
    series_index: usize,
    pointer_start: (f32, f32),
    pan_start: [f32; 2],
    pointer_last: (f32, f32),
}

#[derive(Debug, Clone, Copy)]
struct LabelDrag {
    series_index: usize,
    label_index: usize,
    pointer_start: (f32, f32),
    offset_start: [f32; 2],
    pointer_last: (f32, f32),
}

#[derive(Debug, Clone, Copy)]
struct BrushDrag {
    area_index: usize,
    pointer_start: (f32, f32),
    pointer_last: (f32, f32),
}

impl ChartRenderState {
    #[cfg(test)]
    fn new(option: ChartOption) -> Self {
        Self::with_drivers(
            option,
            ChartTransitionDriver::immediate(),
            ChartTransitionDriver::immediate(),
        )
    }

    fn with_drivers(
        option: ChartOption,
        transition_driver: ChartTransitionDriver,
        state_transition_driver: ChartTransitionDriver,
    ) -> Self {
        let zoom_windows = initial_windows(&option);
        let selected_items = initial_selected_items(&option);
        let hidden_series = initial_hidden_series(&option);
        let option = SharedChartOption::new(option);
        let transition = ChartTransition::initial(option.snapshot(), transition_driver.clone());
        let prop_option = option.borrow().clone();
        let source_option = prop_option.clone();
        Self {
            inner: Rc::new(ChartRenderStateInner {
                prop_option: RefCell::new(prop_option),
                source_option: RefCell::new(source_option),
                option,
                selected: RefCell::new(None),
                action_tooltip: RefCell::new(None),
                highlighted: RefCell::new(None),
                draw_hits: RefCell::new(Vec::new()),
                hidden_series: RefCell::new(hidden_series),
                selected_items: RefCell::new(selected_items),
                zoom_windows: RefCell::new(zoom_windows.clone()),
                zoom_drag: RefCell::new(None),
                map_drag: RefCell::new(None),
                label_drag: RefCell::new(None),
                brush_drag: RefCell::new(None),
                toolbox_zoom_active: Cell::new(false),
                toolbox_zoom_drag: RefCell::new(None),
                toolbox_zoom_history: RefCell::new(vec![zoom_windows]),
                magic_override: Cell::new(MagicOverride::None),
                magic_stack_override: Cell::new(None),
                data_view_visible: Cell::new(false),
                media_signature: RefCell::new(Vec::new()),
                media_timeline_index: Cell::new(usize::MAX),
                media_size: Cell::new((0.0, 0.0)),
                transition: RefCell::new(transition),
                state_transition: RefCell::new(None),
                transition_driver,
                state_transition_driver,
                state_key: RefCell::new(StateKey::default()),
                state_target: RefCell::new(None),
                timeline_elapsed_ms: Cell::new(0),
                effect_elapsed_ms: Cell::new(0),
            }),
        }
    }

    fn animated_option(&self) -> Rc<ChartOption> {
        let mut transition = self.transition.borrow_mut();
        let Some(active) = transition.as_ref() else {
            return self.option.snapshot();
        };
        let (snapshot, finished) = active.snapshot();
        if finished {
            transition.take();
            self.option.snapshot()
        } else {
            snapshot
        }
    }

    fn rendered_option(&self) -> Rc<ChartOption> {
        let base = self.animated_option();
        let selected = self.selected.borrow();
        let action_tooltip = self.action_tooltip.borrow();
        let highlighted = self.highlighted.borrow();
        let active = highlighted
            .as_ref()
            .or(action_tooltip.as_ref())
            .or(selected.as_ref());
        let selected_items = self.selected_items.borrow();
        let key = StateKey {
            hovered: active
                .filter(|event| event.series_index < base.series.len())
                .map(|event| (event.series_index, event.data_index)),
            selected: selected_items.clone(),
        };
        let has_active_state = active.is_some() || !selected_items.is_empty();
        let key_changed = *self.state_key.borrow() != key;
        let main_transition_active = self.transition.borrow().is_some();

        let make_target = || {
            if !has_active_state {
                return base.clone();
            }
            let mut target = (*base).clone();
            crate::state::apply_states(&mut target, active, &selected_items);
            Rc::new(target)
        };

        if key_changed {
            let target = make_target();
            let from = self
                .state_transition
                .borrow()
                .as_ref()
                .map(ChartTransition::current)
                .or_else(|| self.state_target.borrow().clone())
                .unwrap_or_else(|| base.clone());
            self.state_transition.replace(ChartTransition::state(
                from,
                target.clone(),
                self.state_transition_driver.clone(),
            ));
            self.state_target
                .replace(has_active_state.then_some(target));
            self.state_key.replace(key);
        } else if has_active_state
            && main_transition_active
            && self.state_transition.borrow().is_none()
        {
            // Keep an already-active hover/selection state attached to a base
            // series that is itself moving. This is the only overlap that
            // requires rebuilding a state target during a frame.
            self.state_target.replace(Some(make_target()));
        }
        drop(selected_items);
        drop(highlighted);
        drop(action_tooltip);
        drop(selected);
        let state_snapshot = self
            .state_transition
            .borrow()
            .as_ref()
            .map(ChartTransition::snapshot);
        let rendered = if let Some((snapshot, finished)) = state_snapshot {
            if finished {
                self.state_transition.borrow_mut().take();
                self.state_target
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| base.clone())
            } else {
                snapshot
            }
        } else {
            self.state_target
                .borrow()
                .clone()
                .unwrap_or_else(|| base.clone())
        };
        rendered
    }

    fn update_option(&self, option: &ChartOption) {
        if *self.prop_option.borrow() == *option {
            return;
        }
        self.prop_option.replace(option.clone());
        self.source_option.replace(option.clone());
        self.media_signature.borrow_mut().clear();
        self.media_timeline_index.set(usize::MAX);
        let (width, height) = self.media_size.get();
        if width > 0.0 && height > 0.0 {
            self.apply_media(width, height);
        } else {
            self.replace_option(option.clone());
        }
    }

    fn replace_option(&self, mut next: ChartOption) {
        apply_magic_override(&mut next, self.magic_override.get());
        apply_magic_stack_override(&mut next, self.magic_stack_override.get());
        set_toolbox_runtime_status(
            &mut next,
            "dataZoom",
            "__active",
            self.toolbox_zoom_active.get(),
        );
        set_toolbox_runtime_status(
            &mut next,
            "dataZoom",
            "__canBack",
            self.toolbox_zoom_history.borrow().len() > 1,
        );
        set_toolbox_runtime_status(
            &mut next,
            "dataView",
            "__visible",
            self.data_view_visible.get(),
        );
        {
            let current = self.option.borrow();
            if next.media.is_none()
                && !next.timeline_options.is_empty()
                && next.timeline_options == current.timeline_options
            {
                let index = current
                    .timeline
                    .as_ref()
                    .map_or(0, |timeline| timeline.current_index);
                next.apply_timeline_index(index);
            }
            if let (Some(next_brush), Some(current_brush)) =
                (next.brush.as_mut(), current.brush.as_ref())
            {
                let mut next_config = next_brush.clone();
                let mut current_config = current_brush.clone();
                next_config.areas.clear();
                current_config.areas.clear();
                if next_config == current_config {
                    next_brush.areas = current_brush.areas.clone();
                    next_brush.active = current_brush.active;
                }
            }
        }
        {
            let current = self.option.borrow();
            for (index, series) in next.series.iter_mut().enumerate() {
                if let (Some(next_layout), Some(current_layout)) = (
                    series_label_layout_mut(series),
                    current.series.get(index).and_then(series_label_layout),
                ) {
                    let mut next_config = next_layout.clone();
                    let mut current_config = current_layout.clone();
                    let callbacks_compatible =
                        next_config.callback.is_some() == current_config.callback.is_some();
                    next_config.callback = None;
                    current_config.callback = None;
                    next_config.drag_offsets.clear();
                    current_config.drag_offsets.clear();
                    if callbacks_compatible && next_config == current_config {
                        next_layout.drag_offsets = current_layout.drag_offsets.clone();
                    }
                }
                if let (
                    crate::model::Series::Map(next_map),
                    Some(crate::model::Series::Map(current_map)),
                ) = (series, current.series.get(index))
                {
                    next_map.map_options.pan_offset = current_map.map_options.pan_offset;
                }
            }
        }
        if *self.option.borrow() != next {
            let previous_visual = self.animated_option();
            let reset_zoom = self.option.borrow().data_zoom != next.data_zoom;
            let reset_hidden = self.option.borrow().legend != next.legend;
            let initial_hidden = reset_hidden.then(|| initial_hidden_series(&next));
            if let Some(initial_hidden) = initial_hidden {
                self.hidden_series.replace(initial_hidden);
            } else {
                self.hidden_series
                    .borrow_mut()
                    .retain(|index| *index < next.series.len());
            }
            self.selected_items
                .borrow_mut()
                .retain(|(series_index, data_index)| {
                    next.series
                        .get(*series_index)
                        .is_some_and(|series| match series {
                            crate::model::Series::Map(series) => {
                                *data_index < series.features.len()
                            }
                            crate::model::Series::Line(series)
                            | crate::model::Series::Bar(series)
                            | crate::model::Series::Pie(series)
                            | crate::model::Series::Scatter(series)
                            | crate::model::Series::EffectScatter(series)
                            | crate::model::Series::Radar(series)
                            | crate::model::Series::Gauge(series)
                            | crate::model::Series::Funnel(series)
                            | crate::model::Series::Heatmap(series)
                            | crate::model::Series::Candlestick(series)
                            | crate::model::Series::Boxplot(series)
                            | crate::model::Series::PictorialBar(series)
                            | crate::model::Series::Parallel(series)
                            | crate::model::Series::ThemeRiver(series)
                            | crate::model::Series::Treemap(series) => {
                                *data_index < series.data.len()
                            }
                            crate::model::Series::Tree(series)
                            | crate::model::Series::Graph(series) => {
                                *data_index < series.nodes.len()
                            }
                            crate::model::Series::Sankey(series) => {
                                *data_index < series.nodes.len()
                            }
                            _ => false,
                        })
                });
            let initial_selected = initial_selected_items(&next);
            let initial_zoom = reset_zoom.then(|| initial_windows(&next));
            let next = Rc::new(next);
            let transition = ChartTransition::update(
                previous_visual,
                next.clone(),
                self.transition_driver.clone(),
            );
            self.option.replace_shared(next);
            self.transition.replace(transition);
            self.selected.replace(None);
            let remap = |event: Option<ChartEvent>| {
                let event = event?;
                resolve_action_target(
                    &self.option.borrow(),
                    &ChartActionTarget::item(event.series_index, event.data_index),
                )
            };
            let highlighted = remap(self.highlighted.take());
            self.highlighted.replace(highlighted);
            let action_tooltip = remap(self.action_tooltip.take());
            self.action_tooltip.replace(action_tooltip);
            self.selected_items.borrow_mut().extend(initial_selected);
            if let Some(windows) = initial_zoom {
                self.zoom_windows.replace(windows.clone());
                self.toolbox_zoom_history.replace(vec![windows]);
                self.zoom_drag.replace(None);
                self.toolbox_zoom_drag.replace(None);
            }
            self.brush_drag.replace(None);
        }
    }

    fn apply_media(&self, width: f32, height: f32) {
        self.media_size.set((width, height));
        let source = self.source_option.borrow();
        let timeline_index = self
            .option
            .borrow()
            .timeline
            .as_ref()
            .map_or(0, |timeline| timeline.current_index);
        let signature = crate::parser::media_signature(&source, width, height);
        if *self.media_signature.borrow() == signature
            && self.media_timeline_index.get() == timeline_index
        {
            return;
        }
        let resolved = crate::parser::resolve_media_option(&source, width, height, timeline_index)
            .unwrap_or_else(|_| source.clone());
        drop(source);
        self.media_signature.replace(signature);
        self.media_timeline_index.set(timeline_index);
        self.replace_option(resolved);
    }

    fn runtime_option(&self) -> ChartOption {
        let mut snapshot = self.option.borrow().clone();
        if let Some(legend) = snapshot.legend.as_mut() {
            let hidden = self.hidden_series.borrow();
            for (index, series) in snapshot.series.iter().enumerate() {
                if let Some(name) = series.name() {
                    legend
                        .selected
                        .insert(name.to_string(), !hidden.contains(&index));
                }
            }
        }
        for (data_zoom, window) in snapshot
            .data_zoom
            .iter_mut()
            .zip(self.zoom_windows.borrow().iter())
        {
            data_zoom.start = window.start;
            data_zoom.end = window.end;
        }
        apply_selected_snapshot(&mut snapshot, &self.selected_items.borrow());
        snapshot
    }

    fn append_data(&self, chunk: &ChartAppendData) -> bool {
        if chunk.is_empty() {
            return false;
        }
        let previous_visual = self.animated_option();
        let changed = append_data_to_option(&mut self.option.borrow_mut(), chunk);
        if !changed {
            return false;
        }
        append_data_to_option(&mut self.source_option.borrow_mut(), chunk);
        let target = self.option.snapshot();
        self.transition.replace(ChartTransition::update(
            previous_visual,
            target,
            self.transition_driver.clone(),
        ));
        true
    }

    fn clear(&self) {
        let option = ChartOption::new();
        self.source_option.replace(option.clone());
        self.option.replace(option.clone());
        self.selected.replace(None);
        self.action_tooltip.replace(None);
        self.highlighted.replace(None);
        self.draw_hits.borrow_mut().clear();
        self.hidden_series.borrow_mut().clear();
        self.selected_items.borrow_mut().clear();
        let windows = initial_windows(&option);
        self.zoom_windows.replace(windows.clone());
        self.toolbox_zoom_history.replace(vec![windows]);
        self.zoom_drag.replace(None);
        self.map_drag.replace(None);
        self.label_drag.replace(None);
        self.brush_drag.replace(None);
        self.toolbox_zoom_active.set(false);
        self.toolbox_zoom_drag.replace(None);
        self.magic_override.set(MagicOverride::None);
        self.magic_stack_override.set(None);
        self.data_view_visible.set(false);
        self.media_signature.borrow_mut().clear();
        self.media_timeline_index.set(usize::MAX);
        self.transition.replace(None);
        self.state_transition.replace(None);
        self.state_key.replace(StateKey::default());
        self.state_target.replace(None);
    }

    fn begin_brush(&self, x: f32, y: f32) -> bool {
        let mut option = self.option.borrow_mut();
        let Some(brush) = option.brush.as_mut().filter(|brush| brush.active) else {
            return false;
        };
        if brush.brush_mode != "multiple" {
            brush.areas.clear();
        }
        brush.areas.push(BrushArea {
            start: [x, y],
            end: [x, y],
        });
        let area_index = brush.areas.len() - 1;
        self.brush_drag.replace(Some(BrushDrag {
            area_index,
            pointer_start: (x, y),
            pointer_last: (x, y),
        }));
        true
    }

    fn update_brush_drag(&self, mut drag: BrushDrag, x: f32, y: f32) {
        if let Some(area) = self
            .option
            .borrow_mut()
            .brush
            .as_mut()
            .and_then(|brush| brush.areas.get_mut(drag.area_index))
        {
            area.end = [x, y];
        }
        drag.pointer_last = (x, y);
        self.brush_drag.replace(Some(drag));
    }

    fn finish_brush(&self, drag: BrushDrag, x: f32, y: f32) -> ChartEvent {
        self.update_brush_drag(drag, x, y);
        self.brush_drag.replace(None);
        let distance =
            ((x - drag.pointer_start.0).powi(2) + (y - drag.pointer_start.1).powi(2)).sqrt();
        let mut option = self.option.borrow_mut();
        let brush = option.brush.as_mut().expect("active brush must exist");
        if distance < 3.0 && brush.remove_on_click {
            brush.areas.clear();
        }
        let area = brush.areas.get(drag.area_index).copied();
        let value = area.map_or_else(Vec::new, |area| {
            vec![
                area.start[0] as f64,
                area.start[1] as f64,
                area.end[0] as f64,
                area.end[1] as f64,
            ]
        });
        ChartEvent {
            series_index: 0,
            data_index: drag.area_index,
            series_name: None,
            name: Some(String::from("brushSelected")),
            value,
            x,
            y,
            component_type: String::from("brush"),
        }
    }

    fn begin_toolbox_zoom(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        if !self.toolbox_zoom_active.get()
            || cartesian_plot_at(&self.option.borrow(), x, y, width, height).is_none()
        {
            return false;
        }
        self.toolbox_zoom_drag.replace(Some(BrushDrag {
            area_index: 0,
            pointer_start: (x, y),
            pointer_last: (x, y),
        }));
        true
    }

    fn update_toolbox_zoom_drag(&self, mut drag: BrushDrag, x: f32, y: f32) {
        drag.pointer_last = (x, y);
        self.toolbox_zoom_drag.replace(Some(drag));
    }

    fn finish_toolbox_zoom(
        &self,
        drag: BrushDrag,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> ChartEvent {
        let Some((plot_x, plot_y, plot_width, plot_height)) = cartesian_plot_at(
            &self.option.borrow(),
            drag.pointer_start.0,
            drag.pointer_start.1,
            width,
            height,
        ) else {
            return toolbox_event("dataZoom", x, y);
        };
        let x1 = drag.pointer_start.0.clamp(plot_x, plot_x + plot_width);
        let x2 = x.clamp(plot_x, plot_x + plot_width);
        let y1 = drag.pointer_start.1.clamp(plot_y, plot_y + plot_height);
        let y2 = y.clamp(plot_y, plot_y + plot_height);
        let option = self.option.borrow();
        let controls_x = option.data_zoom.iter().any(|data_zoom| {
            data_zoom
                .extra
                .get("toolboxInternal")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                && !data_zoom.x_axis_index.is_empty()
        });
        let controls_y = option.data_zoom.iter().any(|data_zoom| {
            data_zoom
                .extra
                .get("toolboxInternal")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                && !data_zoom.y_axis_index.is_empty()
        });
        if (controls_x && (x2 - x1).abs() < 3.0) || (controls_y && (y2 - y1).abs() < 3.0) {
            return toolbox_event("dataZoom", x, y);
        }
        let x_start = ((x1.min(x2) - plot_x) / plot_width).clamp(0.0, 1.0) as f64;
        let x_end = ((x1.max(x2) - plot_x) / plot_width).clamp(0.0, 1.0) as f64;
        let y_start = (1.0 - (y1.max(y2) - plot_y) / plot_height).clamp(0.0, 1.0) as f64;
        let y_end = (1.0 - (y1.min(y2) - plot_y) / plot_height).clamp(0.0, 1.0) as f64;
        let mut windows = self.zoom_windows.borrow_mut();
        for (index, data_zoom) in option.data_zoom.iter().enumerate() {
            if !data_zoom
                .extra
                .get("toolboxInternal")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            let Some(window) = windows.get_mut(index) else {
                continue;
            };
            let old = *window;
            let span = old.end - old.start;
            let (start, end) = if !data_zoom.x_axis_index.is_empty() {
                (x_start, x_end)
            } else {
                (y_start, y_end)
            };
            *window = ZoomWindow::new(old.start + span * start, old.start + span * end);
        }
        drop(option);
        let snapshot = windows.clone();
        drop(windows);
        if self.toolbox_zoom_history.borrow().last() != Some(&snapshot) {
            self.toolbox_zoom_history.borrow_mut().push(snapshot);
        }
        set_toolbox_runtime_status(
            &mut self.option.borrow_mut(),
            "dataZoom",
            "__canBack",
            self.toolbox_zoom_history.borrow().len() > 1,
        );
        toolbox_event("dataZoom", x, y)
    }

    fn toggle_toolbox_zoom(&self) {
        let active = !self.toolbox_zoom_active.get();
        self.toolbox_zoom_active.set(active);
        if active {
            if let Some(brush) = self.option.borrow_mut().brush.as_mut() {
                brush.active = false;
            }
        } else {
            self.toolbox_zoom_drag.replace(None);
        }
        set_toolbox_runtime_status(
            &mut self.option.borrow_mut(),
            "dataZoom",
            "__active",
            active,
        );
    }

    fn toolbox_zoom_back(&self) {
        let mut history = self.toolbox_zoom_history.borrow_mut();
        if history.len() > 1 {
            history.pop();
            if let Some(previous) = history.last() {
                self.zoom_windows.replace(previous.clone());
            }
        }
        let can_back = history.len() > 1;
        drop(history);
        set_toolbox_runtime_status(
            &mut self.option.borrow_mut(),
            "dataZoom",
            "__canBack",
            can_back,
        );
    }

    fn activate_magic_type(&self, mode: MagicOverride) {
        self.magic_override.set(mode);
        apply_magic_override(&mut self.option.borrow_mut(), mode);
    }

    fn toggle_magic_stack(&self) {
        let stacked = self.magic_stack_override.get() != Some(true);
        self.magic_stack_override.set(Some(stacked));
        apply_magic_stack_override(&mut self.option.borrow_mut(), Some(stacked));
    }

    fn restore(&self) {
        self.magic_override.set(MagicOverride::None);
        self.magic_stack_override.set(None);
        self.data_view_visible.set(false);
        self.toolbox_zoom_active.set(false);
        self.toolbox_zoom_drag.replace(None);
        let source = self.source_option.borrow();
        let (width, height) = self.media_size.get();
        let initial_timeline_index = source
            .timeline
            .as_ref()
            .map_or(0, |timeline| timeline.current_index);
        let restore_timeline_index = if source.timeline_options.is_empty() {
            initial_timeline_index
        } else {
            0
        };
        let mut option = if width > 0.0 && height > 0.0 {
            crate::parser::resolve_media_option(&source, width, height, restore_timeline_index)
                .unwrap_or_else(|_| source.clone())
        } else {
            source.clone()
        };
        let signature = crate::parser::media_signature(&source, width, height);
        drop(source);
        if let Some(brush) = option.brush.as_mut() {
            brush.areas.clear();
            brush.active = false;
        }
        if option
            .timeline
            .as_ref()
            .is_some_and(|timeline| timeline.current_index != restore_timeline_index)
        {
            option.apply_timeline_index(0);
        }
        let windows = initial_windows(&option);
        self.media_signature.replace(signature);
        self.media_timeline_index.set(restore_timeline_index);
        let hidden_series = initial_hidden_series(&option);
        let previous_visual = self.animated_option();
        let option = Rc::new(option);
        let transition = ChartTransition::update(
            previous_visual,
            option.clone(),
            self.transition_driver.clone(),
        );
        self.option.replace_shared(option);
        self.transition.replace(transition);
        self.zoom_windows.replace(windows.clone());
        self.toolbox_zoom_history.replace(vec![windows]);
        self.zoom_drag.replace(None);
        self.hidden_series.replace(hidden_series);
        self.selected.replace(None);
        self.action_tooltip.replace(None);
        self.highlighted.replace(None);
        self.selected_items
            .replace(initial_selected_items(&self.option.borrow()));
    }

    fn dispatch_action(&self, action: ChartAction) -> Option<ChartRuntimeEvent> {
        let action_name = chart_action_name(&action.kind);
        let event_name = chart_action_event_name(&action.kind);
        let mut source = None;
        let changed = match &action.kind {
            ChartActionKind::Highlight(target) => {
                let event = resolve_action_target(&self.option.borrow(), target);
                let changed = *self.highlighted.borrow() != event;
                self.highlighted.replace(event.clone());
                source = event;
                changed
            }
            ChartActionKind::Downplay(target) => {
                let event = resolve_action_target(&self.option.borrow(), target);
                source = event.clone();
                let should_clear = self.highlighted.borrow().as_ref().is_some_and(|current| {
                    event
                        .as_ref()
                        .is_none_or(|event| same_chart_item(current, event))
                });
                if should_clear {
                    self.highlighted.replace(None);
                }
                should_clear
            }
            ChartActionKind::Select(target)
            | ChartActionKind::Unselect(target)
            | ChartActionKind::ToggleSelect(target) => {
                let event = resolve_action_target(&self.option.borrow(), target);
                source = event.clone();
                event.is_some_and(|event| self.apply_selection_action(&action.kind, &event))
            }
            ChartActionKind::ShowTip(target) => {
                let event = resolve_action_target(&self.option.borrow(), target);
                let changed = *self.action_tooltip.borrow() != event;
                self.action_tooltip.replace(event.clone());
                source = event;
                changed
            }
            ChartActionKind::HideTip => self.action_tooltip.take().is_some(),
            ChartActionKind::LegendSelect { name } => self.set_legend_selected(name, true),
            ChartActionKind::LegendUnselect { name } => self.set_legend_selected(name, false),
            ChartActionKind::LegendToggleSelect { name } => self
                .series_index_by_name(name)
                .is_some_and(|index| self.toggle_legend(index)),
            ChartActionKind::DataZoom {
                data_zoom_index,
                start,
                end,
            } => {
                let mut windows = self.zoom_windows.borrow_mut();
                let window = windows.get_mut(*data_zoom_index)?;
                let next = ZoomWindow::new(*start, *end);
                let changed = *window != next;
                *window = next;
                if changed {
                    self.toolbox_zoom_history.borrow_mut().push(windows.clone());
                }
                changed
            }
            ChartActionKind::TimelineChange { current_index } => {
                let previous_visual = self.animated_option();
                let mut option = self.option.borrow_mut();
                let changed = option.apply_timeline_index(*current_index);
                drop(option);
                if changed {
                    self.transition.replace(ChartTransition::update(
                        previous_visual,
                        self.option.snapshot(),
                        self.transition_driver.clone(),
                    ));
                }
                changed
            }
            ChartActionKind::TimelinePlayChange { play_state } => {
                let mut option = self.option.borrow_mut();
                let timeline = option.timeline.as_mut()?;
                let changed = timeline.auto_play != *play_state;
                timeline.auto_play = *play_state;
                self.timeline_elapsed_ms.set(0);
                changed
            }
            ChartActionKind::Restore => {
                self.restore();
                true
            }
        };
        if !changed || action.silent {
            return None;
        }
        Some(self.runtime_event(event_name, Some(action_name), source))
    }

    fn dispatch_actions(&self, actions: Vec<ChartAction>) -> Option<ChartRuntimeEvent> {
        let batch = actions
            .into_iter()
            .filter_map(|action| self.dispatch_action(action))
            .map(|event| ChartRuntimeEventBatchItem {
                event_type: event.event_type,
                source: event.source,
                from_action: event.from_action,
            })
            .collect::<Vec<_>>();
        let first_type = batch.first()?.event_type.as_str();
        let event_type = if batch.iter().all(|item| item.event_type == first_type) {
            first_type.to_string()
        } else {
            String::from("batch")
        };
        let mut event = self.runtime_event(event_type, Some("batch"), None);
        event.batch = batch;
        Some(event)
    }

    fn apply_selection_action(&self, action: &ChartActionKind, event: &ChartEvent) -> bool {
        let mode = self
            .option
            .borrow()
            .series
            .get(event.series_index)
            .and_then(crate::state::selected_mode)
            .map(ToOwned::to_owned);
        if mode.as_deref().is_none_or(|mode| mode == "false") {
            return false;
        }
        let key = (event.series_index, event.data_index);
        let mut selected = self.selected_items.borrow_mut();
        let before = selected.clone();
        match action {
            ChartActionKind::Select(_) => {
                if matches!(mode.as_deref(), Some("single") | Some("true")) {
                    selected.retain(|(series_index, _)| *series_index != event.series_index);
                }
                selected.insert(key);
            }
            ChartActionKind::Unselect(_) => {
                selected.remove(&key);
            }
            ChartActionKind::ToggleSelect(_) => {
                if selected.contains(&key) {
                    selected.remove(&key);
                } else {
                    if matches!(mode.as_deref(), Some("single") | Some("true")) {
                        selected.retain(|(series_index, _)| *series_index != event.series_index);
                    }
                    selected.insert(key);
                }
            }
            _ => {}
        }
        *selected != before
    }

    fn set_legend_selected(&self, name: &str, selected: bool) -> bool {
        let Some(index) = self.series_index_by_name(name) else {
            return false;
        };
        let mut hidden = self.hidden_series.borrow_mut();
        let before = hidden.clone();
        if selected {
            if self
                .option
                .borrow()
                .legend
                .as_ref()
                .is_some_and(|legend| legend.selected_mode == "single")
            {
                let count = self.option.borrow().series.len();
                hidden.extend((0..count).filter(|candidate| *candidate != index));
            }
            hidden.remove(&index);
        } else {
            hidden.insert(index);
        }
        *hidden != before
    }

    fn series_index_by_name(&self, name: &str) -> Option<usize> {
        self.option
            .borrow()
            .series
            .iter()
            .position(|series| series.name() == Some(name))
    }

    fn runtime_event(
        &self,
        event_type: impl Into<String>,
        from_action: Option<impl Into<String>>,
        source: Option<ChartEvent>,
    ) -> ChartRuntimeEvent {
        let selected = self
            .selected_items
            .borrow()
            .iter()
            .fold(
                std::collections::BTreeMap::<usize, Vec<usize>>::new(),
                |mut grouped, (series_index, data_index)| {
                    grouped.entry(*series_index).or_default().push(*data_index);
                    grouped
                },
            )
            .into_iter()
            .map(|(series_index, data_indices)| ChartSelectedItems {
                series_index,
                data_indices,
            })
            .collect();
        let hidden = self.hidden_series.borrow();
        let legend_selected = self
            .option
            .borrow()
            .series
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                Some((series.name()?.to_string(), !hidden.contains(&index)))
            })
            .collect();
        let event_type = event_type.into();
        ChartRuntimeEvent {
            from_action: from_action.map(Into::into),
            event_type,
            source,
            selected,
            legend_selected,
            batch: Vec::new(),
        }
    }

    fn activate_timeline(&self, hit: &ChartEvent) -> bool {
        let previous_visual = self.animated_option();
        let mut option = self.option.borrow_mut();
        let count = option.timeline_options.len();
        let Some(timeline) = option.timeline.as_ref() else {
            return false;
        };
        let current = timeline.current_index.min(count.saturating_sub(1));
        let loop_play = timeline.loop_play;
        let next = match hit.name.as_deref() {
            Some("timeline-prev") => current
                .checked_sub(1)
                .or_else(|| loop_play.then_some(count.saturating_sub(1))),
            Some("timeline-next") => (current + 1 < count)
                .then_some(current + 1)
                .or_else(|| loop_play.then_some(0)),
            Some("timeline-play") => {
                if let Some(timeline) = option.timeline.as_mut() {
                    timeline.auto_play = !timeline.auto_play;
                }
                self.timeline_elapsed_ms.set(0);
                return true;
            }
            _ => (hit.data_index < count).then_some(hit.data_index),
        };
        let changed = next.is_some_and(|index| option.apply_timeline_index(index));
        drop(option);
        if changed {
            self.transition.replace(ChartTransition::update(
                previous_visual,
                self.option.snapshot(),
                self.transition_driver.clone(),
            ));
        }
        changed
    }

    fn needs_animation_clock(&self) -> bool {
        let option = self.option.borrow();
        option
            .timeline
            .as_ref()
            .is_some_and(|timeline| timeline.auto_play && option.timeline_options.len() > 1)
            || option.series.iter().any(|series| match series {
                crate::model::Series::EffectScatter(_) => true,
                crate::model::Series::Lines(series) => series
                    .options
                    .extra
                    .get("effect")
                    .and_then(serde_json::Value::as_object)
                    .and_then(|effect| effect.get("show"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                _ => false,
            })
    }

    fn advance_animation_clock(&self, elapsed_ms: u64) -> bool {
        const EFFECT_CLOCK_WRAP_MS: u64 = 86_400_000;
        self.effect_elapsed_ms
            .set(self.effect_elapsed_ms.get().saturating_add(elapsed_ms) % EFFECT_CLOCK_WRAP_MS);
        let interval = {
            let option = self.option.borrow();
            option
                .timeline
                .as_ref()
                .filter(|timeline| timeline.auto_play && option.timeline_options.len() > 1)
                .map(|timeline| timeline.play_interval)
        };
        let Some(interval) = interval else {
            self.timeline_elapsed_ms.set(0);
            return false;
        };
        let accumulated = self.timeline_elapsed_ms.get().saturating_add(elapsed_ms);
        if accumulated < interval {
            self.timeline_elapsed_ms.set(accumulated);
            return false;
        }
        self.timeline_elapsed_ms.set(accumulated % interval.max(1));
        let previous_visual = self.animated_option();
        let advanced = self.option.borrow_mut().advance_timeline();
        if advanced {
            self.selected.replace(None);
            self.transition.replace(ChartTransition::update(
                previous_visual,
                self.option.snapshot(),
                self.transition_driver.clone(),
            ));
        }
        advanced
    }

    fn animation_time_seconds(&self) -> f64 {
        self.effect_elapsed_ms.get() as f64 / 1_000.0
    }

    fn toggle_legend(&self, series_index: usize) -> bool {
        let option = self.option.borrow();
        let Some(legend) = option.legend.as_ref() else {
            return false;
        };
        if legend.selected_mode == "false" {
            return false;
        }
        let controlled = option
            .series
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                let name = series.name()?;
                (legend.data.is_empty() || legend.data.iter().any(|entry| entry == name))
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        if !controlled.contains(&series_index) {
            return false;
        }
        drop(option);
        let mut hidden = self.hidden_series.borrow_mut();
        let before = hidden.clone();
        if self
            .option
            .borrow()
            .legend
            .as_ref()
            .is_some_and(|legend| legend.selected_mode == "single")
        {
            for index in controlled {
                if index == series_index {
                    hidden.remove(&index);
                } else {
                    hidden.insert(index);
                }
            }
        } else if !hidden.insert(series_index) {
            hidden.remove(&series_index);
        }
        *hidden != before
    }

    fn begin_map_drag(&self, hit: &ChartEvent, x: f32, y: f32) -> bool {
        if hit.component_type != "map" {
            return false;
        }
        let option = self.option.borrow();
        let Some(crate::model::Series::Map(series)) = option.series.get(hit.series_index) else {
            return false;
        };
        if !matches!(series.map_options.roam.as_str(), "true" | "move") {
            return false;
        }
        self.map_drag.replace(Some(MapDrag {
            series_index: hit.series_index,
            pointer_start: (x, y),
            pan_start: series.map_options.pan_offset,
            pointer_last: (x, y),
        }));
        true
    }

    fn cached_label_hit(&self, x: f32, y: f32) -> Option<ChartEvent> {
        self.cached_hit_matching(x, y, |event| event.component_type == "label")
    }

    fn cached_hit(&self, x: f32, y: f32) -> Option<ChartEvent> {
        self.cached_hit_matching(x, y, |_| true)
    }

    fn has_cached_hits(&self) -> bool {
        !self.draw_hits.borrow().is_empty()
    }

    fn cached_hit_matching(
        &self,
        x: f32,
        y: f32,
        predicate: impl Fn(&ChartEvent) -> bool,
    ) -> Option<ChartEvent> {
        self.draw_hits
            .borrow()
            .iter()
            .rev()
            .filter(|hit| predicate(&hit.event))
            .filter_map(|hit| hit.hit(x, y).map(|distance| (distance, hit.event.clone())))
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, event)| event)
    }

    fn cached_nearest_axis_event(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<ChartEvent> {
        let option = self.option.borrow();
        nearest_axis_event_from_hits(&option, &self.draw_hits.borrow(), x, y, width, height)
    }

    fn begin_label_drag(&self, hit: &ChartEvent, x: f32, y: f32) -> bool {
        if hit.component_type != "label" {
            return false;
        }
        let option = self.option.borrow();
        let Some(layout) = option
            .series
            .get(hit.series_index)
            .and_then(series_label_layout)
        else {
            return false;
        };
        let offset_start = layout
            .drag_offsets
            .get(&hit.data_index)
            .copied()
            .unwrap_or([0.0, 0.0]);
        self.label_drag.replace(Some(LabelDrag {
            series_index: hit.series_index,
            label_index: hit.data_index,
            pointer_start: (x, y),
            offset_start,
            pointer_last: (x, y),
        }));
        true
    }

    fn update_label_drag(&self, mut drag: LabelDrag, x: f32, y: f32) {
        if let Some(layout) = self
            .option
            .borrow_mut()
            .series
            .get_mut(drag.series_index)
            .and_then(series_label_layout_mut)
        {
            layout.drag_offsets.insert(
                drag.label_index,
                [
                    drag.offset_start[0] + x - drag.pointer_start.0,
                    drag.offset_start[1] + y - drag.pointer_start.1,
                ],
            );
        }
        drag.pointer_last = (x, y);
        self.label_drag.replace(Some(drag));
    }

    fn finish_label_drag(&self, drag: LabelDrag, x: f32, y: f32) -> ChartEvent {
        self.update_label_drag(drag, x, y);
        self.label_drag.replace(None);
        let offset = self
            .option
            .borrow()
            .series
            .get(drag.series_index)
            .and_then(series_label_layout)
            .and_then(|layout| layout.drag_offsets.get(&drag.label_index))
            .copied()
            .unwrap_or([0.0, 0.0]);
        ChartEvent {
            series_index: drag.series_index,
            data_index: drag.label_index,
            series_name: self
                .option
                .borrow()
                .series
                .get(drag.series_index)
                .and_then(Series::name)
                .map(ToOwned::to_owned),
            name: Some(String::from("label-drag")),
            value: vec![f64::from(offset[0]), f64::from(offset[1])],
            x,
            y,
            component_type: String::from("label"),
        }
    }

    fn update_map_drag(&self, mut drag: MapDrag, x: f32, y: f32) {
        let mut option = self.option.borrow_mut();
        if let Some(crate::model::Series::Map(series)) = option.series.get_mut(drag.series_index) {
            series.map_options.pan_offset = [
                drag.pan_start[0] + x - drag.pointer_start.0,
                drag.pan_start[1] + y - drag.pointer_start.1,
            ];
        }
        drag.pointer_last = (x, y);
        self.map_drag.replace(Some(drag));
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

fn initial_selected_items(option: &ChartOption) -> BTreeSet<(usize, usize)> {
    option
        .series
        .iter()
        .enumerate()
        .flat_map(|(series_index, series)| match series {
            crate::model::Series::Map(series) => series
                .features
                .iter()
                .enumerate()
                .filter(|(_, feature)| feature.selected)
                .map(move |(data_index, _)| (series_index, data_index))
                .collect::<Vec<_>>(),
            crate::model::Series::Line(series)
            | crate::model::Series::Bar(series)
            | crate::model::Series::Pie(series)
            | crate::model::Series::Scatter(series)
            | crate::model::Series::EffectScatter(series)
            | crate::model::Series::Radar(series)
            | crate::model::Series::Gauge(series)
            | crate::model::Series::Funnel(series)
            | crate::model::Series::Heatmap(series)
            | crate::model::Series::Candlestick(series)
            | crate::model::Series::Boxplot(series)
            | crate::model::Series::PictorialBar(series)
            | crate::model::Series::Parallel(series)
            | crate::model::Series::ThemeRiver(series)
            | crate::model::Series::Treemap(series) => series
                .data
                .iter()
                .enumerate()
                .filter(|(_, point)| {
                    point
                        .extra
                        .get("selected")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                })
                .map(move |(data_index, _)| (series_index, data_index))
                .collect::<Vec<_>>(),
            crate::model::Series::Tree(series) | crate::model::Series::Graph(series) => series
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| {
                    node.extra
                        .get("selected")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                })
                .map(move |(data_index, _)| (series_index, data_index))
                .collect::<Vec<_>>(),
            crate::model::Series::Sankey(series) => series
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| {
                    node.extra
                        .get("selected")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                })
                .map(move |(data_index, _)| (series_index, data_index))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

fn apply_selected_snapshot(option: &mut ChartOption, selected: &BTreeSet<(usize, usize)>) {
    for (series_index, series) in option.series.iter_mut().enumerate() {
        match series {
            Series::Map(series) => {
                for (data_index, feature) in series.features.iter_mut().enumerate() {
                    feature.selected = selected.contains(&(series_index, data_index));
                }
            }
            Series::Line(series)
            | Series::Bar(series)
            | Series::Pie(series)
            | Series::Scatter(series)
            | Series::EffectScatter(series)
            | Series::Radar(series)
            | Series::Gauge(series)
            | Series::Funnel(series)
            | Series::Heatmap(series)
            | Series::Candlestick(series)
            | Series::Boxplot(series)
            | Series::PictorialBar(series)
            | Series::Parallel(series)
            | Series::ThemeRiver(series)
            | Series::Treemap(series) => {
                for (data_index, point) in series.data.iter_mut().enumerate() {
                    point.extra.insert(
                        String::from("selected"),
                        serde_json::Value::Bool(selected.contains(&(series_index, data_index))),
                    );
                }
            }
            Series::Tree(series) | Series::Graph(series) => {
                for (data_index, node) in series.nodes.iter_mut().enumerate() {
                    node.extra.insert(
                        String::from("selected"),
                        serde_json::Value::Bool(selected.contains(&(series_index, data_index))),
                    );
                }
            }
            Series::Sankey(series) => {
                for (data_index, node) in series.nodes.iter_mut().enumerate() {
                    node.extra.insert(
                        String::from("selected"),
                        serde_json::Value::Bool(selected.contains(&(series_index, data_index))),
                    );
                }
            }
            _ => {}
        }
    }
}

fn append_data_to_option(option: &mut ChartOption, chunk: &ChartAppendData) -> bool {
    match chunk {
        ChartAppendData::Scatter { series_index, data } => {
            let Some(Series::Scatter(series)) = option.series.get_mut(*series_index) else {
                return false;
            };
            if series.options.extra.contains_key("datasetIndex") {
                return false;
            }
            series.data.extend(data.iter().cloned());
            true
        }
        ChartAppendData::Lines { series_index, data } => {
            let Some(Series::Lines(series)) = option.series.get_mut(*series_index) else {
                return false;
            };
            series.data.extend(data.iter().cloned());
            true
        }
    }
}

fn initial_hidden_series(option: &ChartOption) -> BTreeSet<usize> {
    let Some(legend) = option.legend.as_ref() else {
        return BTreeSet::new();
    };
    let entries = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            let name = series.name()?;
            (legend.data.is_empty() || legend.data.iter().any(|entry| entry == name))
                .then_some((index, name))
        })
        .collect::<Vec<_>>();
    if legend.selected_mode == "single" {
        let selected = entries
            .iter()
            .find(|(_, name)| legend.selected.get(*name).copied().unwrap_or(true))
            .map(|(index, _)| *index)
            .or_else(|| entries.first().map(|(index, _)| *index));
        return entries
            .into_iter()
            .filter_map(|(index, _)| (Some(index) != selected).then_some(index))
            .collect();
    }
    entries
        .into_iter()
        .filter_map(|(index, name)| {
            legend
                .selected
                .get(name)
                .is_some_and(|selected| !selected)
                .then_some(index)
        })
        .collect()
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
    let lifecycle_active = use_app_foreground() && use_component_visibility();
    let transition_progress = arkit_animation::use_animatable(0.0_f32);
    let state_progress = arkit_animation::use_animatable(0.0_f32);
    let clock_pulse = arkit_animation::use_animatable(0.0_f32);
    let transition_driver = ChartTransitionDriver::new(transition_progress.clone());
    let state_transition_driver = ChartTransitionDriver::new(state_progress.clone());
    let animation_clock = ChartAnimationClock::new(clock_pulse);
    let initial_option = props.option.clone();
    let state = use_hook(move || {
        ChartRenderState::with_drivers(initial_option, transition_driver, state_transition_driver)
    });
    state.update_option(&props.option);

    let node_ref = use_ark_node();
    let invalidate_node = node_ref;
    animation_clock.set_invalidator(move || {
        if let Some(node) = invalidate_node.peek() {
            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
        }
    });
    let transition_node = node_ref;
    transition_progress.set_invalidator(move || {
        if let Some(node) = transition_node.peek() {
            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
        }
    });
    let state_node = node_ref;
    state_progress.set_invalidator(move || {
        if let Some(node) = state_node.peek() {
            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
        }
    });
    let clock_state = state.clone();
    let tick_clock = animation_clock.clone();
    animation_clock.on_tick(move || {
        clock_state.advance_animation_clock(33);
        if !clock_state.needs_animation_clock() {
            tick_clock.stop();
        }
    });
    let registered_node = use_hook(|| Rc::new(Cell::new(None::<usize>)));
    let select_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<ChartEvent>>)));
    let event_handler = use_hook(|| Rc::new(Cell::new(None::<EventHandler<ChartRuntimeEvent>>)));
    select_handler.set(props.on_select);
    event_handler.set(props.on_event);
    let controller_binding = use_hook(|| Rc::new(RefCell::new(None::<(ChartController, u64)>)));
    let controller_changed = {
        let binding = controller_binding.borrow();
        match (binding.as_ref(), props.controller.as_ref()) {
            (Some((current, _)), Some(next)) => current != next,
            (None, None) => false,
            _ => true,
        }
    };
    if controller_changed {
        if let Some((controller, binding)) = controller_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
        if let Some(controller) = props.controller.clone() {
            let command_state = state.clone();
            let command_clock = animation_clock.clone();
            let command_node = node_ref;
            let command_events = event_handler.clone();
            let option_state = state.clone();
            let size_state = state.clone();
            let binding = controller.bind(
                Rc::new(move |command| {
                    let event = match command {
                        ChartCommand::Action(action) => command_state.dispatch_action(action),
                        ChartCommand::Actions(actions) => command_state.dispatch_actions(actions),
                        ChartCommand::AppendData(data) => {
                            command_state.append_data(&data);
                            None
                        }
                        ChartCommand::Clear => {
                            command_state.clear();
                            None
                        }
                    };
                    if let Some(node) = command_node.peek() {
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                    }
                    if command_state.needs_animation_clock() {
                        command_clock.start();
                    } else {
                        command_clock.stop();
                    }
                    if let (Some(event), Some(handler)) = (event, command_events.get()) {
                        handler.call(event);
                    }
                }),
                Rc::new(move || option_state.runtime_option()),
                Rc::new(move || {
                    let (width, height) = size_state.media_size.get();
                    [width, height]
                }),
            );
            controller_binding
                .borrow_mut()
                .replace((controller, binding));
        }
    }
    let drop_binding = controller_binding.clone();
    use_drop(move || {
        if let Some((controller, binding)) = drop_binding.borrow_mut().take() {
            controller.unbind(binding);
        }
    });
    let draw_state = state.clone();
    let registered_for_effect = registered_node.clone();
    let draw_clock = animation_clock.clone();
    let draw_transition_progress = transition_progress.clone();
    let draw_state_progress = state_progress.clone();

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
                // SAFETY: ArkUI owns `raw_canvas` for exactly this synchronous
                // custom-draw callback. `Canvas` is borrowed (never destroyed)
                // and does not escape the callback.
                let canvas = unsafe { Canvas::from_raw_borrowed(raw_canvas.as_ptr().cast()) };
                let size = draw_context.size();
                let pixel_ratio = pixel_ratio();
                let logical_size = (
                    size.width as f32 / pixel_ratio,
                    size.height as f32 / pixel_ratio,
                );
                draw_state.apply_media(logical_size.0, logical_size.1);
                let option = draw_state.rendered_option();
                let domain_option = draw_state.option.borrow();
                let selected = draw_state.selected.borrow();
                let action_tooltip = draw_state.action_tooltip.borrow();
                let tooltip = action_tooltip.as_ref().or(selected.as_ref());
                canvas.save();
                canvas.scale(pixel_ratio, pixel_ratio);
                let hits = crate::animation::with_animation_time(
                    draw_state.animation_time_seconds(),
                    || {
                        draw_option_with_domain(
                            &option,
                            &domain_option,
                            tooltip,
                            &draw_state.hidden_series.borrow(),
                            &draw_state.zoom_windows.borrow(),
                            &draw_state.selected_items.borrow(),
                            Some(&canvas),
                            logical_size.0,
                            logical_size.1,
                        )
                    },
                );
                draw_state.draw_hits.replace(hits);
                if let Some(drag) = *draw_state.toolbox_zoom_drag.borrow() {
                    draw_toolbox_zoom_selection(
                        &canvas,
                        BrushArea {
                            start: [drag.pointer_start.0, drag.pointer_start.1],
                            end: [drag.pointer_last.0, drag.pointer_last.1],
                        },
                    );
                }
                canvas.restore();
            });
            registered_for_effect.set(Some(native_key));
        }
        // The initial commands may have been queued before the native node was
        // mounted. Resume once to wake the root FrameDriver with a valid node.
        if lifecycle_active {
            draw_transition_progress.controls().resume();
            draw_state_progress.controls().resume();
        } else {
            draw_transition_progress.controls().pause();
            draw_state_progress.controls().pause();
        }
        if lifecycle_active && draw_state.needs_animation_clock() {
            draw_clock.start();
            if draw_clock.is_running() {
                draw_clock.poke();
            }
        } else {
            draw_clock.stop();
        }
        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
    });

    let lifecycle_clock = animation_clock.clone();
    let lifecycle_state = state.clone();
    let lifecycle_transition_progress = transition_progress.clone();
    let lifecycle_state_progress = state_progress.clone();
    use_effect(use_reactive(&lifecycle_active, move |active| {
        if active {
            lifecycle_transition_progress.controls().resume();
            lifecycle_state_progress.controls().resume();
            if lifecycle_state.needs_animation_clock() {
                lifecycle_clock.start();
                lifecycle_clock.poke();
            }
        } else {
            lifecycle_transition_progress.controls().pause();
            lifecycle_state_progress.controls().pause();
            lifecycle_clock.stop();
        }
    }));

    let fixed_height = if props.height.is_none() && props.percent_height.is_none() {
        Some(320.0)
    } else {
        props.height
    };
    let click_state = state.clone();
    let click_clock = animation_clock.clone();
    let click_handler = select_handler.clone();
    let click_events = event_handler.clone();
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
                click_state.apply_media(logical_size.0, logical_size.1);
                let target_is_logical = pointer.target_width > 0.0
                    && (pointer.target_width - logical_size.0).abs()
                        <= (pointer.target_width - size.0).abs();
                let (x, y) = if target_is_logical || pointer.target_width <= 0.0 {
                    (pointer.x, pointer.y)
                } else {
                    (pointer.x / ratio, pointer.y / ratio)
                };
                let action = pointer.action;
                if matches!(
                    action,
                    dioxus_elements::event::PointerAction::Down
                        | dioxus_elements::event::PointerAction::Move
                ) {
                    click_state.action_tooltip.replace(None);
                }
                if action == dioxus_elements::event::PointerAction::Cancel {
                    click_state.zoom_drag.replace(None);
                    click_state.map_drag.replace(None);
                    click_state.label_drag.replace(None);
                    click_state.brush_drag.replace(None);
                    click_state.toolbox_zoom_drag.replace(None);
                    return;
                }
                if action == dioxus_elements::event::PointerAction::Move {
                    let toolbox_zoom_drag = { *click_state.toolbox_zoom_drag.borrow() };
                    if let Some(drag) = toolbox_zoom_drag {
                        click_state.update_toolbox_zoom_drag(drag, x, y);
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        return;
                    }
                    let brush_drag = { *click_state.brush_drag.borrow() };
                    if let Some(drag) = brush_drag {
                        click_state.update_brush_drag(drag, x, y);
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        return;
                    }
                    let map_drag = { *click_state.map_drag.borrow() };
                    if let Some(drag) = map_drag {
                        click_state.update_map_drag(drag, x, y);
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        return;
                    }
                    let label_drag = { *click_state.label_drag.borrow() };
                    if let Some(drag) = label_drag {
                        click_state.update_label_drag(drag, x, y);
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        return;
                    }
                    let zoom_drag = { *click_state.zoom_drag.borrow() };
                    let Some(drag) = zoom_drag else {
                        if click_state.option.borrow().tooltip.trigger == "axis" {
                            let previous = click_state.selected.borrow().clone();
                            let selected = click_state.cached_nearest_axis_event(
                                x,
                                y,
                                logical_size.0,
                                logical_size.1,
                            ).or_else(|| {
                                (!click_state.has_cached_hits()).then(|| {
                                    nearest_axis_event(
                                        &click_state.option.borrow(),
                                        x,
                                        y,
                                        logical_size.0,
                                        logical_size.1,
                                        &click_state.hidden_series.borrow(),
                                        &click_state.zoom_windows.borrow(),
                                    )
                                }).flatten()
                            });
                            click_state.selected.replace(selected.clone());
                            emit_pointer_transition(
                                &click_state,
                                &click_events,
                                previous,
                                selected,
                            );
                            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        } else {
                            let previous = click_state.selected.borrow().clone();
                            let hovered = click_state
                                .cached_hit(x, y)
                                .or_else(|| {
                                    (!click_state.has_cached_hits())
                                        .then(|| {
                                            hit_test_with_hidden(
                                                &click_state.option.borrow(),
                                                x,
                                                y,
                                                logical_size.0,
                                                logical_size.1,
                                                &click_state.hidden_series.borrow(),
                                                &click_state.zoom_windows.borrow(),
                                            )
                                        })
                                        .flatten()
                                })
                            .filter(|event| {
                                !matches!(
                                    event.component_type.as_str(),
                                    "legend" | "toolbox" | "timeline" | "dataZoom"
                                )
                            });
                            click_state.selected.replace(hovered.clone());
                            emit_pointer_transition(
                                &click_state,
                                &click_events,
                                previous,
                                hovered,
                            );
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
                let hit = click_state
                    .cached_label_hit(x, y)
                    .or_else(|| click_state.cached_hit(x, y))
                    .or_else(|| {
                        (!click_state.has_cached_hits())
                            .then(|| {
                                hit_test_with_hidden(
                                    &click_state.option.borrow(),
                                    x,
                                    y,
                                    logical_size.0,
                                    logical_size.1,
                                    &click_state.hidden_series.borrow(),
                                    &click_state.zoom_windows.borrow(),
                                )
                            })
                            .flatten()
                    });
                if action == dioxus_elements::event::PointerAction::Down {
                    if let (Some(hit), Some(handler)) = (hit.as_ref(), click_events.get()) {
                        handler.call(click_state.runtime_event(
                            "mousedown",
                            None::<String>,
                            Some(hit.clone()),
                        ));
                    }
                    let began_label = hit
                        .as_ref()
                        .is_some_and(|hit| click_state.begin_label_drag(hit, x, y));
                    let began_slider = !began_label && hit
                        .as_ref()
                        .is_some_and(|hit| click_state.begin_zoom_drag(hit, x, y));
                    if !began_label && !began_slider {
                        let began_map = hit
                            .as_ref()
                            .is_some_and(|hit| click_state.begin_map_drag(hit, x, y));
                        if !began_map {
                            let chrome_hit = hit.as_ref().is_some_and(|hit| {
                                matches!(
                                    hit.component_type.as_str(),
                                    "toolbox" | "legend" | "timeline" | "dataZoom"
                                )
                            });
                            let began_toolbox_zoom = !chrome_hit
                                && click_state.begin_toolbox_zoom(
                                    x,
                                    y,
                                    logical_size.0,
                                    logical_size.1,
                                );
                            if began_toolbox_zoom {
                                return;
                            }
                            let began_brush = !chrome_hit && click_state.begin_brush(x, y);
                            if !began_brush {
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
                        }
                    }
                    return;
                }
                if action != dioxus_elements::event::PointerAction::Up {
                    return;
                }
                if let (Some(hit), Some(handler)) = (hit.as_ref(), click_events.get()) {
                    handler.call(click_state.runtime_event(
                        "mouseup",
                        None::<String>,
                        Some(hit.clone()),
                    ));
                }
                if let Some(drag) = click_state.label_drag.take() {
                    let event = click_state.finish_label_drag(drag, x, y);
                    let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                    if let Some(handler) = click_handler.get() {
                        handler.call(event.clone());
                    }
                    if let Some(handler) = click_events.get() {
                        handler.call(click_state.runtime_event(
                            "labeldragend",
                            None::<String>,
                            Some(event.clone()),
                        ));
                    }
                    return;
                }
                if let Some(drag) = click_state.toolbox_zoom_drag.take() {
                    let event = click_state.finish_toolbox_zoom(
                        drag,
                        x,
                        y,
                        logical_size.0,
                        logical_size.1,
                    );
                    click_state.selected.replace(Some(event.clone()));
                    let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                    if let Some(handler) = click_handler.get() {
                        handler.call(event);
                    }
                    return;
                }
                if let Some(drag) = click_state.brush_drag.take() {
                    let event = click_state.finish_brush(drag, x, y);
                    click_state.selected.replace(Some(event.clone()));
                    let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                    if let Some(handler) = click_handler.get() {
                        handler.call(event);
                    }
                    return;
                }
                if let Some(drag) = click_state.map_drag.take() {
                    let distance = ((drag.pointer_last.0 - drag.pointer_start.0).powi(2)
                        + (drag.pointer_last.1 - drag.pointer_start.1).powi(2))
                    .sqrt();
                    if distance >= 3.0 {
                        let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                        return;
                    }
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
                let mut selection_changed = false;
                if let Some(hit) = hit.as_ref().filter(|hit| hit.component_type == "toolbox") {
                    match hit.name.as_deref() {
                        Some("restore") => {
                            click_state.restore();
                        }
                        Some("brush-rect") => {
                            click_state.toolbox_zoom_active.set(false);
                            click_state.toolbox_zoom_drag.replace(None);
                            set_toolbox_runtime_status(
                                &mut click_state.option.borrow_mut(),
                                "dataZoom",
                                "__active",
                                false,
                            );
                            if let Some(brush) = click_state.option.borrow_mut().brush.as_mut() {
                                brush.active = !brush.active;
                                brush.brush_type = String::from("rect");
                            }
                        }
                        Some("brush-clear") => {
                            if let Some(brush) = click_state.option.borrow_mut().brush.as_mut() {
                                brush.areas.clear();
                                brush.active = false;
                            }
                            click_state.selected.replace(None);
                        }
                        Some("data-zoom") => click_state.toggle_toolbox_zoom(),
                        Some("data-zoom-back") => click_state.toolbox_zoom_back(),
                        Some("magic-line") => {
                            click_state.activate_magic_type(MagicOverride::Line)
                        }
                        Some("magic-bar") => {
                            click_state.activate_magic_type(MagicOverride::Bar)
                        }
                        Some("magic-stack") => {
                            click_state.toggle_magic_stack()
                        }
                        Some("data-view") => {
                            click_state.data_view_visible.set(true);
                            set_toolbox_runtime_status(
                                &mut click_state.option.borrow_mut(),
                                "dataView",
                                "__visible",
                                true,
                            );
                        }
                        Some("data-view-close") => {
                            click_state.data_view_visible.set(false);
                            set_toolbox_runtime_status(
                                &mut click_state.option.borrow_mut(),
                                "dataView",
                                "__visible",
                                false,
                            );
                        }
                        Some("save-as-image") => {
                            let option = click_state.option.borrow();
                            let selected = click_state.selected.borrow();
                            let hidden_series = click_state.hidden_series.borrow();
                            let zoom_windows = click_state.zoom_windows.borrow();
                            let selected_items = click_state.selected_items.borrow();
                            let result = save_chart_image(ExportContext {
                                option: &option,
                                selected: selected.as_ref(),
                                hidden_series: &hidden_series,
                                zoom_windows: &zoom_windows,
                                selected_items: &selected_items,
                                width: logical_size.0,
                                height: logical_size.1,
                                device_pixel_ratio: ratio,
                            });
                            drop(selected_items);
                            drop(zoom_windows);
                            drop(hidden_series);
                            drop(selected);
                            drop(option);
                            let mut event = hit.clone();
                            event.name = Some(match result {
                                Ok(path) => format!("save-as-image:{}", path.display()),
                                Err(error) => format!("save-as-image-error:{error}"),
                            });
                            click_state.selected.replace(Some(event.clone()));
                            let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                            if let Some(handler) = click_handler.get() {
                                handler.call(event);
                            }
                            return;
                        }
                        _ => {}
                    }
                } else if let Some(hit) = hit.as_ref().filter(|hit| hit.component_type == "timeline") {
                    if click_state.activate_timeline(hit) {
                        click_state.selected.replace(Some(hit.clone()));
                        if click_state.needs_animation_clock() {
                            click_clock.start();
                        } else {
                            click_clock.stop();
                        }
                    }
                } else if let Some(hit) = hit.as_ref().filter(|hit| hit.component_type == "legend") {
                    if click_state.toggle_legend(hit.series_index) {
                        click_state.selected.replace(None);
                    }
                } else {
                    if let Some(hit) = hit.as_ref().filter(|hit| {
                        click_state
                            .option
                            .borrow()
                            .series
                            .get(hit.series_index)
                            .and_then(crate::state::selected_mode)
                            .is_some()
                    }) {
                        let selected_mode = click_state
                            .option
                            .borrow()
                            .series
                            .get(hit.series_index)
                            .and_then(crate::state::selected_mode)
                            .map(ToOwned::to_owned);
                        let mut selected_items = click_state.selected_items.borrow_mut();
                        let before = selected_items.clone();
                        match selected_mode.as_deref() {
                            Some("multiple") => {
                                let key = (hit.series_index, hit.data_index);
                                if !selected_items.insert(key) {
                                    selected_items.remove(&key);
                                }
                            }
                            Some("single") | Some("true") => {
                                selected_items.retain(|(series_index, _)| {
                                    *series_index != hit.series_index
                                });
                                selected_items.insert((hit.series_index, hit.data_index));
                            }
                            _ => {}
                        }
                        selection_changed = *selected_items != before;
                    }
                    click_state.selected.replace(hit.clone());
                }
                let _ = node.borrow().mark_dirty(NodeDirtyFlag::NeedRender);
                if let (Some(hit), Some(handler)) = (hit.as_ref(), click_events.get()) {
                    let event_type = match hit.component_type.as_str() {
                        "legend" => "legendselectchanged",
                        "dataZoom" => "datazoom",
                        "timeline" => "timelinechanged",
                        _ => "click",
                    };
                    handler.call(click_state.runtime_event(
                        event_type,
                        None::<String>,
                        Some(hit.clone()),
                    ));
                    if selection_changed {
                        handler.call(click_state.runtime_event(
                            "selectchanged",
                            Some("select"),
                            Some(hit.clone()),
                        ));
                    }
                }
                if let (Some(hit), Some(handler)) = (hit, click_handler.get()) {
                    handler.call(hit);
                }
            },
        }
    }
}

fn emit_pointer_transition(
    state: &ChartRenderState,
    handler: &Rc<Cell<Option<EventHandler<ChartRuntimeEvent>>>>,
    previous: Option<ChartEvent>,
    current: Option<ChartEvent>,
) {
    let Some(handler) = handler.get() else {
        return;
    };
    let changed = match (&previous, &current) {
        (Some(previous), Some(current)) => !same_chart_item(previous, current),
        (None, None) => false,
        _ => true,
    };
    if changed {
        if let Some(previous) = previous {
            handler.call(state.runtime_event("mouseout", None::<String>, Some(previous)));
        }
        if let Some(current) = current.clone() {
            handler.call(state.runtime_event("mouseover", None::<String>, Some(current)));
        } else {
            handler.call(state.runtime_event("globalout", None::<String>, None));
        }
    }
    if let Some(current) = current {
        handler.call(state.runtime_event("mousemove", None::<String>, Some(current)));
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

fn chart_action_name(action: &ChartActionKind) -> &'static str {
    match action {
        ChartActionKind::Highlight(_) => "highlight",
        ChartActionKind::Downplay(_) => "downplay",
        ChartActionKind::Select(_) => "select",
        ChartActionKind::Unselect(_) => "unselect",
        ChartActionKind::ToggleSelect(_) => "toggleSelect",
        ChartActionKind::ShowTip(_) => "showTip",
        ChartActionKind::HideTip => "hideTip",
        ChartActionKind::LegendSelect { .. } => "legendSelect",
        ChartActionKind::LegendUnselect { .. } => "legendUnSelect",
        ChartActionKind::LegendToggleSelect { .. } => "legendToggleSelect",
        ChartActionKind::DataZoom { .. } => "dataZoom",
        ChartActionKind::TimelineChange { .. } => "timelineChange",
        ChartActionKind::TimelinePlayChange { .. } => "timelinePlayChange",
        ChartActionKind::Restore => "restore",
    }
}

fn chart_action_event_name(action: &ChartActionKind) -> &'static str {
    match action {
        ChartActionKind::Select(_)
        | ChartActionKind::Unselect(_)
        | ChartActionKind::ToggleSelect(_) => "selectchanged",
        ChartActionKind::LegendSelect { .. }
        | ChartActionKind::LegendUnselect { .. }
        | ChartActionKind::LegendToggleSelect { .. } => "legendselectchanged",
        ChartActionKind::TimelineChange { .. } => "timelinechanged",
        ChartActionKind::TimelinePlayChange { .. } => "timelineplaychanged",
        _ => chart_action_name(action),
    }
}

fn same_chart_item(left: &ChartEvent, right: &ChartEvent) -> bool {
    left.series_index == right.series_index && left.data_index == right.data_index
}

fn resolve_action_target(option: &ChartOption, target: &ChartActionTarget) -> Option<ChartEvent> {
    let series_index = target.series_index.or_else(|| {
        let name = target.series_name.as_deref()?;
        option
            .series
            .iter()
            .position(|series| series.name() == Some(name))
    })?;
    let series = option.series.get(series_index)?;
    let series_name = series.name().map(ToOwned::to_owned);
    let requested_index = target.data_index;
    let requested_name = target.name.as_deref();
    let (data_index, point, component_type) = match series {
        Series::Line(series) => {
            resolve_basic_point(series, requested_index, requested_name, "line")?
        }
        Series::Bar(series) => resolve_basic_point(series, requested_index, requested_name, "bar")?,
        Series::Pie(series) => resolve_basic_point(series, requested_index, requested_name, "pie")?,
        Series::Scatter(series) => {
            resolve_basic_point(series, requested_index, requested_name, "scatter")?
        }
        Series::EffectScatter(series) => {
            resolve_basic_point(series, requested_index, requested_name, "effectScatter")?
        }
        Series::Radar(series) => {
            resolve_basic_point(series, requested_index, requested_name, "radar")?
        }
        Series::Gauge(series) => {
            resolve_basic_point(series, requested_index, requested_name, "gauge")?
        }
        Series::Funnel(series) => {
            resolve_basic_point(series, requested_index, requested_name, "funnel")?
        }
        Series::Heatmap(series) => {
            resolve_basic_point(series, requested_index, requested_name, "heatmap")?
        }
        Series::Candlestick(series) => {
            resolve_basic_point(series, requested_index, requested_name, "candlestick")?
        }
        Series::Boxplot(series) => {
            resolve_basic_point(series, requested_index, requested_name, "boxplot")?
        }
        Series::PictorialBar(series) => {
            resolve_basic_point(series, requested_index, requested_name, "pictorialBar")?
        }
        Series::Parallel(series) => {
            resolve_basic_point(series, requested_index, requested_name, "parallel")?
        }
        Series::ThemeRiver(series) => {
            resolve_basic_point(series, requested_index, requested_name, "themeRiver")?
        }
        Series::Treemap(series) => {
            resolve_basic_point(series, requested_index, requested_name, "treemap")?
        }
        Series::Tree(series) | Series::Graph(series) => {
            let index = requested_index
                .or_else(|| {
                    requested_name
                        .and_then(|name| series.nodes.iter().position(|node| node.name == name))
                })
                .unwrap_or(0);
            let node = series.nodes.get(index)?;
            (
                index,
                DataPoint::named(node.name.clone(), node.value),
                if matches!(option.series.get(series_index), Some(Series::Tree(_))) {
                    "tree"
                } else {
                    "graph"
                },
            )
        }
        Series::Sankey(series) => {
            let index = requested_index
                .or_else(|| {
                    requested_name
                        .and_then(|name| series.nodes.iter().position(|node| node.name == name))
                })
                .unwrap_or(0);
            let node = series.nodes.get(index)?;
            (
                index,
                DataPoint::named(node.name.clone(), node.value),
                "sankey",
            )
        }
        Series::Map(series) => {
            let index = requested_index
                .or_else(|| {
                    requested_name.and_then(|name| {
                        series
                            .features
                            .iter()
                            .position(|feature| feature.name == name)
                    })
                })
                .unwrap_or(0);
            let feature = series.features.get(index)?;
            (
                index,
                DataPoint::named(feature.name.clone(), feature.value),
                "map",
            )
        }
        Series::Lines(series) => {
            let index = requested_index
                .or_else(|| {
                    requested_name.and_then(|name| {
                        series
                            .data
                            .iter()
                            .position(|line| line.name.as_deref() == Some(name))
                    })
                })
                .unwrap_or(0);
            let line = series.data.get(index)?;
            let mut point = DataPoint::scalar(line.value);
            point.name = line.name.clone();
            (index, point, "lines")
        }
        Series::Sunburst(series) => {
            let mut flattened = Vec::new();
            flatten_sunburst_nodes(&series.data, &mut flattened);
            let index = requested_index
                .or_else(|| {
                    requested_name
                        .and_then(|name| flattened.iter().position(|node| node.name == name))
                })
                .unwrap_or(0);
            let node = flattened.get(index)?;
            (
                index,
                DataPoint::named(node.name.clone(), node.value),
                "sunburst",
            )
        }
        Series::Custom(series) => {
            let index = requested_index
                .or_else(|| {
                    requested_name.and_then(|name| {
                        series
                            .data
                            .iter()
                            .position(|point| point.name.as_deref() == Some(name))
                    })
                })
                .unwrap_or(0);
            (index, series.data.get(index)?.clone(), "custom")
        }
    };
    Some(ChartEvent {
        series_index,
        data_index,
        series_name,
        name: point.name,
        value: point
            .values
            .iter()
            .filter_map(crate::model::DataValue::as_f64)
            .collect(),
        x: 0.0,
        y: 0.0,
        component_type: component_type.to_string(),
    })
}

fn resolve_basic_point(
    series: &crate::model::BasicSeries,
    requested_index: Option<usize>,
    requested_name: Option<&str>,
    component_type: &'static str,
) -> Option<(usize, DataPoint, &'static str)> {
    let index = requested_index
        .or_else(|| {
            requested_name.and_then(|name| {
                series
                    .data
                    .iter()
                    .position(|point| point.name.as_deref() == Some(name))
            })
        })
        .unwrap_or(0);
    Some((index, series.data.get(index)?.clone(), component_type))
}

fn flatten_sunburst_nodes<'a>(
    nodes: &'a [crate::model::SunburstNode],
    output: &mut Vec<&'a crate::model::SunburstNode>,
) {
    for node in nodes {
        output.push(node);
        flatten_sunburst_nodes(&node.children, output);
    }
}

fn series_label_layout(series: &Series) -> Option<&crate::model::LabelLayoutOptions> {
    Some(match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Pie(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Radar(value)
        | Series::Gauge(value)
        | Series::Funnel(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value)
        | Series::Parallel(value)
        | Series::ThemeRiver(value)
        | Series::Treemap(value) => &value.options.label_layout,
        Series::Tree(value) | Series::Graph(value) => &value.options.label_layout,
        Series::Sankey(value) => &value.options.label_layout,
        Series::Map(value) => &value.options.label_layout,
        Series::Lines(value) => &value.options.label_layout,
        Series::Sunburst(value) => &value.options.label_layout,
        Series::Custom(_) => return None,
    })
}

fn series_label_layout_mut(series: &mut Series) -> Option<&mut crate::model::LabelLayoutOptions> {
    Some(match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Pie(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Radar(value)
        | Series::Gauge(value)
        | Series::Funnel(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value)
        | Series::Parallel(value)
        | Series::ThemeRiver(value)
        | Series::Treemap(value) => &mut value.options.label_layout,
        Series::Tree(value) | Series::Graph(value) => &mut value.options.label_layout,
        Series::Sankey(value) => &mut value.options.label_layout,
        Series::Map(value) => &mut value.options.label_layout,
        Series::Lines(value) => &mut value.options.label_layout,
        Series::Sunburst(value) => &mut value.options.label_layout,
        Series::Custom(_) => return None,
    })
}

fn apply_magic_override(option: &mut ChartOption, mode: MagicOverride) {
    match mode {
        MagicOverride::Line | MagicOverride::Bar => {
            for series in &mut option.series {
                let replacement = match (mode, &*series) {
                    (MagicOverride::Line, Series::Bar(value)) => Some(Series::Line(value.clone())),
                    (MagicOverride::Bar, Series::Line(value)) => Some(Series::Bar(value.clone())),
                    _ => None,
                };
                if let Some(replacement) = replacement {
                    *series = replacement;
                }
            }
        }
        MagicOverride::None => {}
    }
}

fn apply_magic_stack_override(option: &mut ChartOption, stacked: Option<bool>) {
    let Some(stacked) = stacked else {
        return;
    };
    for series in &mut option.series {
        let options = match series {
            Series::Line(value) | Series::Bar(value) => &mut value.options,
            _ => continue,
        };
        if stacked {
            options.stack = Some(String::from("__ec_magicType_stack__"));
        } else if options.stack.as_deref() == Some("__ec_magicType_stack__") {
            options.stack = None;
        }
    }
}

fn set_toolbox_runtime_status(
    option: &mut ChartOption,
    feature_name: &str,
    key: &str,
    value: bool,
) {
    let Some(features) = option
        .extra
        .get_mut("toolbox")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|toolbox| toolbox.get_mut("feature"))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    let Some(feature) = features.get_mut(feature_name) else {
        return;
    };
    if matches!(feature, serde_json::Value::Bool(true)) {
        *feature = serde_json::Value::Object(Default::default());
    }
    let Some(feature) = feature.as_object_mut() else {
        return;
    };
    feature.insert(String::from(key), serde_json::Value::Bool(value));
}

fn toolbox_event(name: &str, x: f32, y: f32) -> ChartEvent {
    ChartEvent {
        series_index: 0,
        data_index: 0,
        series_name: None,
        name: Some(String::from(name)),
        value: Vec::new(),
        x,
        y,
        component_type: String::from("toolbox"),
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

    #[test]
    fn pie_data_selected_flag_initializes_native_selection() {
        let mut selected = crate::model::DataPoint::named("Selected", 3.0);
        selected
            .extra
            .insert(String::from("selected"), serde_json::Value::Bool(true));
        let option = ChartOption::new().push_series(Series::pie(
            "pie",
            [selected, crate::model::DataPoint::named("Other", 7.0)],
        ));
        let state = ChartRenderState::new(option);
        assert!(state.selected_items.borrow().contains(&(0, 0)));
        assert!(!state.selected_items.borrow().contains(&(0, 1)));
    }

    #[test]
    fn legend_selected_and_single_mode_drive_native_visibility() {
        let option = ChartOption::from_json_str(
            r#"{
                "legend":{"selectedMode":"single","selected":{"Revenue":false}},
                "series":[
                    {"type":"line","name":"Revenue","data":[1]},
                    {"type":"bar","name":"Orders","data":[2]}
                ]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        assert_eq!(*state.hidden_series.borrow(), BTreeSet::from([0]));
        assert!(state.toggle_legend(0));
        assert_eq!(*state.hidden_series.borrow(), BTreeSet::from([1]));
        assert!(!state.toggle_legend(0));
        assert_eq!(*state.hidden_series.borrow(), BTreeSet::from([1]));
    }

    #[test]
    fn disabled_legend_selection_is_non_interactive() {
        let option = ChartOption::from_json_str(
            r#"{
                "legend":{"selectedMode":false,"selected":{"Orders":false}},
                "series":[{"type":"bar","name":"Orders","data":[2]}]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        assert_eq!(*state.hidden_series.borrow(), BTreeSet::from([0]));
        assert!(!state.toggle_legend(0));
        assert_eq!(*state.hidden_series.borrow(), BTreeSet::from([0]));
    }

    #[test]
    fn timeline_actions_replace_the_visible_snapshot() {
        let option = ChartOption::from_json_str(
            r#"{
                "baseOption":{"timeline":{"data":["A","B"]},"series":[{"type":"bar","data":[1]}]},
                "options":[
                    {"series":[{"type":"bar","data":[3]}]},
                    {"series":[{"type":"bar","data":[9]}]}
                ]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        assert!(state.activate_timeline(&ChartEvent {
            series_index: 0,
            data_index: 1,
            series_name: None,
            name: Some(String::from("B")),
            value: vec![1.0],
            x: 0.0,
            y: 0.0,
            component_type: String::from("timeline"),
        }));
        let option = state.option.borrow();
        let Series::Bar(series) = &option.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(9.0));
        assert_eq!(option.timeline.as_ref().unwrap().current_index, 1);
    }

    #[test]
    fn brush_drag_persists_a_native_selection_area() {
        let option = ChartOption::from_json_str(
            r#"{"brush":{"brushType":"rect"},"series":[{"type":"scatter","data":[[1,2]]}]}"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        assert!(state.begin_brush(10.0, 20.0));
        let drag = state.brush_drag.take().unwrap();
        let event = state.finish_brush(drag, 80.0, 90.0);
        assert_eq!(event.component_type, "brush");
        assert_eq!(event.value, [10.0, 20.0, 80.0, 90.0]);
        assert_eq!(state.option.borrow().brush.as_ref().unwrap().areas.len(), 1);
    }

    #[test]
    fn magic_type_switch_and_stack_survive_option_refresh() {
        let source = ChartOption::from_json_str(
            r#"{
                "toolbox":{"feature":{"magicType":{"type":["line","bar","stack"]}}},
                "xAxis":{"type":"category","data":["A","B"]},
                "series":[{"type":"line","data":[1,2]},{"type":"line","data":[2,3]}]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(source.clone());
        state.activate_magic_type(MagicOverride::Bar);
        state.toggle_magic_stack();
        assert!(state
            .option
            .borrow()
            .series
            .iter()
            .all(|series| matches!(series, Series::Bar(_))));
        state.update_option(&source);
        assert!(state.option.borrow().series.iter().all(|series| {
            matches!(series, Series::Bar(value) if value.options.stack.as_deref() == Some("__ec_magicType_stack__"))
        }));
        state.restore();
        assert!(state
            .option
            .borrow()
            .series
            .iter()
            .all(|series| matches!(series, Series::Line(_))));
    }

    #[test]
    fn toolbox_data_zoom_selection_and_back_update_history() {
        let option = ChartOption::from_json_str(
            r#"{
                "toolbox":{"feature":{"dataZoom":{}}},
                "xAxis":{"type":"value"},"yAxis":{"type":"value"},
                "series":[{"type":"line","data":[1,2,3]}]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        state.toggle_toolbox_zoom();
        assert!(state.begin_toolbox_zoom(80.0, 60.0, 360.0, 240.0));
        let drag = state.toolbox_zoom_drag.take().unwrap();
        state.finish_toolbox_zoom(drag, 250.0, 170.0, 360.0, 240.0);
        assert_eq!(state.toolbox_zoom_history.borrow().len(), 2);
        assert!(state
            .zoom_windows
            .borrow()
            .iter()
            .any(|window| window.span() < 100.0));
        state.toolbox_zoom_back();
        assert_eq!(state.toolbox_zoom_history.borrow().len(), 1);
        assert!(state
            .zoom_windows
            .borrow()
            .iter()
            .all(|window| window.start == 0.0 && window.end == 100.0));
    }

    #[test]
    fn media_rules_reapply_when_canvas_size_changes() {
        let option = ChartOption::from_json_str(
            r#"{
                "baseOption":{
                    "xAxis":{"type":"category","data":["A","B"]},
                    "series":[{"type":"line","data":[1,2]}]
                },
                "media":[
                    {"query":{"maxWidth":400},"option":{"series":[{"type":"bar"}]}},
                    {"option":{"series":[{"type":"line"}]}}
                ]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        state.apply_media(360.0, 240.0);
        assert!(matches!(state.option.borrow().series[0], Series::Bar(_)));
        state.apply_media(720.0, 240.0);
        assert!(matches!(state.option.borrow().series[0], Series::Line(_)));
    }

    #[test]
    fn dispatch_action_unifies_highlight_selection_and_tooltip_state() {
        let option = ChartOption::from_json_str(
            r#"{
                "series":[{
                    "type":"bar","name":"Orders","selectedMode":"multiple",
                    "data":[{"name":"A","value":10},{"name":"B","value":20}]
                }]
            }"#,
        )
        .unwrap();
        let state = ChartRenderState::new(option);
        let target = ChartActionTarget::named("Orders", "B");

        let highlight = state
            .dispatch_action(ChartAction::new(ChartActionKind::Highlight(target.clone())))
            .unwrap();
        assert_eq!(highlight.event_type, "highlight");
        assert_eq!(highlight.source.as_ref().unwrap().data_index, 1);
        assert_eq!(state.highlighted.borrow().as_ref().unwrap().data_index, 1);

        let selected = state
            .dispatch_action(ChartAction::new(ChartActionKind::Select(target.clone())))
            .unwrap();
        assert_eq!(selected.event_type, "selectchanged");
        assert_eq!(selected.from_action.as_deref(), Some("select"));
        assert_eq!(selected.selected[0].data_indices, [1]);

        let shown = state
            .dispatch_action(ChartAction::new(ChartActionKind::ShowTip(target.clone())))
            .unwrap();
        assert_eq!(shown.event_type, "showTip");
        assert_eq!(
            state
                .action_tooltip
                .borrow()
                .as_ref()
                .unwrap()
                .name
                .as_deref(),
            Some("B")
        );
        let mut next = state.prop_option.borrow().clone();
        let Series::Bar(series) = &mut next.series[0] else {
            panic!("bar")
        };
        series.data[1] = DataPoint::named("B", 25.0);
        state.update_option(&next);
        assert_eq!(
            state.action_tooltip.borrow().as_ref().unwrap().value,
            [25.0],
            "showTip must remap to updated data instead of being cleared"
        );
        assert_eq!(state.highlighted.borrow().as_ref().unwrap().value, [25.0]);

        assert!(state
            .dispatch_action(ChartAction::new(ChartActionKind::Unselect(target.clone())).silent())
            .is_none());
        assert!(state.selected_items.borrow().is_empty());
        assert!(state
            .dispatch_action(ChartAction::new(ChartActionKind::Downplay(target)))
            .is_some());
        assert!(state.highlighted.borrow().is_none());

        let batch = state
            .dispatch_actions(vec![
                ChartAction::new(ChartActionKind::Select(ChartActionTarget::item(0, 0))),
                ChartAction::new(ChartActionKind::Select(ChartActionTarget::item(0, 1))),
            ])
            .unwrap();
        assert_eq!(batch.event_type, "selectchanged");
        assert_eq!(batch.from_action.as_deref(), Some("batch"));
        assert_eq!(batch.batch.len(), 2);
        assert_eq!(batch.selected[0].data_indices, [0, 1]);
    }

    #[test]
    fn controller_replays_actions_queued_before_mount() {
        let controller = ChartController::new();
        controller.dispatch_action(ChartAction::new(ChartActionKind::HideTip));
        let received = Rc::new(RefCell::new(Vec::new()));
        let output = received.clone();
        let binding = controller.bind(
            Rc::new(move |command| output.borrow_mut().push(command)),
            Rc::new(|| ChartOption::new().title("mounted")),
            Rc::new(|| [320.0, 240.0]),
        );
        assert!(controller.is_mounted());
        assert_eq!(received.borrow().len(), 1);
        assert!(matches!(
            &received.borrow()[0],
            ChartCommand::Action(ChartAction {
                kind: ChartActionKind::HideTip,
                ..
            })
        ));
        assert_eq!(
            controller.get_option().unwrap().title.unwrap().text,
            "mounted"
        );
        assert_eq!(controller.get_size(), Some([320.0, 240.0]));
        controller.unbind(binding);
        assert!(!controller.is_mounted());
        assert!(controller.get_option().is_none());
    }

    #[test]
    fn append_data_updates_supported_series_and_runtime_snapshot() {
        let mut option = ChartOption::new()
            .legend(crate::model::Legend::default())
            .push_series(Series::scatter("Points", [DataPoint::values([1.0, 2.0])]));
        let Series::Scatter(series) = &mut option.series[0] else {
            panic!("scatter")
        };
        series.options.selected_mode = Some(String::from("multiple"));
        let prop_option = option.clone();
        let state = ChartRenderState::new(option);
        assert!(state.append_data(&ChartAppendData::scatter(
            0,
            [DataPoint::values([3.0, 4.0]), DataPoint::values([5.0, 6.0])],
        )));
        assert_eq!(
            match &state.option.borrow().series[0] {
                Series::Scatter(series) => series.data.len(),
                _ => 0,
            },
            3
        );
        state.update_option(&prop_option);
        assert_eq!(
            match &state.option.borrow().series[0] {
                Series::Scatter(series) => series.data.len(),
                _ => 0,
            },
            3,
            "an unchanged controlled prop must not roll back appendData"
        );
        state.selected_items.borrow_mut().insert((0, 2));
        state.hidden_series.borrow_mut().insert(0);
        let snapshot = state.runtime_option();
        assert_eq!(
            snapshot.legend.unwrap().selected.get("Points"),
            Some(&false)
        );
        let Series::Scatter(series) = &snapshot.series[0] else {
            panic!("scatter")
        };
        assert_eq!(
            series.data[2].extra.get("selected"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn label_drag_offsets_persist_across_matching_option_updates() {
        let mut option = ChartOption::new().push_series(Series::bar("Orders", [10.0, 20.0]));
        let Series::Bar(series) = &mut option.series[0] else {
            panic!("bar")
        };
        series.options.label.show = true;
        series.options.label_layout.draggable = true;
        let state = ChartRenderState::new(option.clone());
        let hit = ChartEvent {
            series_index: 0,
            data_index: 1,
            series_name: Some(String::from("Orders")),
            name: Some(String::from("20")),
            value: vec![20.0],
            x: 30.0,
            y: 40.0,
            component_type: String::from("label"),
        };
        assert!(state.begin_label_drag(&hit, 30.0, 40.0));
        let drag = state.label_drag.take().unwrap();
        let event = state.finish_label_drag(drag, 42.0, 58.0);
        assert_eq!(event.value, [12.0, 18.0]);

        let mut next = option;
        let Series::Bar(series) = &mut next.series[0] else {
            panic!("bar")
        };
        series.data[1] = DataPoint::scalar(24.0);
        state.update_option(&next);
        assert_eq!(
            series_label_layout(&state.option.borrow().series[0])
                .unwrap()
                .drag_offsets
                .get(&1),
            Some(&[12.0, 18.0])
        );
    }
}
