//! Symbolic multi-target timeline graph.

use std::num::{NonZeroU16, NonZeroU32};

use crate::{
    Easing, IterationCount, LabelName, PlaybackRate, SourceAnimation, SourceSet, TimeOffset,
    TimePoint, TimeSpan,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TimelinePosition {
    Absolute(TimePoint),
    Label {
        label: LabelName,
        offset: TimeOffset,
    },
    PreviousStart(TimeOffset),
    PreviousEnd(TimeOffset),
    TimelineEnd(TimeOffset),
    Percentage(f32),
}

impl TimelinePosition {
    pub const START: Self = Self::Absolute(TimePoint::ZERO);

    pub fn percentage(value: f32) -> Option<Self> {
        (value.is_finite() && (0.0..=1.0).contains(&value)).then_some(Self::Percentage(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallPolicy {
    ForwardOnly,
    BothDirections,
    Once,
}

#[derive(Debug, Clone)]
pub struct PlaybackSettings {
    pub delay: TimeSpan,
    pub loop_delay: TimeSpan,
    pub iterations: IterationCount,
    pub alternate: bool,
    pub reversed: bool,
    pub playback_rate: PlaybackRate,
    pub frame_rate: Option<NonZeroU16>,
    pub playback_easing: Easing,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            delay: TimeSpan::ZERO,
            loop_delay: TimeSpan::ZERO,
            iterations: IterationCount::ONCE,
            alternate: false,
            reversed: false,
            playback_rate: PlaybackRate::NORMAL,
            frame_rate: None,
            playback_easing: Easing::Linear,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimelineNode {
    Animation {
        animation: SourceAnimation,
        position: TimelinePosition,
    },
    Timer {
        duration: TimeSpan,
        position: TimelinePosition,
    },
    Set(SourceSet),
    Call {
        call: crate::CallId,
        policy: CallPolicy,
        position: TimelinePosition,
    },
    Nested {
        timeline: Box<TimelineSource>,
        position: TimelinePosition,
    },
    Label {
        name: LabelName,
        position: TimelinePosition,
    },
    Barrier {
        participants: NonZeroU32,
        position: TimelinePosition,
    },
}

#[derive(Debug, Clone, Default)]
pub struct TimelineSource {
    pub settings: PlaybackSettings,
    pub nodes: Vec<TimelineNode>,
}

impl TimelineSource {
    pub fn push(&mut self, node: TimelineNode) {
        self.nodes.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_and_alternate_are_independent_settings() {
        let settings = PlaybackSettings {
            reversed: true,
            alternate: false,
            ..PlaybackSettings::default()
        };
        assert!(settings.reversed);
        assert!(!settings.alternate);
    }

    #[test]
    fn percentage_position_rejects_non_finite_values() {
        assert!(TimelinePosition::percentage(0.5).is_some());
        assert!(TimelinePosition::percentage(f32::NAN).is_none());
        assert!(TimelinePosition::percentage(-0.1).is_none());
        assert!(TimelinePosition::percentage(1.1).is_none());
    }
}
