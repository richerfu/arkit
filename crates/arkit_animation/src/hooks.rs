use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arkit_animation_core::{AdapterTargetId, TargetName};
use arkit_hooks::use_ark_node;
use arkit_prelude::*;

use crate::api::Timeline;
use crate::controls::{AnimationControls, ControlsInner};
use crate::{AnimationHost, FrameDriver};

#[derive(Clone)]
pub(crate) struct AnimationHostContext {
    pub host: Rc<AnimationHost>,
    pub driver: Rc<FrameDriver>,
    pub target_version: Signal<u64>,
    pending_controls: Rc<RefCell<Vec<Weak<ControlsInner>>>>,
}

#[derive(Clone, Copy)]
pub struct AnimationTarget {
    ready: Signal<bool>,
}

impl AnimationTarget {
    pub fn is_ready(self) -> bool {
        (self.ready)()
    }
}

#[track_caller]
pub fn use_animation_host_provider() {
    crate::layout::use_layout_registry_provider();
    let frame_node = use_ark_node();
    let context = use_context_provider(|| {
        let host = AnimationHost::new().expect("built-in animation adapters must register");
        AnimationHostContext {
            driver: FrameDriver::new(host.clone(), frame_node),
            host,
            target_version: Signal::new(0),
            pending_controls: Rc::new(RefCell::new(Vec::new())),
        }
    });
    let metrics = arkit_hooks::use_window_metrics();
    let density = metrics.scale.max(f32::EPSILON);
    context
        .host
        .set_window_metrics(arkit_animation_core::WindowMetrics {
            width_vp: metrics.content_rect.width.max(0) as f32 / density,
            height_vp: metrics.content_rect.height.max(0) as f32 / density,
            density,
        });
}

#[track_caller]
pub fn use_animation_target(name: impl Into<String>) -> AnimationTarget {
    let context = use_context::<AnimationHostContext>();
    let node = use_ark_node();
    let name = use_hook(|| TargetName::owned(name.into()));
    let mut ready = use_signal(|| false);
    let registered = use_hook(|| Rc::new(Cell::new(None::<AdapterTargetId>)));
    let register_context = context.clone();
    let register_name = name.clone();
    let register_slot = registered.clone();
    use_effect(move || {
        if register_slot.get().is_some() {
            return;
        }
        let Some(host_node) = node.get() else {
            return;
        };
        match register_context
            .host
            .arkui()
            .register_target(register_name.clone(), host_node, None)
        {
            Ok(id) => {
                register_slot.set(Some(id));
                ready.set(true);
                let mut version = register_context.target_version;
                version += 1;
                retry_pending_controls(&register_context);
            }
            Err(error) => {
                ohos_hilog_binding::error(format!(
                    "arkit_animation target registration failed: {error}"
                ));
            }
        }
    });
    let drop_context = context;
    let drop_slot = registered;
    use_drop(move || {
        if let Some(id) = drop_slot.take() {
            drop_context.host.unregister_arkui_target(id);
            let mut version = drop_context.target_version;
            version += 1;
        }
    });
    AnimationTarget { ready }
}

pub(crate) fn use_animation_context() -> AnimationHostContext {
    use_context::<AnimationHostContext>()
}

#[track_caller]
pub fn use_animation(timeline: Timeline) -> AnimationControls {
    let context = use_animation_context();
    let (source, policy, requirements, calls) = timeline.into_parts();
    let pending_controls = context.pending_controls.clone();
    let inner = use_hook(|| {
        let inner = ControlsInner::new(
            context.host.clone(),
            context.driver.clone(),
            source,
            policy,
            requirements,
            calls,
        );
        pending_controls.borrow_mut().push(Rc::downgrade(&inner));
        inner
    });
    let registration = inner.clone();
    let target_version = context.target_version;
    use_effect(move || {
        let _ = target_version();
        try_register_controls(&registration);
    });
    AnimationControls { inner }
}

fn retry_pending_controls(context: &AnimationHostContext) {
    let pending = {
        let mut entries = context.pending_controls.borrow_mut();
        entries.retain(|entry| entry.strong_count() > 0);
        entries.iter().filter_map(Weak::upgrade).collect::<Vec<_>>()
    };
    for controls in pending {
        try_register_controls(&controls);
    }
}

fn try_register_controls(controls: &ControlsInner) {
    if controls.instance.get().is_some() {
        return;
    }
    match controls.host.insert_timeline_with_policy(
        &controls.source.borrow(),
        controls.policy.get(),
        controls.requirements.get(),
    ) {
        Ok((instance, report)) => {
            controls.instance.set(Some(instance));
            controls.lowering_report.replace(Some(report));
            controls.notify_observers();
        }
        Err(crate::AnimationHostError::Resolve(
            arkit_animation_core::AnimationResolveError::EmptyTargetSelection,
        )) => {}
        Err(error) => {
            ohos_hilog_binding::error(format!("animation insertion failed: {error}"));
        }
    }
}

#[track_caller]
pub fn use_animation_snapshot(
    controls: &AnimationControls,
) -> Signal<Option<arkit_animation_core::AnimationInstanceSnapshot>> {
    let snapshot = use_signal(|| controls.snapshot());
    let observed_controls = controls.clone();
    let snapshot_writer = Rc::new(RefCell::new(snapshot));
    let subscription = use_hook(|| {
        let snapshot_writer = snapshot_writer.clone();
        Rc::new(RefCell::new(Some(observed_controls.subscribe(
            move |value| {
                snapshot_writer.borrow_mut().set(Some(value));
            },
        ))))
    });
    let drop_subscription = subscription.clone();
    use_drop(move || {
        drop_subscription.borrow_mut().take();
    });
    snapshot
}
