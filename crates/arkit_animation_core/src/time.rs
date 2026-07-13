//! Integer time primitives used by source, compiled plans, and players.

use std::num::NonZeroU32;
use std::ops::{Add, Sub};

use crate::TimeError;

pub const NANOS_PER_MILLISECOND: u64 = 1_000_000;
pub const NANOS_PER_SECOND: u64 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeExtent {
    Infinite,
    Finite(TimeSpan),
}

impl Default for TimeExtent {
    fn default() -> Self {
        Self::ZERO
    }
}

impl TimeExtent {
    pub const ZERO: Self = Self::Finite(TimeSpan::ZERO);

    pub const fn finite(self) -> Option<TimeSpan> {
        match self {
            Self::Finite(duration) => Some(duration),
            Self::Infinite => None,
        }
    }

    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Self::Infinite, _) | (_, Self::Infinite) => Self::Infinite,
            (Self::Finite(left), Self::Finite(right)) => Self::Finite(left.max(right)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSpan(u64);

impl TimeSpan {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub const fn from_millis(milliseconds: u64) -> Self {
        Self(milliseconds.saturating_mul(NANOS_PER_MILLISECOND))
    }

    pub fn try_from_millis_f64(milliseconds: f64) -> Result<Self, TimeError> {
        if !milliseconds.is_finite() {
            return Err(TimeError::NonFinite);
        }
        if milliseconds < 0.0 {
            return Err(TimeError::Negative);
        }
        let nanos = milliseconds * NANOS_PER_MILLISECOND as f64;
        if nanos > u64::MAX as f64 {
            return Err(TimeError::Overflow);
        }
        Ok(Self(nanos.round() as u64))
    }

    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    pub fn as_millis_f64(self) -> f64 {
        self.0 as f64 / NANOS_PER_MILLISECOND as f64
    }

    pub const fn saturating_mul(self, factor: u32) -> Self {
        Self(self.0.saturating_mul(factor as u64))
    }

    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        match self.0.checked_add(rhs.0) {
            Some(nanos) => Some(Self(nanos)),
            None => None,
        }
    }

    pub const fn checked_mul(self, factor: u32) -> Option<Self> {
        match self.0.checked_mul(factor as u64) {
            Some(nanos) => Some(Self(nanos)),
            None => None,
        }
    }
}

impl Add for TimeSpan {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for TimeSpan {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimePoint(u64);

impl TimePoint {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub const fn as_nanos(self) -> u64 {
        self.0
    }

    pub const fn saturating_duration_since(self, earlier: Self) -> TimeSpan {
        TimeSpan::from_nanos(self.0.saturating_sub(earlier.0))
    }

    pub const fn checked_add(self, rhs: TimeSpan) -> Option<Self> {
        match self.0.checked_add(rhs.as_nanos()) {
            Some(nanos) => Some(Self(nanos)),
            None => None,
        }
    }
}

impl Add<TimeSpan> for TimePoint {
    type Output = Self;

    fn add(self, rhs: TimeSpan) -> Self::Output {
        Self(self.0.saturating_add(rhs.as_nanos()))
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = TimeSpan;

    fn sub(self, rhs: TimePoint) -> Self::Output {
        self.saturating_duration_since(rhs)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TimeOffset(i64);

impl TimeOffset {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    pub const fn from_millis(milliseconds: i64) -> Self {
        Self(milliseconds.saturating_mul(NANOS_PER_MILLISECOND as i64))
    }

    pub const fn as_nanos(self) -> i64 {
        self.0
    }

    pub fn apply(self, point: TimePoint) -> TimePoint {
        if self.0 >= 0 {
            point + TimeSpan::from_nanos(self.0 as u64)
        } else {
            TimePoint::from_nanos(point.as_nanos().saturating_sub(self.0.unsigned_abs()))
        }
    }

    pub const fn checked_apply(self, point: TimePoint) -> Option<TimePoint> {
        if self.0 >= 0 {
            point.checked_add(TimeSpan::from_nanos(self.0 as u64))
        } else {
            Some(TimePoint::from_nanos(
                point.as_nanos().saturating_sub(self.0.unsigned_abs()),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IterationCount {
    Finite(NonZeroU32),
    Infinite,
}

impl IterationCount {
    pub const ONCE: Self = Self::Finite(NonZeroU32::MIN);

    pub fn finite(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Self::Finite)
    }
}

impl Default for IterationCount {
    fn default() -> Self {
        Self::ONCE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackRate(f64);

impl PlaybackRate {
    pub const NORMAL: Self = Self(1.0);

    pub fn new(value: f64) -> Result<Self, TimeError> {
        if !value.is_finite() {
            return Err(TimeError::NonFinite);
        }
        if value <= 0.0 {
            return Err(TimeError::ZeroPlaybackRate);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> f64 {
        self.0
    }

    pub fn scale(self, elapsed: TimeSpan) -> TimeSpan {
        let nanos = elapsed.as_nanos() as f64 * self.0;
        TimeSpan::from_nanos(nanos.min(u64::MAX as f64).round() as u64)
    }
}

impl Default for PlaybackRate {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_milliseconds_round_to_nanoseconds() {
        let duration = TimeSpan::try_from_millis_f64(1.25).unwrap();
        assert_eq!(duration.as_nanos(), 1_250_000);
        assert_eq!(duration.as_millis_f64(), 1.25);
    }

    #[test]
    fn offsets_saturate_at_zero() {
        let point = TimePoint::from_nanos(10);
        assert_eq!(TimeOffset::from_nanos(-20).apply(point), TimePoint::ZERO);
    }

    #[test]
    fn playback_rate_scales_integer_time() {
        let rate = PlaybackRate::new(1.5).unwrap();
        assert_eq!(rate.scale(TimeSpan::from_nanos(10)).as_nanos(), 15);
        assert!(PlaybackRate::new(0.0).is_err());
        assert!(PlaybackRate::new(f64::NAN).is_err());
    }

    #[test]
    fn checked_time_arithmetic_reports_overflow() {
        assert_eq!(
            TimeSpan::from_nanos(2).checked_add(TimeSpan::from_nanos(3)),
            Some(TimeSpan::from_nanos(5))
        );
        assert_eq!(
            TimePoint::from_nanos(2).checked_add(TimeSpan::from_nanos(3)),
            Some(TimePoint::from_nanos(5))
        );
        assert_eq!(
            TimePoint::from_nanos(u64::MAX).checked_add(TimeSpan::from_nanos(1)),
            None
        );
    }
}
