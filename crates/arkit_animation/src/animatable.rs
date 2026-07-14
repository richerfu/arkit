use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::rc::Weak;

use arkit_animation_core::{
    AnimatableValue, Composition, Easing, IterationCount, Modifier, Property, PropertyDescriptor,
    PropertyName, TargetName, TimeSpan, TimelinePosition,
};
use arkit_prelude::*;

use crate::api::{Animation, Timeline};
use crate::controls::{AnimationControls, ControlsInner};
use crate::hooks::use_animation_context;
use crate::{AnimationSelector, DrawingAdapter, TargetAdapter};

#[derive(Clone)]
pub struct Animatable<T: AnimatableValue> {
    adapter: Rc<DrawingAdapter>,
    target: arkit_animation_core::AdapterTargetId,
    target_name: TargetName,
    property: Property<T>,
    controls: AnimationControls,
    marker: PhantomData<T>,
    _registration: Rc<AnimatableRegistration>,
    defaults: AnimatableDefaults,
}

impl<T: AnimatableValue> PartialEq for Animatable<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.adapter, &other.adapter)
            && self.target == other.target
            && self.property == other.property
    }
}

impl<T: AnimatableValue> Eq for Animatable<T> {}

#[derive(Debug, Clone)]
pub struct AnimatableDefaults {
    pub duration: TimeSpan,
    pub delay: TimeSpan,
    pub easing: Easing,
    pub composition: Composition,
    pub modifier: Modifier,
}

impl Default for AnimatableDefaults {
    fn default() -> Self {
        Self {
            duration: TimeSpan::from_millis(300),
            delay: TimeSpan::ZERO,
            easing: Easing::Linear,
            composition: Composition::Replace,
            modifier: Modifier::Identity,
        }
    }
}

struct AnimatableRegistration {
    host: Weak<crate::AnimationHost>,
    adapter: arkit_animation_core::AdapterId,
}

impl Drop for AnimatableRegistration {
    fn drop(&mut self) {
        if let Some(host) = self.host.upgrade() {
            host.unregister_adapter(self.adapter);
        }
    }
}

impl<T: AnimatableValue> Animatable<T> {
    pub fn get(&self) -> T {
        let value = self
            .adapter
            .value(self.target, self.property.name())
            .expect("animatable target retains its value");
        T::try_from_animation_value(value).expect("animatable descriptor fixes the value kind")
    }

    pub fn retarget(&self, value: T, duration: TimeSpan) {
        self.animate(self.get(), value, duration, TimeSpan::ZERO, Easing::Linear);
    }

    pub fn to(&self, value: T) {
        let defaults = &self.defaults;
        let animation = Animation::new(AnimationSelector::Target(self.target_name.clone()))
            .tween(&self.property, self.get(), value, defaults.duration)
            .configure_last(
                defaults.easing.clone(),
                defaults.composition,
                defaults.modifier.clone(),
                defaults.delay,
                0,
            );
        self.controls
            .set_timeline(Timeline::new().add(animation, TimelinePosition::START));
        self.controls.restart();
    }

    pub fn animate(&self, from: T, to: T, duration: TimeSpan, delay: TimeSpan, easing: Easing) {
        let animation = Animation::new(AnimationSelector::Target(self.target_name.clone()))
            .tween(&self.property, from, to, duration)
            .configure_last(easing, Composition::Replace, Modifier::Identity, delay, 0);
        self.controls
            .set_timeline(Timeline::new().add(animation, TimelinePosition::START));
        self.controls.restart();
    }

    /// Repeats a logical value range on the root-owned animation engine.
    ///
    /// This is intended for drawing invalidation clocks and other continuously
    /// sampled values. It does not allocate a platform or async timer.
    pub fn animate_repeating(&self, from: T, to: T, duration: TimeSpan, easing: Easing) {
        let animation = Animation::new(AnimationSelector::Target(self.target_name.clone()))
            .tween(&self.property, from, to, duration)
            .configure_last(
                easing,
                Composition::Replace,
                Modifier::Identity,
                TimeSpan::ZERO,
                0,
            );
        self.controls.set_timeline(
            Timeline::new()
                .add(animation, TimelinePosition::START)
                .iterations(IterationCount::Infinite),
        );
        self.controls.restart();
    }

    pub fn set(&self, value: T) {
        self.controls.pause();
        self.adapter
            .set_value(
                self.target,
                self.property.name(),
                value.into_animation_value(),
            )
            .expect("animatable descriptor fixes the value kind");
    }

    pub fn set_invalidator(&self, invalidator: impl Fn() + 'static) {
        self.adapter.set_invalidator(move |_| invalidator());
    }

    pub fn controls(&self) -> &AnimationControls {
        &self.controls
    }

    pub fn revert(&self) {
        self.controls.revert();
    }
}

thread_local! {
    static NEXT_ANIMATABLE_ID: Cell<u64> = const { Cell::new(0) };
}

fn next_animatable_name() -> TargetName {
    NEXT_ANIMATABLE_ID.with(|next| {
        let id = next.get();
        next.set(id.checked_add(1).expect("animatable id space exhausted"));
        TargetName::owned(format!("animatable-{id}"))
    })
}

#[track_caller]
pub fn use_animatable<T: AnimatableValue>(initial: T) -> Animatable<T> {
    use_animatable_with_defaults(initial, AnimatableDefaults::default())
}

#[track_caller]
pub fn use_animatable_with_defaults<T: AnimatableValue>(
    initial: T,
    defaults: AnimatableDefaults,
) -> Animatable<T> {
    let context = use_animation_context();
    use_hook(|| {
        let property = Property::<T>::owned("value");
        let descriptor = PropertyDescriptor::new(&property);
        let adapter = Rc::new(DrawingAdapter::new(
            context.host.next_adapter_id(),
            [descriptor],
        ));
        let adapter_id = adapter.id();
        context
            .host
            .register_adapter(adapter.clone() as Rc<dyn TargetAdapter>)
            .expect("animatable adapter ids are allocated by the host");
        let target_name = next_animatable_name();
        let baseline = initial.clone();
        let target = adapter
            .register_target(
                target_name.clone(),
                [(
                    PropertyName::owned("value"),
                    baseline.into_animation_value(),
                )],
            )
            .expect("new animatable target is unique");
        let initial_animation = Animation::new(AnimationSelector::Target(target_name.clone()))
            .tween(&property, initial.clone(), initial, TimeSpan::ZERO);
        let source = Timeline::new()
            .add(initial_animation, TimelinePosition::START)
            .into_source();
        let controls = AnimationControls {
            inner: ControlsInner::new(
                context.host.clone(),
                context.driver.clone(),
                source.clone(),
                crate::ExecutionPolicy::SampledOnly,
                crate::CapabilityRequirements::default(),
                Vec::new(),
            ),
        };
        let instance = context
            .host
            .insert_timeline(&source)
            .expect("animatable source resolves against its drawing adapter");
        controls.inner.instance.set(Some(instance));
        let registration = Rc::new(AnimatableRegistration {
            host: Rc::downgrade(&context.host),
            adapter: adapter_id,
        });
        Animatable {
            adapter,
            target,
            target_name,
            property,
            controls,
            marker: PhantomData,
            _registration: registration,
            defaults,
        }
    })
}
