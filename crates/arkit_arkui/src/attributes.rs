//! Desired attribute storage + native element adapters.
//!
//! Dioxus mutations update a renderer-owned host tree. `DesiredAttrs` stores
//! the declarative attributes for each host node, and every native write goes
//! through the same encoder used for desired-state replay. This keeps immediate
//! mutation writes and attach/patch replays byte-for-byte consistent.

use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

use crate::parse_color;

const ROW_ALIGN_CENTER: i32 = 1;
const JUSTIFY_CENTER: i32 = 2;
const STACK_ALIGNMENT_CENTER: i32 = 4;
const TEXT_ALIGN_CENTER: i32 = 1;
const BUTTON_ACCESSIBILITY_ROLE: u32 = 9;
const BUTTON_DEFAULT_HEIGHT: f32 = 40.0;
const BUTTON_DEFAULT_HORIZONTAL_PADDING: f32 = 16.0;
const BUTTON_DEFAULT_VERTICAL_PADDING: f32 = 8.0;
const BUTTON_DEFAULT_RADIUS: f32 = 20.0;
const BUTTON_DEFAULT_BACKGROUND: u32 = 0xFF00_7DFF;
const BUTTON_DEFAULT_FOREGROUND: u32 = 0xFFFF_FFFF;
const BUTTON_DEFAULT_FONT_SIZE: f32 = 16.0;
const BUTTON_DEFAULT_FONT_WEIGHT: i32 = 500;

#[derive(Clone, Debug)]
enum EncodedAttrValue {
    F32(f32),
    I32(i32),
    Bool(bool),
    U32(u32),
    String(String),
    VecF32(Vec<f32>),
    FlexOptionPart(usize, i32),
    Shadow(i32),
}

impl EncodedAttrValue {
    fn to_item(&self) -> ArkUINodeAttributeItem {
        match self {
            Self::F32(v) => (*v).into(),
            Self::I32(v) => (*v).into(),
            Self::Bool(v) => (*v).into(),
            Self::U32(v) => (*v).into(),
            Self::String(v) => v.clone().into(),
            Self::VecF32(v) => v.clone().into(),
            Self::FlexOptionPart(index, value) => flex_option_with(*index, *value).into(),
            Self::Shadow(v) => vec![*v].into(),
        }
    }
}

#[derive(Clone, Debug)]
struct EncodedAttr {
    name: String,
    ty: ArkUINodeAttributeType,
    value: EncodedAttrValue,
}

