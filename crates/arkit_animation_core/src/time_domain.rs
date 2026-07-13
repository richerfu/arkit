//! Pure mapping from parent-domain time into a compiled local time domain.

use crate::{
    CompiledTimeDomain, IterationCount, PlaybackDirection, TimeExtent, TimePoint, TimeSpan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDomainPhase {
    BeforeDelay,
    Active,
    LoopDelay,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeDomainSample {
    pub phase: TimeDomainPhase,
    pub local_time: TimePoint,
    pub iteration: u64,
    pub direction: PlaybackDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeDomainOptions {
    pub reversed: bool,
    pub alternate: bool,
}

impl TimeDomainOptions {
    pub fn from_domain(domain: &CompiledTimeDomain) -> Self {
        Self {
            reversed: domain.settings.reversed,
            alternate: domain.settings.alternate,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TimeDomainMapper;

impl TimeDomainMapper {
    pub fn sample(domain: &CompiledTimeDomain, parent_time: TimePoint) -> TimeDomainSample {
        Self::sample_with_options(domain, parent_time, TimeDomainOptions::from_domain(domain))
    }

    pub fn sample_with_options(
        domain: &CompiledTimeDomain,
        parent_time: TimePoint,
        options: TimeDomainOptions,
    ) -> TimeDomainSample {
        let initial_direction = direction_for(options, 0);
        let initial_time = boundary_time(domain.extent, initial_direction, false);
        if parent_time < domain.offset {
            return TimeDomainSample {
                phase: TimeDomainPhase::BeforeDelay,
                local_time: initial_time,
                iteration: 0,
                direction: initial_direction,
            };
        }

        let relative = parent_time - domain.offset;
        if relative < domain.settings.delay {
            return TimeDomainSample {
                phase: TimeDomainPhase::BeforeDelay,
                local_time: initial_time,
                iteration: 0,
                direction: initial_direction,
            };
        }
        let elapsed = relative - domain.settings.delay;
        let TimeExtent::Finite(active_duration) = domain.extent else {
            return TimeDomainSample {
                phase: TimeDomainPhase::Active,
                local_time: TimePoint::from_nanos(
                    domain.settings.playback_rate.scale(elapsed).as_nanos(),
                ),
                iteration: 0,
                direction: PlaybackDirection::Forward,
            };
        };

        let active_parent = unscale(active_duration, domain.settings.playback_rate.get());
        let cycle = active_parent + domain.settings.loop_delay;
        let iteration_limit = match domain.settings.iterations {
            IterationCount::Finite(iterations) => Some(u64::from(iterations.get())),
            IterationCount::Infinite => None,
        };
        let occupied = iteration_limit.map(|iterations| {
            active_parent
                .as_nanos()
                .saturating_mul(iterations)
                .saturating_add(
                    domain
                        .settings
                        .loop_delay
                        .as_nanos()
                        .saturating_mul(iterations.saturating_sub(1)),
                )
        });
        if occupied.is_some_and(|occupied| elapsed.as_nanos() >= occupied) {
            let iteration = iteration_limit.unwrap_or(1).saturating_sub(1);
            let direction = direction_for(options, iteration);
            return TimeDomainSample {
                phase: TimeDomainPhase::Complete,
                local_time: boundary_time(domain.extent, direction, true),
                iteration,
                direction,
            };
        }

        let iteration = if cycle == TimeSpan::ZERO {
            0
        } else {
            elapsed.as_nanos() / cycle.as_nanos()
        };
        let within = if cycle == TimeSpan::ZERO {
            TimeSpan::ZERO
        } else {
            TimeSpan::from_nanos(elapsed.as_nanos() % cycle.as_nanos())
        };
        let direction = direction_for(options, iteration);
        if within >= active_parent {
            return TimeDomainSample {
                phase: TimeDomainPhase::LoopDelay,
                local_time: boundary_time(domain.extent, direction, true),
                iteration,
                direction,
            };
        }

        let progress = if active_parent == TimeSpan::ZERO {
            1.0
        } else {
            within.as_nanos() as f32 / active_parent.as_nanos() as f32
        };
        let eased = domain.settings.playback_easing.sample(progress);
        let directed = match direction {
            PlaybackDirection::Forward => eased,
            PlaybackDirection::Reverse => 1.0 - eased,
        };
        TimeDomainSample {
            phase: TimeDomainPhase::Active,
            local_time: TimePoint::from_nanos(
                (active_duration.as_nanos() as f64 * f64::from(directed.clamp(0.0, 1.0))).round()
                    as u64,
            ),
            iteration,
            direction,
        }
    }
}

fn direction_for(options: TimeDomainOptions, iteration: u64) -> PlaybackDirection {
    let reverse = options.reversed ^ (options.alternate && iteration % 2 == 1);
    if reverse {
        PlaybackDirection::Reverse
    } else {
        PlaybackDirection::Forward
    }
}

fn boundary_time(extent: TimeExtent, direction: PlaybackDirection, terminal: bool) -> TimePoint {
    let duration = extent.finite().unwrap_or(TimeSpan::ZERO);
    let at_end = matches!(direction, PlaybackDirection::Forward) == terminal;
    if at_end {
        TimePoint::from_nanos(duration.as_nanos())
    } else {
        TimePoint::ZERO
    }
}

fn unscale(duration: TimeSpan, playback_rate: f64) -> TimeSpan {
    let nanos = duration.as_nanos() as f64 / playback_rate;
    TimeSpan::from_nanos(nanos.min(u64::MAX as f64).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Easing, PlaybackRate, PlaybackSettings};

    fn domain() -> CompiledTimeDomain {
        CompiledTimeDomain {
            parent: Some(crate::TimeDomainId::new(0)),
            offset: TimePoint::from_nanos(20),
            extent: TimeExtent::Finite(TimeSpan::from_nanos(100)),
            settings: PlaybackSettings {
                delay: TimeSpan::from_nanos(10),
                loop_delay: TimeSpan::from_nanos(5),
                iterations: IterationCount::finite(2).unwrap(),
                alternate: true,
                reversed: true,
                playback_rate: PlaybackRate::new(2.0).unwrap(),
                playback_easing: Easing::Linear,
                ..PlaybackSettings::default()
            },
            first_event: None,
            event_count: 0,
        }
    }

    #[test]
    fn mapper_preserves_nested_delay_rate_reverse_alternate_and_loop_delay() {
        let domain = domain();
        let before = TimeDomainMapper::sample(&domain, TimePoint::from_nanos(29));
        assert_eq!(before.phase, TimeDomainPhase::BeforeDelay);
        assert_eq!(before.local_time, TimePoint::from_nanos(100));

        let first_half = TimeDomainMapper::sample(&domain, TimePoint::from_nanos(55));
        assert_eq!(first_half.phase, TimeDomainPhase::Active);
        assert_eq!(first_half.local_time, TimePoint::from_nanos(50));
        assert_eq!(first_half.direction, PlaybackDirection::Reverse);

        let loop_delay = TimeDomainMapper::sample(&domain, TimePoint::from_nanos(80));
        assert_eq!(loop_delay.phase, TimeDomainPhase::LoopDelay);
        assert_eq!(loop_delay.local_time, TimePoint::ZERO);

        let alternate_start = TimeDomainMapper::sample(&domain, TimePoint::from_nanos(85));
        assert_eq!(alternate_start.iteration, 1);
        assert_eq!(alternate_start.direction, PlaybackDirection::Forward);
        assert_eq!(alternate_start.local_time, TimePoint::ZERO);

        let complete = TimeDomainMapper::sample(&domain, TimePoint::from_nanos(135));
        assert_eq!(complete.phase, TimeDomainPhase::Complete);
        assert_eq!(complete.local_time, TimePoint::from_nanos(100));
    }
}
