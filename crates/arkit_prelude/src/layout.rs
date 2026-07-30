//! Start-aligned alternatives to the native ArkUI row and column elements.

use dioxus_core::{Attribute, AttributeValue};

use crate::*;

const START: &str = "start";
const ALIGN_ITEMS: &str = "align_items";
const JUSTIFY_CONTENT: &str = "justify_content";

/// Props for [`Col`].
///
/// Every native [`column`] attribute is forwarded to the underlying node. When
/// omitted, `align_items` and `justify_content` both default to `"start"`.
#[derive(Props, Clone, PartialEq)]
pub struct ColProps {
    #[props(extends = column)]
    attributes: Vec<Attribute>,
    children: Element,
}

/// A native ArkUI [`column`] whose two axes start-aligned by default.
///
/// Callers can still override either axis with the normal `align_items` and
/// `justify_content` attributes.
///
/// ```ignore
/// rsx! {
///     Col {
///         width: "100%",
///         padding: 16.0,
///         text { content: "Top left" }
///     }
/// }
/// ```
#[component]
pub fn Col(props: ColProps) -> Element {
    let attributes = with_start_alignment(props.attributes);
    rsx! {
        column {
            ..attributes,
            {props.children}
        }
    }
}

/// Props for [`Row`].
///
/// Every native [`row`] attribute is forwarded to the underlying node. When
/// omitted, `align_items` and `justify_content` both default to `"start"`.
#[derive(Props, Clone, PartialEq)]
pub struct RowProps {
    #[props(extends = row)]
    attributes: Vec<Attribute>,
    children: Element,
}

/// A native ArkUI [`row`] whose two axes start-aligned by default.
///
/// Callers can still override either axis with the normal `align_items` and
/// `justify_content` attributes.
///
/// ```ignore
/// rsx! {
///     Row {
///         width: "100%",
///         text { content: "Leading" }
///     }
/// }
/// ```
#[component]
pub fn Row(props: RowProps) -> Element {
    let attributes = with_start_alignment(props.attributes);
    rsx! {
        row {
            ..attributes,
            {props.children}
        }
    }
}

fn with_start_alignment(mut attributes: Vec<Attribute>) -> Vec<Attribute> {
    ensure_attribute(&mut attributes, ALIGN_ITEMS, START);
    ensure_attribute(&mut attributes, JUSTIFY_CONTENT, START);
    attributes
}

fn ensure_attribute(attributes: &mut Vec<Attribute>, name: &'static str, default: &'static str) {
    let has_value = attributes
        .iter()
        .any(|attribute| attribute.name == name && attribute.value != AttributeValue::None);
    if has_value {
        return;
    }

    attributes.retain(|attribute| attribute.name != name);
    attributes.push(Attribute::new(name, default, None, false));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_both_axes_to_start() {
        let attributes = with_start_alignment(Vec::new());

        assert_eq!(text_value(&attributes, ALIGN_ITEMS), Some(START));
        assert_eq!(text_value(&attributes, JUSTIFY_CONTENT), Some(START));
    }

    #[test]
    fn preserves_explicit_alignment() {
        let attributes = with_start_alignment(vec![
            Attribute::new(ALIGN_ITEMS, "center", None, false),
            Attribute::new(JUSTIFY_CONTENT, "space_between", None, false),
        ]);

        assert_eq!(text_value(&attributes, ALIGN_ITEMS), Some("center"));
        assert_eq!(
            text_value(&attributes, JUSTIFY_CONTENT),
            Some("space_between")
        );
    }

    #[test]
    fn none_alignment_falls_back_to_start() {
        let attributes = with_start_alignment(vec![
            Attribute::new(ALIGN_ITEMS, AttributeValue::None, None, false),
            Attribute::new(JUSTIFY_CONTENT, AttributeValue::None, None, false),
        ]);

        assert_eq!(text_value(&attributes, ALIGN_ITEMS), Some(START));
        assert_eq!(text_value(&attributes, JUSTIFY_CONTENT), Some(START));
    }

    fn text_value<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a str> {
        attributes.iter().find_map(|attribute| {
            if attribute.name != name {
                return None;
            }
            match &attribute.value {
                AttributeValue::Text(value) => Some(value.as_str()),
                _ => None,
            }
        })
    }
}