impl EncodedAttr {
    fn new(name: impl Into<String>, ty: ArkUINodeAttributeType, value: EncodedAttrValue) -> Self {
        Self {
            name: name.into(),
            ty,
            value,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&self, node: &mut ArkUINode, tag: &str) -> ArkUIResult<()> {
        if tag == "button" && is_button_text_attr(self.name()) {
            return Ok(());
        }
        let ty = self.ty;
        let result = node.set_attribute(ty, self.value.to_item());
        if let Err(error) = &result {
            ohos_hilog_binding::error(format!(
                "arkit_arkui: failed to apply `{}` to <{tag}> ({ty:?}): {error}",
                self.name()
            ));
        }
        result
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct DesiredAttrs {
    attrs: Vec<EncodedAttr>,
}

impl DesiredAttrs {
    pub(crate) fn set(&mut self, tag: &str, name: &str, value: &dioxus_core::AttributeValue) {
        let Some(attr) = encode_attr(tag, name, value) else {
            return;
        };
        if let Some(existing) = self.attrs.iter_mut().find(|item| item.name == attr.name) {
            *existing = attr;
        } else {
            self.attrs.push(attr);
        }
    }

    fn get(&self, name: &str) -> Option<&EncodedAttr> {
        self.attrs.iter().find(|attr| attr.name == name)
    }

    fn has_any(&self, names: &[&str]) -> bool {
        names.iter().any(|name| self.get(name).is_some())
    }

    pub(crate) fn apply_to(&self, node: &mut ArkUINode, tag: &str) {
        for group in [
            AttrGroup::Control,
            AttrGroup::Layout,
            AttrGroup::Visual,
            AttrGroup::Text,
            AttrGroup::Image,
        ] {
            self.apply_group(node, tag, group);
        }
    }

    fn apply_group(&self, node: &mut ArkUINode, tag: &str, group: AttrGroup) {
        if group == AttrGroup::Layout {
            // Geometry must be committed before alignment. Dioxus stores
            // static attributes before dynamic ones, so insertion order is
            // unrelated to native layout dependencies (for example a static
            // `justify_content` can precede a dynamic `percent_height`).
            for attr in self
                .attrs
                .iter()
                .filter(|attr| attr_group(attr.name()) == group)
                .filter(|attr| !is_box_attr(attr.name()))
                .filter(|attr| !is_flex_option_attr(attr.name()))
                .filter(|attr| !is_alignment_attr(attr.name()))
            {
                let _ = attr.apply(node, tag);
            }
            self.apply_box(node, "padding", ArkUINodeAttributeType::Padding);
            self.apply_box(node, "margin", ArkUINodeAttributeType::Margin);
            if tag == "flex" {
                self.apply_flex_option(node);
            }
            for attr in self
                .attrs
                .iter()
                .filter(|attr| attr_group(attr.name()) == group)
                .filter(|attr| is_alignment_attr(attr.name()))
            {
                let _ = attr.apply(node, tag);
            }
            return;
        }

        let mut deferred = Vec::new();
        for attr in self
            .attrs
            .iter()
            .filter(|attr| attr_group(attr.name()) == group)
        {
            if is_deferred_attr(attr.name()) {
                deferred.push(attr);
                continue;
            }
            let _ = attr.apply(node, tag);
        }
        for attr in deferred {
            let _ = attr.apply(node, tag);
        }
    }

    pub(crate) fn apply_named(&self, node: &mut ArkUINode, tag: &str, names: &[&str]) {
        let wants_padding = names.iter().any(|name| name.starts_with("padding"));
        let wants_margin = names.iter().any(|name| name.starts_with("margin"));
        let wants_flex = names.iter().any(|name| is_flex_option_attr(name));
        let mut deferred = Vec::new();
        for name in names {
            if is_box_attr(name) || is_flex_option_attr(name) {
                continue;
            }
            if let Some(attr) = self.get(name) {
                if is_deferred_attr(attr.name()) {
                    deferred.push(attr);
                    continue;
                }
                let _ = attr.apply(node, tag);
            }
        }
        if wants_padding {
            self.apply_box(node, "padding", ArkUINodeAttributeType::Padding);
        }
        if wants_margin {
            self.apply_box(node, "margin", ArkUINodeAttributeType::Margin);
        }
        if wants_flex && tag == "flex" {
            self.apply_flex_option(node);
        }
        for attr in deferred {
            let _ = attr.apply(node, tag);
        }
    }

    pub(crate) fn apply_button_text_attrs(&self, node: &mut ArkUINode) {
        if !self.has_any(&["font_color", "foreground_color"]) {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::FontColor,
                BUTTON_DEFAULT_FOREGROUND.into(),
            );
        }
        if self.get("font_size").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::FontSize,
                BUTTON_DEFAULT_FONT_SIZE.into(),
            );
        }
        if self.get("font_weight").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::FontWeight,
                BUTTON_DEFAULT_FONT_WEIGHT.into(),
            );
        }
        if self.get("text_align").is_none() {
            let _ = node.set_attribute(ArkUINodeAttributeType::TextAlign, TEXT_ALIGN_CENTER.into());
        }
        self.apply_named(
            node,
            "text",
            &[
                "font_color",
                "foreground_color",
                "font_size",
                "font_weight",
                "font_style",
                "font_family",
                "line_height",
                "text_align",
                "text_letter_spacing",
                "text_decoration",
                "text_overflow",
                "max_lines",
            ],
        );
    }

    fn apply_box(&self, node: &mut ArkUINode, base: &str, ty: ArkUINodeAttributeType) {
        let mut sides = self
            .get(base)
            .and_then(|attr| match &attr.value {
                EncodedAttrValue::VecF32(v) if v.len() == 4 => Some([v[0], v[1], v[2], v[3]]),
                _ => None,
            })
            .unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let mut has_any = self.get(base).is_some();

        for (name, index) in [
            (format!("{base}_top"), 0usize),
            (format!("{base}_right"), 1),
            (format!("{base}_bottom"), 2),
            (format!("{base}_left"), 3),
        ] {
            if let Some(attr) = self.get(&name) {
                if let EncodedAttrValue::F32(value) = &attr.value {
                    sides[index] = *value;
                    has_any = true;
                }
            }
        }

        if has_any {
            let _ = node.set_attribute(ty, vec![sides[0], sides[1], sides[2], sides[3]].into());
        }
    }

    fn apply_flex_option(&self, node: &mut ArkUINode) {
        let mut fields = vec![0, 0, 1, 1, 1];
        for attr in self
            .attrs
            .iter()
            .filter(|attr| is_flex_option_attr(attr.name()))
        {
            if let EncodedAttrValue::FlexOptionPart(index, value) = attr.value {
                if index < fields.len() {
                    fields[index] = value;
                }
            }
        }
        let _ = node.set_attribute(ArkUINodeAttributeType::FlexOption, fields.into());
    }
}

