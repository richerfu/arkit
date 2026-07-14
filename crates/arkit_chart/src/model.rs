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
    /// All ECharts dataset components. `dataset` remains the index-0 compatibility view.
    pub datasets: Vec<Dataset>,
    pub visual_map: Option<VisualMap>,
    /// All ECharts visualMap components. `visual_map` remains the index-0 compatibility view.
    pub visual_maps: Vec<VisualMap>,
    pub data_zoom: Vec<DataZoom>,
    pub timeline: Option<Timeline>,
    /// Fully merged snapshots from ECharts `baseOption` + `options`.
    pub timeline_options: Vec<ChartOption>,
    pub brush: Option<BrushOptions>,
    /// Raw responsive option rules from ECharts `baseOption + media`.
    pub media: Option<MediaOptions>,
    /// ECharts global enter/update/state animation policy.
    pub animation: AnimationOptions,
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
            datasets: Vec::new(),
            visual_map: None,
            visual_maps: Vec::new(),
            data_zoom: Vec::new(),
            timeline: None,
            timeline_options: Vec::new(),
            brush: None,
            media: None,
            animation: AnimationOptions::default(),
            visual_style: VisualStyle::default(),
            series: Vec::new(),
            diagnostics: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationTiming {
    pub duration: u64,
    pub easing: String,
    pub delay: u64,
}

