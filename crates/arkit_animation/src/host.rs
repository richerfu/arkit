use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::time::Instant;

use arkit_animation_core::{
    AnimationBaselineSnapshot, AnimationClockMode, AnimationCompileError, AnimationCompiler,
    AnimationEngine, AnimationOutcome, AnimationResolveError, AnimationResolver,
    AnimationRuntimeError, EngineCommand, EngineEvent, InstanceKey, PlaybackDirection,
    TimeDomainId, TimeExtent, TimePoint, TimeSpan, TimelineSource, WindowMetrics,
};
use arkit_arkui::MountedNodeLease;
use rustc_hash::FxHashMap;

use crate::{
    native_instance::ArkUiNodeAnimatorInstance, NativeAnimationInstance, NativeAnimatorSpec,
    NativeInstanceError,
};
use crate::{
    AdapterRegistry, AdapterResolutionSnapshot, AnimationAdapterError, ArkUiAdapter, TargetAdapter,
};
use crate::{
    AnimationBackend, BackendRejection, CapabilityRequirements, ExecutionPolicy, LoweringReport,
    NativeLowerer, NativeLoweringError, UnsupportedFeature,
};

type EngineListener = Rc<dyn Fn(EngineEvent)>;
type ContextNodeProvider = Rc<dyn Fn() -> Option<MountedNodeLease>>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnimationPerformanceCounters {
    pub frames: u64,
    pub frame_callbacks_requested: u64,
    pub dirty_writes: u64,
    pub adapter_failures: u64,
    pub target_misses: u64,
    pub fallback_count: u64,
    pub last_compute_ns: u64,
    pub last_apply_ns: u64,
    pub engine: arkit_animation_core::EngineDiagnostics,
}

#[derive(Default)]
struct HostCounters {
    frames: Cell<u64>,
    frame_callbacks_requested: Cell<u64>,
    dirty_writes: Cell<u64>,
    adapter_failures: Cell<u64>,
    target_misses: Cell<u64>,
    fallback_count: Cell<u64>,
    last_compute_ns: Cell<u64>,
    last_apply_ns: Cell<u64>,
}

#[derive(Debug)]
pub enum AnimationHostError {
    Adapter(AnimationAdapterError),
    Resolve(AnimationResolveError),
    Compile(AnimationCompileError),
    Runtime(AnimationRuntimeError),
    Native(NativeLoweringError),
    NativeInstance(NativeInstanceError),
    ReentrantTick,
}

impl Display for AnimationHostError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for AnimationHostError {}

impl From<AnimationAdapterError> for AnimationHostError {
    fn from(value: AnimationAdapterError) -> Self {
        Self::Adapter(value)
    }
}

impl From<AnimationResolveError> for AnimationHostError {
    fn from(value: AnimationResolveError) -> Self {
        Self::Resolve(value)
    }
}

impl From<AnimationCompileError> for AnimationHostError {
    fn from(value: AnimationCompileError) -> Self {
        Self::Compile(value)
    }
}

