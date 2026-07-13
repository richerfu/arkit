use std::num::{NonZeroU16, NonZeroU32};
use std::rc::Rc;

use arkit_animation_core::{
    AnimatableValue, CallId, CallPolicy, Composition, Easing, FromValue, IterationCount, LabelName,
    Modifier, PlaybackRate, PlaybackSettings, Property, SourceAnimation, SourceSet, SourceTarget,
    TargetName, TimePoint, TimeSpan, TimelineNode, TimelinePosition, TimelineSource, TweenSpec,
    ValueSource,
};

use crate::AnimationSelector;
use crate::{AnimationBuildError, CapabilityRequirements, ExecutionPolicy};

pub(crate) type TimelineParts = (
    TimelineSource,
    ExecutionPolicy,
    CapabilityRequirements,
    Vec<Rc<dyn Fn()>>,
);

#[derive(Debug, Clone)]
pub struct PropertyKeyframe<T> {
    pub offset: f32,
    pub value: T,
    pub easing: Easing,
}

impl<T> PropertyKeyframe<T> {
    pub fn new(offset: f32, value: T) -> Self {
        Self {
            offset,
            value,
            easing: Easing::Linear,
        }
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    source: SourceAnimation,
}

impl Animation {
    pub fn new(target: AnimationSelector) -> Self {
        Self {
            source: SourceAnimation::new(SourceTarget::from(target)),
        }
    }

    pub fn tween<T: AnimatableValue>(
        mut self,
        property: &Property<T>,
        from: T,
        to: T,
        duration: TimeSpan,
    ) -> Self {
        self.source.push(TweenSpec::new(
            property.name().clone(),
            FromValue::Explicit(from.into_animation_value()),
            ValueSource::Fixed(to.into_animation_value()),
            duration,
        ));
        self
    }

    pub fn tween_from_current<T: AnimatableValue>(
        mut self,
        property: &Property<T>,
        to: T,
        duration: TimeSpan,
    ) -> Self {
        self.source.push(TweenSpec::new(
            property.name().clone(),
            FromValue::Current,
            ValueSource::Fixed(to.into_animation_value()),
            duration,
        ));
        self
    }

    pub fn configure_last(
        mut self,
        easing: Easing,
        composition: Composition,
        modifier: Modifier,
        delay: TimeSpan,
        priority: i32,
    ) -> Self {
        if let Some(tween) = self.source.tweens.last_mut() {
            tween.easing = easing;
            tween.composition = composition;
            tween.modifier = modifier;
            tween.delay = delay;
            tween.priority = priority;
        }
        self
    }

    pub fn keyframes<T: AnimatableValue>(
        mut self,
        property: &Property<T>,
        keyframes: impl IntoIterator<Item = PropertyKeyframe<T>>,
        duration: TimeSpan,
    ) -> Result<Self, AnimationBuildError> {
        let keyframes = keyframes.into_iter().collect::<Vec<_>>();
        if keyframes.len() < 2 {
            return Err(AnimationBuildError::EmptyKeyframes);
        }
        for keyframe in &keyframes {
            if !keyframe.offset.is_finite() || !(0.0..=1.0).contains(&keyframe.offset) {
                return Err(AnimationBuildError::InvalidKeyframeOffset(keyframe.offset));
            }
        }
        if keyframes
            .windows(2)
            .any(|pair| pair[0].offset >= pair[1].offset)
        {
            return Err(AnimationBuildError::NonIncreasingKeyframeOffset);
        }
        for pair in keyframes.windows(2) {
            let start = &pair[0];
            let end = &pair[1];
            let mut tween = TweenSpec::new(
                property.name().clone(),
                FromValue::Explicit(start.value.clone().into_animation_value()),
                ValueSource::Fixed(end.value.clone().into_animation_value()),
                scaled_duration(duration, end.offset - start.offset)?,
            );
            tween.delay = scaled_duration(duration, start.offset)?;
            tween.easing = start.easing.clone();
            self.source.push(tween);
        }
        Ok(self)
    }

    pub fn source(&self) -> &SourceAnimation {
        &self.source
    }
}

#[derive(Clone)]
pub struct Timeline {
    source: TimelineSource,
    policy: ExecutionPolicy,
    requirements: CapabilityRequirements,
    calls: Vec<Rc<dyn Fn()>>,
}

impl std::fmt::Debug for Timeline {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Timeline")
            .field("source", &self.source)
            .field("policy", &self.policy)
            .field("requirements", &self.requirements)
            .field("call_count", &self.calls.len())
            .finish()
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            source: TimelineSource::default(),
            policy: ExecutionPolicy::Auto,
            requirements: CapabilityRequirements {
                seek: true,
                pause: true,
                resume: true,
                reverse: true,
                cancel: true,
                callbacks: true,
                ..CapabilityRequirements::default()
            },
            calls: Vec::new(),
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, animation: Animation, position: TimelinePosition) -> Self {
        self.source.push(TimelineNode::Animation {
            animation: animation.source,
            position,
        });
        self
    }

