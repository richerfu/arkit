//! ArkUI native node creation and tag normalization.
//!
//! This is the native backend of the Dioxus renderer. Keeping it in the
//! renderer crate makes the ownership boundary explicit: `arkit_arkui` owns
//! both HostTree projection and the ArkUI nodes produced by that projection.

use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::built_in_component::{
    CalendarPicker, Checkbox, Column, Custom, DatePicker, Flex, FlowItem, Grid, GridItem, Image,
    List, ListItem, LoadingProgress, Progress, Radio, Refresh, Row, Scroll, Slider, Stack, Swiper,
    Text, TextArea, TextInput, Toggle, WaterFlow, XComponent,
};

/// The canonical set of ArkUI built-in component kinds the renderer supports.
///
/// The canonical ArkUI node kinds exposed by the Dioxus element registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Button,
    CalendarPicker,
    Checkbox,
    Column,
    Custom,
    DatePicker,
    Flex,
    FlowItem,
    Grid,
    GridItem,
    Image,
    List,
    ListItem,
    LoadingProgress,
    Progress,
    Radio,
    Refresh,
    Row,
    Scroll,
    Slider,
    Stack,
    Swiper,
    Text,
    TextArea,
    TextInput,
    Toggle,
    WaterFlow,
    XComponent,
}

/// Instantiate a native [`ArkUINode`] for the given component kind.
pub fn create_node(kind: NodeKind) -> ArkUIResult<ArkUINode> {
    Ok(match kind {
        // Dioxus buttons accept arbitrary child trees, whereas ArkUI's native
        // Button accepts a label rather than child nodes. Project the semantic
        // button onto a pressable Stack; the renderer supplies its default
        // Button skin, accessibility role, and internal content Row.
        NodeKind::Button => Stack::new()?.into(),
        NodeKind::CalendarPicker => CalendarPicker::new()?.into(),
        NodeKind::Checkbox => Checkbox::new()?.into(),
        NodeKind::Column => Column::new()?.into(),
        NodeKind::Custom => Custom::new()?.into(),
        NodeKind::DatePicker => DatePicker::new()?.into(),
        NodeKind::Flex => Flex::new()?.into(),
        NodeKind::FlowItem => FlowItem::new()?.into(),
        NodeKind::Grid => Grid::new()?.into(),
        NodeKind::GridItem => GridItem::new()?.into(),
        NodeKind::Image => Image::new()?.into(),
        NodeKind::List => List::new()?.into(),
        NodeKind::ListItem => ListItem::new()?.into(),
        NodeKind::LoadingProgress => LoadingProgress::new()?.into(),
        NodeKind::Progress => Progress::new()?.into(),
        NodeKind::Radio => Radio::new()?.into(),
        NodeKind::Refresh => Refresh::new()?.into(),
        NodeKind::Row => Row::new()?.into(),
        NodeKind::Scroll => Scroll::new()?.into(),
        NodeKind::Slider => Slider::new()?.into(),
        NodeKind::Stack => Stack::new()?.into(),
        NodeKind::Swiper => Swiper::new()?.into(),
        NodeKind::Text => Text::new()?.into(),
        NodeKind::TextArea => TextArea::new()?.into(),
        NodeKind::TextInput => TextInput::new()?.into(),
        NodeKind::Toggle => Toggle::new()?.into(),
        NodeKind::WaterFlow => WaterFlow::new()?.into(),
        NodeKind::XComponent => XComponent::new()?.into(),
    })
}

