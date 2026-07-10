//! ECharts-compatible option and series data model.

use std::collections::BTreeMap;
use std::rc::Rc;

use ohos_drawing_binding::Canvas;
use serde_json::Value;

pub(crate) const DEFAULT_COLORS: [u32; 10] = [
    0xFF5470C6, 0xFF91CC75, 0xFFFAC858, 0xFFEE6666, 0xFF73C0DE, 0xFF3BA272, 0xFFFC8452, 0xFF9A60B4,
    0xFFEA7CCC, 0xFF2F4554,
];

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOption {
    pub title: Option<Title>,
    pub legend: Option<Legend>,
    pub grid: Vec<Grid>,
    pub x_axis: Vec<Axis>,
    pub y_axis: Vec<Axis>,
    pub radar: Vec<RadarCoordinate>,
    pub tooltip: Tooltip,
    pub dataset: Option<Dataset>,
    pub visual_map: Option<VisualMap>,
    pub data_zoom: Vec<DataZoom>,
    pub visual_style: VisualStyle,
    pub series: Vec<Series>,
    pub diagnostics: Vec<Diagnostic>,
    pub extra: BTreeMap<String, Value>,
}

impl Default for ChartOption {
    fn default() -> Self {
        Self {
            title: None,
            legend: None,
            grid: vec![Grid::default()],
            x_axis: vec![Axis::category(Vec::<String>::new())],
            y_axis: vec![Axis::value()],
            radar: Vec::new(),
            tooltip: Tooltip::default(),
            dataset: None,
            visual_map: None,
            data_zoom: Vec::new(),
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
        self.title = Some(Title {
            text: text.into(),
            ..Title::default()
        });
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = vec![axis];
        self
    }

    pub fn grid(mut self, grid: Grid) -> Self {
        self.grid = vec![grid];
        self
    }

    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend = Some(legend);
        self
    }

    pub fn data_zoom(mut self, data_zoom: DataZoom) -> Self {
        self.data_zoom.push(data_zoom);
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
        crate::parser::parse_option_str(input)
    }