fn is_box_attr(name: &str) -> bool {
    matches!(
        name,
        "padding"
            | "padding_top"
            | "padding_right"
            | "padding_bottom"
            | "padding_left"
            | "margin"
            | "margin_top"
            | "margin_right"
            | "margin_bottom"
            | "margin_left"
    )
}

fn is_flex_option_attr(name: &str) -> bool {
    matches!(
        name,
        "flex_direction" | "flex_wrap" | "justify_content" | "align_items" | "flex_align_content"
    )
}

fn is_alignment_attr(name: &str) -> bool {
    matches!(
        name,
        "alignment" | "align_self" | "item_alignment" | "align_items" | "justify_content"
    )
}

fn is_button_text_attr(name: &str) -> bool {
    matches!(
        name,
        "font_color"
            | "font_size"
            | "font_weight"
            | "font_style"
            | "font_family"
            | "line_height"
            | "text_align"
            | "text_letter_spacing"
            | "text_decoration"
            | "text_overflow"
            | "max_lines"
            | "content"
            | "label"
            | "text_content"
    )
}

fn is_deferred_attr(name: &str) -> bool {
    matches!(
        name,
        "border_radius" | "corner_radius" | "clip" | "clip_shape"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttrGroup {
    Control,
    Layout,
    Visual,
    Text,
    Image,
}

fn attr_group(name: &str) -> AttrGroup {
    match name {
        "font_size"
        | "font_color"
        | "font_weight"
        | "font_style"
        | "font_family"
        | "line_height"
        | "text_align"
        | "text_letter_spacing"
        | "text_decoration"
        | "text_overflow"
        | "max_lines"
        | "content"
        | "label"
        | "text_content"
        | "placeholder"
        | "value"
        | "placeholder_color"
        | "caret_color" => AttrGroup::Text,
        "src" | "object_fit" => AttrGroup::Image,
        "background_color"
        | "border_color"
        | "border_width"
        | "border_radius"
        | "corner_radius"
        | "border_style"
        | "shadow"
        | "opacity"
        | "clip"
        | "visibility"
        | "enabled"
        | "foreground_color"
        | "progress_color"
        | "color_blend"
        | "checkbox_select_color"
        | "block_color"
        | "track_color"
        | "selected_color"
        | "toggle_selected_color"
        | "toggle_unselected_color"
        | "toggle_switch_point_color" => AttrGroup::Visual,
        _ => AttrGroup::Layout,
    }
}

fn encode_attr(tag: &str, name: &str, value: &dioxus_core::AttributeValue) -> Option<EncodedAttr> {
    let as_f32 = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Float(f) => Some(*f as f32),
        dioxus_core::AttributeValue::Int(i) => Some(*i as f32),
        dioxus_core::AttributeValue::Text(s) => s.parse::<f32>().ok(),
        _ => None,
    };
    let as_i32 = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Float(f) => Some(*f as i32),
        dioxus_core::AttributeValue::Int(i) => Some(*i as i32),
        dioxus_core::AttributeValue::Text(s) => s.parse::<i32>().ok(),
        _ => None,
    };
    let as_bool = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Bool(b) => Some(*b),
        dioxus_core::AttributeValue::Int(i) => Some(*i != 0),
        _ => None,
    };
    let as_string = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Text(s) => Some(s.clone()),
        dioxus_core::AttributeValue::Float(f) => Some(f.to_string()),
        dioxus_core::AttributeValue::Int(i) => Some(i.to_string()),
        dioxus_core::AttributeValue::Bool(b) => Some(b.to_string()),
        _ => None,
    };
    let as_color = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Text(s) => parse_color(s).ok(),
        dioxus_core::AttributeValue::Int(i) => Some(*i as u32),
        _ => None,
    };
    let as_radius_vec = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Text(s) => parse_f32_list(s).and_then(|values| {
            if values.len() == 4 {
                Some(values)
            } else {
                None
            }
        }),
        _ => as_f32(v).map(|v| vec![v, v, v, v]),
    };
    let as_box_vec = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Text(s) => parse_f32_list(s).and_then(|values| {
            if values.len() == 4 {
                Some(values)
            } else {
                None
            }
        }),
        _ => as_f32(v).map(|v| vec![v, v, v, v]),
    };

    if let Some(ty) = color_attr(name, tag) {
        return as_color(value).map(|v| EncodedAttr::new(name, ty, EncodedAttrValue::U32(v)));
    }

    let attr = match name {
        "font_size" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::FontSize,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "font_weight" => {
            let raw = as_i32(value)?;
            let mapped = if raw >= 100 {
                ((raw / 100).saturating_sub(1)).min(8)
            } else {
                raw
            };
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FontWeight,
                EncodedAttrValue::I32(mapped),
            )
        }
        "font_style" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::FontStyle,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "font_family" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::FontFamily,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "line_height" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextLineHeight,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "text_align" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextAlign,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "text_letter_spacing" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextLetterSpacing,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "text_decoration" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextDecoration,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "text_overflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextOverflow,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "max_lines" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextMaxLines,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "content" | "label" | "text_content" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextContent,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "value" if matches!(tag, "textinput" | "textarea") => {
            let ty = match tag {
                "textinput" => ArkUINodeAttributeType::TextInputText,
                "textarea" => ArkUINodeAttributeType::TextAreaText,
                _ => return None,
            };
            EncodedAttr::new(name, ty, EncodedAttrValue::String(as_string(value)?))
        }
        "placeholder" => {
            let ty = match tag {
                "textinput" => ArkUINodeAttributeType::TextInputPlaceholder,
                "textarea" => ArkUINodeAttributeType::TextAreaPlaceholder,
                _ => return None,
            };
            EncodedAttr::new(name, ty, EncodedAttrValue::String(as_string(value)?))
        }
        "padding" => {
            let v = as_f32(value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Padding,
                EncodedAttrValue::VecF32(vec![v, v, v, v]),
            )
        }
        "padding_top" | "padding_right" | "padding_bottom" | "padding_left" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Padding,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "margin" => {
            let v = as_f32(value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Margin,
                EncodedAttrValue::VecF32(vec![v, v, v, v]),
            )
        }
        "margin_top" | "margin_right" | "margin_bottom" | "margin_left" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Margin,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "percent_width" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WidthPercent,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "percent_height" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::HeightPercent,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "width" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Width,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "height" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Height,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "max_width_constraint" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ConstraintSize,
            EncodedAttrValue::VecF32(vec![0.0, as_f32(value)?, 0.0, 100_000.0]),
        ),
        "constraint_size" => {
            let values = match value {
                dioxus_core::AttributeValue::Text(s) => parse_f32_list(s)?,
                _ => return None,
            };
            if values.len() != 4 {
                return None;
            }
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::ConstraintSize,
                EncodedAttrValue::VecF32(values),
            )
        }
        "opacity" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Opacity,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "layout_weight" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::LayoutWeight,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "aspect_ratio" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::AspectRatio,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "position" => {
            let values = match value {
                dioxus_core::AttributeValue::Text(s) => {
                    let values = parse_f32_list(s)?;
                    match values.as_slice() {
                        [value] => vec![*value, *value],
                        [x, y] => vec![*x, *y],
                        _ => return None,
                    }
                }
                _ => {
                    let v = as_f32(value)?;
                    vec![v, v]
                }
            };
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Position,
                EncodedAttrValue::VecF32(values),
            )
        }
        "z_index" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ZIndex,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "border_radius" | "corner_radius" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::BorderRadius,
            EncodedAttrValue::VecF32(as_radius_vec(value)?),
        ),
        "border_width" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::BorderWidth,
            EncodedAttrValue::VecF32(as_box_vec(value)?),
        ),
        "border_style" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::BorderStyle,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "shadow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Shadow,
            EncodedAttrValue::Shadow(as_i32(value)?),
        ),
        "enabled" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Enabled,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "clip" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Clip,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "visibility" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Visibility,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "focusable" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Focusable,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "focus_on_touch" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::FocusOnTouch,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "hit_test_behavior" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::HitTestBehavior,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "alignment" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Alignment,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "align_self" | "item_alignment" => {
            let v = as_i32(value)
                .or_else(|| as_string(value).and_then(|s| item_alignment_value(&s)))?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::AlignSelf,
                EncodedAttrValue::I32(v),
            )
        }
        "align_items" => {
            if tag == "flex" {
                let v = as_i32(value)
                    .or_else(|| as_string(value).and_then(|s| item_alignment_value(&s)))?;
                EncodedAttr::new(
                    name,
                    ArkUINodeAttributeType::FlexOption,
                    EncodedAttrValue::FlexOptionPart(3, v),
                )
            } else {
                let attr = align_items_attr(tag)?;
                let v = as_i32(value)
                    .or_else(|| as_string(value).and_then(|s| align_items_value(tag, &s)))?;
                EncodedAttr::new(name, attr, EncodedAttrValue::I32(v))
            }
        }
        "justify_content" => {
            let v = as_i32(value)
                .or_else(|| as_string(value).and_then(|s| justify_content_value(&s)))?;
            if tag == "flex" {
                EncodedAttr::new(
                    name,
                    ArkUINodeAttributeType::FlexOption,
                    EncodedAttrValue::FlexOptionPart(2, v),
                )
            } else {
                EncodedAttr::new(name, justify_content_attr(tag)?, EncodedAttrValue::I32(v))
            }
        }
        "flex_direction" => {
            let v = as_i32(value)
                .or_else(|| as_string(value).and_then(|s| flex_direction_value(&s)))?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(0, v),
            )
        }
        "flex_wrap" => {
            let v = as_i32(value).or_else(|| as_string(value).and_then(|s| flex_wrap_value(&s)))?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(1, v),
            )
        }
        "flex_align_content" => {
            let v = as_i32(value)
                .or_else(|| as_string(value).and_then(|s| justify_content_value(&s)))?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(4, v),
            )
        }
        "scroll_bar" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollBarDisplayMode,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "scroll_enabled" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollEnableScrollInteraction,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "scroll_edge_effect" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollEdgeEffect,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_index" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperIndex,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_loop" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperLoop,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "swiper_cached_count" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperCachedCount,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_display_count" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperDisplayCount,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_vertical" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperVertical,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "swiper_interval" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperInterval,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_duration" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperDuration,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "grid_column_template" if tag == "grid" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::GridColumnTemplate,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "grid_row_template" if tag == "grid" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::GridRowTemplate,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "grid_column_gap" if tag == "grid" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::GridColumnGap,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "grid_row_gap" if tag == "grid" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::GridRowGap,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "grid_cached_count" if tag == "grid" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::GridCachedCount,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "list_cached_count" if tag == "list" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ListCachedCount,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "list_sticky" if tag == "list" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ListSticky,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "water_flow_column_template" if tag == "waterflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WaterFlowColumnTemplate,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "water_flow_row_template" if tag == "waterflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WaterFlowRowTemplate,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "water_flow_column_gap" if tag == "waterflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WaterFlowColumnGap,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "water_flow_row_gap" if tag == "waterflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WaterFlowRowGap,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "water_flow_cached_count" if tag == "waterflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::WaterFlowCachedCount,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "refresh_state" | "refreshing" if tag == "refresh" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::RefreshRefreshing,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "refresh_offset" if tag == "refresh" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::RefreshOffset,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "refresh_pull_to_refresh" if tag == "refresh" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::RefreshPullToRefresh,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "object_fit" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ImageObjectFit,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "src" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ImageSrc,
            match value {
                dioxus_core::AttributeValue::Any(_) => return None,
                _ => EncodedAttrValue::String(as_string(value)?),
            },
        ),
        "checked" => {
            let ty = match tag {
                "checkbox" => ArkUINodeAttributeType::CheckboxSelect,
                "toggle" => ArkUINodeAttributeType::ToggleValue,
                "radio" => ArkUINodeAttributeType::RadioChecked,
                _ => return None,
            };
            EncodedAttr::new(name, ty, EncodedAttrValue::Bool(as_bool(value)?))
        }
        "radio_value" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::RadioValue,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "slider_value" | "value" if tag == "slider" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SliderValue,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "slider_min" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SliderMinValue,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "slider_max" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SliderMaxValue,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "slider_step" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SliderStep,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "progress_value" | "value" if tag == "progress" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ProgressValue,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "progress_total" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ProgressTotal,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "progress_type" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ProgressType,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        _ => return None,
    };

    Some(attr)
}

