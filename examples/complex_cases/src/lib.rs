use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    List,
    Grid,
    WaterFlow,
    Grouped,
}

#[derive(Debug, Clone)]
enum Message {
    Select(Case),
    VisibleRange(VirtualVisibleRange),
    Refresh,
    LoadMore,
}

#[derive(Debug, Clone)]
struct AppState {
    active: Case,
    total_count: u32,
    visible_range: VirtualVisibleRange,
    refreshing: bool,
    loading_more: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active: Case::List,
            total_count: 10_000,
            visible_range: VirtualVisibleRange::default(),
            refreshing: false,
            loading_more: false,
        }
    }
}

impl AppState {
    fn new() -> Self {
        Self::default()
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Select(case) => {
            if state.active == case {
                return Task::none();
            }
            state.active = case;
            state.visible_range = VirtualVisibleRange::default();
        }
        Message::VisibleRange(range) => {
            if state.visible_range == range {
                return Task::none();
            }
            state.visible_range = range;
        }
        Message::Refresh => {
            state.refreshing = false;
            state.total_count = 10_000;
            state.visible_range = VirtualVisibleRange::default();
        }
        Message::LoadMore => {
            if !state.loading_more {
                state.loading_more = false;
                state.total_count += 500;
            }
        }
    }

    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    Element::new(ComplexCasesView {
        state: state.clone(),
    })
}

struct ComplexCasesView {
    state: AppState,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for ComplexCasesView {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = &self.state;
        Some(
            column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .background_color(0xfff8fafc)
                .children(vec![
                    Element::new(Toolbar {
                        active: state.active,
                    }),
                    Element::new(StatusBar {
                        total_count: state.total_count,
                        visible_range: state.visible_range.clone(),
                    }),
                    Element::new(Content {
                        state: state.clone(),
                    }),
                ])
                .into(),
        )
    }
}

struct Toolbar {
    active: Case,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for Toolbar {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let case_button = |label: &'static str, case: Case| {
            let background = if case == self.active {
                0xff111827
            } else {
                0xffffffff
            };
            let foreground = if case == self.active {
                0xffffffff
            } else {
                0xff111827
            };

            button(label)
                .font_size(14.0)
                .font_color(foreground)
                .background_color(background)
                .border_radius(6.0)
                .padding([8.0, 12.0, 8.0, 12.0])
                .on_press(Message::Select(case))
        };

        Some(
            row_component()
                .percent_width(1.0)
                .padding([16.0, 16.0, 8.0, 16.0])
                .children(vec![
                    case_button("List", Case::List).into(),
                    case_button("Grid", Case::Grid).margin_left(8.0).into(),
                    case_button("WaterFlow", Case::WaterFlow)
                        .margin_left(8.0)
                        .into(),
                    case_button("Grouped", Case::Grouped)
                        .margin_left(8.0)
                        .into(),
                ])
                .into(),
        )
    }
}

struct StatusBar {
    total_count: u32,
    visible_range: VirtualVisibleRange,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for StatusBar {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            row_component()
                .percent_width(1.0)
                .padding([0.0, 16.0, 12.0, 16.0])
                .children(vec![
                    text(format!("total {}", self.total_count))
                        .font_size(13.0)
                        .font_color(0xff475569)
                        .into(),
                    text(format!(
                        "visible {}..{}",
                        self.visible_range.first_index, self.visible_range.last_index
                    ))
                    .margin_left(16.0)
                    .font_size(13.0)
                    .font_color(0xff475569)
                    .into(),
                ])
                .into(),
        )
    }
}