/// Map an rsx tag string to its [`NodeKind`].
///
/// Returns `None` for unknown tags; the caller decides on a fallback (the
/// renderer uses `Stack`).
pub fn kind_from_tag(tag: &str) -> Option<NodeKind> {
    Some(match tag {
        "button" => NodeKind::Button,
        "calendar" | "calendarpicker" => NodeKind::CalendarPicker,
        "checkbox" => NodeKind::Checkbox,
        "column" => NodeKind::Column,
        "custom" => NodeKind::Custom,
        "datepicker" => NodeKind::DatePicker,
        "flex" => NodeKind::Flex,
        "flowitem" => NodeKind::FlowItem,
        "grid" => NodeKind::Grid,
        "griditem" => NodeKind::GridItem,
        "image" => NodeKind::Image,
        "list" => NodeKind::List,
        "listitem" => NodeKind::ListItem,
        "loadingprogress" => NodeKind::LoadingProgress,
        "progress" => NodeKind::Progress,
        "portal" => NodeKind::Stack,
        "radio" => NodeKind::Radio,
        "refresh" => NodeKind::Refresh,
        "row" => NodeKind::Row,
        "scroll" => NodeKind::Scroll,
        "slider" => NodeKind::Slider,
        "stack" => NodeKind::Stack,
        "swiper" => NodeKind::Swiper,
        "text" => NodeKind::Text,
        "textarea" => NodeKind::TextArea,
        "textinput" => NodeKind::TextInput,
        "toggle" => NodeKind::Toggle,
        "waterflow" => NodeKind::WaterFlow,
        "xcomponent" => NodeKind::XComponent,
        _ => return None,
    })
}

/// Map an rsx tag string to a freshly created [`ArkUINode`].
///
/// Unknown tags fall back to a `Stack` container (matching the renderer's
/// placeholder behavior) and emit a warning.
pub fn create_node_by_tag(tag: &str) -> ArkUIResult<ArkUINode> {
    match kind_from_tag(tag) {
        Some(kind) => create_node(kind),
        None => {
            ohos_hilog_binding::warn(format!(
                "arkit_arkui: unknown element tag `{tag}`, falling back to Stack"
            ));
            Stack::new().map(Into::into)
        }
    }
}

/// Resolve a tag string to its canonical static form for storage.
///
/// Unknown tags map to `"stack"`. The returned slice is `'static`.
pub fn canonical_tag(tag: &str) -> &'static str {
    match tag {
        "button" => "button",
        "calendar" | "calendarpicker" => "calendarpicker",
        "checkbox" => "checkbox",
        "column" => "column",
        "custom" => "custom",
        "datepicker" => "datepicker",
        "flex" => "flex",
        "flowitem" => "flowitem",
        "grid" => "grid",
        "griditem" => "griditem",
        "image" => "image",
        "list" => "list",
        "listitem" => "listitem",
        "loadingprogress" => "loadingprogress",
        "progress" => "progress",
        "portal" => "portal",
        "radio" => "radio",
        "refresh" => "refresh",
        "row" => "row",
        "scroll" => "scroll",
        "slider" => "slider",
        "stack" => "stack",
        "swiper" => "swiper",
        "text" => "text",
        "textarea" => "textarea",
        "textinput" => "textinput",
        "toggle" => "toggle",
        "waterflow" => "waterflow",
        "xcomponent" => "xcomponent",
        _ => "stack",
    }
}

/// Parse a CSS-like hex color (`"#RRGGBB"` / `"#AARRGGBB"` / `"RRGGBB"`) into a
/// 32-bit ARGB value. Returns `Err` if the string is not a hex color.
pub fn parse_color(s: &str) -> Result<u32, ()> {
    let s = s.strip_prefix('#').unwrap_or(s);
    match s.len() {
        6 => u32::from_str_radix(s, 16)
            .map(|v| 0xFF00_0000 | v)
            .map_err(|_| ()),
        8 => u32::from_str_radix(s, 16).map_err(|_| ()),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{canonical_tag, kind_from_tag, NodeKind};

    #[test]
    fn loading_progress_tag_maps_to_native_kind() {
        assert_eq!(
            kind_from_tag("loadingprogress"),
            Some(NodeKind::LoadingProgress)
        );
        assert_eq!(canonical_tag("loadingprogress"), "loadingprogress");
    }
}