fn parse_f32_list(value: &str) -> Option<Vec<f32>> {
    let values = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn flex_option_with(index: usize, value: i32) -> Vec<i32> {
    let mut fields = vec![0, 0, 1, 1, 1];
    if index < fields.len() {
        fields[index] = value;
    }
    fields
}

fn color_attr(name: &str, tag: &str) -> Option<ArkUINodeAttributeType> {
    Some(match name {
        "font_color" => ArkUINodeAttributeType::FontColor,
        "background_color" => ArkUINodeAttributeType::BackgroundColor,
        "border_color" => ArkUINodeAttributeType::BorderColor,
        "foreground_color" => ArkUINodeAttributeType::ForegroundColor,
        "progress_color" => ArkUINodeAttributeType::ProgressColor,
        "color_blend" => ArkUINodeAttributeType::ColorBlend,
        "placeholder_color" => match tag {
            "textinput" => ArkUINodeAttributeType::TextInputPlaceholderColor,
            "textarea" => ArkUINodeAttributeType::TextAreaPlaceholderColor,
            _ => return None,
        },
        "caret_color" => match tag {
            "textinput" => ArkUINodeAttributeType::TextInputCaretColor,
            "textarea" => ArkUINodeAttributeType::TextAreaCaretColor,
            _ => return None,
        },
        "checkbox_select_color" => ArkUINodeAttributeType::CheckboxSelectColor,
        "block_color" if tag == "slider" => ArkUINodeAttributeType::SliderBlockColor,
        "track_color" if tag == "slider" => ArkUINodeAttributeType::SliderTrackColor,
        "selected_color" if tag == "slider" => ArkUINodeAttributeType::SliderSelectedColor,
        "toggle_selected_color" => ArkUINodeAttributeType::ToggleSelectedColor,
        "toggle_unselected_color" => ArkUINodeAttributeType::ToggleUnselectedColor,
        "toggle_switch_point_color" => ArkUINodeAttributeType::ToggleSwitchPointColor,
        _ => return None,
    })
}

fn align_items_attr(tag: &str) -> Option<ArkUINodeAttributeType> {
    match tag {
        "column" => Some(ArkUINodeAttributeType::ColumnAlignItems),
        "row" => Some(ArkUINodeAttributeType::RowAlignItems),
        _ => None,
    }
}

fn enum_token(s: &str) -> String {
    s.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase()
}

fn align_items_value(tag: &str, s: &str) -> Option<i32> {
    let lower = enum_token(s);
    match tag {
        "column" | "row" => match lower.as_str() {
            "start" | "top" => Some(0),
            "center" => Some(1),
            "end" | "bottom" => Some(2),
            _ => None,
        },
        _ => None,
    }
}

fn justify_content_attr(tag: &str) -> Option<ArkUINodeAttributeType> {
    match tag {
        "column" => Some(ArkUINodeAttributeType::ColumnJustifyContent),
        "row" => Some(ArkUINodeAttributeType::RowJustifyContent),
        _ => None,
    }
}

fn justify_content_value(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "start" => Some(1),
        "center" => Some(2),
        "end" => Some(3),
        "space_between" | "spacebetween" => Some(6),
        "space_around" | "spacearound" => Some(7),
        "space_evenly" | "spaceevenly" => Some(8),
        _ => None,
    }
}

