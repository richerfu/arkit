use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::time::Instant;

use arkit_animation_core::{
    AnimationBaselineSnapshot, AnimationCompileError, AnimationCompiler, AnimationEngine,
    AnimationResolveError, AnimationResolver, AnimationRuntimeError, EngineCommand, EngineEvent,
    InstanceId, TimelineSource, WindowMetrics,
};

use crate::{
    AdapterRegistry, AdapterResolutionSnapshot, AnimationAdapterError, ArkUiAdapter, TargetAdapter,
};
use crate::{
    AnimationBackend, BackendRejection, CapabilityRequirements, ExecutionPolicy, LoweringReport,
    NativeLowerer, NativeLoweringError, UnsupportedFeature,
};

type EngineListener = Rc<dyn Fn(EngineEvent)>;

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

pub struct AnimationHost {
    engine: RefCell<AnimationEngine>,
    registry: RefCell<AdapterRegistry>,
    arkui: Rc<ArkUiAdapter>,
    window_metrics: RefCell<WindowMetrics>,
    listeners: RefCell<Vec<Option<EngineListener>>>,
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
            counters: HostCounters::default(),
        }))
    }

    pub fn arkui(&self) -> &Rc<ArkUiAdapter> {
        &self.arkui
    }

    pub fn unregister_arkui_target(&self, target: arkit_animation_core::AdapterTargetId) -> bool {
        let events = {
            let mut engine = self.engine.borrow_mut();
            engine.detach_target(self.arkui.id(), target);
            engine.drain_events().collect::<Vec<_>>()
        };
        let removed = self.arkui.unregister_target(target);
        self.publish_events(&events);
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
        removed
    }

    pub fn insert_timeline(
        &self,
        source: &TimelineSource,
    ) -> Result<InstanceId, AnimationHostError> {
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
    ) -> Result<(InstanceId, LoweringReport), AnimationHostError> {
        let (plan, baselines) = match self.resolve(source) {
            Ok(resolved) => resolved,
            Err(AnimationHostError::Resolve(AnimationResolveError::EmptyTargetSelection)) => {
                increment(&self.counters.target_misses);
                return Err(AnimationResolveError::EmptyTargetSelection.into());
            }
            Err(error) => return Err(error),
        };
        let mut report = NativeLowerer.lower_plan(policy, &plan, requirements)?;
        if report.selected != AnimationBackend::Sampled {
            if policy == ExecutionPolicy::NativeOnly {
                return Err(NativeLoweringError::BackendUnavailable {
                    backend: report.selected,
                }
                .into());
            }
            let backend = report.selected;
            report.selected = AnimationBackend::Sampled;
            report.rejected_native.push(backend);
            report.rejections.push(BackendRejection {
                backend,
                unsupported: vec![UnsupportedFeature::BackendUnavailable],
            });
            report.fallback_reason = Some(
                "native UIContext was not installed; using the semantically equivalent sampled backend"
                    .into(),
            );
        }
        if report.fallback_reason.is_some() {
            increment(&self.counters.fallback_count);
        }
        let instance = self
            .engine
            .borrow_mut()
            .insert(plan, baselines)
            .map_err(AnimationHostError::from)?;
        Ok((instance, report))
    }

    pub fn refresh_timeline(
        &self,
        instance: InstanceId,
        source: &TimelineSource,
    ) -> Result<(), AnimationHostError> {
        let (plan, baselines) = self.resolve(source)?;
        self.engine
            .borrow_mut()
            .replace_resolution(instance, plan, baselines)
            .map_err(Into::into)
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
        self.engine.borrow_mut().enqueue(command);
    }

    pub fn tick(
        &self,
        frame_time: arkit_animation_core::TimePoint,
    ) -> Result<Vec<EngineEvent>, AnimationHostError> {
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
        let events: Vec<_> = engine.drain_events().collect();
        drop(engine);
        self.publish_events(&events);
        Ok(events)
    }

    pub fn snapshot(
        &self,
        instance: InstanceId,
    ) -> Option<arkit_animation_core::AnimationInstanceSnapshot> {
        self.engine.borrow().snapshot(instance)
    }

    pub fn has_work(&self) -> bool {
        self.engine.borrow().has_work()
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

fn increment(counter: &Cell<u64>) {
    counter.set(counter.get().saturating_add(1));
}

#[cfg(debug_assertions)]
fn elapsed_nanos(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}