    pub fn from_json_value(value: Value) -> Result<Self, ChartParseError> {
        crate::parser::parse_option_value(value)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Title {
    pub text: String,
    pub subtext: Option<String>,
    pub left: Value,
    pub top: Value,
    pub text_style: TextOptions,
    pub subtext_style: TextOptions,
}

impl Default for Title {
    fn default() -> Self {
        Self {
            text: String::new(),
            subtext: None,
            left: Value::Number(5.into()),
            top: Value::Number(5.into()),
            text_style: TextOptions {
                font_size: 18.0,
                font_weight: 700,
                ..TextOptions::default()
            },
            subtext_style: TextOptions {
                color: Some(0xFF6B7280),
                font_size: 12.0,
                ..TextOptions::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Legend {
    pub show: bool,
    pub orient: String,
    pub left: Value,
    pub top: Value,
    pub data: Vec<String>,
    pub item_width: f32,
    pub item_height: f32,
    pub text_style: TextOptions,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            show: true,
            orient: String::from("horizontal"),
            left: Value::String("center".into()),
            top: Value::String("top".into()),
            data: Vec::new(),
            item_width: 25.0,
            item_height: 14.0,
            text_style: TextOptions::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextOptions {
    pub color: Option<u32>,
    pub font_size: f32,
    pub font_weight: i32,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            color: None,
            font_size: 12.0,
            font_weight: 400,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub left: Length,
    pub right: Length,
    pub top: Length,
    pub bottom: Length,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub contain_label: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            left: Length::Percent(10.0),
            right: Length::Percent(10.0),
            top: Length::Px(60.0),
            bottom: Length::Px(70.0),
            width: None,
            height: None,
            contain_label: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
}

impl Length {
    pub fn resolve(self, total: f32) -> f32 {
        match self {
            Length::Px(value) => value,
            Length::Percent(value) => total * value / 100.0,
        }
    }
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Px(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tooltip {
    pub show: bool,
    pub trigger: String,
    pub formatter: Option<String>,
    pub background_color: u32,
    pub border_color: u32,
    pub text_color: u32,
    pub padding: f32,
    pub axis_pointer: AxisPointer,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            show: true,
            trigger: String::from("item"),
            formatter: None,
            background_color: 0xE6333333,
            border_color: 0x00000000,
            text_color: 0xFFFFFFFF,
            padding: 8.0,
            axis_pointer: AxisPointer::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisPointer {
    pub show: bool,
    pub kind: String,
    pub snap: bool,
    pub line_style: LineStyle,
    pub label: LabelStyle,
}

impl Default for AxisPointer {
    fn default() -> Self {
        Self {
            show: true,
            kind: String::from("line"),
            snap: false,
            line_style: LineStyle {
                color: Some(0xFF777777),
                width: 1.0,
                opacity: 1.0,
            },
            label: LabelStyle {
                show: true,
                color: Some(0xFFFFFFFF),
                font_size: 10.0,
                ..LabelStyle::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    pub source: Vec<Vec<DataValue>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualMap {
    pub show: bool,
    pub min: f64,
    pub max: f64,
    pub colors: Vec<u32>,
}

/// ECharts `dataZoom` component shared by slider and inside interactions.
#[derive(Debug, Clone, PartialEq)]
pub struct DataZoom {
    pub show: bool,
    pub kind: String,
    pub start: f64,
    pub end: f64,
    pub start_value: Option<DataValue>,
    pub end_value: Option<DataValue>,
    pub x_axis_index: Vec<usize>,
    pub y_axis_index: Vec<usize>,
    pub filter_mode: String,
    pub orient: String,
    pub zoom_lock: bool,
    pub height: f32,
    pub extra: BTreeMap<String, Value>,
}

impl Default for DataZoom {
    fn default() -> Self {
        Self {
            show: true,
            kind: String::from("slider"),
            start: 0.0,
            end: 100.0,
            start_value: None,
            end_value: None,
            x_axis_index: vec![0],
            y_axis_index: Vec::new(),
            filter_mode: String::from("filter"),
            orient: String::from("horizontal"),
            zoom_lock: false,
            height: 20.0,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadarIndicator {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadarCoordinate {
    pub indicators: Vec<RadarIndicator>,
    pub center: [Value; 2],
    pub radius: Value,
    pub start_angle: f32,
    pub split_number: usize,
    pub shape: String,
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
    /// Category gap semantics used by ECharts (`true` places ticks at the
    /// center of bands, `false` places them on band boundaries).
    pub boundary_gap: bool,
    pub inverse: bool,
    pub split_number: usize,
    pub show: bool,
    pub split_line: bool,
    pub axis_label: bool,
    pub grid_index: usize,
}

impl Axis {
    pub fn category(data: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            axis_type: AxisType::Category,
            name: None,
            data: data.into_iter().map(Into::into).collect(),
            min: None,
            max: None,
            boundary_gap: true,
            inverse: false,
            split_number: 5,
            show: true,
            split_line: false,
            axis_label: true,
            grid_index: 0,
        }
    }

    pub fn value() -> Self {
        Self {
            axis_type: AxisType::Value,
            name: None,
            data: Vec::new(),
            min: None,
            max: None,
            boundary_gap: false,
            inverse: false,
            split_number: 5,
            show: true,
            split_line: true,
            axis_label: true,
            grid_index: 0,
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
    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            DataValue::Number(value) => Some(*value),
            DataValue::String(value) => value.parse().ok().or_else(|| parse_echarts_time(value)),
        }
    }
}

fn parse_echarts_time(value: &str) -> Option<f64> {
    let value = value.trim();
    let (date, time) = value
        .split_once('T')
        .or_else(|| value.split_once(' '))
        .unwrap_or((value, "00:00:00"));
    let mut date = date.split('-');
    let year = date.next()?.parse::<i32>().ok()?;
    let month = date.next()?.parse::<u32>().ok()?;
    let day = date.next()?.parse::<u32>().ok()?;

    let (time, timezone_minutes) = if let Some(time) = time.strip_suffix('Z') {
        (time, 0i64)
    } else if let Some(index) = time
        .char_indices()
        .skip(1)
        .find_map(|(index, value)| matches!(value, '+' | '-').then_some(index))
    {
        let (clock, timezone) = time.split_at(index);
        let sign = if timezone.starts_with('-') { -1 } else { 1 };
        let mut timezone = timezone[1..].split(':');
        let hours = timezone.next()?.parse::<i64>().ok()?;
        let minutes = timezone.next().unwrap_or("0").parse::<i64>().ok()?;
        (clock, sign * (hours * 60 + minutes))
    } else {
        (time, 0)
    };
    let mut clock = time.split(':');
    let hour = clock.next().unwrap_or("0").parse::<i64>().ok()?;
    let minute = clock.next().unwrap_or("0").parse::<i64>().ok()?;
    let second = clock.next().unwrap_or("0").parse::<f64>().ok()?;
    let days = days_from_civil(year, month, day)?;
    Some(
        days as f64 * 86_400_000.0
            + hour as f64 * 3_600_000.0
            + minute as f64 * 60_000.0
            + second * 1_000.0
            - timezone_minutes as f64 * 60_000.0,
    )
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month as i32 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day as i32 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some((era * 146_097 + day_of_era - 719_468) as i64)
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
    pub item_style: ItemStyle,
    pub label: LabelStyle,
    pub extra: BTreeMap<String, Value>,
}

impl DataPoint {
    pub fn scalar(value: impl Into<DataValue>) -> Self {
        Self {
            name: None,
            values: vec![value.into()],
            style: None,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: BTreeMap::new(),
        }
    }

    pub fn named(name: impl Into<String>, value: impl Into<DataValue>) -> Self {
        Self {
            name: Some(name.into()),
            values: vec![value.into()],
            style: None,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: BTreeMap::new(),
        }
    }

    pub fn values(values: impl IntoIterator<Item = impl Into<DataValue>>) -> Self {
        Self {
            name: None,
            values: values.into_iter().map(Into::into).collect(),
            style: None,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: BTreeMap::new(),
        }
    }

    pub(crate) fn number(&self, index: usize) -> f64 {
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
    pub options: SeriesOptions,
}

impl BasicSeries {
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            name: Some(name.into()),
            data: values.into_iter().map(DataPoint::scalar).collect(),
            style: None,
            options: SeriesOptions::default(),
        }
    }

    pub fn data(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self {
            name: Some(name.into()),
            data: data.into_iter().collect(),
            style: None,
            options: SeriesOptions::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStyle {
    pub color: Option<u32>,
    /// ECharts `color0`, used by falling candlesticks.
    pub color0: Option<u32>,
    pub border_color: Option<u32>,
    /// ECharts `borderColor0`, used by falling candlesticks.
    pub border_color0: Option<u32>,
    pub border_width: f32,
    pub opacity: f32,
}

impl Default for ItemStyle {
    fn default() -> Self {
        Self {
            color: None,
            color0: None,
            border_color: None,
            border_color0: None,
            border_width: 0.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    pub color: Option<u32>,
    pub width: f32,
    pub opacity: f32,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            color: None,
            width: 2.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyle {
    pub show: bool,
    pub color: Option<u32>,
    pub font_size: f32,
    pub font_weight: i32,
    pub position: String,
    pub formatter: Option<String>,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            show: false,
            color: None,
            font_size: 12.0,
            font_weight: 400,
            position: String::from("top"),
            formatter: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesOptions {
    pub item_style: ItemStyle,
    pub line_style: LineStyle,
    pub area_style: Option<ItemStyle>,
    pub label: LabelStyle,
    pub smooth: f32,
    pub show_symbol: bool,
    pub symbol_size: f32,
    pub bar_width: Option<f32>,
    pub stack: Option<String>,
    pub selected_mode: Option<String>,
    pub extra: BTreeMap<String, Value>,
}

impl Default for SeriesOptions {
    fn default() -> Self {
        Self {
            item_style: ItemStyle::default(),
            line_style: LineStyle::default(),
            area_style: None,
            label: LabelStyle::default(),
            smooth: 0.0,
            show_symbol: true,
            symbol_size: 7.0,
            bar_width: None,
            stack: None,
            selected_mode: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeData {
    pub name: String,
    pub value: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub category: Option<usize>,
    pub symbol_size: Option<f32>,
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
    pub options: SeriesOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SankeySeries {
    pub name: Option<String>,
    pub nodes: Vec<NodeData>,
    pub links: Vec<LinkData>,
    pub options: SeriesOptions,
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
    pub options: SeriesOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub name: Option<String>,
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinesSeries {
    pub name: Option<String>,
    pub data: Vec<LineSegment>,
    pub options: SeriesOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SunburstNode {
    pub name: String,
    pub value: f64,
    pub children: Vec<SunburstNode>,
    pub item_style: ItemStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SunburstSeries {
    pub name: Option<String>,
    pub data: Vec<SunburstNode>,
    pub options: SeriesOptions,
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
        self.name == other.name
            && self.data == other.data
            && Rc::ptr_eq(&self.renderer, &other.renderer)
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
    EffectScatter(BasicSeries),
    Radar(BasicSeries),
    Gauge(BasicSeries),
    Funnel(BasicSeries),
    Heatmap(BasicSeries),
    Candlestick(BasicSeries),
    Boxplot(BasicSeries),
    PictorialBar(BasicSeries),
    Parallel(BasicSeries),
    ThemeRiver(BasicSeries),
    Tree(GraphSeries),
    Treemap(BasicSeries),
    Graph(GraphSeries),
    Sankey(SankeySeries),
    Map(MapSeries),
    Lines(LinesSeries),
    Sunburst(SunburstSeries),
    Custom(CustomSeries),
}

impl std::fmt::Debug for Series {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Series::Line(value) => f.debug_tuple("Line").field(value).finish(),
            Series::Bar(value) => f.debug_tuple("Bar").field(value).finish(),
            Series::Pie(value) => f.debug_tuple("Pie").field(value).finish(),
            Series::Scatter(value) => f.debug_tuple("Scatter").field(value).finish(),
            Series::EffectScatter(value) => f.debug_tuple("EffectScatter").field(value).finish(),
            Series::Radar(value) => f.debug_tuple("Radar").field(value).finish(),
            Series::Gauge(value) => f.debug_tuple("Gauge").field(value).finish(),
            Series::Funnel(value) => f.debug_tuple("Funnel").field(value).finish(),
            Series::Heatmap(value) => f.debug_tuple("Heatmap").field(value).finish(),
            Series::Candlestick(value) => f.debug_tuple("Candlestick").field(value).finish(),
            Series::Boxplot(value) => f.debug_tuple("Boxplot").field(value).finish(),
            Series::PictorialBar(value) => f.debug_tuple("PictorialBar").field(value).finish(),
            Series::Parallel(value) => f.debug_tuple("Parallel").field(value).finish(),
            Series::ThemeRiver(value) => f.debug_tuple("ThemeRiver").field(value).finish(),
            Series::Tree(value) => f.debug_tuple("Tree").field(value).finish(),
            Series::Treemap(value) => f.debug_tuple("Treemap").field(value).finish(),
            Series::Graph(value) => f.debug_tuple("Graph").field(value).finish(),
            Series::Sankey(value) => f.debug_tuple("Sankey").field(value).finish(),
            Series::Map(value) => f.debug_tuple("Map").field(value).finish(),
            Series::Lines(value) => f.debug_tuple("Lines").field(value).finish(),
            Series::Sunburst(value) => f.debug_tuple("Sunburst").field(value).finish(),
            Series::Custom(value) => f.debug_tuple("Custom").field(value).finish(),
        }
    }
}

impl PartialEq for Series {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Line(left), Self::Line(right))
            | (Self::Bar(left), Self::Bar(right))
            | (Self::Pie(left), Self::Pie(right))
            | (Self::Scatter(left), Self::Scatter(right))
            | (Self::EffectScatter(left), Self::EffectScatter(right))
            | (Self::Radar(left), Self::Radar(right))
            | (Self::Gauge(left), Self::Gauge(right))
            | (Self::Funnel(left), Self::Funnel(right))
            | (Self::Heatmap(left), Self::Heatmap(right))
            | (Self::Candlestick(left), Self::Candlestick(right))
            | (Self::Boxplot(left), Self::Boxplot(right))
            | (Self::PictorialBar(left), Self::PictorialBar(right))
            | (Self::Parallel(left), Self::Parallel(right))
            | (Self::ThemeRiver(left), Self::ThemeRiver(right))
            | (Self::Treemap(left), Self::Treemap(right)) => left == right,
            (Self::Tree(left), Self::Tree(right)) | (Self::Graph(left), Self::Graph(right)) => {
                left == right
            }
            (Self::Sankey(left), Self::Sankey(right)) => left == right,
            (Self::Map(left), Self::Map(right)) => left == right,
            (Self::Lines(left), Self::Lines(right)) => left == right,
            (Self::Sunburst(left), Self::Sunburst(right)) => left == right,
            (Self::Custom(left), Self::Custom(right)) => left == right,
            _ => false,
        }
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
        let mut series = BasicSeries::data(name, data);
        series.options.label.show = true;
        series.options.label.formatter = Some(String::from("{b}"));
        Self::Pie(series)
    }

    pub fn scatter(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Scatter(BasicSeries::data(name, data))
    }

    pub fn effect_scatter(
        name: impl Into<String>,
        data: impl IntoIterator<Item = DataPoint>,
    ) -> Self {
        Self::EffectScatter(BasicSeries::data(name, data))
    }

    pub fn radar(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self::Radar(BasicSeries::data(name, [DataPoint::values(values)]))
    }

    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self::Gauge(BasicSeries::new(name, [value]))
    }

    pub fn funnel(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        let mut series = BasicSeries::data(name, data);
        series.options.label.show = true;
        series.options.label.formatter = Some(String::from("{b}"));
        Self::Funnel(series)
    }

    pub fn heatmap(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Heatmap(BasicSeries::data(name, data))
    }

    pub fn candlestick(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Candlestick(BasicSeries::data(name, data))
    }

    pub fn boxplot(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Boxplot(BasicSeries::data(name, data))
    }

    pub fn pictorial_bar(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self::PictorialBar(BasicSeries::new(name, values))
    }

    pub fn parallel(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Parallel(BasicSeries::data(name, data))
    }

    pub fn theme_river(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::ThemeRiver(BasicSeries::data(name, data))
    }

    pub fn tree(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        let mut options = SeriesOptions::default();
        options.label.show = true;
        options.label.formatter = Some(String::from("{b}"));
        Self::Tree(GraphSeries {
            name: Some(name.into()),
            nodes,
            links,
            options,
        })
    }

    pub fn treemap(name: impl Into<String>, data: impl IntoIterator<Item = DataPoint>) -> Self {
        let mut series = BasicSeries::data(name, data);
        series.options.label.show = true;
        series.options.label.formatter = Some(String::from("{b}"));
        Self::Treemap(series)
    }

    pub fn graph(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        Self::Graph(GraphSeries {
            name: Some(name.into()),
            nodes,
            links,
            options: SeriesOptions::default(),
        })
    }

    pub fn sankey(name: impl Into<String>, nodes: Vec<NodeData>, links: Vec<LinkData>) -> Self {
        Self::Sankey(SankeySeries {
            name: Some(name.into()),
            nodes,
            links,
            options: SeriesOptions::default(),
        })
    }

    pub fn map(name: impl Into<String>, features: Vec<MapFeature>) -> Self {
        Self::Map(MapSeries {
            name: Some(name.into()),
            features,
            options: SeriesOptions::default(),
        })
    }

    pub fn lines(name: impl Into<String>, data: Vec<LineSegment>) -> Self {
        Self::Lines(LinesSeries {
            name: Some(name.into()),
            data,
            options: SeriesOptions::default(),
        })
    }

    pub fn sunburst(name: impl Into<String>, data: Vec<SunburstNode>) -> Self {
        let mut options = SeriesOptions::default();
        options.label.show = true;
        options.label.formatter = Some(String::from("{b}"));
        Self::Sunburst(SunburstSeries {
            name: Some(name.into()),
            data,
            options,
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

    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Series::Line(v)
            | Series::Bar(v)
            | Series::Pie(v)
            | Series::Scatter(v)
            | Series::EffectScatter(v)
            | Series::Radar(v)
            | Series::Gauge(v)
            | Series::Funnel(v)
            | Series::Heatmap(v)
            | Series::Candlestick(v)
            | Series::Boxplot(v)
            | Series::PictorialBar(v)
            | Series::Parallel(v)
            | Series::ThemeRiver(v)
            | Series::Treemap(v) => v.name.as_deref(),
            Series::Tree(v) | Series::Graph(v) => v.name.as_deref(),
            Series::Sankey(v) => v.name.as_deref(),
            Series::Map(v) => v.name.as_deref(),
            Series::Lines(v) => v.name.as_deref(),
            Series::Sunburst(v) => v.name.as_deref(),
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