struct Content {
    state: AppState,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for Content {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = &self.state;
        Some(match state.active {
            Case::List => refresh_component()
                .percent_width(1.0)
                .layout_weight(1.0)
                .refreshing(state.refreshing)
                .refresh_offset(72.0)
                .on_refresh(Message::Refresh)
                .child(
                    virtual_list_component(state.total_count, |index| {
                        Element::new(ListRow { index })
                    })
                    .list_cached_count(8)
                    .on_visible_range_change(Message::VisibleRange)
                    .on_load_more(
                        state.total_count,
                        12,
                        state.loading_more,
                        Message::LoadMore,
                    ),
                )
                .into(),
            Case::Grid => {
                virtual_grid_component(state.total_count, |index| Element::new(GridTile { index }))
                    .layout_weight(1.0)
                    .grid_column_template("1fr 1fr 1fr")
                    .grid_column_gap(8.0)
                    .grid_row_gap(8.0)
                    .grid_cached_count(18)
                    .padding([0.0, 16.0, 16.0, 16.0])
                    .on_visible_range_change(Message::VisibleRange)
                    .on_load_more(state.total_count, 18, state.loading_more, Message::LoadMore)
                    .into()
            }
            Case::WaterFlow => virtual_water_flow_component(state.total_count, |index| {
                Element::new(FlowTile { index })
            })
            .layout_weight(1.0)
            .water_flow_column_template("1fr 1fr")
            .water_flow_column_gap(8.0)
            .water_flow_row_gap(8.0)
            .water_flow_cached_count(12)
            .padding([0.0, 16.0, 16.0, 16.0])
            .on_visible_range_change(Message::VisibleRange)
            .on_load_more(state.total_count, 12, state.loading_more, Message::LoadMore)
            .into(),
            Case::Grouped => grouped_virtual_list(
                group_data(),
                |group| Element::new(GroupHeader { group }),
                |group, index| Element::new(GroupRow { group, index }),
            ),
        })
    }
}

struct ListRow {
    index: u32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for ListRow {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let index = self.index;
        Some(
            row_component()
                .percent_width(1.0)
                .height(64.0)
                .align_items_center()
                .padding([0.0, 16.0, 0.0, 16.0])
                .background_color(if index % 2 == 0 {
                    0xffffffff
                } else {
                    0xfff1f5f9
                })
                .children(vec![
                    text(format!("#{:05}", index))
                        .font_size(16.0)
                        .font_weight(FontWeight::W600)
                        .font_color(0xff111827)
                        .into(),
                    text(format!("native adapter row {}", index))
                        .margin_left(16.0)
                        .font_size(14.0)
                        .font_color(0xff64748b)
                        .into(),
                ])
                .into(),
        )
    }
}

fn group_data() -> Vec<VirtualListGroup> {
    (0..40)
        .map(|index| VirtualListGroup::new(format!("group-{index}"), 250))
        .collect()
}

struct GridTile {
    index: u32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for GridTile {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let index = self.index;
        Some(
            column_component()
                .height(96.0)
                .border_radius(8.0)
                .background_color(0xffffffff)
                .padding(12.0)
                .children(vec![
                    text(format!("Grid {}", index))
                        .font_size(15.0)
                        .font_weight(FontWeight::W600)
                        .font_color(0xff111827)
                        .into(),
                    text(format!("cell {}", index % 12))
                        .margin_top(8.0)
                        .font_size(13.0)
                        .font_color(0xff64748b)
                        .into(),
                ])
                .into(),
        )
    }
}

struct FlowTile {
    index: u32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for FlowTile {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let index = self.index;
        let height = 88.0 + ((index % 5) as f32 * 18.0);
        Some(
            column_component()
                .height(height)
                .border_radius(8.0)
                .background_color(if index % 2 == 0 {
                    0xffffffff
                } else {
                    0xffeef2ff
                })
                .padding(12.0)
                .children(vec![
                    text(format!("Flow {}", index))
                        .font_size(15.0)
                        .font_weight(FontWeight::W600)
                        .font_color(0xff111827)
                        .into(),
                    text(format!("height {}", height as i32))
                        .margin_top(8.0)
                        .font_size(13.0)
                        .font_color(0xff64748b)
                        .into(),
                ])
                .into(),
        )
    }
}

struct GroupHeader {
    group: u32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for GroupHeader {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            row_component()
                .percent_width(1.0)
                .height(44.0)
                .align_items_center()
                .background_color(0xffe2e8f0)
                .padding([0.0, 16.0, 0.0, 16.0])
                .children(vec![text(format!("Group {}", self.group))
                    .font_size(15.0)
                    .font_weight(FontWeight::W600)
                    .font_color(0xff0f172a)
                    .into()])
                .into(),
        )
    }
}

struct GroupRow {
    group: u32,
    index: u32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for GroupRow {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            row_component()
                .percent_width(1.0)
                .height(56.0)
                .align_items_center()
                .padding([0.0, 16.0, 0.0, 16.0])
                .background_color(0xffffffff)
                .children(vec![text(format!(
                    "Group {} item {}",
                    self.group, self.index
                ))
                .font_size(14.0)
                .font_color(0xff334155)
                .into()])
                .into(),
        )
    }
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::new, update, view)
}
