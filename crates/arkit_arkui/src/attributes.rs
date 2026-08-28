//! Desired attribute storage + native element adapters.
//!
//! Dioxus mutations update a renderer-owned host tree. `DesiredAttrs` stores
//! persistent declarative attributes for each host node, and every native
//! write goes through the same encoder used for desired-state replay. Commands
//! such as `scroll_offset` are typed separately and consumed exactly once.

use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

use crate::css_value::{
    self, expand_box_shorthand, i32_or_keyword, map_font_weight_to_arkui, object_fit_value,
    parse_css_color, parse_length, parse_opacity, parse_vp, CssLength,
};
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

#[derive(Clone, Debug, PartialEq)]
enum EncodedAttrValue {
    F32(f32),
    I32(i32),
    Bool(bool),
    U32(u32),
    String(String),
    VecF32(Vec<f32>),
    VecI32(Vec<i32>),
    ScrollOffset { x: f32, y: f32, options: Vec<i32> },
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
            Self::VecI32(v) => v.clone().into(),
            Self::ScrollOffset { x, y, options } => {
                let mut values = Vec::with_capacity(2 + options.len());
                values.push(ArkUINodeAttributeNumber::Float(*x));
                values.push(ArkUINodeAttributeNumber::Float(*y));
                values.extend(options.iter().copied().map(ArkUINodeAttributeNumber::Int));
                ArkUINodeAttributeItem::NumberValue(values)
            }
            Self::FlexOptionPart(index, value) => flex_option_with(*index, *value).into(),
            Self::Shadow(v) => vec![*v].into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

/// Imperative Scroll position change carried through Dioxus attributes.
///
/// ArkUI defines `NODE_SCROLL_OFFSET` as a scroll-to operation. It is not
/// declarative node state and must never enter [`DesiredAttrs`], otherwise
/// attach or global style replay can unexpectedly reset a user's live scroll.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScrollOffsetCommand {
    x: f32,
    y: f32,
    options: Vec<i32>,
}

impl ScrollOffsetCommand {
    pub(crate) fn from_attribute(value: &dioxus_core::AttributeValue) -> Option<Self> {
        let dioxus_core::AttributeValue::Text(text) = value else {
            return None;
        };
        let (x, y, options) = parse_scroll_offset(text)?;
        Some(Self { x, y, options })
    }

    pub(crate) fn apply(&self, node: &mut ArkUINode) -> ArkUIResult<()> {
        EncodedAttr::new(
            "scroll_offset",
            ArkUINodeAttributeType::ScrollOffset,
            EncodedAttrValue::ScrollOffset {
                x: self.x,
                y: self.y,
                options: self.options.clone(),
            },
        )
        .apply(node, "scroll")
    }
}

/// Imperative List jump carried through Dioxus attributes.
///
/// ArkUI `NODE_LIST_SCROLL_TO_INDEX` is a scroll-to operation. It must not
/// enter [`DesiredAttrs`], or attach/style replay would yank the list back.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ListScrollToIndexCommand {
    index: i32,
    smooth: i32,
    align: i32,
}

impl ListScrollToIndexCommand {
    pub(crate) fn from_attribute(value: &dioxus_core::AttributeValue) -> Option<Self> {
        let (index, smooth, align) = parse_scroll_to_index(value)?;
        Some(Self {
            index,
            smooth,
            align,
        })
    }