fn item_alignment_value(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "auto" => Some(0),
        "start" => Some(1),
        "center" => Some(2),
        "end" => Some(3),
        "stretch" => Some(4),
        "baseline" => Some(5),
        _ => None,
    }
}

fn flex_direction_value(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "row" => Some(0),
        "column" => Some(1),
        "row_reverse" | "rowreverse" => Some(2),
        "column_reverse" | "columnreverse" => Some(3),
        _ => None,
    }
}

fn flex_wrap_value(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "nowrap" | "no_wrap" => Some(0),
        "wrap" => Some(1),
        "wrap_reverse" | "wrapreverse" => Some(2),
        _ => None,
    }
}

impl DesiredAttrs {
    pub(crate) fn apply_initial(&self, node: &mut ArkUINode, tag: &str) {
        if tag == "button" {
            let _ = node.set_attribute(ArkUINodeAttributeType::Focusable, true.into());
            let _ = node.set_attribute(
                ArkUINodeAttributeType::AccessibilityRole,
                BUTTON_ACCESSIBILITY_ROLE.into(),
            );
            let _ = node.set_attribute(ArkUINodeAttributeType::AccessibilityGroup, true.into());
        }
    }

    pub(crate) fn after_attach(&self, node: &mut ArkUINode, tag: &str) {
        // Several ArkUI layout attributes are accepted only after a node is in
        // the mounted tree. Reapply the complete declarative state here; this
        // is the same encoder used for initial creation and later patches.
        self.apply_to(node, tag);
        self.after_patch(node, tag);
    }