impl From<AnimationRuntimeError> for AnimationHostError {
    fn from(value: AnimationRuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<NativeLoweringError> for AnimationHostError {
    fn from(value: NativeLoweringError) -> Self {
        Self::Native(value)
    }
}

impl From<NativeInstanceError> for AnimationHostError {
    fn from(value: NativeInstanceError) -> Self {
        Self::NativeInstance(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostedNativeState {
    Idle,
    Running,
    Paused,
    Terminal,
}

struct HostedNativeInstance {
    owner: Box<dyn NativeAnimationInstance>,
    duration: TimeSpan,
    policy: ExecutionPolicy,
    context_node: MountedNodeLease,
    state: HostedNativeState,
    direction: PlaybackDirection,
}

pub struct AnimationHost {
    engine: RefCell<AnimationEngine>,
    registry: RefCell<AdapterRegistry>,
    arkui: Rc<ArkUiAdapter>,
    window_metrics: RefCell<WindowMetrics>,
    listeners: RefCell<Vec<Option<EngineListener>>>,
    native_instances: RefCell<FxHashMap<InstanceKey, HostedNativeInstance>>,
    lowering_reports: RefCell<FxHashMap<InstanceKey, LoweringReport>>,
    native_commands: RefCell<VecDeque<EngineCommand>>,
    context_node_provider: RefCell<Option<ContextNodeProvider>>,
    event_scratch: RefCell<Vec<EngineEvent>>,
    ticking: Cell<bool>,
    counters: HostCounters,
}

impl AnimationHost {
    pub fn new() -> Result<Rc<Self>, AnimationHostError> {
        let arkui = Rc::new(ArkUiAdapter::new(arkit_animation_core::AdapterId::new(0)));
        let mut registry = AdapterRegistry::default();
        registry.register(arkui.clone() as Rc<dyn TargetAdapter>)?;
        Ok(Rc::new(Self {
            engine: RefCell::new(AnimationEngine::new()),
            registry: RefCell::new(registry),
            arkui,
            window_metrics: RefCell::new(WindowMetrics::default()),
            listeners: RefCell::new(Vec::new()),
            native_instances: RefCell::new(FxHashMap::default()),
            lowering_reports: RefCell::new(FxHashMap::default()),
            native_commands: RefCell::new(VecDeque::new()),
            context_node_provider: RefCell::new(None),
            event_scratch: RefCell::new(Vec::new()),
            ticking: Cell::new(false),
            counters: HostCounters::default(),
        }))
    }

    pub fn arkui(&self) -> &Rc<ArkUiAdapter> {
        &self.arkui
    }

    pub(crate) fn set_context_node_provider(&self, provider: ContextNodeProvider) {
        self.context_node_provider.replace(Some(provider));
    }

    pub(crate) fn release_native_context(&self) {
        let instances = self
            .native_instances
            .borrow()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        if instances.is_empty() {
            return;
        }

        {
            let mut engine = self.engine.borrow_mut();
            for instance in &instances {
                if let Err(error) = engine.set_clock_mode(*instance, AnimationClockMode::Internal) {
                    ohos_hilog_binding::error(format!(
                        "arkit_animation: failed to detach native clock before context teardown: {error:?}"
                    ));
                }
            }
            for command in self.native_commands.borrow_mut().drain(..) {
                engine.enqueue(command);
            }
        }

        // Drop native animator callbacks and handles while their ArkUI context
        // node is still valid. The sampled engine remains usable if Dioxus
        // later rebinds the animation root.
        self.native_instances.borrow_mut().clear();
        for instance in instances {
            self.record_runtime_fallback(
                instance,
                UnsupportedFeature::BackendUnavailable,
                "native animation context unmounted; using sampled execution",
            );
        }
    }

    pub fn unregister_arkui_target(&self, target: arkit_animation_core::AdapterTargetId) -> bool {
        let events = {
            let mut engine = self.engine.borrow_mut();
            engine.detach_target(self.arkui.id(), target);
            engine.drain_events().collect::<Vec<_>>()
        };
        let removed = self.arkui.unregister_target(target);
        self.publish_events(&events);
        self.remove_hosted_for_events(&events);
        removed
    }

    pub fn set_window_metrics(&self, metrics: WindowMetrics) {
        *self.window_metrics.borrow_mut() = metrics;
    }

    pub fn register_adapter(
        &self,
        adapter: Rc<dyn TargetAdapter>,
    ) -> Result<arkit_animation_core::AdapterId, AnimationHostError> {
        self.registry
            .borrow_mut()
            .register(adapter)
            .map_err(Into::into)
    }

    pub fn next_adapter_id(&self) -> arkit_animation_core::AdapterId {
        self.registry.borrow().next_id()
    }

    pub fn unregister_adapter(&self, adapter: arkit_animation_core::AdapterId) -> bool {
        if adapter == self.arkui.id() {
            return false;
        }
        let events = {
            let mut engine = self.engine.borrow_mut();
            engine.detach_adapter(adapter);
            engine.drain_events().collect::<Vec<_>>()
        };
        let removed = self.registry.borrow_mut().unregister(adapter).is_some();
        self.publish_events(&events);
        self.remove_hosted_for_events(&events);
        removed
    }

    pub fn insert_timeline(
        &self,
        source: &TimelineSource,
    ) -> Result<InstanceKey, AnimationHostError> {
        self.insert_timeline_with_policy(
            source,
            ExecutionPolicy::SampledOnly,
            CapabilityRequirements::default(),
        )
        .map(|(instance, _)| instance)
    }

    pub fn insert_timeline_with_policy(
        &self,
        source: &TimelineSource,
        policy: ExecutionPolicy,
        requirements: CapabilityRequirements,
    ) -> Result<(InstanceKey, LoweringReport), AnimationHostError> {
        let (plan, baselines) = match self.resolve(source) {
            Ok(resolved) => resolved,
            Err(AnimationHostError::Resolve(AnimationResolveError::EmptyTargetSelection)) => {
                increment(&self.counters.target_misses);
                return Err(AnimationResolveError::EmptyTargetSelection.into());
            }
            Err(error) => return Err(error),
        };
        let mut report = NativeLowerer.lower_plan(policy, &plan, requirements)?;
        let mut native = None;
        if report.selected == AnimationBackend::ArkUiAnimator {
            let duration = match plan.domains()[TimeDomainId::new(0)].parent_extent() {
                TimeExtent::Finite(duration) => duration,
                TimeExtent::Infinite => unreachable!(
                    "infinite timelines are rejected by ArkUI Animator capability lowering"
                ),
            };
            let owner = self.context_node_for_plan(&plan).ok_or_else(|| {
                NativeInstanceError::Native(
                    "no mounted ArkUI node is available for native context lookup".into(),
                )
            });
            let owner = owner.and_then(|node| {
                ArkUiNodeAnimatorInstance::new(&node, NativeAnimatorSpec::progress(duration))
                    .map(|owner| (Box::new(owner) as Box<dyn NativeAnimationInstance>, node))
            });
            match owner {
                Ok((owner, context_node)) => native = Some((owner, duration, context_node)),
                Err(error) if policy == ExecutionPolicy::NativeOnly => return Err(error.into()),
                Err(error) => mark_native_fallback(
                    &mut report,
                    AnimationBackend::ArkUiAnimator,
                    UnsupportedFeature::BackendUnavailable,
                    format!("native Animator setup failed ({error}); using sampled execution"),
                ),
            }
        } else if report.selected != AnimationBackend::Sampled {
            if policy == ExecutionPolicy::NativeOnly {
                return Err(NativeLoweringError::BackendUnavailable {
                    backend: report.selected,
                }
                .into());
            }
            let backend = report.selected;
            mark_native_fallback(
                &mut report,
                backend,
                UnsupportedFeature::BackendUnavailable,
                "selected native backend is not root-host executable; using sampled execution",
            );
        }
        if report.fallback_reason.is_some() {
            increment(&self.counters.fallback_count);
        }
        let clock_mode = if native.is_some() {
            AnimationClockMode::External
        } else {
            AnimationClockMode::Internal
        };
        let instance = self
            .engine
            .borrow_mut()
            .insert_with_clock(plan, baselines, clock_mode)?;
        if let Some((owner, duration, context_node)) = native {
            self.native_instances.borrow_mut().insert(
                instance,
                HostedNativeInstance {
                    owner,
                    duration,
                    policy,
                    context_node,
                    state: HostedNativeState::Idle,
                    direction: PlaybackDirection::Forward,
                },
            );
        }
        self.lowering_reports
            .borrow_mut()
            .insert(instance, report.clone());
        Ok((instance, report))
    }

    pub fn refresh_timeline(
        &self,
        instance: InstanceKey,
        source: &TimelineSource,
        policy: ExecutionPolicy,
        requirements: CapabilityRequirements,
    ) -> Result<(), AnimationHostError> {
        let (plan, baselines) = self.resolve(source)?;
        let mut report = NativeLowerer.lower_plan(policy, &plan, requirements)?;
        if report.selected != AnimationBackend::Sampled {
            if policy == ExecutionPolicy::NativeOnly {
                return Err(NativeInstanceError::UnsupportedControl("refresh").into());
            }
            let backend = report.selected;
            mark_native_fallback(
                &mut report,
                backend,
                UnsupportedFeature::Refresh,
                "hot timeline replacement uses sampled execution to preserve current position",
            );
        }
        self.transition_native_to_sampled(instance)?;
        self.engine
            .borrow_mut()
            .replace_resolution(instance, plan, baselines)?;
        if report.fallback_reason.is_some() {
            increment(&self.counters.fallback_count);
        }
        self.lowering_reports.borrow_mut().insert(instance, report);
        Ok(())
    }

    fn context_node_for_plan(
        &self,
        plan: &arkit_animation_core::CompiledAnimation,
    ) -> Option<MountedNodeLease> {
        if let Some(node) = self
            .context_node_provider
            .borrow()
            .as_ref()
            .and_then(|provider| provider())
        {
            return Some(node);
        }
        plan.targets().iter().find_map(|target| {
            (target.adapter == self.arkui.id())
                .then(|| self.arkui.node(target.adapter_target))
                .flatten()
        })
    }

    fn resolve(
        &self,
        source: &TimelineSource,
    ) -> Result<
        (
            Arc<arkit_animation_core::CompiledAnimation>,
            AnimationBaselineSnapshot,
        ),
        AnimationHostError,
    > {
        let metrics = *self.window_metrics.borrow();
        let registry = self.registry.borrow();
        let snapshot = AdapterResolutionSnapshot::new(&registry, metrics);
        let resolved = AnimationResolver::new(&snapshot).resolve_timeline(source)?;
        let plan = AnimationCompiler.compile(resolved)?;
        let mut baselines = Vec::with_capacity(plan.outputs().len());
        for output in plan.outputs() {
            let target = &plan.targets()[output.target];
            let property = &plan.properties()[output.property];
            baselines.push(
                registry
                    .get(target.adapter)?
                    .read_baseline(target.adapter_target, property.adapter_property)?,
            );
        }
        Ok((
            plan,
            AnimationBaselineSnapshot::from_output_values(baselines),
        ))
    }

    pub fn enqueue(&self, command: EngineCommand) {
        if self
            .native_instances
            .borrow()
            .contains_key(&host_command_instance(command))
        {
            self.native_commands.borrow_mut().push_back(command);
        } else {
            self.engine.borrow_mut().enqueue(command);
        }
    }

    pub fn tick(
        &self,
        frame_time: arkit_animation_core::TimePoint,
    ) -> Result<(), AnimationHostError> {
        if self.ticking.replace(true) {
            return Err(AnimationHostError::ReentrantTick);
        }
        let _tick_guard = TickGuard(&self.ticking);
        self.flush_native_commands()?;
        self.poll_native_instances();
        #[cfg(debug_assertions)]
        let compute_started = Instant::now();
        let mut engine = self.engine.borrow_mut();
        let frame = engine.tick(frame_time)?;
        #[cfg(debug_assertions)]
        self.counters
            .last_compute_ns
            .set(elapsed_nanos(compute_started));
        let dirty_writes = engine.frame_batch().len() as u64;
        #[cfg(debug_assertions)]
        let apply_started = Instant::now();
        if let Err(error) = self.registry.borrow().apply(engine.frame_batch()) {
            increment(&self.counters.adapter_failures);
            engine.reject_frame(frame)?;
            return Err(error.into());
        }
        #[cfg(debug_assertions)]
        self.counters
            .last_apply_ns
            .set(elapsed_nanos(apply_started));
        increment(&self.counters.frames);
        self.counters.dirty_writes.set(
            self.counters
                .dirty_writes
                .get()
                .saturating_add(dirty_writes),
        );
        engine.acknowledge_frame(frame)?;
        // Listener callbacks require the engine borrow to be released. Reuse
        // one host-owned buffer instead of allocating a fresh Vec every frame.
        let mut events = self.event_scratch.take();
        debug_assert!(events.is_empty());
        events.extend(engine.drain_events());
        drop(engine);
        self.publish_events(&events);
        self.remove_hosted_for_events(&events);
        events.clear();
        let displaced = self.event_scratch.replace(events);
        debug_assert!(displaced.is_empty());
        Ok(())
    }

    pub fn snapshot(
        &self,
        instance: InstanceKey,
    ) -> Option<arkit_animation_core::AnimationInstanceSnapshot> {
        self.engine.borrow().snapshot(instance)
    }

    pub fn lowering_report(&self, instance: InstanceKey) -> Option<LoweringReport> {
        self.lowering_reports.borrow().get(&instance).cloned()
    }

    pub fn has_work(&self) -> bool {
        self.engine.borrow().has_work()
            || !self.native_commands.borrow().is_empty()
            || self
                .native_instances
                .borrow()
                .values()
                .any(|instance| instance.state == HostedNativeState::Running)
    }

    fn flush_native_commands(&self) -> Result<(), AnimationHostError> {
        while let Some(command) = self.native_commands.borrow_mut().pop_front() {
            let instance = host_command_instance(command);
            if !self.native_instances.borrow().contains_key(&instance) {
                self.engine.borrow_mut().enqueue(command);
                continue;
            }
            if !native_command_supported(command) {
                self.fallback_native(
                    instance,
                    native_control_name(command),
                    runtime_unsupported_feature(command),
                )?;
                self.engine.borrow_mut().enqueue(command);
                continue;
            }

            let operation = self.apply_native_command(instance, command);
            if let Err(error) = operation {
                let native_only = self
                    .native_instances
                    .borrow()
                    .get(&instance)
                    .is_some_and(|native| native.policy == ExecutionPolicy::NativeOnly);
                if native_only {
                    return Err(error.into());
                }
                self.transition_native_to_sampled(instance)?;
                self.record_runtime_fallback(
                    instance,
                    runtime_unsupported_feature(command),
                    format!(
                        "native {} failed ({error}); using sampled execution",
                        native_control_name(command)
                    ),
                );
                increment(&self.counters.fallback_count);
                self.engine.borrow_mut().enqueue(command);
            }
        }
        Ok(())
    }

    fn apply_native_command(
        &self,
        instance: InstanceKey,
        command: EngineCommand,
    ) -> Result<(), NativeInstanceError> {
        if matches!(command, EngineCommand::Remove(_)) {
            self.native_instances.borrow_mut().remove(&instance);
            self.engine.borrow_mut().enqueue(command);
            return Ok(());
        }

        let (state, direction) = {
            let natives = self.native_instances.borrow();
            let native = &natives[&instance];
            (native.state, native.direction)
        };
        if matches!(command, EngineCommand::Reverse(_)) && state != HostedNativeState::Running {
            return Err(NativeInstanceError::UnsupportedControl(
                "reverse while native animator is not running",
            ));
        }
        if (matches!(command, EngineCommand::Restart(_))
            || matches!(command, EngineCommand::Play(_)) && state == HostedNativeState::Terminal)
            && direction == PlaybackDirection::Reverse
        {
            return Err(NativeInstanceError::UnsupportedControl(
                "native replay while reversed",
            ));
        }
        if matches!(command, EngineCommand::Restart(_))
            || matches!(command, EngineCommand::Play(_)) && state == HostedNativeState::Terminal
        {
            self.recreate_native(instance)?;
        }

        let mut natives = self.native_instances.borrow_mut();
        let native = natives
            .get_mut(&instance)
            .expect("native instance membership checked before command dispatch");
        match command {
            EngineCommand::Play(_) if native.state != HostedNativeState::Running => {
                native.owner.play()?;
                native.state = HostedNativeState::Running;
            }
            EngineCommand::Pause(_) if native.state == HostedNativeState::Running => {
                native.owner.pause()?;
                native.state = HostedNativeState::Paused;
            }
            EngineCommand::Resume(_) if native.state == HostedNativeState::Paused => {
                native.owner.play()?;
                native.state = HostedNativeState::Running;
            }
            EngineCommand::Restart(_) => {
                native.owner.play()?;
                native.state = HostedNativeState::Running;
            }
            EngineCommand::Reverse(_) if native.state == HostedNativeState::Running => {
                native.owner.reverse()?;
                native.direction = native.direction.reversed();
            }
            EngineCommand::Complete(_) if native.state != HostedNativeState::Terminal => {
                native.owner.complete()?;
                native.state = HostedNativeState::Terminal;
            }
            EngineCommand::Cancel(_) if native.state != HostedNativeState::Terminal => {
                native.owner.cancel()?;
                native.state = HostedNativeState::Terminal;
            }
            EngineCommand::SetAlternate { .. }
            | EngineCommand::Play(_)
            | EngineCommand::Pause(_)
            | EngineCommand::Resume(_)
            | EngineCommand::Reverse(_)
            | EngineCommand::Complete(_)
            | EngineCommand::Cancel(_) => {}
            _ => unreachable!("unsupported native commands fall back before dispatch"),
        }
        drop(natives);
        self.engine.borrow_mut().enqueue(command);
        Ok(())
    }

    fn recreate_native(&self, instance: InstanceKey) -> Result<(), NativeInstanceError> {
        let (duration, context_node) = {
            let natives = self.native_instances.borrow();
            let native = &natives[&instance];
            (native.duration, native.context_node.clone())
        };
        let owner =
            ArkUiNodeAnimatorInstance::new(&context_node, NativeAnimatorSpec::progress(duration))?;
        let mut natives = self.native_instances.borrow_mut();
        let native = natives
            .get_mut(&instance)
            .expect("native instance cannot disappear during UI-thread recreation");
        native.owner = Box::new(owner);
        native.state = HostedNativeState::Idle;
        native.direction = PlaybackDirection::Forward;
        Ok(())
    }

    fn poll_native_instances(&self) {
        let mut natives = self.native_instances.borrow_mut();
        let mut engine = self.engine.borrow_mut();
        for (instance, native) in natives.iter_mut() {
            if native.state == HostedNativeState::Running {
                if let Some(progress) = native
                    .owner
                    .take_progress()
                    .filter(|value| value.is_finite())
                {
                    let position = TimePoint::from_nanos(
                        (native.duration.as_nanos() as f64 * f64::from(progress.clamp(0.0, 1.0)))
                            .round() as u64,
                    );
                    engine.enqueue(EngineCommand::AdvanceExternal {
                        instance: *instance,
                        position,
                    });
                }
            } else {
                let _ = native.owner.take_progress();
            }
            if let Some(outcome) = native.owner.take_terminal() {
                if native.state != HostedNativeState::Terminal {
                    engine.enqueue(match outcome {
                        AnimationOutcome::Completed => EngineCommand::Complete(*instance),
                        AnimationOutcome::Cancelled => EngineCommand::Cancel(*instance),
                        AnimationOutcome::Reverted => EngineCommand::Revert(*instance),
                    });
                }
                native.state = HostedNativeState::Terminal;
            }
        }
    }

    fn fallback_native(
        &self,
        instance: InstanceKey,
        control: &'static str,
        unsupported: UnsupportedFeature,
    ) -> Result<(), AnimationHostError> {
        let Some(policy) = self
            .native_instances
            .borrow()
            .get(&instance)
            .map(|native| native.policy)
        else {
            return Ok(());
        };
        if policy == ExecutionPolicy::NativeOnly {
            return Err(NativeInstanceError::UnsupportedControl(control).into());
        }
        self.transition_native_to_sampled(instance)?;
        self.record_runtime_fallback(
            instance,
            unsupported,
            format!("native {control} is unsupported at runtime; using sampled execution"),
        );
        increment(&self.counters.fallback_count);
        Ok(())
    }

    fn transition_native_to_sampled(
        &self,
        instance: InstanceKey,
    ) -> Result<(), AnimationHostError> {
        if self.native_instances.borrow().contains_key(&instance) {
            self.engine
                .borrow_mut()
                .set_clock_mode(instance, AnimationClockMode::Internal)?;
            self.native_instances.borrow_mut().remove(&instance);
        }
        Ok(())
    }

    fn record_runtime_fallback(
        &self,
        instance: InstanceKey,
        unsupported: UnsupportedFeature,
        reason: impl Into<Box<str>>,
    ) {
        let mut reports = self.lowering_reports.borrow_mut();
        let Some(report) = reports.get_mut(&instance) else {
            return;
        };
        let backend = report.selected;
        if backend != AnimationBackend::Sampled {
            mark_native_fallback(report, backend, unsupported, reason);
        }
    }

    fn remove_hosted_for_events(&self, events: &[EngineEvent]) {
        let removed = events
            .iter()
            .filter_map(|event| match event {
                EngineEvent::Removed { instance } => Some(*instance),
                _ => None,
            })
            .collect::<Vec<_>>();
        if removed.is_empty() {
            return;
        }
        let mut natives = self.native_instances.borrow_mut();
        for instance in &removed {
            natives.remove(instance);
        }
        drop(natives);
        let mut reports = self.lowering_reports.borrow_mut();
        for instance in &removed {
            reports.remove(instance);
        }
        self.native_commands
            .borrow_mut()
            .retain(|command| !removed.contains(&host_command_instance(*command)));
    }

    pub fn performance_counters(&self) -> AnimationPerformanceCounters {
        AnimationPerformanceCounters {
            frames: self.counters.frames.get(),
            frame_callbacks_requested: self.counters.frame_callbacks_requested.get(),
            dirty_writes: self.counters.dirty_writes.get(),
            adapter_failures: self.counters.adapter_failures.get(),
            target_misses: self.counters.target_misses.get(),
            fallback_count: self.counters.fallback_count.get(),
            last_compute_ns: self.counters.last_compute_ns.get(),
            last_apply_ns: self.counters.last_apply_ns.get(),
            engine: self.engine.borrow().diagnostics(),
        }
    }

    pub(crate) fn record_frame_callback_requested(&self) {
        increment(&self.counters.frame_callbacks_requested);
    }

    pub fn subscribe(&self, listener: Rc<dyn Fn(EngineEvent)>) -> usize {
        let mut listeners = self.listeners.borrow_mut();
        if let Some((index, slot)) = listeners
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(listener);
            return index;
        }
        listeners.push(Some(listener));
        listeners.len() - 1
    }

    pub fn unsubscribe(&self, listener: usize) {
        if let Some(slot) = self.listeners.borrow_mut().get_mut(listener) {
            *slot = None;
        }
    }

    fn publish_events(&self, events: &[EngineEvent]) {
        for event in events.iter().copied() {
            let listener_count = self.listeners.borrow().len();
            for index in 0..listener_count {
                let listener = self.listeners.borrow().get(index).and_then(Clone::clone);
                if let Some(listener) = listener {
                    listener(event);
                }
            }
        }
    }
}

struct TickGuard<'a>(&'a Cell<bool>);

impl Drop for TickGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn mark_native_fallback(
    report: &mut LoweringReport,
    backend: AnimationBackend,
    unsupported: UnsupportedFeature,
    reason: impl Into<Box<str>>,
) {
    report.selected = AnimationBackend::Sampled;
    if !report.rejected_native.contains(&backend) {
        report.rejected_native.push(backend);
    }
    report.rejections.push(BackendRejection {
        backend,
        unsupported: vec![unsupported],
    });
    report.fallback_reason = Some(reason.into());
}

const fn host_command_instance(command: EngineCommand) -> InstanceKey {
    match command {
        EngineCommand::Play(instance)
        | EngineCommand::Pause(instance)
        | EngineCommand::Resume(instance)
        | EngineCommand::Restart(instance)
        | EngineCommand::Reverse(instance)
        | EngineCommand::Complete(instance)
        | EngineCommand::Cancel(instance)
        | EngineCommand::Reset(instance)
        | EngineCommand::Revert(instance)
        | EngineCommand::Refresh(instance)
        | EngineCommand::Remove(instance) => instance,
        EngineCommand::SetAlternate { instance, .. }
        | EngineCommand::Seek { instance, .. }
        | EngineCommand::AdvanceExternal { instance, .. }
        | EngineCommand::SeekOutputs { instance, .. }
        | EngineCommand::Stretch { instance, .. }
        | EngineCommand::SetPlaybackRate { instance, .. } => instance,
    }
}

const fn native_command_supported(command: EngineCommand) -> bool {
    matches!(
        command,
        EngineCommand::Play(_)
            | EngineCommand::Pause(_)
            | EngineCommand::Resume(_)
            | EngineCommand::Restart(_)
            | EngineCommand::Reverse(_)
            | EngineCommand::SetAlternate { .. }
            | EngineCommand::Complete(_)
            | EngineCommand::Cancel(_)
            | EngineCommand::Remove(_)
    )
}

const fn native_control_name(command: EngineCommand) -> &'static str {
    match command {
        EngineCommand::Play(_) => "play",
        EngineCommand::Pause(_) => "pause",
        EngineCommand::Resume(_) => "resume",
        EngineCommand::Restart(_) => "restart",
        EngineCommand::Reverse(_) => "reverse",
        EngineCommand::Complete(_) => "complete",
        EngineCommand::Cancel(_) => "cancel",
        EngineCommand::Seek { .. } => "seek",
        EngineCommand::AdvanceExternal { .. } => "external advance",
        EngineCommand::SeekOutputs { .. } => "output seek",
        EngineCommand::Reset(_) => "reset",
        EngineCommand::Revert(_) => "revert",
        EngineCommand::Stretch { .. } => "stretch",
        EngineCommand::Refresh(_) => "refresh",
        EngineCommand::Remove(_) => "remove",
        EngineCommand::SetAlternate { .. } => "alternate mode",
        EngineCommand::SetPlaybackRate { .. } => "playback rate",
    }
}

const fn runtime_unsupported_feature(command: EngineCommand) -> UnsupportedFeature {
    match command {
        EngineCommand::Pause(_) => UnsupportedFeature::Pause,
        EngineCommand::Resume(_) => UnsupportedFeature::Resume,
        EngineCommand::Reverse(_) => UnsupportedFeature::Reverse,
        EngineCommand::Cancel(_) => UnsupportedFeature::Cancel,
        EngineCommand::SetAlternate { .. } => UnsupportedFeature::Alternate,
        EngineCommand::Seek { .. } | EngineCommand::SeekOutputs { .. } => UnsupportedFeature::Seek,
        EngineCommand::Reset(_) => UnsupportedFeature::Reset,
        EngineCommand::Revert(_) => UnsupportedFeature::Revert,
        EngineCommand::Refresh(_) => UnsupportedFeature::Refresh,
        EngineCommand::Stretch { .. } => UnsupportedFeature::Stretch,
        EngineCommand::SetPlaybackRate { .. } => UnsupportedFeature::PlaybackRate,
        EngineCommand::AdvanceExternal { .. } => UnsupportedFeature::ExternalAdvance,
        EngineCommand::Play(_)
        | EngineCommand::Restart(_)
        | EngineCommand::Complete(_)
        | EngineCommand::Remove(_) => UnsupportedFeature::BackendUnavailable,
    }
}

fn increment(counter: &Cell<u64>) {
    counter.set(counter.get().saturating_add(1));
}

#[cfg(debug_assertions)]
fn elapsed_nanos(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}
