use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};

pub(crate) fn numbers(item: ArkUINodeAttributeItem) -> Option<Vec<f32>> {
    let values = match item {
        ArkUINodeAttributeItem::NumberValue(values) => values,
        ArkUINodeAttributeItem::Composite(value) => value.number_values,
        ArkUINodeAttributeItem::String(_) | ArkUINodeAttributeItem::Object(_) => return None,
    };
    Some(
        values
            .into_iter()
            .map(|value| match value {
                ArkUINodeAttributeNumber::Float(value) => value,
                ArkUINodeAttributeNumber::Int(value) => value as f32,
                ArkUINodeAttributeNumber::Uint(value) => value as f32,
            })
            .collect(),
    )
}

pub(crate) fn first_f32(item: ArkUINodeAttributeItem) -> Option<f32> {
    numbers(item)?.first().copied()
}

pub(crate) fn first_u32(item: ArkUINodeAttributeItem) -> Option<u32> {
    match item {
        ArkUINodeAttributeItem::NumberValue(values) => values.first().map(|value| match value {
            ArkUINodeAttributeNumber::Float(value) => *value as u32,
            ArkUINodeAttributeNumber::Int(value) => *value as u32,
            ArkUINodeAttributeNumber::Uint(value) => *value,
        }),
        ArkUINodeAttributeItem::Composite(value) => {
            value.number_values.first().map(|value| match value {
                ArkUINodeAttributeNumber::Float(value) => *value as u32,
                ArkUINodeAttributeNumber::Int(value) => *value as u32,
                ArkUINodeAttributeNumber::Uint(value) => *value,
            })
        }
        ArkUINodeAttributeItem::String(_) | ArkUINodeAttributeItem::Object(_) => None,
    }
}
