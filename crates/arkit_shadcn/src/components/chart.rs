//! Chart — shadcn-style series chart.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Renders a card-surfaced list of series rows; each row shows a muted
//! `Series N` label, a foreground percent value, and a thin `progress` bar
//! colored from the theme's `chart_1..chart_5` palette. Series are stacked with
//! `XXL` vertical gap (legacy `card` → `stack(children, XXL)`); the progress
//! bar uses `rounded_progress` styling (`full` radius, `secondary` track,
//! clipped, 8px tall).

use crate::{i18n::use_component_i18n, theme::*};
use arkit_prelude::*;

/// Props for [`Chart`].
#[derive(Props, Clone, PartialEq)]
pub struct ChartProps {
    pub values: Vec<f32>,
    /// Optional caller-provided labels by series index. Missing entries use
    /// the active i18n locale's built-in `Series N` equivalent.
    #[props(default)]
    pub series_labels: Vec<String>,
}

/// A list of series progress bars on a card surface.
#[component]
pub fn Chart(props: ChartProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let palette = [
        theme.colors.chart_1,
        theme.colors.chart_2,
        theme.colors.chart_3,
        theme.colors.chart_4,
        theme.colors.chart_5,
    ];

    let rows: Vec<Element> = props
        .values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let percent = value.clamp(0.0, 100.0);
            let tone = palette[idx % palette.len()];
            let label = props
                .series_labels
                .get(idx)
                .filter(|label| !label.is_empty())
                .cloned()
                .unwrap_or_else(|| i18n.chart_series(idx + 1));
            let pct = format!("{percent:.0}%");
            let margin_top = if idx == 0 { 0.0 } else { spacing::XXL };
            rsx! {
                row {
                    width: "100%",
                    margin_top: margin_top,
                    column {
                        width: "100%",
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "space_between",
                            text {
                                content: label,
                                font_size: typography::SM,
                                font_color: theme.colors.muted_foreground,
                                line_height: 20.0,
                            }
                            text {
                                content: pct,
                                font_size: typography::MD,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                        }
                        row {
                            margin_top: spacing::XXS,
                            progress {
                                progress_value: percent,
                                progress_total: 100.0,
                                progress_color: tone,
                                height: 8.0,
                                border_radius: theme.radii.full,
                                background_color: theme.colors.secondary,
                                clip: true,
                            }
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            background_color: theme.colors.card,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.xl,
            shadow: "sm",
            padding_top: 0.0,
            padding_right: spacing::XXL,
            padding_bottom: 0.0,
            padding_left: spacing::XXL,
            {rows.into_iter()}
        }
    }
}

/// Props for [`ChartCard`].
#[derive(Props, Clone, PartialEq)]
pub struct ChartCardProps {
    pub title: String,
    pub values: Vec<f32>,
    #[props(default)]
    pub series_labels: Vec<String>,
}

/// A titled chart card — a heading above a [`Chart`].
#[component]
pub fn ChartCard(props: ChartCardProps) -> Element {
    let theme = use_theme();
    let title = props.title.clone();
    rsx! {
        column {
            width: "100%",
            background_color: theme.colors.card,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.xl,
            shadow: "sm",
            padding_top: 0.0,
            padding_right: spacing::XXL,
            padding_bottom: 0.0,
            padding_left: spacing::XXL,
            text {
                content: title,
                font_size: typography::LG,
                font_weight: 600,
                font_color: theme.colors.foreground,
                line_height: 20.0,
            }
            row {
                width: "100%",
                margin_top: spacing::XXL,
                Chart {
                    values: props.values,
                    series_labels: props.series_labels,
                }
            }
        }
    }
}
