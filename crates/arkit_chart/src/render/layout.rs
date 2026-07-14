//! Reusable deterministic layout algorithms for free-coordinate series.

use super::geometry::Plot;
use crate::model::{ChartOption, LinkData, NodeData};

pub(super) fn grid_plot(option: &ChartOption, index: usize, width: f32, height: f32) -> Plot {
    let grid = option.grid.get(index).cloned().unwrap_or_default();
    let left = grid.left.resolve(width);
    let right = grid.right.resolve(width);
    let top = grid.top.resolve(height);
    let bottom = grid.bottom.resolve(height);
    let mut plot = Plot {
        x: left,
        y: top,
        width: grid
            .width
            .map(|value| value.resolve(width))
            .unwrap_or(width - left - right)
            .max(1.0),
        height: grid
            .height
            .map(|value| value.resolve(height))
            .unwrap_or(height - top - bottom)
            .max(1.0),
    };
    if grid.contain_label {
        plot.x += 38.0;
        plot.width = (plot.width - 38.0).max(1.0);
        plot.height = (plot.height - 22.0).max(1.0);
    }
    plot
}

pub(super) fn squarify(weights: &[f64], plot: Plot) -> Vec<Plot> {
    let mut output = vec![
        Plot {
            x: plot.x,
            y: plot.y,
            width: 0.0,
            height: 0.0,
        };
        weights.len()
    ];
    let total: f64 = weights.iter().map(|value| value.max(0.0)).sum();
    if weights.is_empty() || total <= 0.0 || plot.width <= 0.0 || plot.height <= 0.0 {
        return output;
    }

    let scale = plot.width as f64 * plot.height as f64 / total;
    let mut items: Vec<(usize, f64)> = weights
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value > 0.0).then_some((index, *value * scale)))
        .collect();
    items.sort_by(|left, right| right.1.total_cmp(&left.1));
    let mut remaining = plot;
    let mut row: Vec<(usize, f64)> = Vec::new();

    while let Some(item) = items.first().copied() {
        let side = remaining.width.min(remaining.height).max(1.0) as f64;
        if row.is_empty() || worst_ratio(&row, side) >= worst_ratio_with(&row, item, side) {
            row.push(item);
            items.remove(0);
        } else {
            layout_row(&row, &mut remaining, &mut output);
            row.clear();
        }
    }
    if !row.is_empty() {
        layout_row(&row, &mut remaining, &mut output);
    }
    output
}

pub(super) fn tree_layout(
    node_count: usize,
    links: &[LinkData],
    plot: Plot,
    orientation: &str,
) -> Vec<(f32, f32)> {
    if node_count == 0 {
        return Vec::new();
    }
    let mut indegree = vec![0usize; node_count];
    let mut children = vec![Vec::new(); node_count];
    for link in links {
        if link.source < node_count && link.target < node_count {
            indegree[link.target] += 1;
            children[link.source].push(link.target);
        }
    }
    let mut depths = vec![0usize; node_count];
    let mut queue: std::collections::VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();
    if queue.is_empty() {
        queue.push_back(0);
    }
    let mut visited = vec![false; node_count];
    while let Some(node) = queue.pop_front() {
        visited[node] = true;
        for child in &children[node] {
            depths[*child] = depths[*child].max(depths[node] + 1);
            indegree[*child] = indegree[*child].saturating_sub(1);
            if indegree[*child] == 0 {
                queue.push_back(*child);
            }
        }
    }
    for index in 0..node_count {
        if !visited[index] {
            depths[index] = 0;
        }
    }
    let level_count = depths.iter().copied().max().unwrap_or(0) + 1;
    let mut levels = vec![Vec::new(); level_count];
    for (node, depth) in depths.into_iter().enumerate() {
        levels[depth].push(node);
    }
    let mut positions = vec![(plot.x, plot.y); node_count];
    for (depth, nodes) in levels.iter().enumerate() {
        for (offset, node) in nodes.iter().enumerate() {
            let main = if level_count <= 1 {
                0.5
            } else {
                depth as f32 / (level_count - 1) as f32
            };
            let cross = (offset as f32 + 0.5) / nodes.len().max(1) as f32;
            positions[*node] = match orientation {
                "RL" => (
                    plot.x + plot.width * (1.0 - main),
                    plot.y + plot.height * cross,
                ),
                "TB" => (plot.x + plot.width * cross, plot.y + plot.height * main),
                "BT" => (
                    plot.x + plot.width * cross,
                    plot.y + plot.height * (1.0 - main),
                ),
                _ => (plot.x + plot.width * main, plot.y + plot.height * cross),
            };
        }
    }
    positions
}

