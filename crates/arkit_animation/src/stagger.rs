//! Dioxus-friendly staggered timing for lists of animated components.

/// Where a staggered sequence starts within a collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StaggerFrom {
    #[default]
    First,
    Center,
    Last,
    Index(usize),
}

/// Ordering applied after calculating distance from [`StaggerFrom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StaggerDirection {
    #[default]
    Normal,
    Reverse,
}

/// A reusable delay distributor, equivalent to Anime.js-style `stagger()` for
/// Dioxus list components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stagger {
    step_ms: u32,
    start_ms: u32,
    from: StaggerFrom,
    direction: StaggerDirection,
}

/// Create a stagger distributor with `step_ms` between adjacent positions.
pub const fn stagger(step_ms: u32) -> Stagger {
    Stagger::new(step_ms)
}

impl Stagger {
    pub const fn new(step_ms: u32) -> Self {
        Self {
            step_ms,
            start_ms: 0,
            from: StaggerFrom::First,
            direction: StaggerDirection::Normal,
        }
    }

    pub const fn start_ms(mut self, value: u32) -> Self {
        self.start_ms = value;
        self
    }

    pub const fn from(mut self, value: StaggerFrom) -> Self {
        self.from = value;
        self
    }

    pub const fn from_center(self) -> Self {
        self.from(StaggerFrom::Center)
    }

    pub const fn from_last(self) -> Self {
        self.from(StaggerFrom::Last)
    }

    pub const fn reverse(mut self) -> Self {
        self.direction = StaggerDirection::Reverse;
        self
    }

    pub const fn step_ms(self) -> u32 {
        self.step_ms
    }

    /// Calculate the delay for `index` in a collection containing `total`
    /// items. Out-of-range indexes are clamped to the final item.
    pub fn delay(self, index: usize, total: usize) -> u32 {
        if total <= 1 {
            return self.start_ms;
        }

        let distance = self.distance(index, total);
        self.start_ms
            .saturating_add((distance * self.step_ms as f32).round() as u32)
    }

    /// Distribute a numeric value using the same origin and direction as the
    /// delay sequence. Positions with the minimum delay map to `from`, and
    /// positions with the maximum delay map to `to`.
    pub fn value(self, from: f32, to: f32, index: usize, total: usize) -> f32 {
        if total <= 1 {
            return from;
        }
        let max_distance = self.max_distance(total);
        if max_distance <= f32::EPSILON {
            from
        } else {
            let factor = self.distance(index, total) / max_distance;
            from + (to - from) * factor
        }
    }

    fn origin(self, total: usize) -> f32 {
        match self.from {
            StaggerFrom::First => 0.0,
            StaggerFrom::Center => (total - 1) as f32 / 2.0,
            StaggerFrom::Last => (total - 1) as f32,
            StaggerFrom::Index(value) => value.min(total - 1) as f32,
        }
    }

    fn max_distance(self, total: usize) -> f32 {
        let origin = self.origin(total);
        origin.max((total - 1) as f32 - origin)
    }

    fn distance(self, index: usize, total: usize) -> f32 {
        let origin = self.origin(total);
        let distance = (index.min(total - 1) as f32 - origin).abs();
        match self.direction {
            StaggerDirection::Normal => distance,
            StaggerDirection::Reverse => self.max_distance(total) - distance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distributes_from_first_and_last() {
        let first = stagger(40).start_ms(10);
        assert_eq!(
            [first.delay(0, 4), first.delay(1, 4), first.delay(3, 4)],
            [10, 50, 130]
        );

        let last = stagger(40).from_last();
        assert_eq!(
            [last.delay(0, 4), last.delay(2, 4), last.delay(3, 4)],
            [120, 40, 0]
        );
    }

    #[test]
    fn distributes_from_center_and_reverses() {
        let center = stagger(40).from_center();
        assert_eq!(
            (0..5)
                .map(|index| center.delay(index, 5))
                .collect::<Vec<_>>(),
            [80, 40, 0, 40, 80]
        );

        let reversed = center.reverse();
        assert_eq!(
            (0..5)
                .map(|index| reversed.delay(index, 5))
                .collect::<Vec<_>>(),
            [0, 40, 80, 40, 0]
        );
        assert_eq!(center.value(1.0, 0.5, 2, 5), 1.0);
        assert_eq!(center.value(1.0, 0.5, 0, 5), 0.5);
    }
}