    pub(crate) fn apply(&self, node: &mut ArkUINode) -> ArkUIResult<()> {
        EncodedAttr::new(
            "scroll_to_index",
            ArkUINodeAttributeType::ListScrollToIndex,
            EncodedAttrValue::VecI32(vec![self.index, self.smooth, self.align]),
        )
        .apply(node, "list")
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct DesiredAttrs {
    attrs: Vec<EncodedAttr>,
}

#[derive(Clone, Copy)]
pub(crate) enum AttrMutation {
    Unchanged,
    Set,
    Removed(ArkUINodeAttributeType),
}

impl DesiredAttrs {
    pub(crate) fn set(
        &mut self,
        tag: &str,
        name: &str,
        value: &dioxus_core::AttributeValue,
    ) -> AttrMutation {
        if matches!(value, dioxus_core::AttributeValue::None) {
            return self
                .attrs
                .iter()
                .position(|item| item.name == name)
                .map(|index| AttrMutation::Removed(self.attrs.remove(index).ty))
                .unwrap_or(AttrMutation::Unchanged);
        }
        let Some(attr) = encode_attr(tag, name, value) else {
            return AttrMutation::Unchanged;
        };
        if let Some(existing) = self.attrs.iter_mut().find(|item| item.name == attr.name) {
            if *existing == attr {
                return AttrMutation::Unchanged;
            }
            *existing = attr;
        } else {
            self.attrs.push(attr);
        }
        AttrMutation::Set
    }

    fn get(&self, name: &str) -> Option<&EncodedAttr> {
        self.attrs.iter().find(|attr| attr.name == name)
    }

    fn has_any(&self, names: &[&str]) -> bool {
        names.iter().any(|name| self.get(name).is_some())
    }

    pub(crate) fn apply_to(&self, node: &mut ArkUINode, tag: &str) {
        self.apply_to_skipping(node, tag, &rustc_hash::FxHashSet::default());
    }

    /// Apply every desired attribute except those named in `skip`.
    ///
    /// Animation-driven attributes are kept out of native writes this way: the
    /// declarative value stays authoritative in storage while the animation
    /// owns the live value on the node.
    pub(crate) fn apply_to_skipping(
        &self,
        node: &mut ArkUINode,
        tag: &str,
        skip: &rustc_hash::FxHashSet<String>,
    ) {
        for group in [
            AttrGroup::Control,
            AttrGroup::Layout,
            AttrGroup::Visual,
            AttrGroup::Text,
            AttrGroup::Image,
        ] {
            self.apply_group(node, tag, group, skip);
        }
    }

    fn apply_group(
        &self,
        node: &mut ArkUINode,
        tag: &str,
        group: AttrGroup,
        skip: &rustc_hash::FxHashSet<String>,
    ) {
        if group == AttrGroup::Layout {
            // Geometry must be committed before alignment. Dioxus stores
            // static attributes before dynamic ones, so insertion order is
            // unrelated to native layout dependencies (for example a static
            // `justify_content` can precede a dynamic height percent).
            for attr in self
                .attrs
                .iter()
                .filter(|attr| attr_group(attr.name()) == group)
                .filter(|attr| !is_box_attr(attr.name()))
                .filter(|attr| !is_constraint_attr(attr.name()))
                .filter(|attr| !is_flex_option_attr(attr.name()))
                .filter(|attr| !is_alignment_attr(attr.name()))
                .filter(|attr| !skip.contains(attr.name()))
            {
                let _ = attr.apply(node, tag);
            }
            self.apply_box(node, "padding", ArkUINodeAttributeType::Padding);
            self.apply_box(node, "margin", ArkUINodeAttributeType::Margin);
            self.apply_constraint_size(node);
            if tag == "flex" {
                self.apply_flex_option(node);
            }
            for attr in self
                .attrs
                .iter()
                .filter(|attr| attr_group(attr.name()) == group)
                .filter(|attr| is_alignment_attr(attr.name()))
                .filter(|attr| !skip.contains(attr.name()))
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
            .filter(|attr| !skip.contains(attr.name()))
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
            if is_box_attr(name) || is_flex_option_attr(name) || is_constraint_attr(name) {
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
        let wants_constraint = names.iter().any(|name| {
            matches!(
                *name,
                "constraint_size"
                    | "min_width"
                    | "max_width"
                    | "min_height"
                    | "max_height"
                    | "max_width_constraint"
            )
        });
        if wants_constraint {
            self.apply_constraint_size(node);
        }
        if wants_flex && tag == "flex" {
            self.apply_flex_option(node);
        }
        for attr in deferred {
            let _ = attr.apply(node, tag);
        }
    }

    pub(crate) fn apply_mutation(
        &self,
        node: &mut ArkUINode,
        tag: &str,
        name: &str,
        mutation: AttrMutation,
    ) {
        match mutation {
            AttrMutation::Unchanged => return,
            AttrMutation::Removed(ty) => {
                let _ = node.reset_attribute(ty);
            }
            AttrMutation::Set => {}
        }
        self.apply_named(node, tag, &[name]);
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
        // Cascade (CSS-like): shorthand → axis (x/y) → individual longhands.
        let mut sides = self
            .get(base)
            .and_then(|attr| match &attr.value {
                EncodedAttrValue::VecF32(v) if v.len() == 4 => Some([v[0], v[1], v[2], v[3]]),
                EncodedAttrValue::F32(v) => Some([*v, *v, *v, *v]),
                _ => None,
            })
            .unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let mut has_any = self.get(base).is_some();

        let axis_x = [format!("{base}_x"), format!("{base}_horizontal")];
        let axis_y = [format!("{base}_y"), format!("{base}_vertical")];
        for name in &axis_y {
            if let Some(v) = self.box_side_f32(name) {
                sides[0] = v;
                sides[2] = v;
                has_any = true;
            }
        }
        for name in &axis_x {
            if let Some(v) = self.box_side_f32(name) {
                sides[1] = v;
                sides[3] = v;
                has_any = true;
            }
        }

        for (name, index) in [
            (format!("{base}_top"), 0usize),
            (format!("{base}_right"), 1),
            (format!("{base}_bottom"), 2),
            (format!("{base}_left"), 3),
        ] {
            if let Some(v) = self.box_side_f32(&name) {
                sides[index] = v;
                has_any = true;
            }
        }

        if has_any {
            let _ = node.set_attribute(ty, vec![sides[0], sides[1], sides[2], sides[3]].into());
        }
    }

    fn box_side_f32(&self, name: &str) -> Option<f32> {
        match &self.get(name)?.value {
            EncodedAttrValue::F32(v) => Some(*v),
            EncodedAttrValue::VecF32(v) if v.len() == 1 => Some(v[0]),
            _ => None,
        }
    }

    fn apply_constraint_size(&self, node: &mut ArkUINode) {
        // ConstraintSize = [minWidth, maxWidth, minHeight, maxHeight]
        let mut values = [0.0_f32, 100_000.0, 0.0, 100_000.0];
        let mut has_any = false;
        if let Some(attr) = self.get("constraint_size") {
            if let EncodedAttrValue::VecF32(v) = &attr.value {
                if v.len() == 4 {
                    values = [v[0], v[1], v[2], v[3]];
                    has_any = true;
                }
            }
        }
        for (name, index) in [
            ("min_width", 0usize),
            ("max_width", 1),
            ("min_height", 2),
            ("max_height", 3),
            ("max_width_constraint", 1),
        ] {
            if let Some(attr) = self.get(name) {
                if let EncodedAttrValue::F32(v) = attr.value {
                    values[index] = v;
                    has_any = true;
                }
            }
        }
        if has_any {
            let _ = node.set_attribute(
                ArkUINodeAttributeType::ConstraintSize,
                vec![values[0], values[1], values[2], values[3]].into(),
            );
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

#[cfg(test)]
mod tests {
    use dioxus_core::AttributeValue;
    use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;

    use super::{
        encode_attr, parse_scroll_offset, parse_scroll_to_index, AttrMutation, DesiredAttrs,
        EncodedAttrValue, ListScrollToIndexCommand, ScrollOffsetCommand,
    };

    #[test]
    fn margin_padding_accept_css_shorthand() {
        let m = encode_attr("column", "margin", &AttributeValue::Text("8 16".into()))
            .expect("margin shorthand");
        assert_eq!(
            m.value,
            EncodedAttrValue::VecF32(vec![8.0, 16.0, 8.0, 16.0])
        );
        let p = encode_attr(
            "column",
            "padding",
            &AttributeValue::Text("8px 16vp 4 2".into()),
        )
        .expect("padding shorthand");
        assert_eq!(p.value, EncodedAttrValue::VecF32(vec![8.0, 16.0, 4.0, 2.0]));
        let mx = encode_attr("column", "margin_x", &AttributeValue::Float(12.0)).expect("margin_x");
        assert_eq!(mx.value, EncodedAttrValue::F32(12.0));
    }

    #[test]
    fn width_percent_string_maps_to_width_percent() {
        let w = encode_attr("column", "width", &AttributeValue::Text("50%".into()))
            .expect("width percent");
        assert_eq!(w.ty, ArkUINodeAttributeType::WidthPercent);
        assert_eq!(w.value, EncodedAttrValue::F32(0.5));
        let h = encode_attr("column", "height", &AttributeValue::Float(40.0)).expect("height vp");
        assert_eq!(h.ty, ArkUINodeAttributeType::Height);
        assert_eq!(h.value, EncodedAttrValue::F32(40.0));
    }

    #[test]
    fn text_and_visibility_keywords() {
        let a = encode_attr("text", "text_align", &AttributeValue::Text("center".into()))
            .expect("text_align");
        assert_eq!(a.value, EncodedAttrValue::I32(1));
        let v = encode_attr(
            "column",
            "visibility",
            &AttributeValue::Text("hidden".into()),
        )
        .expect("visibility");
        assert_eq!(v.value, EncodedAttrValue::I32(1));
        let fw = encode_attr("text", "font_weight", &AttributeValue::Text("bold".into()))
            .expect("font_weight bold");
        assert_eq!(fw.value, EncodedAttrValue::I32(6)); // 700 → index 6
        let of = encode_attr("image", "object_fit", &AttributeValue::Text("cover".into()))
            .expect("object_fit");
        assert_eq!(of.value, EncodedAttrValue::I32(1));
    }

    #[test]
    fn remaining_enum_keywords() {
        let d = encode_attr(
            "text",
            "text_decoration",
            &AttributeValue::Text("underline".into()),
        )
        .expect("decoration");
        assert_eq!(d.value, EncodedAttrValue::I32(1));
        let al = encode_attr("stack", "alignment", &AttributeValue::Text("center".into()))
            .expect("alignment");
        assert_eq!(al.value, EncodedAttrValue::I32(4));
        let se = encode_attr(
            "scroll",
            "scroll_edge_effect",
            &AttributeValue::Text("none".into()),
        )
        .expect("edge effect");
        assert_eq!(se.value, EncodedAttrValue::I32(2));
        let it = encode_attr(
            "textinput",
            "input_type",
            &AttributeValue::Text("password".into()),
        )
        .expect("input_type");
        assert_eq!(it.value, EncodedAttrValue::I32(7));
        let pt = encode_attr(
            "progress",
            "progress_type",
            &AttributeValue::Text("ring".into()),
        )
        .expect("progress_type");
        assert_eq!(pt.value, EncodedAttrValue::I32(1));
        let bt = encode_attr(
            "button",
            "button_type",
            &AttributeValue::Text("capsule".into()),
        )
        .expect("button_type");
        assert_eq!(bt.value, EncodedAttrValue::I32(1));
        let op = encode_attr("column", "opacity", &AttributeValue::Text("50%".into()))
            .expect("opacity percent");
        assert_eq!(op.value, EncodedAttrValue::F32(0.5));
        assert!(
            encode_attr("column", "shadow", &AttributeValue::Text("none".into())).is_none(),
            "shadow none is no-op"
        );
        let sh =
            encode_attr("column", "shadow", &AttributeValue::Text("sm".into())).expect("shadow sm");
        assert_eq!(sh.value, EncodedAttrValue::Shadow(1));
    }

    #[test]
    fn scroll_bar_accepts_css_keywords_only() {
        let off =
            encode_attr("scroll", "scroll_bar", &AttributeValue::Text("off".into())).expect("off");
        assert_eq!(off.ty, ArkUINodeAttributeType::ScrollBarDisplayMode);
        assert_eq!(off.value, EncodedAttrValue::I32(0));

        let on = encode_attr("list", "scroll_bar", &AttributeValue::Bool(true)).expect("bool true");
        assert_eq!(on.value, EncodedAttrValue::I32(2));

        let auto =
            encode_attr("grid", "scroll_bar", &AttributeValue::Text("auto".into())).expect("auto");
        assert_eq!(auto.value, EncodedAttrValue::I32(1));

        assert!(
            encode_attr("scroll", "scroll_bar", &AttributeValue::Int(2)).is_none(),
            "raw integers are rejected"
        );
    }

    #[test]
    fn scroll_direction_maps_to_native_axis() {
        let vertical = encode_attr(
            "scroll",
            "scroll_direction",
            &AttributeValue::Text("vertical".into()),
        )
        .expect("vertical");
        let horizontal = encode_attr(
            "scroll",
            "scroll_direction",
            &AttributeValue::Text("horizontal".into()),
        )
        .expect("horizontal");

        assert_eq!(vertical.ty, ArkUINodeAttributeType::ScrollScrollDirection);
        assert_eq!(vertical.value, EncodedAttrValue::I32(0));
        assert_eq!(horizontal.ty, ArkUINodeAttributeType::ScrollScrollDirection);
        assert_eq!(horizontal.value, EncodedAttrValue::I32(1));
    }

    #[test]
    fn scroll_offset_preserves_float_offsets_and_integer_options() {
        assert_eq!(
            parse_scroll_offset("12.5, 24.25, 0, 3"),
            Some((12.5, 24.25, vec![0, 3]))
        );
    }

    #[test]
    fn scroll_offset_rejects_invalid_or_excess_options() {
        assert_eq!(parse_scroll_offset("0"), None);
        assert_eq!(parse_scroll_offset("0,0,0,1,0,0,0,1"), None);
        assert_eq!(parse_scroll_offset("0,0,fast"), None);
    }

    #[test]
    fn scroll_offset_is_a_command_not_declarative_state() {
        let value = AttributeValue::Text("12.5,24.25,0".into());
        assert!(ScrollOffsetCommand::from_attribute(&value).is_some());

        let mut attrs = DesiredAttrs::default();
        assert!(matches!(
            attrs.set("scroll", "scroll_offset", &value),
            AttrMutation::Unchanged
        ));
        assert!(attrs.get("scroll_offset").is_none());
    }

    #[test]
    fn scroll_to_index_accepts_int_or_index_smooth_align_text() {
        assert_eq!(
            parse_scroll_to_index(&AttributeValue::Int(12)),
            Some((12, 0, 0))
        );
        assert_eq!(
            parse_scroll_to_index(&AttributeValue::Text("18,1,0".into())),
            Some((18, 1, 0))
        );
        assert_eq!(
            parse_scroll_to_index(&AttributeValue::Text("x".into())),
            None
        );
        assert_eq!(
            parse_scroll_to_index(&AttributeValue::Text("1,0,0,1".into())),
            None
        );
    }

    #[test]
    fn scroll_to_index_is_a_command_not_declarative_state() {
        let value = AttributeValue::Text("4,0,0".into());
        assert!(ListScrollToIndexCommand::from_attribute(&value).is_some());

        let mut attrs = DesiredAttrs::default();
        assert!(matches!(
            attrs.set("list", "scroll_to_index", &value),
            AttrMutation::Unchanged
        ));
        assert!(attrs.get("scroll_to_index").is_none());
    }

    #[test]
    fn loading_progress_attributes_use_native_types() {
        let color = encode_attr(
            "loadingprogress",
            "loading_progress_color",
            &AttributeValue::Int(i64::from(0xFF00_7DFF_u32)),
        )
        .expect("loading color is supported");
        assert_eq!(color.ty, ArkUINodeAttributeType::LoadingProgressColor);
        assert_eq!(color.value, EncodedAttrValue::U32(0xFF00_7DFF));

        let enabled = encode_attr(
            "loadingprogress",
            "loading_progress_enable_loading",
            &AttributeValue::Bool(false),
        )
        .expect("loading state is supported");
        assert_eq!(
            enabled.ty,
            ArkUINodeAttributeType::LoadingProgressEnableLoading
        );
        assert_eq!(enabled.value, EncodedAttrValue::Bool(false));
    }

    #[test]
    fn text_input_otp_attributes_use_native_types() {
        let input_type = encode_attr(
            "textinput",
            "input_type",
            &AttributeValue::Text("otp".into()),
        )
        .expect("one-time-code input type is supported");
        assert_eq!(input_type.ty, ArkUINodeAttributeType::TextInputType);
        assert_eq!(input_type.value, EncodedAttrValue::I32(14));
        assert!(
            encode_attr("textinput", "input_type", &AttributeValue::Int(14)).is_none(),
            "raw input_type integers are rejected"
        );

        let input_filter = encode_attr(
            "textinput",
            "input_filter",
            &AttributeValue::Text("[0-9]".into()),
        )
        .expect("text input filter is supported");
        assert_eq!(
            input_filter.ty,
            ArkUINodeAttributeType::TextInputInputFilter
        );
        assert_eq!(input_filter.value, EncodedAttrValue::String("[0-9]".into()));

        let show_password_icon = encode_attr(
            "textinput",
            "show_password_icon",
            &AttributeValue::Bool(true),
        )
        .expect("password visibility icon is supported");
        assert_eq!(
            show_password_icon.ty,
            ArkUINodeAttributeType::TextInputShowPasswordIcon
        );
        assert_eq!(show_password_icon.value, EncodedAttrValue::Bool(true));

        let max_length = encode_attr("textinput", "max_length", &AttributeValue::Int(6))
            .expect("text input max length is supported");
        assert_eq!(max_length.ty, ArkUINodeAttributeType::TextInputMaxLength);
        assert_eq!(max_length.value, EncodedAttrValue::I32(6));
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
            | "padding_x"
            | "padding_y"
            | "padding_horizontal"
            | "padding_vertical"
            | "margin"
            | "margin_top"
            | "margin_right"
            | "margin_bottom"
            | "margin_left"
            | "margin_x"
            | "margin_y"
            | "margin_horizontal"
            | "margin_vertical"
    ) || name.starts_with("padding_")
        || name.starts_with("margin_")
}

fn is_constraint_attr(name: &str) -> bool {
    matches!(
        name,
        "constraint_size"
            | "min_width"
            | "max_width"
            | "min_height"
            | "max_height"
            | "max_width_constraint"
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
        "focusable" | "focus_on_touch" | "focused" | "focus_status" | "enabled" => {
            AttrGroup::Control
        }
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
        | "foreground_color"
        | "progress_color"
        | "loading_progress_color"
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
    let as_f32 = |v: &dioxus_core::AttributeValue| {
        parse_vp(v).or_else(|| match v {
            dioxus_core::AttributeValue::Float(f) => Some(*f as f32),
            dioxus_core::AttributeValue::Int(i) => Some(*i as f32),
            dioxus_core::AttributeValue::Text(s) => s.parse::<f32>().ok(),
            _ => None,
        })
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
        dioxus_core::AttributeValue::Text(s) => match css_value::enum_token(s).as_str() {
            "true" | "yes" | "on" => Some(true),
            "false" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    };
    let as_string = |v: &dioxus_core::AttributeValue| match v {
        dioxus_core::AttributeValue::Text(s) => Some(s.clone()),
        dioxus_core::AttributeValue::Float(f) => Some(f.to_string()),
        dioxus_core::AttributeValue::Int(i) => Some(i.to_string()),
        dioxus_core::AttributeValue::Bool(b) => Some(b.to_string()),
        _ => None,
    };
    let as_color = |v: &dioxus_core::AttributeValue| {
        parse_css_color(v).or_else(|| match v {
            dioxus_core::AttributeValue::Text(s) => parse_color(s).ok(),
            dioxus_core::AttributeValue::Int(i) => Some(*i as u32),
            _ => None,
        })
    };
    let as_radius_vec =
        |v: &dioxus_core::AttributeValue| expand_box_shorthand(v).map(|sides| sides.to_vec());
    let as_box_vec =
        |v: &dioxus_core::AttributeValue| expand_box_shorthand(v).map(|sides| sides.to_vec());

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
            let raw = css_value::font_weight_value(value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FontWeight,
                EncodedAttrValue::I32(map_font_weight_to_arkui(raw)),
            )
        }
        "font_style" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::FontStyle,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::font_style_keyword)?),
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
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::text_align_keyword)?),
        ),
        "text_letter_spacing" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextLetterSpacing,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "text_decoration" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextDecoration,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::text_decoration_keyword)?),
        ),
        "text_overflow" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextOverflow,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::text_overflow_keyword)?),
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
        "input_type" if tag == "textinput" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextInputType,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::input_type_keyword)?),
        ),
        "input_filter" if tag == "textinput" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextInputInputFilter,
            EncodedAttrValue::String(as_string(value)?),
        ),
        "show_password_icon" if tag == "textinput" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextInputShowPasswordIcon,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "max_length" if tag == "textinput" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::TextInputMaxLength,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "padding" => {
            let sides = expand_box_shorthand(value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Padding,
                EncodedAttrValue::VecF32(sides.to_vec()),
            )
        }
        "padding_top" | "padding_right" | "padding_bottom" | "padding_left" | "padding_x"
        | "padding_y" | "padding_horizontal" | "padding_vertical" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Padding,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        "margin" => {
            let sides = expand_box_shorthand(value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Margin,
                EncodedAttrValue::VecF32(sides.to_vec()),
            )
        }
        "margin_top" | "margin_right" | "margin_bottom" | "margin_left" | "margin_x"
        | "margin_y" | "margin_horizontal" | "margin_vertical" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Margin,
            EncodedAttrValue::F32(as_f32(value)?),
        ),
        // CSS-only sizing: use `width`/`height` with vp or `"N%"` — no percent_* attrs.
        // CSS-like: width: 100 / "100px" → vp; width: "50%" → percent.
        "width" => match parse_length(value)? {
            CssLength::Vp(v) => EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Width,
                EncodedAttrValue::F32(v),
            ),
            CssLength::Percent(p) => EncodedAttr::new(
                name,
                ArkUINodeAttributeType::WidthPercent,
                EncodedAttrValue::F32(p),
            ),
        },
        "height" => match parse_length(value)? {
            CssLength::Vp(v) => EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Height,
                EncodedAttrValue::F32(v),
            ),
            CssLength::Percent(p) => EncodedAttr::new(
                name,
                ArkUINodeAttributeType::HeightPercent,
                EncodedAttrValue::F32(p),
            ),
        },
        "min_width" | "max_width" | "min_height" | "max_height" | "max_width_constraint" => {
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::ConstraintSize,
                EncodedAttrValue::F32(as_f32(value)?),
            )
        }
        "constraint_size" => {
            let values =
                expand_box_shorthand(value)
                    .map(|s| s.to_vec())
                    .or_else(|| match value {
                        dioxus_core::AttributeValue::Text(s) => parse_f32_list(s),
                        _ => None,
                    })?;
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
            EncodedAttrValue::F32(parse_opacity(value).or_else(|| as_f32(value))?),
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
                    let parts = css_value::split_css_list(s);
                    let nums: Option<Vec<f32>> =
                        parts.into_iter().map(css_value::parse_vp_number).collect();
                    let nums = nums.or_else(|| parse_f32_list(s))?;
                    match nums.as_slice() {
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
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::border_style_keyword)?),
        ),
        "shadow" => {
            let v = i32_or_keyword(value, css_value::shadow_keyword)?;
            if v < 0 {
                // "none" — treat as no-op encode so callers can pass the keyword
                return None;
            }
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::Shadow,
                EncodedAttrValue::Shadow(v),
            )
        }
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
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::visibility_keyword)?),
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
        // ArkUI NODE_FOCUS_STATUS: 1 = request focus.
        // Only encode when true — writing 0 on every idle frame would steal
        // focus from sibling inputs (SSH form, etc.).
        "focused" | "focus_status" => {
            if as_bool(value)? {
                EncodedAttr::new(
                    name,
                    ArkUINodeAttributeType::FocusStatus,
                    EncodedAttrValue::I32(1),
                )
            } else {
                return None;
            }
        }
        "hit_test_behavior" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::HitTestBehavior,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::hit_test_keyword)?),
        ),
        "alignment" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::Alignment,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::alignment_keyword)?),
        ),
        "align_self" | "item_alignment" => {
            let v = i32_or_keyword(value, |s| {
                css_value::flex_align_items_keyword(s).or_else(|| item_alignment_value(s))
            })?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::AlignSelf,
                EncodedAttrValue::I32(v),
            )
        }
        "align_items" => {
            if tag == "flex" {
                let v = i32_or_keyword(value, |s| {
                    css_value::flex_align_items_keyword(s).or_else(|| item_alignment_value(s))
                })?;
                EncodedAttr::new(
                    name,
                    ArkUINodeAttributeType::FlexOption,
                    EncodedAttrValue::FlexOptionPart(3, v),
                )
            } else {
                let attr = align_items_attr(tag)?;
                let v = i32_or_keyword(value, |s| align_items_value(tag, s))?;
                EncodedAttr::new(name, attr, EncodedAttrValue::I32(v))
            }
        }
        "justify_content" => {
            let v = i32_or_keyword(value, justify_content_value)?;
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
            let v = i32_or_keyword(value, flex_direction_value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(0, v),
            )
        }
        "flex_wrap" => {
            let v = i32_or_keyword(value, flex_wrap_value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(1, v),
            )
        }
        "flex_align_content" => {
            let v = i32_or_keyword(value, justify_content_value)?;
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::FlexOption,
                EncodedAttrValue::FlexOptionPart(4, v),
            )
        }
        "button_type" if tag == "button" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ButtonType,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::button_type_keyword)?),
        ),
        // ArkUI ScrollBarDisplayMode: Off=0, Auto=1, On=2.
        "scroll_bar" if matches!(tag, "scroll" | "list" | "grid" | "waterflow") => {
            EncodedAttr::new(
                name,
                ArkUINodeAttributeType::ScrollBarDisplayMode,
                EncodedAttrValue::I32(scroll_bar_display_mode(value)?),
            )
        }
        "scroll_direction" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollScrollDirection,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::scroll_direction_keyword)?),
        ),
        "scroll_enabled" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollEnableScrollInteraction,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "scroll_edge_effect" if tag == "scroll" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::ScrollEdgeEffect,
            EncodedAttrValue::I32(i32_or_keyword(
                value,
                css_value::scroll_edge_effect_keyword,
            )?),
        ),
        "swiper_index" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperIndex,
            EncodedAttrValue::I32(as_i32(value)?),
        ),
        "swiper_swipe_to_index" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperSwipeToIndex,
            EncodedAttrValue::VecI32(vec![as_i32(value)?, 1]),
        ),
        "swiper_loop" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperLoop,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "swiper_auto_play" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperAutoPlay,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "swiper_show_indicator" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperShowIndicator,
            EncodedAttrValue::Bool(as_bool(value)?),
        ),
        "swiper_disable_swipe" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperDisableSwipe,
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
        "swiper_curve" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperCurve,
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::animation_curve_keyword)?),
        ),
        "swiper_item_space" if tag == "swiper" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::SwiperItemSpace,
            EncodedAttrValue::F32(as_f32(value)?),
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
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::list_sticky_keyword)?),
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
            EncodedAttrValue::I32(i32_or_keyword(value, object_fit_value)?),
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
            EncodedAttrValue::I32(i32_or_keyword(value, css_value::progress_type_keyword)?),
        ),
        "loading_progress_enable_loading" if tag == "loadingprogress" => EncodedAttr::new(
            name,
            ArkUINodeAttributeType::LoadingProgressEnableLoading,
            EncodedAttrValue::Bool(as_bool(value)?),
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

fn parse_scroll_to_index(value: &dioxus_core::AttributeValue) -> Option<(i32, i32, i32)> {
    match value {
        dioxus_core::AttributeValue::Int(index) => Some((*index as i32, 0, 0)),
        dioxus_core::AttributeValue::Float(index) => Some((*index as i32, 0, 0)),
        dioxus_core::AttributeValue::Text(text) => {
            let mut fields = text.split(',').map(str::trim);
            let index = fields.next()?.parse().ok()?;
            let smooth = match fields.next() {
                Some(field) => field.parse().ok()?,
                None => 0,
            };
            let align = match fields.next() {
                Some(field) => field.parse().ok()?,
                None => 0,
            };
            if fields.next().is_some() {
                return None;
            }
            Some((index, smooth, align))
        }
        _ => None,
    }
}

fn parse_scroll_offset(value: &str) -> Option<(f32, f32, Vec<i32>)> {
    let mut fields = value.split(',').map(str::trim);
    let x = fields.next()?.parse().ok()?;
    let y = fields.next()?.parse().ok()?;
    let options = fields
        .map(str::parse::<i32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if options.len() > 5 {
        return None;
    }
    Some((x, y, options))
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
        "loading_progress_color" if tag == "loadingprogress" => {
            ArkUINodeAttributeType::LoadingProgressColor
        }
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

fn align_items_value(tag: &str, s: &str) -> Option<i32> {
    let lower = css_value::enum_token(s);
    match tag {
        // Column/Row AlignItems enum is distinct from Flex AlignSelf.
        "column" | "row" => match lower.as_str() {
            "start" | "top" | "left" | "flex_start" => Some(0),
            "center" => Some(1),
            "end" | "bottom" | "right" | "flex_end" => Some(2),
            // stretch not always available; map closest to start for safety
            "stretch" => Some(0),
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
    match css_value::enum_token(s).as_str() {
        "start" | "flex_start" | "left" | "top" => Some(1),
        "center" => Some(2),
        "end" | "flex_end" | "right" | "bottom" => Some(3),
        "space_between" | "spacebetween" => Some(6),
        "space_around" | "spacearound" => Some(7),
        "space_evenly" | "spaceevenly" => Some(8),
        _ => None,
    }
}

fn item_alignment_value(s: &str) -> Option<i32> {
    css_value::flex_align_items_keyword(s)
}

fn flex_direction_value(s: &str) -> Option<i32> {
    match css_value::enum_token(s).as_str() {
        "row" | "horizontal" => Some(0),
        "column" | "vertical" => Some(1),
        "row_reverse" | "rowreverse" => Some(2),
        "column_reverse" | "columnreverse" => Some(3),
        _ => None,
    }
}

fn flex_wrap_value(s: &str) -> Option<i32> {
    match css_value::enum_token(s).as_str() {
        "nowrap" | "no_wrap" | "false" | "off" => Some(0),
        "wrap" | "true" | "on" => Some(1),
        "wrap_reverse" | "wrapreverse" => Some(2),
        _ => None,
    }
}

/// Map declarative `scroll_bar` values to ArkUI `ScrollBarDisplayMode`.
fn scroll_bar_display_mode(value: &dioxus_core::AttributeValue) -> Option<i32> {
    i32_or_keyword(value, css_value::scroll_bar_keyword)
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
        if !self.has_any(&["height", "constraint_size"]) {
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