pub(super) fn circular_layout(node_count: usize, plot: Plot) -> Vec<(f32, f32)> {
    let radius = plot.width.min(plot.height) * 0.38;
    (0..node_count)
        .map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2
                + std::f32::consts::TAU * index as f32 / node_count.max(1) as f32;
            (
                plot.x + plot.width / 2.0 + angle.cos() * radius,
                plot.y + plot.height / 2.0 + angle.sin() * radius,
            )
        })
        .collect()
}

pub(super) fn positioned_graph_layout(nodes: &[NodeData], plot: Plot) -> Option<Vec<(f32, f32)>> {
    if !nodes
        .iter()
        .all(|node| node.x.is_some() && node.y.is_some())
    {
        return None;
    }
    let min_x = nodes.iter().filter_map(|node| node.x).reduce(f64::min)?;
    let max_x = nodes.iter().filter_map(|node| node.x).reduce(f64::max)?;
    let min_y = nodes.iter().filter_map(|node| node.y).reduce(f64::min)?;
    let max_y = nodes.iter().filter_map(|node| node.y).reduce(f64::max)?;
    Some(
        nodes
            .iter()
            .map(|node| {
                (
                    plot.x
                        + ((node.x.unwrap_or_default() - min_x) / (max_x - min_x).max(1e-12))
                            as f32
                            * plot.width,
                    plot.y
                        + ((node.y.unwrap_or_default() - min_y) / (max_y - min_y).max(1e-12))
                            as f32
                            * plot.height,
                )
            })
            .collect(),
    )
}

pub(super) fn force_layout(
    node_count: usize,
    links: &[LinkData],
    plot: Plot,
    repulsion: f32,
    gravity: f32,
    edge_length: f32,
) -> Vec<(f32, f32)> {
    let mut positions = circular_layout(node_count, plot);
    let iterations = if node_count > 120 { 18 } else { 48 };
    let center = (plot.x + plot.width / 2.0, plot.y + plot.height / 2.0);
    for iteration in 0..iterations {
        let mut forces = vec![(0.0f32, 0.0f32); node_count];
        for left in 0..node_count {
            for right in left + 1..node_count {
                let dx = positions[left].0 - positions[right].0;
                let dy = positions[left].1 - positions[right].1;
                let distance_sq = (dx * dx + dy * dy).max(1.0);
                let distance = distance_sq.sqrt();
                let force = repulsion / distance_sq;
                let fx = dx / distance * force;
                let fy = dy / distance * force;
                forces[left].0 += fx;
                forces[left].1 += fy;
                forces[right].0 -= fx;
                forces[right].1 -= fy;
            }
        }
        for link in links {
            if link.source >= node_count || link.target >= node_count {
                continue;
            }
            let dx = positions[link.target].0 - positions[link.source].0;
            let dy = positions[link.target].1 - positions[link.source].1;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            let force = (distance - edge_length) * 0.035;
            let fx = dx / distance * force;
            let fy = dy / distance * force;
            forces[link.source].0 += fx;
            forces[link.source].1 += fy;
            forces[link.target].0 -= fx;
            forces[link.target].1 -= fy;
        }
        let cooling = 1.0 - iteration as f32 / iterations as f32;
        for index in 0..node_count {
            forces[index].0 += (center.0 - positions[index].0) * gravity * 0.01;
            forces[index].1 += (center.1 - positions[index].1) * gravity * 0.01;
            positions[index].0 =
                (positions[index].0 + forces[index].0 * cooling).clamp(plot.x, plot.x + plot.width);
            positions[index].1 = (positions[index].1 + forces[index].1 * cooling)
                .clamp(plot.y, plot.y + plot.height);
        }
    }
    positions
}

pub(super) struct SankeyLayout {
    pub(super) nodes: Vec<Plot>,
    pub(super) links: Vec<SankeyLinkLayout>,
}

pub(super) struct SankeyLinkLayout {
    pub(super) source: (f32, f32),
    pub(super) target: (f32, f32),
    pub(super) width: f32,
}

