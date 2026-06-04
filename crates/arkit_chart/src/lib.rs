use std::cell::RefCell;
use std::collections::BTreeMap;
use std::f32::consts::TAU;
use std::rc::Rc;

use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::types::advanced::NodeDirtyFlag;
use arkit::prelude::*;
use ohos_drawing_binding::{
    Brush, Canvas, FontCollection, Path, Pen, Point, Rect, TextStyle, TypographyBuilder,
    TypographyStyle,
};
use serde_json::Value;

const DEFAULT_COLORS: [u32; 10] = [
    0xFF5470C6, 0xFF91CC75, 0xFFFAC858, 0xFFEE6666, 0xFF73C0DE, 0xFF3BA272, 0xFFFC8452, 0xFF9A60B4,
    0xFFEA7CCC, 0xFF2F4554,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOption {
    pub title: Option<Title>,
    pub legend: Option<Legend>,
    pub grid: Grid,
    pub x_axis: Vec<Axis>,
    pub y_axis: Vec<Axis>,
    pub tooltip: Tooltip,
    pub dataset: Option<Dataset>,
    pub visual_style: VisualStyle,
    pub series: Vec<Series>,
    pub diagnostics: Vec<Diagnostic>,
    pub extra: BTreeMap<String, Value>,
}

impl Default for ChartOption {
    fn default() -> Self {
        Self {
            title: None,
            legend: Some(Legend::default()),
            grid: Grid::default(),
            x_axis: vec![Axis::category(Vec::<String>::new())],
            y_axis: vec![Axis::value()],
            tooltip: Tooltip::default(),
            dataset: None,
            visual_style: VisualStyle::default(),
            series: Vec::new(),
            diagnostics: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

impl ChartOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, text: impl Into<String>) -> Self {
        self.title = Some(Title { text: text.into() });
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = vec![axis];
        self
    }

    pub fn y_axis(mut self, axis: Axis) -> Self {
        self.y_axis = vec![axis];
        self
    }

    pub fn series(mut self, series: impl IntoIterator<Item = Series>) -> Self {
        self.series = series.into_iter().collect();
        self
    }

    pub fn push_series(mut self, series: Series) -> Self {
        self.series.push(series);
        self
    }

    pub fn from_json_str(input: &str) -> Result<Self, ChartParseError> {
        let value = serde_json::from_str(input).map_err(|error| ChartParseError {
            message: error.to_string(),
        })?;
        Self::from_json_value(value)
    }

    pub fn from_json_value(value: Value) -> Result<Self, ChartParseError> {
        let mut option = ChartOption::default();
        let Value::Object(mut object) = value else {
            return Err(ChartParseError {
                message: String::from("chart option must be a JSON object"),
            });
        };

        if let Some(value) = object.remove("title") {
            option.title = parse_title(value);
        }
        if let Some(value) = object.remove("legend") {
            option.legend = parse_legend(value);
        }
        if let Some(value) = object.remove("grid") {
            option.grid = parse_grid(value);
        }
        if let Some(value) = object.remove("tooltip") {
            option.tooltip = parse_tooltip(value);
        }
        if let Some(value) = object.remove("xAxis").or_else(|| object.remove("x_axis")) {
            option.x_axis = parse_axis_list(value, AxisOrientation::X);
        }
        if let Some(value) = object.remove("yAxis").or_else(|| object.remove("y_axis")) {
            option.y_axis = parse_axis_list(value, AxisOrientation::Y);
        }
        if let Some(value) = object.remove("dataset") {
            option.dataset = parse_dataset(value);
        }
        if let Some(value) = object
            .remove("color")
            .or_else(|| object.remove("colors"))
            .and_then(parse_color_palette)
        {
            option.visual_style.palette = value;
        }
        if let Some(value) = object.remove("series") {
            let (series, diagnostics) = parse_series_list(value);
            option.series = series;
            option.diagnostics.extend(diagnostics);
        }

        option.extra = object.into_iter().collect();
        Ok(option)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartParseError {
    pub message: String,
}

impl std::fmt::Display for ChartParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ChartParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legend {
    pub show: bool,
}

impl Default for Legend {
    fn default() -> Self {
        Self { show: true }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            left: 48.0,
            right: 24.0,
            top: 48.0,
            bottom: 38.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tooltip {
    pub show: bool,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self { show: true }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    pub source: Vec<Vec<DataValue>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualStyle {
    pub palette: Vec<u32>,
    pub background_color: u32,
    pub text_color: u32,
    pub axis_color: u32,
    pub split_line_color: u32,
}

impl Default for VisualStyle {
    fn default() -> Self {
        Self {
            palette: DEFAULT_COLORS.to_vec(),
            background_color: 0xFFFFFFFF,
            text_color: 0xFF1F2937,
            axis_color: 0xFF94A3B8,
            split_line_color: 0xFFE5E7EB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisOrientation {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisType {
    Category,
    Value,
    Time,
    Log,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    pub axis_type: AxisType,
    pub name: Option<String>,
    pub data: Vec<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Axis {
    pub fn category(data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            axis_type: AxisType::Category,
            name: None,
            data: data.into_iter().map(Into::into).collect(),
            min: None,
            max: None,
        }
    }

    pub fn value() -> Self {
        Self {
            axis_type: AxisType::Value,
            name: None,
            data: Vec::new(),
            min: None,
            max: None,
        }
    }

    pub fn time() -> Self {
        Self {
            axis_type: AxisType::Time,
            ..Self::value()
        }
    }

    pub fn log() -> Self {
        Self {
            axis_type: AxisType::Log,
            ..Self::value()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Number(f64),
    String(String),
}

impl DataValue {
    fn as_f64(&self) -> Option<f64> {
        match self {
            DataValue::Number(value) => Some(*value),
            DataValue::String(value) => value.parse().ok(),
        }
    }
}

impl From<f64> for DataValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for DataValue {
    fn from(value: i32) -> Self {
        Self::Number(value as f64)
    }
}

impl From<&str> for DataValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    pub name: Option<String>,
    pub values: Vec<DataValue>,
    pub style: Option<VisualStyle>,
}

impl DataPoint {
    pub fn scalar(value: impl Into<DataValue>) -> Self {
        Self {
            name: None,
            values: vec![value.into()],
            style: None,
        }
    }

    pub fn named(name: impl Into<String>, value: impl Into<DataValue>) -> Self {
        Self {
            name: Some(name.into()),
            values: vec![value.into()],
            style: None,
        }
    }

    pub fn values(values: impl IntoIterator<Item = impl Into<DataValue>>) -> Self {
        Self {
            name: None,
            values: values.into_iter().map(Into::into).collect(),
            style: None,
        }
    }

    fn number(&self, index: usize) -> f64 {
        self.values
            .get(index)
            .and_then(DataValue::as_f64)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicSeries {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub style: Option<VisualStyle>,
}

impl BasicSeries {
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            name: Some(name.into()),
            data: values.into_iter().map(DataPoint::scalar).collect(),
            style: None,
        }
    }

    pub fn data(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().collect(),
            style: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeData {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkData {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphSeries {
    pub name: Option<String>,
    pub nodes: Vec<NodeData>,
    pub links: Vec<LinkData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SankeySeries {
    pub name: Option<String>,
    pub nodes: Vec<NodeData>,
    pub links: Vec<LinkData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapFeature {
    pub name: String,
    pub value: f64,
    pub polygons: Vec<Vec<(f64, f64)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSeries {
    pub name: Option<String>,
    pub features: Vec<MapFeature>,
}

pub type CustomSeriesRenderer = Rc<dyn for<'a> Fn(CustomRenderContext<'a>)>;

#[derive(Clone)]
pub struct CustomSeries {
    pub name: Option<String>,
    pub data: Vec<DataPoint>,
    pub renderer: CustomSeriesRenderer,
}

impl std::fmt::Debug for CustomSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomSeries")
            .field("name", &self.name)
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

impl PartialEq for CustomSeries {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.data == other.data
    }
}

pub struct CustomRenderContext<'a> {
    pub canvas: &'a Canvas,
    pub width: f32,
    pub height: f32,
    pub palette: &'a [u32],
}

#[derive(Clone)]
pub enum Series {
    Line(BasicSeries),
    Bar(BasicSeries),
    Pie(BasicSeries),
    Scatter(BasicSeries),
    Radar(BasicSeries),
    Gauge(BasicSeries),
    Funnel(BasicSeries),
    Heatmap(BasicSeries),
    Candlestick(BasicSeries),
    Tree(GraphSeries),
    Treemap(BasicSeries),
    Graph(GraphSeries),
    Sankey(SankeySeries),
    Map(MapSeries),
    Custom(CustomSeries),
}

impl std::fmt::Debug for Series {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Series::Line(value) => f.debug_tuple("Line").field(value).finish(),
            Series::Bar(value) => f.debug_tuple("Bar").field(value).finish(),
            Series::Pie(value) => f.debug_tuple("Pie").field(value).finish(),
            Series::Scatter(value) => f.debug_tuple("Scatter").field(value).finish(),
            Series::Radar(value) => f.debug_tuple("Radar").field(value).finish(),
            Series::Gauge(value) => f.debug_tuple("Gauge").field(value).finish(),
            Series::Funnel(value) => f.debug_tuple("Funnel").field(value).finish(),
            Series::Heatmap(value) => f.debug_tuple("Heatmap").field(value).finish(),
            Series::Candlestick(value) => f.debug_tuple("Candlestick").field(value).finish(),
            Series::Tree(value) => f.debug_tuple("Tree").field(value).finish(),
            Series::Treemap(value) => f.debug_tuple("Treemap").field(value).finish(),
            Series::Graph(value) => f.debug_tuple("Graph").field(value).finish(),
            Series::Sankey(value) => f.debug_tuple("Sankey").field(value).finish(),
            Series::Map(value) => f.debug_tuple("Map").field(value).finish(),
            Series::Custom(value) => f.debug_tuple("Custom").field(value).finish(),
        }
    }
}

impl PartialEq for Series {
    fn eq(&self, other: &Self) -> bool {
        format!("{self:?}") == format!("{other:?}")
    }
}

impl Series {
    pub fn line(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self::Line(BasicSeries::new(name, values))
    }

    pub fn bar(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self::Bar(BasicSeries::new(name, values))
    }

    pub fn pie(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Pie(BasicSeries::data(name, data))
    }

    pub fn scatter(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Scatter(BasicSeries::data(name, data))
    }

    pub fn radar(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self::Radar(BasicSeries::new(name, values))
    }

    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self::Gauge(BasicSeries::new(name, [value]))
    }

    pub fn funnel(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Funnel(BasicSeries::data(name, data))
    }

    pub fn heatmap(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Heatmap(BasicSeries::data(name, data))
    }

    pub fn candlestick(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Candlestick(BasicSeries::data(name, data))
    }

    pub fn tree(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        Self::Tree(GraphSeries {
            name: Some(name.into()),
            nodes,
            links,
        })
    }

    pub fn treemap(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Treemap(BasicSeries::data(name, data))
    }

    pub fn graph(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        Self::Graph(GraphSeries {
            name: Some(name.into()),
            nodes,
            links,
        })
    }

    pub fn sankey(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        Self::Sankey(SankeySeries {
            name: Some(name.into()),
            nodes,
            links,
        })
    }

    pub fn map(name: impl Into<String>, features: Vec<MapFeature>) -> Self {
        Self::Map(MapSeries {
            name: Some(name.into()),
            features,
        })
    }

    pub fn custom(
        name: impl Into<String>,
        data: Vec<DataPoint>,
        renderer: impl for<'a> Fn(CustomRenderContext<'a>) + 'static,
    ) -> Self {
        Self::Custom(CustomSeries {
            name: Some(name.into()),
            data,
            renderer: Rc::new(renderer),
        })
    }

    fn name(&self) -> Option<&str> {
        match self {
            Series::Line(v)
            | Series::Bar(v)
            | Series::Pie(v)
            | Series::Scatter(v)
            | Series::Radar(v)
            | Series::Gauge(v)
            | Series::Funnel(v)
            | Series::Heatmap(v)
            | Series::Candlestick(v)
            | Series::Treemap(v) => v.name.as_deref(),
            Series::Tree(v) | Series::Graph(v) => v.name.as_deref(),
            Series::Sankey(v) => v.name.as_deref(),
            Series::Map(v) => v.name.as_deref(),
            Series::Custom(v) => v.name.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartEvent {
    pub series_index: usize,
    pub data_index: usize,
    pub series_name: Option<String>,
    pub name: Option<String>,
    pub value: Vec<f64>,
    pub x: f32,
    pub y: f32,
    pub component_type: String,
}

pub struct Chart<Message> {
    option: ChartOption,
    on_select: Option<Rc<dyn Fn(ChartEvent) -> Message>>,
}

pub fn chart<Message>(option: impl Into<ChartOption>) -> Chart<Message> {
    Chart::new(option)
}

impl<Message> Chart<Message> {
    pub fn new(option: impl Into<ChartOption>) -> Self {
        Self {
            option: option.into(),
            on_select: None,
        }
    }

    pub fn on_select(mut self, handler: impl Fn(ChartEvent) -> Message + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> From<Chart<Message>> for Element<Message> {
    fn from(value: Chart<Message>) -> Self {
        Element::new(value)
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Chart<Message>
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Element<Message> {
        let option_for_draw = self.option.clone();
        let option_for_hit = self.option.clone();
        let selected = Rc::new(RefCell::new(None::<ChartEvent>));
        let selected_for_draw = selected.clone();
        let node_ref = Rc::new(RefCell::new(
            None::<arkit::ohos_arkui_binding::common::node::ArkUINode>,
        ));
        let node_for_event = node_ref.clone();
        let handler = self.on_select.clone();

        custom_canvas_component(move |ctx| {
            render_option(
                &option_for_draw,
                selected_for_draw.borrow().as_ref(),
                Some(ctx.canvas()),
                ctx.width,
                ctx.height,
            );
        })
        .percent_width(1.0)
        .percent_height(1.0)
        .hit_test_behavior(HitTestBehavior::Default)
        .with_patch(move |node| {
            node_ref.replace(Some(node.clone()));
            node.mark_dirty(NodeDirtyFlag::NeedRender)?;
            Ok(())
        })
        .on_event(NodeEventType::TouchEvent, move |event| {
            let Some(input) = event.input_event() else {
                return;
            };
            if !matches!(input.action, UIInputAction::Up) {
                return;
            }
            let x = input.pointer_x();
            let y = input.pointer_y();
            let size = node_for_event
                .borrow()
                .as_ref()
                .and_then(|node| node.layout_size().ok())
                .map(|size| (size.width.max(1) as f32, size.height.max(1) as f32))
                .unwrap_or((1.0, 1.0));
            let hit = hit_test(&option_for_hit, x, y, size.0, size.1);
            selected.replace(hit.clone());
            if let Some(node) = node_for_event.borrow().as_ref() {
                let _ = node.mark_dirty(NodeDirtyFlag::NeedRender);
            }
            if let (Some(hit), Some(handler)) = (hit, handler.as_ref()) {
                arkit::internal::dispatch(handler(hit));
            }
        })
        .into()
    }
}

pub fn hit_test(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<ChartEvent> {
    render_option(option, None, None, width, height)
        .into_iter()
        .filter_map(|region| region.hit(x, y).map(|distance| (distance, region.event)))
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, event)| event)
}

#[derive(Debug, Clone, Copy)]
struct Plot {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone)]
enum HitShape {
    Point {
        x: f32,
        y: f32,
        radius: f32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Sector {
        cx: f32,
        cy: f32,
        inner: f32,
        outer: f32,
        start: f32,
        sweep: f32,
    },
}

#[derive(Debug, Clone)]
struct HitRegion {
    shape: HitShape,
    event: ChartEvent,
}

impl HitRegion {
    fn hit(&self, x: f32, y: f32) -> Option<f32> {
        match self.shape {
            HitShape::Point {
                x: px,
                y: py,
                radius,
            } => {
                let distance = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                (distance <= radius).then_some(distance)
            }
            HitShape::Rect {
                x: rx,
                y: ry,
                width,
                height,
            } => (x >= rx && x <= rx + width && y >= ry && y <= ry + height).then_some(0.0),
            HitShape::Sector {
                cx,
                cy,
                inner,
                outer,
                start,
                sweep,
            } => {
                let dx = x - cx;
                let dy = y - cy;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < inner || distance > outer {
                    return None;
                }
                let mut angle = dy.atan2(dx);
                if angle < 0.0 {
                    angle += TAU;
                }
                let mut local = angle - start;
                if local < 0.0 {
                    local += TAU;
                }
                (local <= sweep).then_some((outer - distance).abs())
            }
        }
    }
}

fn render_option(
    option: &ChartOption,
    selected: Option<&ChartEvent>,
    canvas: Option<&Canvas>,
    width: f32,
    height: f32,
) -> Vec<HitRegion> {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let mut hits = Vec::new();
    let palette = effective_palette(option);
    let plot = Plot {
        x: option.grid.left,
        y: option.grid.top + option.title.as_ref().map(|_| 8.0).unwrap_or_default(),
        width: (width - option.grid.left - option.grid.right).max(1.0),
        height: (height - option.grid.top - option.grid.bottom).max(1.0),
    };

    if let Some(canvas) = canvas {
        fill_rect(
            canvas,
            0.0,
            0.0,
            width,
            height,
            option.visual_style.background_color,
        );
        if let Some(title) = &option.title {
            draw_text(
                canvas,
                &title.text,
                14.0,
                8.0,
                20.0,
                option.visual_style.text_color,
                700,
            );
        }
        if option
            .legend
            .as_ref()
            .map(|legend| legend.show)
            .unwrap_or(false)
        {
            draw_legend(canvas, option, width, &palette);
        }
    }

    let cartesian_indices: Vec<usize> = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| is_cartesian_series(series).then_some(index))
        .collect();
    if !cartesian_indices.is_empty() {
        draw_cartesian(
            option,
            &cartesian_indices,
            &plot,
            &palette,
            canvas,
            &mut hits,
        );
    }

    let free_series: Vec<(usize, &Series)> = option
        .series
        .iter()
        .enumerate()
        .filter(|(_, series)| !is_cartesian_series(series))
        .collect();
    let slots = free_series.len().max(1);
    for (slot, (series_index, series)) in free_series.into_iter().enumerate() {
        let area = free_plot(plot, slot, slots);
        match series {
            Series::Pie(series) => {
                draw_pie(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Radar(series) => {
                draw_radar(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Gauge(series) => {
                draw_gauge(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Funnel(series) => {
                draw_funnel(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Tree(series) => draw_graph_like(
                "tree",
                series_index,
                series,
                area,
                &palette,
                canvas,
                &mut hits,
            ),
            Series::Treemap(series) => {
                draw_treemap(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Graph(series) => draw_graph_like(
                "graph",
                series_index,
                series,
                area,
                &palette,
                canvas,
                &mut hits,
            ),
            Series::Sankey(series) => {
                draw_sankey(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Map(series) => {
                draw_map(series_index, series, area, &palette, canvas, &mut hits)
            }
            Series::Custom(series) => {
                if let Some(canvas) = canvas {
                    (series.renderer)(CustomRenderContext {
                        canvas,
                        width: area.width,
                        height: area.height,
                        palette: &palette,
                    });
                }
            }
            Series::Line(_)
            | Series::Bar(_)
            | Series::Scatter(_)
            | Series::Heatmap(_)
            | Series::Candlestick(_) => {}
        }
    }

    if let (Some(canvas), Some(selected)) = (canvas, selected) {
        if option.tooltip.show {
            draw_tooltip(canvas, selected, width, height);
        }
    }

    hits
}

fn free_plot(plot: Plot, index: usize, count: usize) -> Plot {
    if count <= 1 {
        return plot;
    }
    let columns = (count as f32).sqrt().ceil() as usize;
    let rows = count.div_ceil(columns);
    let col = index % columns;
    let row = index / columns;
    Plot {
        x: plot.x + plot.width * col as f32 / columns as f32,
        y: plot.y + plot.height * row as f32 / rows as f32,
        width: plot.width / columns as f32,
        height: plot.height / rows as f32,
    }
}

fn is_cartesian_series(series: &Series) -> bool {
    matches!(
        series,
        Series::Line(_)
            | Series::Bar(_)
            | Series::Scatter(_)
            | Series::Heatmap(_)
            | Series::Candlestick(_)
    )
}

fn effective_palette(option: &ChartOption) -> Vec<u32> {
    if option.visual_style.palette.is_empty() {
        DEFAULT_COLORS.to_vec()
    } else {
        option.visual_style.palette.clone()
    }
}

fn color(palette: &[u32], index: usize) -> u32 {
    palette[index % palette.len().max(1)]
}

fn draw_cartesian(
    option: &ChartOption,
    series_indices: &[usize],
    plot: &Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let mut values = Vec::new();
    let mut max_count = 0;
    for index in series_indices {
        let data = series_data(&option.series[*index]);
        max_count = max_count.max(data.len());
        for point in data {
            values.extend(point.values.iter().filter_map(DataValue::as_f64));
        }
    }
    let (min_y, max_y) = value_extent(&values).unwrap_or((0.0, 1.0));
    let categories = option
        .x_axis
        .first()
        .map(|axis| axis.data.clone())
        .filter(|data| !data.is_empty())
        .unwrap_or_else(|| {
            (0..max_count)
                .map(|index| (index + 1).to_string())
                .collect()
        });

    if let Some(canvas) = canvas {
        stroke_line(
            canvas,
            plot.x,
            plot.y,
            plot.x,
            plot.y + plot.height,
            option.visual_style.axis_color,
            1.0,
        );
        stroke_line(
            canvas,
            plot.x,
            plot.y + plot.height,
            plot.x + plot.width,
            plot.y + plot.height,
            option.visual_style.axis_color,
            1.0,
        );
        for step in 0..=4 {
            let y = plot.y + plot.height * step as f32 / 4.0;
            stroke_line(
                canvas,
                plot.x,
                y,
                plot.x + plot.width,
                y,
                option.visual_style.split_line_color,
                0.6,
            );
        }
        for (index, label) in categories.iter().take(6).enumerate() {
            let x = plot.x + plot.width * (index as f32 + 0.5) / categories.len().max(1) as f32;
            draw_text(
                canvas,
                label,
                x - 12.0,
                plot.y + plot.height + 18.0,
                10.0,
                option.visual_style.text_color,
                400,
            );
        }
    }

    let bar_series_count = series_indices
        .iter()
        .filter(|index| matches!(option.series[**index], Series::Bar(_)))
        .count()
        .max(1);
    let mut bar_offset = 0;
    for series_index in series_indices {
        match &option.series[*series_index] {
            Series::Line(series) => draw_line_series(
                *series_index,
                series,
                plot,
                min_y,
                max_y,
                palette,
                canvas,
                hits,
            ),
            Series::Bar(series) => {
                draw_bar_series(
                    *series_index,
                    bar_offset,
                    bar_series_count,
                    series,
                    plot,
                    min_y,
                    max_y,
                    palette,
                    canvas,
                    hits,
                );
                bar_offset += 1;
            }
            Series::Scatter(series) => draw_scatter_series(
                *series_index,
                series,
                plot,
                min_y,
                max_y,
                palette,
                canvas,
                hits,
            ),
            Series::Heatmap(series) => {
                draw_heatmap_series(*series_index, series, plot, palette, canvas, hits)
            }
            Series::Candlestick(series) => draw_candlestick_series(
                *series_index,
                series,
                plot,
                min_y,
                max_y,
                palette,
                canvas,
                hits,
            ),
            _ => {}
        }
    }
}

fn draw_line_series(
    series_index: usize,
    series: &BasicSeries,
    plot: &Plot,
    min_y: f64,
    max_y: f64,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let count = series.data.len().max(1);
    let mut path = Path::new();
    for (index, point) in series.data.iter().enumerate() {
        let x = x_at(plot, index, count);
        let y = y_at(plot, point.number(0), min_y, max_y);
        if index == 0 {
            path.move_to(x, y);
        } else {
            path.line_to(x, y);
        }
        if let Some(canvas) = canvas {
            fill_circle(canvas, x, y, 3.5, color(palette, series_index));
        }
        hits.push(point_hit(
            "line",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            y,
            8.0,
        ));
    }
    if let Some(canvas) = canvas {
        stroke_path(canvas, &path, color(palette, series_index), 2.0);
    }
}

fn draw_bar_series(
    series_index: usize,
    bar_offset: usize,
    bar_count: usize,
    series: &BasicSeries,
    plot: &Plot,
    min_y: f64,
    max_y: f64,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let count = series.data.len().max(1);
    let slot = plot.width / count as f32;
    let width = slot * 0.7 / bar_count as f32;
    for (index, point) in series.data.iter().enumerate() {
        let value = point.number(0);
        let base = y_at(plot, 0.0_f64.max(min_y), min_y, max_y);
        let y = y_at(plot, value, min_y, max_y);
        let x = plot.x + slot * index as f32 + slot * 0.15 + width * bar_offset as f32;
        let top = y.min(base);
        let height = (base - y).abs().max(1.0);
        if let Some(canvas) = canvas {
            fill_rect(canvas, x, top, width, height, color(palette, series_index));
        }
        hits.push(rect_hit(
            "bar",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            top,
            width,
            height,
        ));
    }
}

fn draw_scatter_series(
    series_index: usize,
    series: &BasicSeries,
    plot: &Plot,
    min_y: f64,
    max_y: f64,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let count = series.data.len().max(1);
    for (index, point) in series.data.iter().enumerate() {
        let x = if point.values.len() > 1 {
            x_value_at(plot, point.number(0), 0.0, count.saturating_sub(1) as f64)
        } else {
            x_at(plot, index, count)
        };
        let y = y_at(
            plot,
            if point.values.len() > 1 {
                point.number(1)
            } else {
                point.number(0)
            },
            min_y,
            max_y,
        );
        if let Some(canvas) = canvas {
            fill_circle(canvas, x, y, 4.5, color(palette, series_index));
        }
        hits.push(point_hit(
            "scatter",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            y,
            9.0,
        ));
    }
}

fn draw_heatmap_series(
    series_index: usize,
    series: &BasicSeries,
    plot: &Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let max_x = series
        .data
        .iter()
        .map(|p| p.number(0) as usize)
        .max()
        .unwrap_or(0)
        + 1;
    let max_y = series
        .data
        .iter()
        .map(|p| p.number(1) as usize)
        .max()
        .unwrap_or(0)
        + 1;
    let cell_w = plot.width / max_x.max(1) as f32;
    let cell_h = plot.height / max_y.max(1) as f32;
    let max_v = series
        .data
        .iter()
        .map(|p| p.number(2))
        .fold(0.0, f64::max)
        .max(1.0);
    for (index, point) in series.data.iter().enumerate() {
        let x = plot.x + point.number(0) as f32 * cell_w;
        let y = plot.y + plot.height - (point.number(1) as f32 + 1.0) * cell_h;
        let color_index =
            ((point.number(2) / max_v) * (palette.len().saturating_sub(1)) as f64).round() as usize;
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                y,
                cell_w - 1.0,
                cell_h - 1.0,
                color(palette, color_index),
            );
        }
        hits.push(rect_hit(
            "heatmap",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            y,
            cell_w,
            cell_h,
        ));
    }
}

fn draw_candlestick_series(
    series_index: usize,
    series: &BasicSeries,
    plot: &Plot,
    min_y: f64,
    max_y: f64,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let count = series.data.len().max(1);
    let width = plot.width / count as f32 * 0.5;
    for (index, point) in series.data.iter().enumerate() {
        let x = x_at(plot, index, count);
        let open = point.number(0);
        let close = point.number(1);
        let low = point.number(2);
        let high = point.number(3);
        let color_value = if close >= open {
            color(palette, 3)
        } else {
            color(palette, 1)
        };
        let high_y = y_at(plot, high, min_y, max_y);
        let low_y = y_at(plot, low, min_y, max_y);
        let open_y = y_at(plot, open, min_y, max_y);
        let close_y = y_at(plot, close, min_y, max_y);
        if let Some(canvas) = canvas {
            stroke_line(canvas, x, high_y, x, low_y, color_value, 1.2);
            fill_rect(
                canvas,
                x - width / 2.0,
                open_y.min(close_y),
                width,
                (open_y - close_y).abs().max(1.0),
                color_value,
            );
        }
        hits.push(rect_hit(
            "candlestick",
            series_index,
            index,
            series.name.clone(),
            point,
            x - width / 2.0,
            high_y,
            width,
            low_y - high_y,
        ));
    }
}

fn draw_pie(
    series_index: usize,
    series: &BasicSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let cx = plot.x + plot.width / 2.0;
    let cy = plot.y + plot.height / 2.0;
    let radius = plot.width.min(plot.height) * 0.36;
    let total: f64 = series
        .data
        .iter()
        .map(|p| p.number(0).max(0.0))
        .sum::<f64>()
        .max(1.0);
    let mut start = -TAU / 4.0;
    for (index, point) in series.data.iter().enumerate() {
        let sweep = (point.number(0).max(0.0) / total) as f32 * TAU;
        if let Some(canvas) = canvas {
            fill_sector(canvas, cx, cy, radius, start, sweep, color(palette, index));
        }
        hits.push(HitRegion {
            shape: HitShape::Sector {
                cx,
                cy,
                inner: 0.0,
                outer: radius,
                start: normalize_angle(start),
                sweep,
            },
            event: chart_event(
                "pie",
                series_index,
                index,
                series.name.clone(),
                point,
                cx,
                cy,
            ),
        });
        start += sweep;
    }
}

fn draw_radar(
    series_index: usize,
    series: &BasicSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let cx = plot.x + plot.width / 2.0;
    let cy = plot.y + plot.height / 2.0;
    let radius = plot.width.min(plot.height) * 0.35;
    let max_value = series
        .data
        .iter()
        .map(|p| p.number(0))
        .fold(0.0, f64::max)
        .max(1.0);
    let count = series.data.len().max(3);
    let mut path = Path::new();
    for (index, point) in series.data.iter().enumerate() {
        let angle = -TAU / 4.0 + TAU * index as f32 / count as f32;
        let r = radius * (point.number(0) / max_value) as f32;
        let x = cx + angle.cos() * r;
        let y = cy + angle.sin() * r;
        if index == 0 {
            path.move_to(x, y);
        } else {
            path.line_to(x, y);
        }
        if let Some(canvas) = canvas {
            stroke_line(
                canvas,
                cx,
                cy,
                cx + angle.cos() * radius,
                cy + angle.sin() * radius,
                0xFFE5E7EB,
                1.0,
            );
            fill_circle(canvas, x, y, 3.5, color(palette, series_index));
        }
        hits.push(point_hit(
            "radar",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            y,
            8.0,
        ));
    }
    path.close();
    if let Some(canvas) = canvas {
        stroke_path(canvas, &path, color(palette, series_index), 2.0);
    }
}

fn draw_gauge(
    series_index: usize,
    series: &BasicSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let cx = plot.x + plot.width / 2.0;
    let cy = plot.y + plot.height * 0.62;
    let radius = plot.width.min(plot.height) * 0.34;
    let value = series
        .data
        .first()
        .map(|p| p.number(0))
        .unwrap_or_default()
        .clamp(0.0, 100.0);
    let start = TAU * 0.55;
    let sweep = TAU * 0.9;
    let angle = start + sweep * (value as f32 / 100.0);
    if let Some(canvas) = canvas {
        stroke_arc(canvas, cx, cy, radius, start, sweep, 0xFFE5E7EB, 8.0);
        stroke_arc(
            canvas,
            cx,
            cy,
            radius,
            start,
            sweep * (value as f32 / 100.0),
            color(palette, series_index),
            8.0,
        );
        stroke_line(
            canvas,
            cx,
            cy,
            cx + angle.cos() * radius * 0.82,
            cy + angle.sin() * radius * 0.82,
            color(palette, series_index),
            3.0,
        );
        fill_circle(canvas, cx, cy, 5.0, color(palette, series_index));
    }
    if let Some(point) = series.data.first() {
        hits.push(point_hit(
            "gauge",
            series_index,
            0,
            series.name.clone(),
            point,
            cx,
            cy,
            radius,
        ));
    }
}

fn draw_funnel(
    series_index: usize,
    series: &BasicSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let max_value = series
        .data
        .iter()
        .map(|p| p.number(0))
        .fold(0.0, f64::max)
        .max(1.0);
    let h = plot.height / series.data.len().max(1) as f32;
    for (index, point) in series.data.iter().enumerate() {
        let top_w = plot.width * (point.number(0) / max_value) as f32 * 0.82;
        let next = series
            .data
            .get(index + 1)
            .map(|p| p.number(0))
            .unwrap_or(point.number(0));
        let bottom_w = plot.width * (next / max_value) as f32 * 0.82;
        let y = plot.y + h * index as f32;
        let mut path = Path::new();
        path.move_to(plot.x + (plot.width - top_w) / 2.0, y);
        path.line_to(plot.x + (plot.width + top_w) / 2.0, y);
        path.line_to(plot.x + (plot.width + bottom_w) / 2.0, y + h - 2.0);
        path.line_to(plot.x + (plot.width - bottom_w) / 2.0, y + h - 2.0);
        path.close();
        if let Some(canvas) = canvas {
            fill_path(canvas, &path, color(palette, index));
        }
        hits.push(rect_hit(
            "funnel",
            series_index,
            index,
            series.name.clone(),
            point,
            plot.x,
            y,
            plot.width,
            h,
        ));
    }
}

fn draw_treemap(
    series_index: usize,
    series: &BasicSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let total = series
        .data
        .iter()
        .map(|p| p.number(0).max(0.0))
        .sum::<f64>()
        .max(1.0);
    let mut x = plot.x;
    for (index, point) in series.data.iter().enumerate() {
        let width = plot.width * (point.number(0).max(0.0) / total) as f32;
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                plot.y,
                width.max(1.0) - 1.0,
                plot.height,
                color(palette, index),
            );
        }
        hits.push(rect_hit(
            "treemap",
            series_index,
            index,
            series.name.clone(),
            point,
            x,
            plot.y,
            width,
            plot.height,
        ));
        x += width;
    }
}

fn draw_graph_like(
    component: &str,
    series_index: usize,
    series: &GraphSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let count = series.nodes.len().max(1);
    let mut positions = Vec::with_capacity(count);
    for index in 0..count {
        let angle = -TAU / 4.0 + TAU * index as f32 / count as f32;
        positions.push((
            plot.x + plot.width / 2.0 + angle.cos() * plot.width.min(plot.height) * 0.32,
            plot.y + plot.height / 2.0 + angle.sin() * plot.width.min(plot.height) * 0.32,
        ));
    }
    if let Some(canvas) = canvas {
        for link in &series.links {
            if let (Some(a), Some(b)) = (positions.get(link.source), positions.get(link.target)) {
                stroke_line(
                    canvas,
                    a.0,
                    a.1,
                    b.0,
                    b.1,
                    0xFFCBD5E1,
                    1.2 + link.value as f32,
                );
            }
        }
    }
    for (index, node) in series.nodes.iter().enumerate() {
        let (x, y) = positions[index];
        if let Some(canvas) = canvas {
            fill_circle(canvas, x, y, 9.0 + node.value as f32, color(palette, index));
        }
        hits.push(HitRegion {
            shape: HitShape::Point { x, y, radius: 18.0 },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: Some(node.name.clone()),
                value: vec![node.value],
                x,
                y,
                component_type: component.to_string(),
            },
        });
    }
}

fn draw_sankey(
    series_index: usize,
    series: &SankeySeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let left_count = series.nodes.len().div_ceil(2).max(1);
    let right_count = (series.nodes.len() - left_count).max(1);
    let mut positions = Vec::new();
    for (index, node) in series.nodes.iter().enumerate() {
        let left = index < left_count;
        let group_index = if left { index } else { index - left_count };
        let group_count = if left { left_count } else { right_count };
        let x = if left {
            plot.x
        } else {
            plot.x + plot.width - 42.0
        };
        let y = plot.y + (group_index as f32 + 0.5) * plot.height / group_count as f32 - 12.0;
        positions.push((x, y));
        if let Some(canvas) = canvas {
            fill_rect(canvas, x, y, 42.0, 24.0, color(palette, index));
            draw_text(canvas, &node.name, x + 3.0, y + 16.0, 9.0, 0xFFFFFFFF, 500);
        }
        hits.push(HitRegion {
            shape: HitShape::Rect {
                x,
                y,
                width: 42.0,
                height: 24.0,
            },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: Some(node.name.clone()),
                value: vec![node.value],
                x,
                y,
                component_type: String::from("sankey"),
            },
        });
    }
    if let Some(canvas) = canvas {
        for link in &series.links {
            if let (Some(a), Some(b)) = (positions.get(link.source), positions.get(link.target)) {
                stroke_line(
                    canvas,
                    a.0 + 42.0,
                    a.1 + 12.0,
                    b.0,
                    b.1 + 12.0,
                    0x8894A3B8,
                    2.0 + link.value as f32,
                );
            }
        }
    }
}

fn draw_map(
    series_index: usize,
    series: &MapSeries,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let bounds = map_bounds(&series.features).unwrap_or((0.0, 0.0, 1.0, 1.0));
    for (index, feature) in series.features.iter().enumerate() {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for polygon in &feature.polygons {
            let mut path = Path::new();
            for (point_index, point) in polygon.iter().enumerate() {
                let x = plot.x
                    + ((*point).0 - bounds.0) as f32 / (bounds.2 - bounds.0).max(1e-6) as f32
                        * plot.width;
                let y = plot.y + plot.height
                    - ((*point).1 - bounds.1) as f32 / (bounds.3 - bounds.1).max(1e-6) as f32
                        * plot.height;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                if point_index == 0 {
                    path.move_to(x, y);
                } else {
                    path.line_to(x, y);
                }
            }
            path.close();
            if let Some(canvas) = canvas {
                fill_path(canvas, &path, color(palette, index));
                stroke_path(canvas, &path, 0xFFFFFFFF, 1.0);
            }
        }
        let point = DataPoint::named(feature.name.clone(), feature.value);
        hits.push(rect_hit(
            "map",
            series_index,
            index,
            series.name.clone(),
            &point,
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
        ));
    }
}

fn draw_legend(canvas: &Canvas, option: &ChartOption, width: f32, palette: &[u32]) {
    let mut x = 12.0;
    let y = 32.0;
    for (index, series) in option.series.iter().enumerate().take(5) {
        let Some(name) = series.name() else { continue };
        fill_rect(canvas, x, y - 8.0, 8.0, 8.0, color(palette, index));
        draw_text(
            canvas,
            name,
            x + 12.0,
            y,
            10.0,
            option.visual_style.text_color,
            400,
        );
        x += (name.len() as f32 * 6.0 + 28.0).min(width * 0.25);
    }
}

fn draw_tooltip(canvas: &Canvas, event: &ChartEvent, width: f32, height: f32) {
    let label = format!(
        "{} {}",
        event
            .name
            .as_deref()
            .or(event.series_name.as_deref())
            .unwrap_or("value"),
        event
            .value
            .first()
            .map(|value| format!("{value:.2}"))
            .unwrap_or_default()
    );
    let w = (label.len() as f32 * 7.0 + 16.0).clamp(80.0, 180.0);
    let h = 30.0;
    let x = event.x.min(width - w - 8.0).max(8.0);
    let y = (event.y - h - 10.0).min(height - h - 8.0).max(8.0);
    fill_rect(canvas, x, y, w, h, 0xEE111827);
    draw_text(canvas, &label, x + 8.0, y + 20.0, 11.0, 0xFFFFFFFF, 500);
}

fn point_hit(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    x: f32,
    y: f32,
    radius: f32,
) -> HitRegion {
    HitRegion {
        shape: HitShape::Point { x, y, radius },
        event: chart_event(
            component,
            series_index,
            data_index,
            series_name,
            point,
            x,
            y,
        ),
    }
}

fn rect_hit(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HitRegion {
    HitRegion {
        shape: HitShape::Rect {
            x,
            y,
            width,
            height,
        },
        event: chart_event(
            component,
            series_index,
            data_index,
            series_name,
            point,
            x + width / 2.0,
            y + height / 2.0,
        ),
    }
}

fn chart_event(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    x: f32,
    y: f32,
) -> ChartEvent {
    ChartEvent {
        series_index,
        data_index,
        series_name,
        name: point.name.clone(),
        value: point.values.iter().filter_map(DataValue::as_f64).collect(),
        x,
        y,
        component_type: component.to_string(),
    }
}

fn series_data(series: &Series) -> &[DataPoint] {
    match series {
        Series::Line(v)
        | Series::Bar(v)
        | Series::Pie(v)
        | Series::Scatter(v)
        | Series::Radar(v)
        | Series::Gauge(v)
        | Series::Funnel(v)
        | Series::Heatmap(v)
        | Series::Candlestick(v)
        | Series::Treemap(v) => &v.data,
        Series::Custom(v) => &v.data,
        Series::Tree(_) | Series::Graph(_) | Series::Sankey(_) | Series::Map(_) => &[],
    }
}

fn value_extent(values: &[f64]) -> Option<(f64, f64)> {
    let min = values.iter().copied().reduce(f64::min)?;
    let max = values.iter().copied().reduce(f64::max)?;
    if (max - min).abs() < f64::EPSILON {
        Some((min.min(0.0), max + 1.0))
    } else {
        Some((min.min(0.0), max))
    }
}

fn x_at(plot: &Plot, index: usize, count: usize) -> f32 {
    plot.x + plot.width * (index as f32 + 0.5) / count.max(1) as f32
}

fn x_value_at(plot: &Plot, value: f64, min: f64, max: f64) -> f32 {
    plot.x + ((value - min) / (max - min).max(1e-6)) as f32 * plot.width
}

fn y_at(plot: &Plot, value: f64, min: f64, max: f64) -> f32 {
    plot.y + plot.height - ((value - min) / (max - min).max(1e-6)) as f32 * plot.height
}

fn normalize_angle(angle: f32) -> f32 {
    let mut angle = angle % TAU;
    if angle < 0.0 {
        angle += TAU;
    }
    angle
}

fn map_bounds(features: &[MapFeature]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut any = false;
    for feature in features {
        for polygon in &feature.polygons {
            for (x, y) in polygon {
                any = true;
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
        }
    }
    any.then_some((min_x, min_y, max_x, max_y))
}

fn fill_rect(canvas: &Canvas, x: f32, y: f32, width: f32, height: f32, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    let rect = Rect::new(x, y, x + width.max(0.0), y + height.max(0.0));
    canvas.attach_brush(&brush);
    canvas.draw_rect(&rect);
    canvas.detach_brush();
}

fn fill_circle(canvas: &Canvas, x: f32, y: f32, radius: f32, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    let point = Point::new(x, y);
    canvas.attach_brush(&brush);
    canvas.draw_circle(&point, radius);
    canvas.detach_brush();
}

fn stroke_line(canvas: &Canvas, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, width: f32) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    canvas.attach_pen(&pen);
    canvas.draw_line(x1, y1, x2, y2);
    canvas.detach_pen();
}

fn stroke_path(canvas: &Canvas, path: &Path, color: u32, width: f32) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    canvas.attach_pen(&pen);
    canvas.draw_path(path);
    canvas.detach_pen();
}

fn fill_path(canvas: &Canvas, path: &Path, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    canvas.attach_brush(&brush);
    canvas.draw_path(path);
    canvas.detach_brush();
}

fn fill_sector(canvas: &Canvas, cx: f32, cy: f32, radius: f32, start: f32, sweep: f32, color: u32) {
    let mut path = Path::new();
    path.move_to(cx, cy);
    path.line_to(cx + start.cos() * radius, cy + start.sin() * radius);
    path.arc_to(
        cx - radius,
        cy - radius,
        cx + radius,
        cy + radius,
        start.to_degrees(),
        sweep.to_degrees(),
    );
    path.close();
    fill_path(canvas, &path, color);
}

fn stroke_arc(
    canvas: &Canvas,
    cx: f32,
    cy: f32,
    radius: f32,
    start: f32,
    sweep: f32,
    color: u32,
    width: f32,
) {
    let mut path = Path::new();
    path.arc_to(
        cx - radius,
        cy - radius,
        cx + radius,
        cy + radius,
        start.to_degrees(),
        sweep.to_degrees(),
    );
    stroke_path(canvas, &path, color, width);
}

fn draw_text(canvas: &Canvas, text: &str, x: f32, y: f32, size: f64, color: u32, weight: i32) {
    if text.is_empty() {
        return;
    }
    let mut font_collection = FontCollection::global_instance().unwrap_or_default();
    let mut typography_style = TypographyStyle::new();
    let mut text_style = TextStyle::new();
    text_style.set_color(color);
    text_style.set_font_size(size);
    text_style.set_font_weight(weight);
    let mut builder = TypographyBuilder::new(&mut typography_style, &mut font_collection);
    builder.push_text_style(&mut text_style);
    builder.add_text(text);
    builder.pop_text_style();
    let mut typography = builder.build();
    typography.layout(260.0);
    typography.paint(canvas, x as f64, (y - size as f32) as f64);
}

fn parse_title(value: Value) -> Option<Title> {
    match value {
        Value::String(text) => Some(Title { text }),
        Value::Object(object) => object
            .get("text")
            .and_then(Value::as_str)
            .map(|text| Title {
                text: text.to_string(),
            }),
        _ => None,
    }
}

fn parse_legend(value: Value) -> Option<Legend> {
    match value {
        Value::Bool(show) => Some(Legend { show }),
        Value::Object(object) => Some(Legend {
            show: object.get("show").and_then(Value::as_bool).unwrap_or(true),
        }),
        _ => None,
    }
}

fn parse_tooltip(value: Value) -> Tooltip {
    match value {
        Value::Bool(show) => Tooltip { show },
        Value::Object(object) => Tooltip {
            show: object.get("show").and_then(Value::as_bool).unwrap_or(true),
        },
        _ => Tooltip::default(),
    }
}

fn parse_grid(value: Value) -> Grid {
    let mut grid = Grid::default();
    let Value::Object(object) = value else {
        return grid;
    };
    grid.left = object.get("left").and_then(parse_f32).unwrap_or(grid.left);
    grid.right = object
        .get("right")
        .and_then(parse_f32)
        .unwrap_or(grid.right);
    grid.top = object.get("top").and_then(parse_f32).unwrap_or(grid.top);
    grid.bottom = object
        .get("bottom")
        .and_then(parse_f32)
        .unwrap_or(grid.bottom);
    grid
}

fn parse_axis_list(value: Value, orientation: AxisOrientation) -> Vec<Axis> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| parse_axis(value, orientation))
            .collect(),
        value => vec![parse_axis(value, orientation)],
    }
}

fn parse_axis(value: Value, orientation: AxisOrientation) -> Axis {
    let default = if matches!(orientation, AxisOrientation::X) {
        Axis::category(Vec::<String>::new())
    } else {
        Axis::value()
    };
    let Value::Object(object) = value else {
        return default;
    };
    let axis_type = match object.get("type").and_then(Value::as_str) {
        Some("value") => AxisType::Value,
        Some("time") => AxisType::Time,
        Some("log") => AxisType::Log,
        Some("category") => AxisType::Category,
        _ => default.axis_type,
    };
    Axis {
        axis_type,
        name: object
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        data: object
            .get("data")
            .and_then(Value::as_array)
            .map(|data| data.iter().map(label_from_value).collect())
            .unwrap_or_default(),
        min: object.get("min").and_then(Value::as_f64),
        max: object.get("max").and_then(Value::as_f64),
    }
}

fn parse_dataset(value: Value) -> Option<Dataset> {
    let source = match value {
        Value::Object(object) => object.get("source")?.clone(),
        value => value,
    };
    let rows = source.as_array()?;
    Some(Dataset {
        source: rows
            .iter()
            .filter_map(|row| {
                row.as_array().map(|cols| {
                    cols.iter()
                        .map(|value| {
                            value
                                .as_f64()
                                .map(DataValue::Number)
                                .unwrap_or_else(|| DataValue::String(label_from_value(value)))
                        })
                        .collect()
                })
            })
            .collect(),
    })
}

fn parse_color_palette(value: Value) -> Option<Vec<u32>> {
    value
        .as_array()
        .map(|values| values.iter().filter_map(parse_color).collect())
}

fn parse_series_list(value: Value) -> (Vec<Series>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let series = value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value])
        .into_iter()
        .filter_map(|value| parse_series(value, &mut diagnostics))
        .collect();
    (series, diagnostics)
}

fn parse_series(value: Value, diagnostics: &mut Vec<Diagnostic>) -> Option<Series> {
    let Value::Object(object) = value else {
        return None;
    };
    let kind = object.get("type").and_then(Value::as_str).unwrap_or("line");
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(kind)
        .to_string();
    let data = object
        .get("data")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(parse_data_point).collect())
        .unwrap_or_default();
    Some(match kind {
        "line" => Series::Line(BasicSeries::data(name, data)),
        "bar" => Series::Bar(BasicSeries::data(name, data)),
        "pie" => Series::Pie(BasicSeries::data(name, data)),
        "scatter" => Series::Scatter(BasicSeries::data(name, data)),
        "radar" => Series::Radar(BasicSeries::data(name, data)),
        "gauge" => Series::Gauge(BasicSeries::data(name, data)),
        "funnel" => Series::Funnel(BasicSeries::data(name, data)),
        "heatmap" => Series::Heatmap(BasicSeries::data(name, data)),
        "candlestick" => Series::Candlestick(BasicSeries::data(name, data)),
        "treemap" => Series::Treemap(BasicSeries::data(name, data)),
        "tree" => Series::Tree(parse_graph_series(name, &object)),
        "graph" => Series::Graph(parse_graph_series(name, &object)),
        "sankey" => Series::Sankey(parse_sankey_series(name, &object)),
        "map" => Series::Map(parse_map_series(name, &object)),
        "custom" => {
            diagnostics.push(Diagnostic {
                field: String::from("series.custom"),
                message: String::from(
                    "JSON custom series is parsed but not executed; use typed Rust custom renderer",
                ),
            });
            Series::Custom(CustomSeries {
                name: Some(name),
                data,
                renderer: Rc::new(|_| {}),
            })
        }
        other => {
            diagnostics.push(Diagnostic {
                field: format!("series.{other}"),
                message: String::from("unsupported series type"),
            });
            return None;
        }
    })
}

fn parse_graph_series(name: String, object: &serde_json::Map<String, Value>) -> GraphSeries {
    GraphSeries {
        name: Some(name),
        nodes: object
            .get("data")
            .or_else(|| object.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.iter().enumerate().map(parse_node_data).collect())
            .unwrap_or_default(),
        links: object
            .get("links")
            .or_else(|| object.get("edges"))
            .and_then(Value::as_array)
            .map(|links| links.iter().filter_map(parse_link_data).collect())
            .unwrap_or_default(),
    }
}

fn parse_sankey_series(name: String, object: &serde_json::Map<String, Value>) -> SankeySeries {
    let graph = parse_graph_series(name, object);
    SankeySeries {
        name: graph.name,
        nodes: graph.nodes,
        links: graph.links,
    }
}

fn parse_map_series(name: String, object: &serde_json::Map<String, Value>) -> MapSeries {
    let features = object
        .get("geoJson")
        .or_else(|| object.get("geoJSON"))
        .or_else(|| object.get("features"))
        .and_then(parse_geo_features)
        .unwrap_or_default();
    MapSeries {
        name: Some(name),
        features,
    }
}

fn parse_geo_features(value: &Value) -> Option<Vec<MapFeature>> {
    let features = match value {
        Value::Object(object) => object.get("features")?.as_array()?,
        Value::Array(values) => values,
        _ => return None,
    };
    Some(features.iter().filter_map(parse_geo_feature).collect())
}

fn parse_geo_feature(value: &Value) -> Option<MapFeature> {
    let object = value.as_object()?;
    let name = object
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get("name"))
        .and_then(Value::as_str)
        .or_else(|| object.get("name").and_then(Value::as_str))
        .unwrap_or("feature")
        .to_string();
    let feature_value = object
        .get("value")
        .and_then(Value::as_f64)
        .or_else(|| {
            object
                .get("properties")
                .and_then(Value::as_object)
                .and_then(|p| p.get("value"))
                .and_then(Value::as_f64)
        })
        .unwrap_or(1.0);
    let geometry = object.get("geometry").unwrap_or(value);
    let polygons = parse_polygons(geometry)?;
    Some(MapFeature {
        name,
        value: feature_value,
        polygons,
    })
}

fn parse_polygons(value: &Value) -> Option<Vec<Vec<(f64, f64)>>> {
    let object = value.as_object()?;
    let kind = object.get("type").and_then(Value::as_str)?;
    let coordinates = object.get("coordinates")?;
    match kind {
        "Polygon" => Some(parse_polygon_array(coordinates)),
        "MultiPolygon" => Some(
            coordinates
                .as_array()?
                .iter()
                .flat_map(parse_polygon_array)
                .collect(),
        ),
        _ => None,
    }
}

fn parse_polygon_array(value: &Value) -> Vec<Vec<(f64, f64)>> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|ring| {
            ring.as_array()
                .into_iter()
                .flatten()
                .filter_map(|point| {
                    let point = point.as_array()?;
                    Some((point.first()?.as_f64()?, point.get(1)?.as_f64()?))
                })
                .collect()
        })
        .collect()
}

fn parse_node_data((index, value): (usize, &Value)) -> NodeData {
    match value {
        Value::Object(object) => NodeData {
            name: object
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("node {index}")),
            value: object.get("value").and_then(Value::as_f64).unwrap_or(1.0),
        },
        Value::String(name) => NodeData {
            name: name.clone(),
            value: 1.0,
        },
        _ => NodeData {
            name: format!("node {index}"),
            value: value.as_f64().unwrap_or(1.0),
        },
    }
}

fn parse_link_data(value: &Value) -> Option<LinkData> {
    let object = value.as_object()?;
    Some(LinkData {
        source: object.get("source")?.as_u64()? as usize,
        target: object.get("target")?.as_u64()? as usize,
        value: object.get("value").and_then(Value::as_f64).unwrap_or(1.0),
    })
}

fn parse_data_point(value: &Value) -> DataPoint {
    match value {
        Value::Number(number) => DataPoint::scalar(number.as_f64().unwrap_or_default()),
        Value::String(text) => DataPoint::scalar(text.as_str()),
        Value::Array(values) => DataPoint::values(values.iter().map(|value| {
            value
                .as_f64()
                .map(DataValue::Number)
                .unwrap_or_else(|| DataValue::String(label_from_value(value)))
        })),
        Value::Object(object) => {
            let mut point = object
                .get("value")
                .map(parse_data_point)
                .unwrap_or_else(|| DataPoint::scalar(0.0));
            point.name = object
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            point
        }
        _ => DataPoint::scalar(0.0),
    }
}

fn parse_color(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().map(|value| value as u32),
        Value::String(text) => parse_hex_color(text),
        _ => None,
    }
}

fn parse_hex_color(value: &str) -> Option<u32> {
    let value = value.strip_prefix('#')?;
    let color = u32::from_str_radix(value, 16).ok()?;
    Some(match value.len() {
        6 => 0xFF000000 | color,
        8 => color,
        _ => return None,
    })
}

fn parse_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Number(number) => number.as_f64().map(|value| value as f32),
        Value::String(text) => text.trim_end_matches('%').parse().ok(),
        _ => None,
    }
}

fn label_from_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_echarts_like_json() {
        let option = ChartOption::from_json_str(
            r##"{
                "title": {"text": "Sales"},
                "xAxis": {"type": "category", "data": ["Mon", "Tue"]},
                "yAxis": {"type": "value"},
                "color": ["#ff0000"],
                "series": [{"type": "bar", "name": "A", "data": [3, 5]}]
            }"##,
        )
        .unwrap();

        assert_eq!(option.title.unwrap().text, "Sales");
        assert_eq!(option.x_axis[0].data, vec!["Mon", "Tue"]);
        assert_eq!(option.visual_style.palette, vec![0xFFFF0000]);
        assert!(matches!(option.series[0], Series::Bar(_)));
    }

    #[test]
    fn typed_builder_creates_series() {
        let option = ChartOption::new()
            .title("Typed")
            .x_axis(Axis::category(["A", "B"]))
            .push_series(Series::line("L", [1.0, 2.0]));

        assert_eq!(option.title.unwrap().text, "Typed");
        assert_eq!(option.series.len(), 1);
    }

    #[test]
    fn hit_test_returns_data_event() {
        let option = ChartOption::new().push_series(Series::bar("B", [10.0]));
        let hit = hit_test(&option, 210.0, 60.0, 320.0, 180.0);

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.component_type, "bar");
        assert_eq!(hit.series_index, 0);
        assert_eq!(hit.data_index, 0);
    }

    #[test]
    fn custom_json_reports_diagnostic() {
        let option =
            ChartOption::from_json_str(r#"{"series":[{"type":"custom","data":[1]}]}"#).unwrap();

        assert_eq!(option.diagnostics.len(), 1);
    }

    #[test]
    fn map_bounds_cover_geojson() {
        let option = ChartOption::from_json_str(
            r#"{"series":[{"type":"map","geoJson":{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"name":"A","value":2},"geometry":{"type":"Polygon","coordinates":[[[0,0],[2,0],[2,1],[0,1],[0,0]]]}}]}}]}"#,
        )
        .unwrap();

        let Series::Map(map) = &option.series[0] else {
            panic!("expected map");
        };
        assert_eq!(map.features[0].name, "A");
        assert_eq!(map_bounds(&map.features), Some((0.0, 0.0, 2.0, 1.0)));
    }
}