    pub(crate) fn after_patch(&self, node: &mut ArkUINode, tag: &str) {
        if tag != "button" {
            return;
        }
        if !self.has_any(&["height", "percent_height", "constraint_size"]) {
            let _ =
                node.set_attribute(ArkUINodeAttributeType::Height, BUTTON_DEFAULT_HEIGHT.into());
        }
        if !self.has_any(&[
            "padding",
            "padding_top",
            "padding_right",
            "padding_bottom",
            "padding_left",
        ]) {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::Padding,
                vec![
                    BUTTON_DEFAULT_VERTICAL_PADDING,
                    BUTTON_DEFAULT_HORIZONTAL_PADDING,
                    BUTTON_DEFAULT_VERTICAL_PADDING,
                    BUTTON_DEFAULT_HORIZONTAL_PADDING,
                ]
                .into(),
            );
        }
        if self.get("background_color").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::BackgroundColor,
                BUTTON_DEFAULT_BACKGROUND.into(),
            );
        }
        if !self.has_any(&["border_radius", "corner_radius"]) {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::BorderRadius,
                vec![BUTTON_DEFAULT_RADIUS; 4].into(),
            );
        }
        if self.get("clip").is_none() {
            let _ = node.set_attribute(ArkUINodeAttributeType::Clip, true.into());
        }
        if self.get("alignment").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::Alignment,
                STACK_ALIGNMENT_CENTER.into(),
            );
        }
        self.apply_named(
            node,
            "button",
            &[
                "height",
                "width",
                "percent_width",
                "padding",
                "padding_top",
                "padding_right",
                "padding_bottom",
                "padding_left",
                "background_color",
                "border_style",
                "border_width",
                "border_color",
                "border_radius",
                "clip",
                "font_color",
                "font_size",
                "font_weight",
                "opacity",
                "enabled",
                "alignment",
                "focusable",
                "focus_on_touch",
                "hit_test_behavior",
                "shadow",
            ],
        );
    }

    pub(crate) fn apply_content(&self, node: &mut ArkUINode, tag: &str) {
        if tag != "button" {
            return;
        }
        if self.get("align_items").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::RowAlignItems,
                ROW_ALIGN_CENTER.into(),
            );
        }
        if self.get("justify_content").is_none() {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::RowJustifyContent,
                JUSTIFY_CENTER.into(),
            );
        }
        self.apply_named(node, "row", &["align_items", "justify_content"]);
    }
}