pub(super) fn sankey_layout(
    nodes: &[NodeData],
    links: &[LinkData],
    plot: Plot,
    node_width: f32,
    node_gap: f32,
) -> SankeyLayout {
    if nodes.is_empty() {
        return SankeyLayout {
            nodes: Vec::new(),
            links: Vec::new(),
        };
    }
    let mut depth = vec![0usize; nodes.len()];
    for _ in 0..nodes.len() {
        let mut changed = false;
        for link in links {
            if link.source >= nodes.len() || link.target >= nodes.len() {
                continue;
            }
            let candidate = depth[link.source].saturating_add(1).min(nodes.len() - 1);
            if candidate > depth[link.target] {
                depth[link.target] = candidate;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let mut outgoing = vec![0.0f64; nodes.len()];
    let mut incoming = vec![0.0f64; nodes.len()];
    for link in links {
        if link.source < nodes.len() && link.target < nodes.len() {
            outgoing[link.source] += link.value.max(0.0);
            incoming[link.target] += link.value.max(0.0);
        }
    }
    let values: Vec<f64> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            node.value
                .max(outgoing[index])
                .max(incoming[index])
                .max(1e-6)
        })
        .collect();
    let mut levels = vec![Vec::new(); max_depth + 1];
    for (node, depth) in depth.iter().copied().enumerate() {
        levels[depth].push(node);
    }
    let scale = levels
        .iter()
        .filter(|level| !level.is_empty())
        .map(|level| {
            let available =
                (plot.height - node_gap * level.len().saturating_sub(1) as f32).max(1.0);
            let total: f64 = level.iter().map(|node| values[*node]).sum();
            available as f64 / total.max(1e-12)
        })
        .reduce(f64::min)
        .unwrap_or(1.0) as f32;
    let mut node_plots = vec![plot; nodes.len()];
    for (level_index, level) in levels.iter().enumerate() {
        let total_height: f32 = level
            .iter()
            .map(|node| values[*node] as f32 * scale)
            .sum::<f32>()
            + node_gap * level.len().saturating_sub(1) as f32;
        let mut y = plot.y + (plot.height - total_height) / 2.0;
        let x = if max_depth == 0 {
            plot.x + (plot.width - node_width) / 2.0
        } else {
            plot.x + (plot.width - node_width) * level_index as f32 / max_depth as f32
        };
        for node in level {
            let height = (values[*node] as f32 * scale).max(2.0);
            node_plots[*node] = Plot {
                x,
                y,
                width: node_width,
                height,
            };
            y += height + node_gap;
        }
    }

    let mut source_offsets = vec![0.0f32; nodes.len()];
    let mut target_offsets = vec![0.0f32; nodes.len()];
    let mut link_plots = Vec::with_capacity(links.len());
    for link in links {
        if link.source >= nodes.len() || link.target >= nodes.len() {
            link_plots.push(SankeyLinkLayout {
                source: (0.0, 0.0),
                target: (0.0, 0.0),
                width: 0.0,
            });
            continue;
        }
        let width = (link.value.max(0.0) as f32 * scale).max(1.0);
        let source_node = node_plots[link.source];
        let target_node = node_plots[link.target];
        let source = (
            source_node.x + source_node.width,
            source_node.y + source_offsets[link.source] + width / 2.0,
        );
        let target = (
            target_node.x,
            target_node.y + target_offsets[link.target] + width / 2.0,
        );
        source_offsets[link.source] += width;
        target_offsets[link.target] += width;
        link_plots.push(SankeyLinkLayout {
            source,
            target,
            width,
        });
    }
    SankeyLayout {
        nodes: node_plots,
        links: link_plots,
    }
}

fn worst_ratio(row: &[(usize, f64)], side: f64) -> f64 {
    let sum: f64 = row.iter().map(|item| item.1).sum();
    let min = row
        .iter()
        .map(|item| item.1)
        .reduce(f64::min)
        .unwrap_or(1.0);
    let max = row
        .iter()
        .map(|item| item.1)
        .reduce(f64::max)
        .unwrap_or(1.0);
    ((side * side * max) / (sum * sum).max(1e-12)).max((sum * sum) / (side * side * min).max(1e-12))
}

fn worst_ratio_with(row: &[(usize, f64)], item: (usize, f64), side: f64) -> f64 {
    let mut row = row.to_vec();
    row.push(item);
    worst_ratio(&row, side)
}

fn layout_row(row: &[(usize, f64)], remaining: &mut Plot, output: &mut [Plot]) {
    let area: f64 = row.iter().map(|item| item.1).sum();
    if remaining.width >= remaining.height {
        let row_width = (area / remaining.height.max(1.0) as f64) as f32;
        let mut y = remaining.y;
        for (index, item_area) in row {
            let height = (*item_area / row_width.max(1.0) as f64) as f32;
            output[*index] = Plot {
                x: remaining.x,
                y,
                width: row_width,
                height,
            };
            y += height;
        }
        remaining.x += row_width;
        remaining.width = (remaining.width - row_width).max(0.0);
    } else {
        let row_height = (area / remaining.width.max(1.0) as f64) as f32;
        let mut x = remaining.x;
        for (index, item_area) in row {
            let width = (*item_area / row_height.max(1.0) as f64) as f32;
            output[*index] = Plot {
                x,
                y: remaining.y,
                width,
                height: row_height,
            };
            x += width;
        }
        remaining.y += row_height;
        remaining.height = (remaining.height - row_height).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squarify_preserves_total_area() {
        let plot = Plot {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
        };
        let areas = squarify(&[6.0, 3.0, 1.0], plot);
        let total: f32 = areas.iter().map(|area| area.width * area.height).sum();
        assert!((total - 8_000.0).abs() < 1.0);
        assert!(areas
            .iter()
            .all(|area| area.width > 0.0 && area.height > 0.0));
    }
}
