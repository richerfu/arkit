//! Native ECharts-compatible charts for the Arkit Dioxus renderer.
//!
//! The crate is split into four layers: option/data model, JSON parser,
//! atomic rendering capabilities, and the Dioxus host component. Series
//! renderers are independent compositions over shared canvas, geometry, and
//! hit-testing atoms.

mod animation;
mod component;
mod export;
mod model;
mod parser;
mod registry;
mod render;
mod state;

pub use component::{ChartController, ECharts, EChartsProps};
pub use model::*;
pub use registry::{register_map, register_map_str, unregister_map, MapRegistrationError};
pub use render::hit_test;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn typed_api_covers_every_supported_series_family() {
        let nodes = vec![NodeData {
            name: String::from("node"),
            value: 1.0,
            x: None,
            y: None,
            category: None,
            symbol_size: None,
            symbol_size_dimensions: None,
            symbol: None,
            symbol_rotate: 0.0,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: std::collections::BTreeMap::new(),
        }];
        let option = ChartOption::new().series([
            Series::line("line", [1.0]),
            Series::bar("bar", [1.0]),
            Series::pie("pie", [DataPoint::named("slice", 1.0)]),
            Series::scatter("scatter", [DataPoint::values([1.0, 2.0])]),
            Series::effect_scatter("effect", [DataPoint::values([1.0, 2.0])]),
            Series::radar("radar", [1.0]),
            Series::gauge("gauge", 1.0),
            Series::funnel("funnel", [DataPoint::named("stage", 1.0)]),
            Series::heatmap("heatmap", [DataPoint::values([0.0, 0.0, 1.0])]),
            Series::candlestick("candlestick", [DataPoint::values([1.0, 2.0, 0.5, 2.5])]),
            Series::boxplot("boxplot", [DataPoint::values([0.5, 1.0, 1.5, 2.0, 2.5])]),
            Series::pictorial_bar("pictorial", [1.0]),
            Series::parallel("parallel", [DataPoint::values([1.0, 2.0, 3.0])]),
            Series::theme_river("river", [DataPoint::values(["2026-01-01", "1", "A"])]),
            Series::tree("tree", nodes.clone(), Vec::new()),
            Series::treemap("treemap", [DataPoint::named("area", 1.0)]),
            Series::graph("graph", nodes.clone(), Vec::new()),
            Series::sankey("sankey", nodes, Vec::new()),
            Series::map(
                "map",
                vec![MapFeature::new(
                    "feature",
                    vec![MapPolygon::new([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)])],
                )
                .with_value(1.0)],
            ),
            Series::lines(
                "lines",
                vec![LineSegment {
                    name: Some(String::from("route")),
                    from: (0.0, 0.0),
                    to: (1.0, 1.0),
                    coords: vec![(0.0, 0.0), (1.0, 1.0)],
                    value: 1.0,
                }],
            ),
            Series::sunburst(
                "sunburst",
                vec![SunburstNode {
                    name: String::from("root"),
                    value: 1.0,
                    children: Vec::new(),
                    item_style: ItemStyle::default(),
                }],
            ),
            Series::custom("custom", Vec::new(), |_| {}),
        ]);

        assert_eq!(option.series.len(), 22);
    }
}