impl AnimationTiming {
    fn new(duration: u64, easing: &str) -> Self {
        Self {
            duration,
            easing: easing.to_string(),
            delay: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationOptions {
    pub enabled: bool,
    pub threshold: usize,
    pub initial: AnimationTiming,
    pub update: AnimationTiming,
    pub state: AnimationTiming,
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 2_000,
            initial: AnimationTiming::new(1_000, "cubicInOut"),
            update: AnimationTiming::new(500, "cubicInOut"),
            state: AnimationTiming::new(300, "cubicOut"),
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

    pub(crate) fn visual_map_for_series(&self, series_index: usize) -> Option<&VisualMap> {
        if self.visual_maps.is_empty() {
            return self.visual_map.as_ref();
        }
        self.visual_maps
            .iter()
            .find(|visual_map| {
                visual_map.series_indices.is_empty()
                    || visual_map.series_indices.contains(&series_index)
            })
            .or_else(|| self.visual_maps.first())
    }

    pub(crate) fn apply_timeline_index(&mut self, index: usize) -> bool {
        if self.timeline_options.is_empty() {
            return false;
        }
        let index = index.min(self.timeline_options.len() - 1);
        let Some(mut frame) = self.timeline_options.get(index).cloned() else {
            return false;
        };
        let mut timeline = self.timeline.clone().unwrap_or_default();
        timeline.current_index = index;
        let frames = self.timeline_options.clone();
        let media = self.media.clone();
        frame.timeline = Some(timeline);
        frame.timeline_options = frames;
        frame.media = media;
        *self = frame;
        true
    }

    pub(crate) fn advance_timeline(&mut self) -> bool {
        let Some(timeline) = self.timeline.as_ref() else {
            return false;
        };
        if self.timeline_options.len() < 2 {
            return false;
        }
        let current = timeline.current_index.min(self.timeline_options.len() - 1);
        let reverse = timeline.rewind;
        let next = if reverse {
            current.checked_sub(1).or_else(|| {
                timeline
                    .loop_play
                    .then_some(self.timeline_options.len() - 1)
            })
        } else if current + 1 < self.timeline_options.len() {
            Some(current + 1)
        } else {
            timeline.loop_play.then_some(0)
        };
        let Some(next) = next else {
            if let Some(timeline) = self.timeline.as_mut() {
                timeline.auto_play = false;
            }
            return false;
        };
        self.apply_timeline_index(next)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaOptions {
    pub base_option: Value,
    pub timeline_options: Vec<Value>,
    pub rules: Vec<MediaRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaRule {
    pub query: Option<MediaQuery>,
    pub option: Value,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaQuery {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub min_aspect_ratio: Option<f32>,
    pub max_aspect_ratio: Option<f32>,
}

impl MediaQuery {
    pub(crate) fn matches(&self, width: f32, height: f32) -> bool {
        let aspect_ratio = width / height.max(1.0);
        self.min_width.is_none_or(|value| width >= value)
            && self.max_width.is_none_or(|value| width <= value)
            && self.min_height.is_none_or(|value| height >= value)
            && self.max_height.is_none_or(|value| height <= value)
            && self
                .min_aspect_ratio
                .is_none_or(|value| aspect_ratio >= value)
            && self
                .max_aspect_ratio
                .is_none_or(|value| aspect_ratio <= value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub show: bool,
    pub current_index: usize,
    pub auto_play: bool,
    pub rewind: bool,
    pub loop_play: bool,
    pub play_interval: u64,
    pub orient: String,
    pub inverse: bool,
    pub left: Value,
    pub right: Value,
    pub top: Value,
    pub bottom: Value,
    pub data: Vec<String>,
    pub label: LabelStyle,
    pub line_style: LineStyle,
    pub item_style: ItemStyle,
    pub checkpoint_style: ItemStyle,
    pub control_style: ItemStyle,
    pub extra: BTreeMap<String, Value>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            show: true,
            current_index: 0,
            auto_play: false,
            rewind: false,
            loop_play: true,
            play_interval: 2_000,
            orient: String::from("horizontal"),
            inverse: false,
            left: Value::Number(40.into()),
            right: Value::Number(40.into()),
            top: Value::Null,
            bottom: Value::Number(8.into()),
            data: Vec::new(),
            label: LabelStyle {
                show: true,
                position: String::from("bottom"),
                color: Some(0xFF475569),
                font_size: 10.0,
                ..LabelStyle::default()
            },
            line_style: LineStyle {
                color: Some(0xFFCBD5E1),
                width: 2.0,
                ..LineStyle::default()
            },
            item_style: ItemStyle {
                color: Some(0xFFFFFFFF),
                border_color: Some(0xFF94A3B8),
                border_width: 1.5,
                ..ItemStyle::default()
            },
            checkpoint_style: ItemStyle {
                color: Some(0xFF5470C6),
                border_color: Some(0xFFFFFFFF),
                border_width: 2.0,
                ..ItemStyle::default()
            },
            control_style: ItemStyle {
                color: Some(0xFF475569),
                border_color: Some(0xFFCBD5E1),
                border_width: 1.0,
                ..ItemStyle::default()
            },
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrushOptions {
    pub active: bool,
    pub brush_type: String,
    pub brush_mode: String,
    pub transformable: bool,
    pub remove_on_click: bool,
    pub brush_style: ItemStyle,
    pub in_brush_color: Option<u32>,
    pub out_of_brush_opacity: f32,
    /// Runtime pixel-space selections. They intentionally survive controlled option refreshes.
    pub areas: Vec<BrushArea>,
    pub extra: BTreeMap<String, Value>,
}

impl Default for BrushOptions {
    fn default() -> Self {
        Self {
            active: false,
            brush_type: String::from("rect"),
            brush_mode: String::from("single"),
            transformable: true,
            remove_on_click: true,
            brush_style: ItemStyle {
                color: Some(0x335470C6),
                border_color: Some(0xFF5470C6),
                border_width: 1.0,
                ..ItemStyle::default()
            },
            in_brush_color: None,
            out_of_brush_opacity: 0.3,
            areas: Vec::new(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushArea {
    pub start: [f32; 2],
    pub end: [f32; 2],
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
    pub align: String,
    pub left: Value,
    pub top: Value,
    pub data: Vec<String>,
    pub item_width: f32,
    pub item_height: f32,
    pub item_gap: f32,
    pub icon: String,
    pub inactive_color: u32,
    pub formatter: Option<String>,
    /// ECharts `selectedMode`: `true`, `false`, `single`, or `multiple`.
    pub selected_mode: String,
    /// Initial visibility by legend entry name. Missing names default to selected.
    pub selected: BTreeMap<String, bool>,
    /// Per-entry icon overrides from object-form `legend.data`.
    pub data_icons: BTreeMap<String, String>,
    pub text_style: TextOptions,
    pub extra: BTreeMap<String, Value>,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            show: true,
            orient: String::from("horizontal"),
            align: String::from("auto"),
            left: Value::String("center".into()),
            top: Value::String("top".into()),
            data: Vec::new(),
            item_width: 25.0,
            item_height: 14.0,
            item_gap: 8.0,
            icon: String::from("roundRect"),
            inactive_color: 0xFFB8B8B8,
            formatter: None,
            selected_mode: String::from("true"),
            selected: BTreeMap::new(),
            data_icons: BTreeMap::new(),
            text_style: TextOptions::default(),
            extra: BTreeMap::new(),
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
                kind: String::from("solid"),
                specified: std::collections::BTreeSet::new(),
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
    pub dimensions: Vec<String>,
    pub source_header: bool,
    pub id: Option<String>,
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualMap {
    pub show: bool,
    pub min: f64,
    pub max: f64,
    pub colors: Vec<u32>,
    /// Data dimension used by visual encodings. The last dimension is used
    /// when omitted, matching the common ECharts scatter/heatmap convention.
    pub dimension: Option<usize>,
    /// ECharts `inRange.symbolSize` minimum and maximum.
    pub symbol_size_range: Option<[f32; 2]>,
    pub pieces: Vec<VisualPiece>,
    pub series_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualPiece {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub value: Option<f64>,
    pub color: Option<u32>,
    pub symbol_size: Option<f32>,
    pub label: Option<String>,
}

impl VisualPiece {
    pub(crate) fn contains(&self, candidate: f64) -> bool {
        if let Some(value) = self.value {
            return (candidate - value).abs() < 1e-12;
        }
        self.min.is_none_or(|min| candidate >= min) && self.max.is_none_or(|max| candidate <= max)
    }
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
    pub extra: BTreeMap<String, Value>,
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
    /// ECharts axis side (`top`/`bottom` for x, `left`/`right` for y).
    pub position: String,
    /// Distance in vp from the grid edge. Useful when multiple axes share a side.
    pub offset: f32,
    pub axis_line: AxisLine,
    pub axis_tick: AxisTick,
    pub split_line: bool,
    pub split_line_style: LineStyle,
    pub axis_label: bool,
    pub axis_label_style: AxisLabelStyle,
    pub grid_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisLine {
    pub show: bool,
    pub on_zero: bool,
    pub line_style: LineStyle,
}

impl Default for AxisLine {
    fn default() -> Self {
        Self {
            show: true,
            on_zero: true,
            line_style: LineStyle {
                width: 1.0,
                ..LineStyle::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisTick {
    pub show: bool,
    pub align_with_label: bool,
    pub inside: bool,
    pub length: f32,
    pub line_style: LineStyle,
}

impl Default for AxisTick {
    fn default() -> Self {
        Self {
            show: true,
            align_with_label: false,
            inside: false,
            length: 5.0,
            line_style: LineStyle {
                width: 1.0,
                ..LineStyle::default()
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisLabelStyle {
    pub color: Option<u32>,
    pub font_size: f32,
    pub font_weight: i32,
    pub rotate: f32,
    pub margin: f32,
    /// A numeric ECharts interval. `None` retains automatic sampling.
    pub interval: Option<usize>,
    pub formatter: Option<String>,
}

impl Default for AxisLabelStyle {
    fn default() -> Self {
        Self {
            color: None,
            font_size: 12.0,
            font_weight: 400,
            rotate: 0.0,
            margin: 8.0,
            interval: None,
            formatter: None,
        }
    }
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
            position: String::from("bottom"),
            offset: 0.0,
            axis_line: AxisLine::default(),
            axis_tick: AxisTick::default(),
            split_line: false,
            split_line_style: LineStyle {
                color: Some(0xFFE0E6F1),
                width: 1.0,
                opacity: 1.0,
                kind: String::from("solid"),
                specified: std::collections::BTreeSet::new(),
            },
            axis_label: true,
            axis_label_style: AxisLabelStyle::default(),
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
            position: String::from("left"),
            offset: 0.0,
            axis_line: AxisLine::default(),
            axis_tick: AxisTick::default(),
            split_line: true,
            split_line_style: LineStyle {
                color: Some(0xFFE0E6F1),
                width: 1.0,
                opacity: 1.0,
                kind: String::from("solid"),
                specified: std::collections::BTreeSet::new(),
            },
            axis_label: true,
            axis_label_style: AxisLabelStyle::default(),
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
    /// An ECharts missing value (`null`, `"-"`, or a non-finite typed value).
    Null,
}

impl DataValue {
    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            DataValue::Number(value) => value.is_finite().then_some(*value),
            DataValue::String(value) if value == "-" => None,
            DataValue::String(value) => value.parse().ok().or_else(|| parse_echarts_time(value)),
            DataValue::Null => None,
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
        if value == "-" {
            Self::Null
        } else {
            Self::String(value.to_string())
        }
    }
}

impl From<Option<f64>> for DataValue {
    fn from(value: Option<f64>) -> Self {
        value.map(Self::Number).unwrap_or(Self::Null)
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
    pub fn missing() -> Self {
        Self::scalar(DataValue::Null)
    }

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
        self.number_opt(index).unwrap_or_default()
    }

    pub(crate) fn number_opt(&self, index: usize) -> Option<f64> {
        self.values.get(index).and_then(DataValue::as_f64)
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
    /// Corner radii in ECharts order: top-left, top-right, bottom-right,
    /// bottom-left.
    pub border_radius: [f32; 4],
    pub opacity: f32,
    /// Fields explicitly present in parsed JSON, used for state/data-item merges.
    pub specified: std::collections::BTreeSet<String>,
}

impl Default for ItemStyle {
    fn default() -> Self {
        Self {
            color: None,
            color0: None,
            border_color: None,
            border_color0: None,
            border_width: 0.0,
            border_radius: [0.0; 4],
            opacity: 1.0,
            specified: std::collections::BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    pub color: Option<u32>,
    pub width: f32,
    pub opacity: f32,
    /// ECharts stroke type: `solid`, `dashed`, or `dotted`.
    pub kind: String,
    /// Fields explicitly present in parsed JSON, used for state merges.
    pub specified: std::collections::BTreeSet<String>,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            color: None,
            width: 2.0,
            opacity: 1.0,
            kind: String::from("solid"),
            specified: std::collections::BTreeSet::new(),
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
    pub distance: f32,
    pub rotate: f32,
    pub offset: [f32; 2],
    pub formatter: Option<String>,
    /// Fields explicitly present in parsed JSON, used for state/data-item merges.
    pub specified: std::collections::BTreeSet<String>,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            show: false,
            color: None,
            font_size: 12.0,
            font_weight: 400,
            position: String::from("top"),
            distance: 5.0,
            rotate: 0.0,
            offset: [0.0, 0.0],
            formatter: None,
            specified: std::collections::BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesState {
    pub item_style: ItemStyle,
    pub line_style: LineStyle,
    pub label: LabelStyle,
    pub focus: Option<String>,
    pub blur_scope: String,
    pub scale: Option<f32>,
}

impl Default for SeriesState {
    fn default() -> Self {
        Self {
            item_style: ItemStyle::default(),
            line_style: LineStyle::default(),
            label: LabelStyle::default(),
            focus: None,
            blur_scope: String::from("coordinateSystem"),
            scale: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLayoutCallbackParams {
    pub series_index: usize,
    pub data_index: Option<usize>,
    pub text: String,
    pub align: String,
    pub vertical_align: String,
    pub rect: [f32; 4],
    pub label_rect: [f32; 4],
    pub label_line_points: Vec<[f32; 2]>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LabelLayoutCallbackResult {
    pub hide_overlap: Option<bool>,
    pub move_overlap: Option<String>,
    pub draggable: Option<bool>,
    pub x: Option<Value>,
    pub y: Option<Value>,
    pub dx: Option<f32>,
    pub dy: Option<f32>,
    pub rotate: Option<f32>,
    pub align: Option<String>,
    pub vertical_align: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub font_size: Option<f32>,
    pub label_line_points: Option<Vec<[f32; 2]>>,
}

pub type LabelLayoutCallback = Rc<dyn Fn(LabelLayoutCallbackParams) -> LabelLayoutCallbackResult>;

#[derive(Clone, Default)]
pub struct LabelLayoutOptions {
    pub hide_overlap: bool,
    pub move_overlap: Option<String>,
    pub draggable: bool,
    pub x: Option<Value>,
    pub y: Option<Value>,
    pub dx: Option<f32>,
    pub dy: Option<f32>,
    pub rotate: Option<f32>,
    pub align: Option<String>,
    pub vertical_align: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub font_size: Option<f32>,
    pub label_line_points: Vec<[f32; 2]>,
    pub callback: Option<LabelLayoutCallback>,
    /// Runtime offsets populated by native label dragging. JSON options start
    /// empty and controlled option updates preserve matching entries.
    pub drag_offsets: BTreeMap<usize, [f32; 2]>,
}

impl LabelLayoutOptions {
    pub fn with_callback(
        mut self,
        callback: impl Fn(LabelLayoutCallbackParams) -> LabelLayoutCallbackResult + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl std::fmt::Debug for LabelLayoutOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LabelLayoutOptions")
            .field("hide_overlap", &self.hide_overlap)
            .field("move_overlap", &self.move_overlap)
            .field("draggable", &self.draggable)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("dx", &self.dx)
            .field("dy", &self.dy)
            .field("rotate", &self.rotate)
            .field("align", &self.align)
            .field("vertical_align", &self.vertical_align)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("font_size", &self.font_size)
            .field("label_line_points", &self.label_line_points)
            .field("has_callback", &self.callback.is_some())
            .field("drag_offsets", &self.drag_offsets)
            .finish()
    }
}

impl PartialEq for LabelLayoutOptions {
    fn eq(&self, other: &Self) -> bool {
        self.hide_overlap == other.hide_overlap
            && self.move_overlap == other.move_overlap
            && self.draggable == other.draggable
            && self.x == other.x
            && self.y == other.y
            && self.dx == other.dx
            && self.dy == other.dy
            && self.rotate == other.rotate
            && self.align == other.align
            && self.vertical_align == other.vertical_align
            && self.width == other.width
            && self.height == other.height
            && self.font_size == other.font_size
            && self.label_line_points == other.label_line_points
            && match (&self.callback, &other.callback) {
                (Some(left), Some(right)) => Rc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
            && self.drag_offsets == other.drag_offsets
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeriesOptions {
    pub item_style: ItemStyle,
    pub line_style: LineStyle,
    pub area_style: Option<ItemStyle>,
    pub label: LabelStyle,
    pub smooth: f32,
    pub smooth_monotone: Option<String>,
    pub connect_nulls: bool,
    pub show_symbol: bool,
    pub show_all_symbol: Option<bool>,
    pub symbol: String,
    pub symbol_size: f32,
    /// Width/height form of ECharts `symbolSize`; scalar callers continue to
    /// use `symbol_size`.
    pub symbol_size_dimensions: Option<[f32; 2]>,
    pub symbol_rotate: f32,
    pub symbol_offset: [Value; 2],
    pub step: Option<String>,
    pub clip: bool,
    pub sampling: String,
    pub end_label: LabelStyle,
    pub area_origin: Value,
    pub bar_width: Option<Length>,
    pub bar_max_width: Option<Length>,
    pub bar_min_width: Option<Length>,
    /// Minimum rendered extent along the value axis.
    pub bar_min_height: f32,
    /// Gap between bar groups, as a ratio of the automatic bar width.
    pub bar_gap: f32,
    /// Gap reserved between adjacent categories.
    pub bar_category_gap: Length,
    pub show_background: bool,
    pub background_style: ItemStyle,
    pub stack: Option<String>,
    pub selected_mode: Option<String>,
    pub emphasis: SeriesState,
    pub blur: SeriesState,
    pub select: SeriesState,
    pub label_layout: LabelLayoutOptions,
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
            smooth_monotone: None,
            connect_nulls: false,
            show_symbol: true,
            show_all_symbol: None,
            symbol: String::from("circle"),
            symbol_size: 7.0,
            symbol_size_dimensions: None,
            symbol_rotate: 0.0,
            symbol_offset: [Value::from(0), Value::from(0)],
            step: None,
            clip: true,
            sampling: String::from("none"),
            end_label: LabelStyle::default(),
            area_origin: Value::String(String::from("auto")),
            bar_width: None,
            bar_max_width: None,
            bar_min_width: None,
            bar_min_height: 0.0,
            bar_gap: 0.3,
            bar_category_gap: Length::Percent(20.0),
            show_background: false,
            background_style: ItemStyle {
                color: Some(0x33B4B4B4),
                ..ItemStyle::default()
            },
            stack: None,
            selected_mode: None,
            emphasis: SeriesState::default(),
            blur: SeriesState::default(),
            select: SeriesState::default(),
            label_layout: LabelLayoutOptions::default(),
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
    pub symbol_size_dimensions: Option<[f32; 2]>,
    pub symbol: Option<String>,
    pub symbol_rotate: f32,
    pub item_style: ItemStyle,
    pub label: LabelStyle,
    pub extra: BTreeMap<String, Value>,
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
pub struct MapPolygon {
    pub exterior: Vec<(f64, f64)>,
    pub holes: Vec<Vec<(f64, f64)>>,
}

impl MapPolygon {
    pub fn new(exterior: impl IntoIterator<Item = (f64, f64)>) -> Self {
        Self {
            exterior: exterior.into_iter().collect(),
            holes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapFeature {
    pub name: String,
    /// `None` is ECharts' no-data state and must not participate in visualMap.
    pub value: Option<f64>,
    pub polygons: Vec<MapPolygon>,
    pub center: Option<(f64, f64)>,
    pub item_style: ItemStyle,
    pub label: LabelStyle,
    pub emphasis_item_style: ItemStyle,
    pub emphasis_label: LabelStyle,
    pub select_item_style: ItemStyle,
    pub select_label: LabelStyle,
    pub properties: BTreeMap<String, Value>,
    pub selected: bool,
}

impl MapFeature {
    pub fn new(name: impl Into<String>, polygons: Vec<MapPolygon>) -> Self {
        Self {
            name: name.into(),
            value: None,
            polygons,
            center: None,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            emphasis_item_style: ItemStyle::default(),
            emphasis_label: LabelStyle::default(),
            select_item_style: ItemStyle::default(),
            select_label: LabelStyle::default(),
            properties: BTreeMap::new(),
            selected: false,
        }
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapOptions {
    pub left: Value,
    pub top: Value,
    pub right: Option<Value>,
    pub bottom: Option<Value>,
    pub width: Option<Value>,
    pub height: Option<Value>,
    pub layout_center: Option<[Value; 2]>,
    pub layout_size: Option<Value>,
    pub aspect_scale: f32,
    pub center: Option<(f64, f64)>,
    pub zoom: f32,
    /// Runtime/native pan offset in logical pixels. JSON options start at zero.
    pub pan_offset: [f32; 2],
    pub scale_limit: Option<(f32, f32)>,
    pub bounding_coords: Option<[(f64, f64); 2]>,
    pub roam: String,
    pub name_property: String,
    pub name_map: BTreeMap<String, String>,
    pub emphasis_item_style: ItemStyle,
    pub emphasis_label: LabelStyle,
    pub select_item_style: ItemStyle,
    pub select_label: LabelStyle,
}

impl Default for MapOptions {
    fn default() -> Self {
        let emphasis_item_style = ItemStyle {
            color: Some(0xFF389BB7),
            ..ItemStyle::default()
        };
        let emphasis_label = LabelStyle {
            show: true,
            ..LabelStyle::default()
        };
        let select_item_style = ItemStyle {
            color: Some(0xFFE6B600),
            ..ItemStyle::default()
        };
        let select_label = LabelStyle {
            show: true,
            ..LabelStyle::default()
        };
        Self {
            left: Value::String(String::from("center")),
            top: Value::String(String::from("center")),
            right: None,
            bottom: None,
            width: None,
            height: None,
            layout_center: None,
            layout_size: None,
            aspect_scale: 0.75,
            center: None,
            zoom: 1.0,
            pan_offset: [0.0, 0.0],
            scale_limit: None,
            bounding_coords: None,
            roam: String::from("false"),
            name_property: String::from("name"),
            name_map: BTreeMap::new(),
            emphasis_item_style,
            emphasis_label,
            select_item_style,
            select_label,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapSeries {
    pub name: Option<String>,
    pub features: Vec<MapFeature>,
    pub options: SeriesOptions,
    pub map_options: Box<MapOptions>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSegment {
    pub name: Option<String>,
    pub from: (f64, f64),
    pub to: (f64, f64),
    /// Full ECharts `coords` path. `from`/`to` remain for typed API compatibility.
    pub coords: Vec<(f64, f64)>,
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
        let mut series = BasicSeries::new(name, values);
        series.options.symbol = String::from("emptyCircle");
        series.options.symbol_size = 6.0;
        Self::Line(series)
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
        let mut options = SeriesOptions::default();
        options.item_style.color = Some(0xFFEEEEEE);
        options.item_style.border_color = Some(0xFF444444);
        options.item_style.border_width = 0.5;
        Self::Map(MapSeries {
            name: Some(name.into()),
            features,
            options,
            map_options: Box::new(MapOptions::default()),
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

/// ECharts-style target used by programmatic chart actions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChartActionTarget {
    pub series_index: Option<usize>,
    pub series_name: Option<String>,
    pub data_index: Option<usize>,
    pub name: Option<String>,
}

impl ChartActionTarget {
    pub fn item(series_index: usize, data_index: usize) -> Self {
        Self {
            series_index: Some(series_index),
            data_index: Some(data_index),
            ..Self::default()
        }
    }

    pub fn named(series_name: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            series_name: Some(series_name.into()),
            name: Some(name.into()),
            ..Self::default()
        }
    }
}

/// Native equivalents of the commonly used ECharts `dispatchAction` payloads.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartActionKind {
    Highlight(ChartActionTarget),
    Downplay(ChartActionTarget),
    Select(ChartActionTarget),
    Unselect(ChartActionTarget),
    ToggleSelect(ChartActionTarget),
    ShowTip(ChartActionTarget),
    HideTip,
    LegendSelect {
        name: String,
    },
    LegendUnselect {
        name: String,
    },
    LegendToggleSelect {
        name: String,
    },
    DataZoom {
        data_zoom_index: usize,
        start: f64,
        end: f64,
    },
    TimelineChange {
        current_index: usize,
    },
    TimelinePlayChange {
        play_state: bool,
    },
    Restore,
}

/// Programmatic chart action. `silent` suppresses the corresponding event but
/// never suppresses the state change, matching ECharts `dispatchAction`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartAction {
    pub kind: ChartActionKind,
    pub silent: bool,
}

impl ChartAction {
    pub fn new(kind: ChartActionKind) -> Self {
        Self {
            kind,
            silent: false,
        }
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartSelectedItems {
    pub series_index: usize,
    pub data_indices: Vec<usize>,
}

/// Strongly typed payload for the ECharts `appendData` instance method.
/// ECharts incremental rendering supports native scatter and lines series.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartAppendData {
    Scatter {
        series_index: usize,
        data: Vec<DataPoint>,
    },
    Lines {
        series_index: usize,
        data: Vec<LineSegment>,
    },
}

impl ChartAppendData {
    pub fn scatter(series_index: usize, data: impl IntoIterator<Item = DataPoint>) -> Self {
        Self::Scatter {
            series_index,
            data: data.into_iter().collect(),
        }
    }

    pub fn lines(series_index: usize, data: impl IntoIterator<Item = LineSegment>) -> Self {
        Self::Lines {
            series_index,
            data: data.into_iter().collect(),
        }
    }

    pub fn series_index(&self) -> usize {
        match self {
            Self::Scatter { series_index, .. } | Self::Lines { series_index, .. } => *series_index,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Scatter { data, .. } => data.len(),
            Self::Lines { data, .. } => data.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Component selector used by `convertToPixel`, `convertFromPixel`, and
/// `containPixel`. Cartesian series and grid/axis finders are supported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChartCoordinateFinder {
    pub series_index: Option<usize>,
    pub grid_index: Option<usize>,
    pub x_axis_index: Option<usize>,
    pub y_axis_index: Option<usize>,
}

impl ChartCoordinateFinder {
    pub fn series(series_index: usize) -> Self {
        Self {
            series_index: Some(series_index),
            ..Self::default()
        }
    }

    pub fn grid(grid_index: usize) -> Self {
        Self {
            grid_index: Some(grid_index),
            ..Self::default()
        }
    }

    pub fn axes(x_axis_index: usize, y_axis_index: usize) -> Self {
        Self {
            x_axis_index: Some(x_axis_index),
            y_axis_index: Some(y_axis_index),
            ..Self::default()
        }
    }
}

/// Coordinate value before or after pixel conversion. Category axes preserve
/// their string labels; value/time/log axes use numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartCoordinatePoint {
    pub x: DataValue,
    pub y: DataValue,
}

impl ChartCoordinatePoint {
    pub fn values(x: impl Into<DataValue>, y: impl Into<DataValue>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn numbers(x: f64, y: f64) -> Self {
        Self::values(x, y)
    }
}

/// Unified callback payload for pointer and component-action events.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartRuntimeEventBatchItem {
    pub event_type: String,
    pub source: Option<ChartEvent>,
    pub from_action: Option<String>,
}

/// Unified callback payload for pointer and component-action events.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartRuntimeEvent {
    pub event_type: String,
    pub source: Option<ChartEvent>,
    pub from_action: Option<String>,
    pub selected: Vec<ChartSelectedItems>,
    pub legend_selected: BTreeMap<String, bool>,
    /// Action results when `dispatch_actions` uses ECharts batch semantics.
    pub batch: Vec<ChartRuntimeEventBatchItem>,
}