    pub fn timer(mut self, duration: TimeSpan, position: TimelinePosition) -> Self {
        self.source.push(TimelineNode::Timer { duration, position });
        self
    }

    pub fn set<T: AnimatableValue>(
        mut self,
        target: AnimationSelector,
        property: &Property<T>,
        value: T,
        position: TimelinePosition,
    ) -> Self {
        self.source.push(TimelineNode::Set(SourceSet {
            target: SourceTarget::from(target),
            property: property.name().clone(),
            value: value.into_animation_value(),
            position,
        }));
        self
    }

    pub fn label(mut self, name: LabelName, position: TimelinePosition) -> Self {
        self.source.push(TimelineNode::Label { name, position });
        self
    }

    pub fn call(
        mut self,
        callback: impl Fn() + 'static,
        policy: CallPolicy,
        position: TimelinePosition,
    ) -> Self {
        let call = CallId::new(self.calls.len());
        self.calls.push(Rc::new(callback));
        self.source.push(TimelineNode::Call {
            call,
            policy,
            position,
        });
        self
    }

    pub fn nested(mut self, mut timeline: Timeline, position: TimelinePosition) -> Self {
        remap_calls(&mut timeline.source, self.calls.len());
        self.calls.append(&mut timeline.calls);
        self.source.push(TimelineNode::Nested {
            timeline: Box::new(timeline.source),
            position,
        });
        self
    }

    pub fn barrier(mut self, participants: NonZeroU32, position: TimelinePosition) -> Self {
        self.source.push(TimelineNode::Barrier {
            participants,
            position,
        });
        self
    }

    pub fn remove_target(mut self, target: &TargetName) -> Self {
        remove_target_nodes(&mut self.source, target);
        self
    }

    pub fn settings(mut self, settings: PlaybackSettings) -> Self {
        self.source.settings = settings;
        self
    }

    pub fn delay(mut self, delay: TimeSpan) -> Self {
        self.source.settings.delay = delay;
        self
    }

    pub fn loop_delay(mut self, delay: TimeSpan) -> Self {
        self.source.settings.loop_delay = delay;
        self
    }

    pub fn iterations(mut self, iterations: IterationCount) -> Self {
        self.source.settings.iterations = iterations;
        self
    }

    pub fn alternate(mut self, alternate: bool) -> Self {
        self.source.settings.alternate = alternate;
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.source.settings.reversed = reversed;
        self
    }

    pub fn playback_rate(mut self, rate: PlaybackRate) -> Self {
        self.source.settings.playback_rate = rate;
        self
    }

    pub fn frame_rate(mut self, frame_rate: Option<NonZeroU16>) -> Self {
        self.source.settings.frame_rate = frame_rate;
        self
    }

    pub fn playback_easing(mut self, easing: Easing) -> Self {
        self.source.settings.playback_easing = easing;
        self
    }

    pub fn execution_policy(mut self, policy: ExecutionPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Declares that this timeline is a one-shot island and will not use the
    /// full imperative control surface. This makes native implicit/keyframe
    /// lowering eligible when the plan is otherwise representable.
    pub fn native_island(mut self) -> Self {
        self.requirements = CapabilityRequirements {
            cancel: true,
            callbacks: true,
            ..CapabilityRequirements::default()
        };
        self
    }

    pub fn capability_requirements(mut self, requirements: CapabilityRequirements) -> Self {
        self.requirements = requirements;
        self
    }

    pub fn source(&self) -> &TimelineSource {
        &self.source
    }

    pub fn into_source(self) -> TimelineSource {
        self.source
    }

    pub(crate) fn into_parts(self) -> TimelineParts {
        (self.source, self.policy, self.requirements, self.calls)
    }

    pub fn at(milliseconds: u64) -> Option<TimelinePosition> {
        milliseconds
            .checked_mul(arkit_animation_core::NANOS_PER_MILLISECOND)
            .map(TimePoint::from_nanos)
            .map(TimelinePosition::Absolute)
    }
}

fn scaled_duration(duration: TimeSpan, factor: f32) -> Result<TimeSpan, AnimationBuildError> {
    TimeSpan::try_from_millis_f64(duration.as_millis_f64() * f64::from(factor))
        .map_err(|_| AnimationBuildError::TimeOverflow)
}

fn remap_calls(source: &mut TimelineSource, offset: usize) {
    for node in &mut source.nodes {
        match node {
            TimelineNode::Call { call, .. } => *call = CallId::new(call.index() + offset),
            TimelineNode::Nested { timeline, .. } => remap_calls(timeline, offset),
            _ => {}
        }
    }
}

fn remove_target_nodes(source: &mut TimelineSource, target: &TargetName) {
    source.nodes.retain_mut(|node| match node {
        TimelineNode::Animation { animation, .. } => {
            !matches!(&animation.target, SourceTarget::One(name) if name == target)
        }
        TimelineNode::Set(set) => !matches!(&set.target, SourceTarget::One(name) if name == target),
        TimelineNode::Nested { timeline, .. } => {
            remove_target_nodes(timeline, target);
            !timeline.nodes.is_empty()
        }
        _ => true,
    });
}
