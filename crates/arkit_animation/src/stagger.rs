//! Deterministic time/value distribution for one- and multi-dimensional target sets.

use arkit_animation_core::{AnimationValue, Easing, Modifier, TimeSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StaggerFrom {
    #[default]
    First,
    Center,
    Last,
    Index(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StaggerDirection {
    #[default]
    Normal,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaggerAxis {
    X,
    Y,
    Z,
    Radial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaggerGrid {
    pub columns: usize,
    pub rows: usize,
    pub layers: usize,
}

impl StaggerGrid {
    pub const fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns,
            rows,
            layers: 1,
        }
    }

    pub const fn with_layers(mut self, layers: usize) -> Self {
        self.layers = layers;
        self
    }

    fn normalized(self) -> Self {
        Self {
            columns: self.columns.max(1),
            rows: self.rows.max(1),
            layers: self.layers.max(1),
        }
    }

    fn point(self, index: usize) -> [f32; 3] {
        let grid = self.normalized();
        let layer_size = grid.columns.saturating_mul(grid.rows).max(1);
        let index = index.min(layer_size.saturating_mul(grid.layers).saturating_sub(1));
        [
            (index % grid.columns) as f32,
            ((index / grid.columns) % grid.rows) as f32,
            (index / layer_size) as f32,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Stagger {
    step_ms: f32,
    start_ms: f32,
    from: StaggerFrom,
    direction: StaggerDirection,
    grid: Option<StaggerGrid>,
    axis: StaggerAxis,
    easing: Easing,
    modifier: Modifier,
    jitter: f32,
    seed: u64,
}

pub fn stagger(step_ms: u32) -> Stagger {
    Stagger::new(step_ms)
}

impl Stagger {
    pub fn new(step_ms: u32) -> Self {
        Self {
            step_ms: step_ms as f32,
            start_ms: 0.0,
            from: StaggerFrom::First,
            direction: StaggerDirection::Normal,
            grid: None,
            axis: StaggerAxis::Radial,
            easing: Easing::Linear,
            modifier: Modifier::Identity,
            jitter: 0.0,
            seed: 0,
        }
    }

    pub fn start_ms(mut self, value: u32) -> Self {
        self.start_ms = value as f32;
        self
    }

    pub fn from(mut self, value: StaggerFrom) -> Self {
        self.from = value;
        self
    }

    pub fn from_center(self) -> Self {
        self.from(StaggerFrom::Center)
    }

    pub fn from_last(self) -> Self {
        self.from(StaggerFrom::Last)
    }

    pub fn reverse(mut self) -> Self {
        self.direction = StaggerDirection::Reverse;
        self
    }

    pub fn grid(mut self, grid: StaggerGrid) -> Self {
        self.grid = Some(grid.normalized());
        self
    }

    pub fn axis(mut self, axis: StaggerAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifier = modifier;
        self
    }

    pub fn jitter(mut self, amount: f32, seed: u64) -> Self {
        self.jitter = amount.clamp(0.0, 1.0);
        self.seed = seed;
        self
    }

    pub fn step_ms(&self) -> u32 {
        self.step_ms.round().clamp(0.0, u32::MAX as f32) as u32
    }

    pub fn delay(&self, index: usize, total: usize) -> u32 {
        self.delay_span(index, total)
            .as_millis_f64()
            .round()
            .clamp(0.0, u32::MAX as f64) as u32
    }

    pub fn delay_span(&self, index: usize, total: usize) -> TimeSpan {
        let value = self.distributed(
            self.start_ms,
            self.start_ms + self.step_ms * self.max_distance(total),
            index,
            total,
        );
        TimeSpan::try_from_millis_f64(f64::from(value.max(0.0)))
            .unwrap_or(TimeSpan::from_nanos(u64::MAX))
    }

    pub fn value(&self, from: f32, to: f32, index: usize, total: usize) -> f32 {
        self.distributed(from, to, index, total)
    }

    fn distributed(&self, from: f32, to: f32, index: usize, total: usize) -> f32 {
        if total <= 1 {
            return from;
        }
        let max_distance = self.max_distance(total);
        let raw = if max_distance <= f32::EPSILON {
            0.0
        } else {
            self.distance(index, total) / max_distance
        };
        let directed = match self.direction {
            StaggerDirection::Normal => raw,
            StaggerDirection::Reverse => 1.0 - raw,
        };
        let jitter = self.seeded_jitter(index) / max_distance.max(1.0);
        let progress = self.easing.sample((directed + jitter).clamp(0.0, 1.0));
        let value = from + (to - from) * progress;
        match self.modifier.apply(AnimationValue::Scalar(value)) {
            Ok(AnimationValue::Scalar(value)) => value,
            _ => value,
        }
    }

    fn origin_index(&self, total: usize) -> usize {
        match self.from {
            StaggerFrom::First => 0,
            StaggerFrom::Center => total.saturating_sub(1) / 2,
            StaggerFrom::Last => total.saturating_sub(1),
            StaggerFrom::Index(value) => value.min(total.saturating_sub(1)),
        }
    }

    fn max_distance(&self, total: usize) -> f32 {
        if total <= 1 {
            return 0.0;
        }
        (0..total)
            .map(|index| self.distance(index, total))
            .fold(0.0, f32::max)
    }

    fn distance(&self, index: usize, total: usize) -> f32 {
        let index = index.min(total.saturating_sub(1));
        let origin = self.origin_index(total);
        let Some(grid) = self.grid else {
            return index.abs_diff(origin) as f32;
        };
        let point = grid.point(index);
        let origin = grid.point(origin);
        let delta = [
            (point[0] - origin[0]).abs(),
            (point[1] - origin[1]).abs(),
            (point[2] - origin[2]).abs(),
        ];
        match self.axis {
            StaggerAxis::X => delta[0],
            StaggerAxis::Y => delta[1],
            StaggerAxis::Z => delta[2],
            StaggerAxis::Radial => delta.iter().map(|value| value * value).sum::<f32>().sqrt(),
        }
    }

    fn seeded_jitter(&self, index: usize) -> f32 {
        if self.jitter <= f32::EPSILON {
            return 0.0;
        }
        let mut value = self.seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        value ^= value >> 30;
        value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value ^= value >> 27;
        value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^= value >> 31;
        let unit = (value >> 40) as f32 / ((1_u32 << 24) - 1) as f32;
        (unit * 2.0 - 1.0) * self.jitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distributes_linear_and_grid_axes() {
        let linear = stagger(40).start_ms(10);
        assert_eq!(
            [linear.delay(0, 4), linear.delay(1, 4), linear.delay(3, 4)],
            [10, 50, 130]
        );
        let grid = stagger(10)
            .grid(StaggerGrid::new(3, 3))
            .from_center()
            .axis(StaggerAxis::X);
        assert_eq!(
            (0..3).map(|index| grid.delay(index, 9)).collect::<Vec<_>>(),
            [10, 0, 10]
        );
    }

    #[test]
    fn seeded_jitter_is_repeatable() {
        let distribution = stagger(20).jitter(0.8, 42);
        let first = (0..10)
            .map(|index| distribution.delay(index, 10))
            .collect::<Vec<_>>();
        let second = (0..10)
            .map(|index| distribution.delay(index, 10))
            .collect::<Vec<_>>();
        assert_eq!(first, second);
    }
}
