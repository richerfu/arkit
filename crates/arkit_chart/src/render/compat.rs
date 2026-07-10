//! Normalization helpers for ECharts JSON values retained by the model.

use std::collections::BTreeMap;

use serde_json::Value;

use super::geometry::Plot;

pub(super) fn number(options: &BTreeMap<String, Value>, key: &str, default: f64) -> f64 {
    options.get(key).and_then(Value::as_f64).unwrap_or(default)
}

pub(super) fn boolean(options: &BTreeMap<String, Value>, key: &str, default: bool) -> bool {
    options.get(key).and_then(Value::as_bool).unwrap_or(default)
}

pub(super) fn string<'a>(options: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    options.get(key).and_then(Value::as_str)
}

pub(super) fn length(value: Option<&Value>, total: f32, default: f32) -> f32 {
    match value {
        Some(Value::Number(value)) => value.as_f64().map(|value| value as f32).unwrap_or(default),
        Some(Value::String(value)) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<f32>()
            .map(|value| total * value / 100.0)
            .unwrap_or(default),
        Some(Value::String(value)) => value.parse().unwrap_or(default),
        _ => default,
    }
}

pub(super) fn position(value: Option<&Value>, start: f32, total: f32, default: f32) -> f32 {
    start + length(value, total, default - start)
}

pub(super) fn pair<'a>(options: &'a BTreeMap<String, Value>, key: &str) -> Option<[&'a Value; 2]> {
    let values = options.get(key)?.as_array()?;
    Some([values.first()?, values.get(1)?])
}

pub(super) fn inset_plot(
    options: &BTreeMap<String, Value>,
    plot: Plot,
    defaults: [f32; 4],
) -> Plot {
    let left = length(options.get("left"), plot.width, defaults[0]);
    let top = length(options.get("top"), plot.height, defaults[1]);
    let right = length(options.get("right"), plot.width, defaults[2]);
    let bottom = length(options.get("bottom"), plot.height, defaults[3]);
    Plot {
        x: plot.x + left,
        y: plot.y + top,
        width: (plot.width - left - right).max(1.0),
        height: (plot.height - top - bottom).max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_length_uses_supplied_extent() {
        assert_eq!(
            length(Some(&Value::String("75%".into())), 200.0, 0.0),
            150.0
        );
    }

    #[test]
    fn inset_plot_resolves_echarts_box_positions() {
        let options = BTreeMap::from([
            (String::from("left"), Value::String(String::from("10%"))),
            (String::from("right"), Value::from(20)),
        ]);
        let plot = inset_plot(
            &options,
            Plot {
                x: 5.0,
                y: 10.0,
                width: 200.0,
                height: 100.0,
            },
            [0.0, 5.0, 0.0, 5.0],
        );
        assert_eq!(plot.x, 25.0);
        assert_eq!(plot.y, 15.0);
        assert_eq!(plot.width, 160.0);
        assert_eq!(plot.height, 90.0);
    }
}
