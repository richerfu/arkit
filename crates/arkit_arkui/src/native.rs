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
        "button" | "Button" => NodeKind::Button,
        "calendar" | "calendarpicker" | "Calendar" | "CalendarPicker" => NodeKind::CalendarPicker,
        "checkbox" | "Checkbox" => NodeKind::Checkbox,
        "column" | "Column" => NodeKind::Column,
        "custom" | "Custom" => NodeKind::Custom,
        "datepicker" | "DatePicker" => NodeKind::DatePicker,
        "flex" | "Flex" => NodeKind::Flex,
        "flowitem" | "FlowItem" => NodeKind::FlowItem,
        "grid" | "Grid" => NodeKind::Grid,
        "griditem" | "GridItem" => NodeKind::GridItem,
        "image" | "Image" => NodeKind::Image,
        "list" | "List" => NodeKind::List,
        "listitem" | "ListItem" => NodeKind::ListItem,
        "loadingprogress" | "LoadingProgress" => NodeKind::LoadingProgress,
        "progress" | "Progress" => NodeKind::Progress,
        "portal" | "Portal" => NodeKind::Stack,
        "radio" | "Radio" => NodeKind::Radio,
        "refresh" | "Refresh" => NodeKind::Refresh,
        "row" | "Row" => NodeKind::Row,
        "scroll" | "Scroll" => NodeKind::Scroll,
        "slider" | "Slider" => NodeKind::Slider,
        "stack" | "Stack" => NodeKind::Stack,
        "swiper" | "Swiper" => NodeKind::Swiper,
        "text" | "Text" => NodeKind::Text,
        "textarea" | "TextArea" => NodeKind::TextArea,
        "textinput" | "TextInput" => NodeKind::TextInput,
        "toggle" | "Toggle" => NodeKind::Toggle,
        "waterflow" | "WaterFlow" => NodeKind::WaterFlow,
        "xcomponent" | "XComponent" => NodeKind::XComponent,
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
        "button" | "Button" => "button",
        "calendar" | "calendarpicker" | "Calendar" | "CalendarPicker" => "calendarpicker",
        "checkbox" | "Checkbox" => "checkbox",
        "column" | "Column" => "column",
        "custom" | "Custom" => "custom",
        "datepicker" | "DatePicker" => "datepicker",
        "flex" | "Flex" => "flex",
        "flowitem" | "FlowItem" => "flowitem",
        "grid" | "Grid" => "grid",
        "griditem" | "GridItem" => "griditem",
        "image" | "Image" => "image",
        "list" | "List" => "list",
        "listitem" | "ListItem" => "listitem",
        "loadingprogress" | "LoadingProgress" => "loadingprogress",
        "progress" | "Progress" => "progress",
        "portal" | "Portal" => "portal",
        "radio" | "Radio" => "radio",
        "refresh" | "Refresh" => "refresh",
        "row" | "Row" => "row",
        "scroll" | "Scroll" => "scroll",
        "slider" | "Slider" => "slider",
        "stack" | "Stack" => "stack",
        "swiper" | "Swiper" => "swiper",
        "text" | "Text" => "text",
        "textarea" | "TextArea" => "textarea",
        "textinput" | "TextInput" => "textinput",
        "toggle" | "Toggle" => "toggle",
        "waterflow" | "WaterFlow" => "waterflow",
        "xcomponent" | "XComponent" => "xcomponent",
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
            kind_from_tag("LoadingProgress"),
            Some(NodeKind::LoadingProgress)
        );
        assert_eq!(canonical_tag("LoadingProgress"), "loadingprogress");
    }
}
