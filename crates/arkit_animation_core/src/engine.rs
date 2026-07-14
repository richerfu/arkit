//! Single-owner command and logical-clock engine shared by every backend.

use std::collections::VecDeque;
use std::sync::Arc;

use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationOutcome, AnimationRuntimeError,
    AnimationSampler, AnimationValue, CallPolicy, CompiledAnimation, CompiledEvent, EngineCommand,
    EngineEvent, EngineOutputId, FrameBatch, FrameId, InstanceId, InstanceKey, InvalidationClass,
    OutputId, OutputSeek, PlaybackDirection, PlaybackRate, PlaybackState, PropertyDescriptor,
    PropertyUpdate, SeekMode, TimeDomainId, TimeDomainMapper, TimeDomainOptions, TimeDomainPhase,
    TimeDomainSample, TimeExtent, TimePoint, TimeSpan, TimelineNodeId, TrackCursor, TrackId,
    TrackSampleContext,
};

pub struct AnimationBaselineSnapshot {
    values: IndexVec<OutputId, AnimationValue>,
}

impl AnimationBaselineSnapshot {
    pub fn from_output_values(values: Vec<AnimationValue>) -> Self {
        Self {
            values: IndexVec::from_vec(values),
        }
    }

    pub fn values(&self) -> &oxc_index::IndexSlice<OutputId, [AnimationValue]> {
        &self.values
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlobalOutputKey {
    adapter: AdapterId,
    target: AdapterTargetId,
    property: AdapterPropertyId,
}

#[derive(Debug, Clone, Copy)]
struct OutputContributor {
    instance: InstanceKey,
    output: OutputId,
}

struct EngineOutputSlot {
    key: GlobalOutputKey,
    descriptor: PropertyDescriptor,
    contributors: Vec<OutputContributor>,
    current: Option<AnimationValue>,
    touched: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SampledOutput {
    replace: Option<AnimationValue>,
    additive: Option<AnimationValue>,
}

impl SampledOutput {
    fn approximately_eq(&self, other: &Self, precision: f32) -> bool {
        optional_value_approximately_eq(&self.replace, &other.replace, precision)
            && optional_value_approximately_eq(&self.additive, &other.additive, precision)
    }
}

#[derive(Default)]
struct PendingFrameEvents {
    deferred: Vec<EngineEvent>,
}

fn optional_value_approximately_eq(
    left: &Option<AnimationValue>,
    right: &Option<AnimationValue>,
    precision: f32,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.approximately_eq(right, precision),
        (None, None) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationInstanceSnapshot {
    pub state: PlaybackState,
    pub direction: PlaybackDirection,
    pub elapsed: TimePoint,
    pub local_time: TimePoint,
    pub completed_iterations: u64,
    pub alternate: bool,
}

/// Selects who advances an animation instance's absolute clock.
///
/// External clocks are used by platform animators: the native backend owns
/// scheduling and feeds positions back through [`EngineCommand::Seek`], while
/// the engine remains the sole owner of sampling, composition, and events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AnimationClockMode {
    #[default]
    Internal,
    External,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EngineDiagnostics {
    pub live_instances: usize,
    pub active_instances: usize,
    pub output_slots: usize,
    pub pending_commands: usize,
    pub dirty_writes: usize,
}

pub struct AnimationEngine {
    instances: IndexVec<InstanceId, Option<AnimationInstance>>,
    generations: IndexVec<InstanceId, u64>,
    active: Vec<InstanceKey>,
    command_queue: VecDeque<EngineCommand>,
    event_queue: Vec<EngineEvent>,
    frame_batch: FrameBatch,
    output_slots: IndexVec<EngineOutputId, EngineOutputSlot>,
    output_lookup: FxHashMap<GlobalOutputKey, EngineOutputId>,
    ordered_outputs: Vec<EngineOutputId>,
    free_output_slots: Vec<EngineOutputId>,
    next_activation_sequence: u64,
    next_frame_sequence: u64,
    pending_frame: Option<FrameId>,
    frame_events: PendingFrameEvents,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            instances: IndexVec::new(),
            generations: IndexVec::new(),
            active: Vec::new(),
            command_queue: VecDeque::new(),
            event_queue: Vec::new(),
            frame_batch: FrameBatch::new(),
            output_slots: IndexVec::new(),
            output_lookup: FxHashMap::default(),
            ordered_outputs: Vec::new(),
            free_output_slots: Vec::new(),
            next_activation_sequence: 0,
            next_frame_sequence: 0,
            pending_frame: None,
            frame_events: PendingFrameEvents::default(),
        }
    }

    pub fn insert(
        &mut self,
        plan: Arc<CompiledAnimation>,
        baselines: AnimationBaselineSnapshot,
    ) -> Result<InstanceKey, AnimationRuntimeError> {
        self.insert_with_clock(plan, baselines, AnimationClockMode::Internal)
    }

    pub fn insert_with_clock(
        &mut self,
        plan: Arc<CompiledAnimation>,
        baselines: AnimationBaselineSnapshot,
        clock_mode: AnimationClockMode,
    ) -> Result<InstanceKey, AnimationRuntimeError> {
        validate_baselines(&plan, &baselines)?;
        let slot = self
            .instances
            .iter_enumerated()
            .find_map(|(id, instance)| instance.is_none().then_some(id))
            .unwrap_or_else(|| InstanceId::new(self.instances.len()));
        let generation = if slot.index() == self.instances.len() {
            1
        } else {
            self.generations[slot]
                .checked_add(1)
                .ok_or(AnimationRuntimeError::InstanceGenerationExhausted(slot))?
        };
        let instance = InstanceKey::from_parts(slot, generation);
        let output_slots = self.register_outputs(instance, &plan)?;
        let value = Some(AnimationInstance::new(
            plan,
            baselines,
            output_slots,
            clock_mode,
        ));
        if slot.index() == self.instances.len() {
            let inserted = self.instances.push(value);
            let generation_slot = self.generations.push(generation);
            debug_assert_eq!(inserted, slot);
            debug_assert_eq!(generation_slot, slot);
        } else {
            self.generations[slot] = generation;
            self.instances[slot] = value;
        }
        Ok(instance)
    }

    /// Changes the clock owner without changing playback state or position.
    /// The first internal frame after a handoff establishes a new time origin,
    /// so backend setup/fallback latency is never added to animation elapsed
    /// time.
    pub fn set_clock_mode(
        &mut self,
        instance: InstanceKey,
        clock_mode: AnimationClockMode,
    ) -> Result<(), AnimationRuntimeError> {
        if let Some(frame) = self.pending_frame {
            return Err(AnimationRuntimeError::FrameNotAcknowledged(frame));
        }
        let instance_id = instance.slot();
        if self.generations.raw.get(instance_id.index()).copied() != Some(instance.generation()) {
            return Err(AnimationRuntimeError::UnknownInstance(instance));
        }
        let Some(value) = self.instances[instance_id].as_mut() else {
            return Err(AnimationRuntimeError::UnknownInstance(instance));
        };
        if value.clock_mode == clock_mode {
            return Ok(());
        }
        value.clock_mode = clock_mode;
        value.last_frame = None;
        match clock_mode {
            AnimationClockMode::Internal if value.state == PlaybackState::Running => {
                ensure_active(&mut self.active, instance);
            }
            AnimationClockMode::External => {
                self.active.retain(|candidate| *candidate != instance);
            }
            AnimationClockMode::Internal => {}
        }
        Ok(())
    }

    pub fn enqueue(&mut self, command: EngineCommand) {
        if let Some(pending) = self.command_queue.back_mut() {
            match (pending, command) {
                (
                    EngineCommand::Seek {
                        instance: pending_instance,
                        position: pending_position,
                        mode: SeekMode::SuppressEvents,
                    },
                    EngineCommand::Seek {
                        instance,
                        position,
                        mode: SeekMode::SuppressEvents,
                    },
                ) if *pending_instance == instance => {
                    *pending_position = position;
                    return;
                }
                (
                    EngineCommand::SeekOutputs {
                        instance: pending_instance,
                        first: pending_first,
                        second: pending_second,
                    },
                    EngineCommand::SeekOutputs {
                        instance,
                        first,
                        second,
                    },
                ) if *pending_instance == instance => {
                    *pending_first = first;
                    *pending_second = second;
                    return;
                }
                _ => {}
            }
        }
        self.command_queue.push_back(command);
    }

    pub fn tick(&mut self, frame_time: TimePoint) -> Result<FrameId, AnimationRuntimeError> {
        if let Some(frame) = self.pending_frame {
            return Err(AnimationRuntimeError::FrameNotAcknowledged(frame));
        }
        self.next_frame_sequence = self
            .next_frame_sequence
            .checked_add(1)
            .ok_or(AnimationRuntimeError::FrameSequenceExhausted)?;
        let frame = FrameId::new(self.next_frame_sequence);
        self.frame_events.deferred.clear();
        self.frame_batch.clear();
        while let Some(command) = self.command_queue.pop_front() {
            self.process_command(command, frame_time, frame);
        }

        let instances = &mut self.instances;
        let events = &mut self.event_queue;
        let output_slots = &mut self.output_slots;
        for instance_key in self.active.iter().copied() {
            let instance_id = instance_key.slot();
            if self.generations[instance_id] == instance_key.generation() {
                if let Some(instance) = instances[instance_id].as_mut() {
                    advance_instance(
                        instance_key,
                        instance,
                        frame_time,
                        frame,
                        events,
                        &mut self.frame_events,
                        output_slots,
                    );
                }
            }
        }
        flush_output_slots(
            &self.instances,
            &self.generations,
            &mut self.output_slots,
            &self.ordered_outputs,
            &mut self.frame_batch,
            &mut self.event_queue,
        );
        self.active.retain(|instance| {
            self.generations[instance.slot()] == instance.generation()
                && self.instances[instance.slot()]
                    .as_ref()
                    .is_some_and(|instance| instance.state == PlaybackState::Running)
        });
        self.pending_frame = Some(frame);
        Ok(frame)
    }

    pub fn acknowledge_frame(&mut self, frame: FrameId) -> Result<(), AnimationRuntimeError> {
        let Some(expected) = self.pending_frame else {
            return Err(AnimationRuntimeError::NoFramePending(frame));
        };
        if expected != frame {
            return Err(AnimationRuntimeError::UnexpectedFrameAcknowledgement {
                expected,
                actual: frame,
            });
        }
        for instance_index in 0..self.instances.len() {
            let instance_id = InstanceId::new(instance_index);
            let Some(instance) = self.instances[instance_id].as_mut() else {
                continue;
            };
            if instance.pending_render_frame == Some(frame) {
                self.event_queue.push(EngineEvent::Render {
                    instance: InstanceKey::from_parts(instance_id, self.generations[instance_id]),
                    at: instance.pending_render_at,
                });
                instance.pending_render_frame = None;
            }
        }
        self.event_queue.append(&mut self.frame_events.deferred);
        self.pending_frame = None;
        Ok(())
    }

    pub fn reject_frame(&mut self, frame: FrameId) -> Result<(), AnimationRuntimeError> {
        let Some(expected) = self.pending_frame else {
            return Err(AnimationRuntimeError::NoFramePending(frame));
        };
        if expected != frame {
            return Err(AnimationRuntimeError::UnexpectedFrameAcknowledgement {
                expected,
                actual: frame,
            });
        }
        for instance in self.instances.iter_mut().flatten() {
            if instance.pending_render_frame == Some(frame) {
                instance.pending_render_frame = None;
            }
        }
        self.frame_events.deferred.clear();
        self.pending_frame = None;
        Ok(())
    }

    pub const fn pending_frame(&self) -> Option<FrameId> {
        self.pending_frame
    }

    pub fn replace_resolution(
        &mut self,
        instance: InstanceKey,
        plan: Arc<CompiledAnimation>,
        baselines: AnimationBaselineSnapshot,
    ) -> Result<(), AnimationRuntimeError> {
        if let Some(frame) = self.pending_frame {
            return Err(AnimationRuntimeError::FrameNotAcknowledged(frame));
        }
        validate_baselines(&plan, &baselines)?;
        self.validate_output_contracts(&plan)?;
        let instance_id = instance.slot();
        if self.generations.raw.get(instance_id.index()).copied() != Some(instance.generation()) {
            return Err(AnimationRuntimeError::UnknownInstance(instance));
        }
        let Some(previous) = self
            .instances
            .raw
            .get_mut(instance_id.index())
            .and_then(Option::take)
        else {
            return Err(AnimationRuntimeError::UnknownInstance(instance));
        };
        self.unregister_outputs(instance, &previous.output_slots);
        let output_slots = self.register_outputs(instance, &plan)?;
        let mut refreshed =
            AnimationInstance::new(plan, baselines, output_slots, previous.clock_mode);
        refreshed.inherit_runtime(&previous);
        refreshed.update_local_time();
        touch_instance_outputs(&refreshed, &mut self.output_slots);
        self.instances[instance_id] = Some(refreshed);
        Ok(())
    }

    pub fn snapshot(&self, instance: InstanceKey) -> Option<AnimationInstanceSnapshot> {
        if self.generations.raw.get(instance.slot().index()).copied()? != instance.generation() {
            return None;
        }
        self.instances
            .raw
            .get(instance.slot().index())?
            .as_ref()
            .map(AnimationInstance::snapshot)
    }

    pub fn events(&self) -> &[EngineEvent] {
        &self.event_queue
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = EngineEvent> + '_ {
        self.event_queue.drain(..)
    }

    pub const fn frame_batch(&self) -> &FrameBatch {
        &self.frame_batch
    }

    pub fn has_work(&self) -> bool {
        !self.active.is_empty() || !self.command_queue.is_empty() || self.pending_frame.is_some()
    }

    pub fn diagnostics(&self) -> EngineDiagnostics {
        EngineDiagnostics {
            live_instances: self.instances.iter().flatten().count(),
            active_instances: self.active.len(),
            output_slots: self.ordered_outputs.len(),
            pending_commands: self.command_queue.len(),
            dirty_writes: self.frame_batch.len(),
        }
    }

    /// Detaches a disposed adapter target on the cold lifecycle path. Any
    /// instance touching the target is cancelled as a unit so no later frame
    /// can attempt to write a stale native handle.
    pub fn detach_target(&mut self, adapter: AdapterId, target: AdapterTargetId) -> usize {
        let affected = self
            .instances
            .iter_enumerated()
            .filter_map(|(instance_id, instance)| {
                instance.as_ref().and_then(|instance| {
                    instance
                        .plan
                        .targets()
                        .iter()
                        .any(|candidate| {
                            candidate.adapter == adapter && candidate.adapter_target == target
                        })
                        .then(|| {
                            InstanceKey::from_parts(instance_id, self.generations[instance_id])
                        })
                })
            })
            .collect::<Vec<_>>();
        for instance in affected.iter().copied() {
            self.event_queue.push(EngineEvent::Cancel { instance });
            self.event_queue.push(EngineEvent::StateChanged {
                instance,
                state: PlaybackState::Cancelled,
            });
            self.event_queue.push(EngineEvent::Settled {
                instance,
                outcome: AnimationOutcome::Cancelled,
            });
            self.remove_instance(instance);
        }
        self.active.retain(|instance| {
            self.generations[instance.slot()] == instance.generation()
                && self.instances[instance.slot()].is_some()
        });
        let removed_slots = self
            .ordered_outputs
            .iter()
            .copied()
            .filter(|slot| {
                let key = self.output_slots[*slot].key;
                key.adapter == adapter && key.target == target
            })
            .collect::<Vec<_>>();
        self.ordered_outputs
            .retain(|slot| !removed_slots.contains(slot));
        for slot in removed_slots {
            self.output_lookup.remove(&self.output_slots[slot].key);
            self.output_slots[slot].contributors.clear();
            self.output_slots[slot].current = None;
            self.output_slots[slot].touched = false;
            self.free_output_slots.push(slot);
        }
        affected.len()
    }

    pub fn detach_adapter(&mut self, adapter: AdapterId) -> usize {
        let affected = self
            .instances
            .iter_enumerated()
            .filter_map(|(instance_id, instance)| {
                instance.as_ref().and_then(|instance| {
                    instance
                        .plan
                        .targets()
                        .iter()
                        .any(|candidate| candidate.adapter == adapter)
                        .then(|| {
                            InstanceKey::from_parts(instance_id, self.generations[instance_id])
                        })
                })
            })
            .collect::<Vec<_>>();
        for instance in affected.iter().copied() {
            self.event_queue.push(EngineEvent::Cancel { instance });
            self.event_queue.push(EngineEvent::StateChanged {
                instance,
                state: PlaybackState::Cancelled,
            });
            self.event_queue.push(EngineEvent::Settled {
                instance,
                outcome: AnimationOutcome::Cancelled,
            });
            self.remove_instance(instance);
        }
        self.active.retain(|instance| {
            self.generations[instance.slot()] == instance.generation()
                && self.instances[instance.slot()].is_some()
        });
        let removed_slots = self
            .ordered_outputs
            .iter()
            .copied()
            .filter(|slot| self.output_slots[*slot].key.adapter == adapter)
            .collect::<Vec<_>>();
        self.ordered_outputs
            .retain(|slot| !removed_slots.contains(slot));
        for slot in removed_slots {
            self.output_lookup.remove(&self.output_slots[slot].key);
            self.output_slots[slot].contributors.clear();
            self.output_slots[slot].current = None;
            self.output_slots[slot].touched = false;
            self.free_output_slots.push(slot);
        }
        affected.len()
    }

    fn register_outputs(
        &mut self,
        instance: InstanceKey,
        plan: &CompiledAnimation,
    ) -> Result<IndexVec<OutputId, EngineOutputId>, AnimationRuntimeError> {
        self.validate_output_contracts(plan)?;

        let mut instance_slots = IndexVec::with_capacity(plan.outputs().len());
        for (output_id, output) in plan.outputs().iter_enumerated() {
            let target = &plan.targets()[output.target];
            let property = &plan.properties()[output.property];
            let key = GlobalOutputKey {
                adapter: target.adapter,
                target: target.adapter_target,
                property: property.adapter_property,
            };
            let slot_id = if let Some(slot) = self.output_lookup.get(&key).copied() {
                slot
            } else {
                let new_slot = EngineOutputSlot {
                    key,
                    descriptor: property.descriptor.clone(),
                    contributors: Vec::new(),
                    current: None,
                    touched: true,
                };
                let slot = if let Some(slot) = self.free_output_slots.pop() {
                    self.output_slots[slot] = new_slot;
                    slot
                } else {
                    self.output_slots.push(new_slot)
                };
                self.output_lookup.insert(key, slot);
                let order_key = output_order_key(&self.output_slots[slot]);
                let insertion = self
                    .ordered_outputs
                    .binary_search_by_key(&order_key, |candidate| {
                        output_order_key(&self.output_slots[*candidate])
                    })
                    .unwrap_or_else(|index| index);
                self.ordered_outputs.insert(insertion, slot);
                slot
            };
            self.output_slots[slot_id]
                .contributors
                .push(OutputContributor {
                    instance,
                    output: output_id,
                });
            self.output_slots[slot_id].touched = true;
            instance_slots.push(slot_id);
        }
        Ok(instance_slots)
    }

    fn validate_output_contracts(
        &self,
        plan: &CompiledAnimation,
    ) -> Result<(), AnimationRuntimeError> {
        for output in plan.outputs() {
            let target = &plan.targets()[output.target];
            let property = &plan.properties()[output.property];
            let key = GlobalOutputKey {
                adapter: target.adapter,
                target: target.adapter_target,
                property: property.adapter_property,
            };
            if let Some(slot) = self.output_lookup.get(&key).copied() {
                if self.output_slots[slot].descriptor != property.descriptor {
                    return Err(AnimationRuntimeError::GlobalPropertyContractMismatch {
                        adapter: key.adapter,
                        target: key.target,
                        property: key.property,
                    });
                }
            }
        }
        Ok(())
    }

    fn unregister_outputs(
        &mut self,
        instance: InstanceKey,
        output_slots: &oxc_index::IndexSlice<OutputId, [EngineOutputId]>,
    ) {
        for slot_id in output_slots.iter().copied() {
            let slot = &mut self.output_slots[slot_id];
            slot.contributors
                .retain(|contributor| contributor.instance != instance);
            slot.touched = true;
        }
    }

    fn process_command(&mut self, command: EngineCommand, frame_time: TimePoint, frame: FrameId) {
        let instance_key = command_instance(command);
        if matches!(command, EngineCommand::Remove(_)) {
            self.remove_instance(instance_key);
            return;
        }
        let activation_sequence = matches!(
            command,
            EngineCommand::Play(_) | EngineCommand::Resume(_) | EngineCommand::Restart(_)
        )
        .then(|| {
            self.next_activation_sequence = self
                .next_activation_sequence
                .checked_add(1)
                .expect("animation activation sequence space exhausted");
            self.next_activation_sequence
        });
        let instance_id = instance_key.slot();
        let generation_matches = self
            .generations
            .raw
            .get(instance_id.index())
            .is_some_and(|generation| *generation == instance_key.generation());
        let Some(instance) = generation_matches
            .then(|| {
                self.instances
                    .raw
                    .get_mut(instance_id.index())
                    .and_then(Option::as_mut)
            })
            .flatten()
        else {
            self.event_queue.push(EngineEvent::Error {
                instance: instance_key,
                error: AnimationRuntimeError::UnknownInstance(instance_key),
            });
            return;
        };

        match command {
            EngineCommand::Play(_) => {
                if instance.state != PlaybackState::Running {
                    if matches!(
                        instance.state,
                        PlaybackState::Completed
                            | PlaybackState::Cancelled
                            | PlaybackState::Reverted
                    ) {
                        instance.reset_clock();
                    }
                    instance.state = PlaybackState::Running;
                    instance.last_frame = Some(frame_time);
                    instance.activation_sequence = activation_sequence.unwrap_or_default();
                    touch_instance_outputs(instance, &mut self.output_slots);
                    if instance.clock_mode == AnimationClockMode::Internal {
                        ensure_active(&mut self.active, instance_key);
                    }
                    self.event_queue.push(EngineEvent::StateChanged {
                        instance: instance_key,
                        state: PlaybackState::Running,
                    });
                }
            }
            EngineCommand::Pause(_) if instance.state == PlaybackState::Running => {
                instance.state = PlaybackState::Paused;
                instance.last_frame = None;
                self.event_queue.push(EngineEvent::Pause {
                    instance: instance_key,
                });
                self.event_queue.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Paused,
                });
            }
            EngineCommand::Resume(_) if instance.state == PlaybackState::Paused => {
                instance.state = PlaybackState::Running;
                instance.last_frame = Some(frame_time);
                instance.activation_sequence = activation_sequence.unwrap_or_default();
                touch_instance_outputs(instance, &mut self.output_slots);
                if instance.clock_mode == AnimationClockMode::Internal {
                    ensure_active(&mut self.active, instance_key);
                }
                self.event_queue.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Running,
                });
            }
            EngineCommand::Restart(_) => {
                instance.reset_clock();
                instance.state = PlaybackState::Running;
                instance.last_frame = Some(frame_time);
                instance.activation_sequence = activation_sequence.unwrap_or_default();
                touch_instance_outputs(instance, &mut self.output_slots);
                if instance.clock_mode == AnimationClockMode::Internal {
                    ensure_active(&mut self.active, instance_key);
                }
                self.event_queue.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Running,
                });
            }
            EngineCommand::Reverse(_) => {
                instance.direction = instance.direction.reversed();
            }
            EngineCommand::SetAlternate { enabled, .. } => {
                instance.alternate = enabled;
                instance.update_local_time();
                sample_and_emit_update(
                    instance_key,
                    instance,
                    frame,
                    &mut self.event_queue,
                    &mut self.output_slots,
                );
            }
            EngineCommand::Seek { position, mode, .. } => {
                instance.elapsed = position;
                instance.update_local_time();
                if mode == SeekMode::FireCrossingEvents {
                    fire_crossing_events(instance_key, instance, &mut self.frame_events.deferred);
                }
                sample_and_emit_update(
                    instance_key,
                    instance,
                    frame,
                    &mut self.event_queue,
                    &mut self.output_slots,
                );
            }
            EngineCommand::AdvanceExternal { position, .. }
                if instance.clock_mode == AnimationClockMode::External
                    && instance.state == PlaybackState::Running =>
            {
                advance_instance_to(
                    instance_key,
                    instance,
                    position,
                    frame,
                    &mut self.event_queue,
                    &mut self.frame_events,
                    &mut self.output_slots,
                );
            }
            EngineCommand::AdvanceExternal { .. } => {}
            EngineCommand::SeekOutputs { first, second, .. } => {
                sample_output_seeks(
                    instance_key,
                    instance,
                    frame,
                    first,
                    second,
                    &mut self.event_queue,
                    &mut self.output_slots,
                );
            }
            EngineCommand::Complete(_)
                if matches!(
                    instance.state,
                    PlaybackState::Completed | PlaybackState::Cancelled | PlaybackState::Reverted
                ) => {}
            EngineCommand::Complete(_) => match instance.parent_extent() {
                TimeExtent::Infinite => self.event_queue.push(EngineEvent::Error {
                    instance: instance_key,
                    error: AnimationRuntimeError::InfiniteAnimationCannotComplete(instance_key),
                }),
                TimeExtent::Finite(duration) => {
                    instance.elapsed = match instance.direction {
                        PlaybackDirection::Forward => TimePoint::from_nanos(duration.as_nanos()),
                        PlaybackDirection::Reverse => TimePoint::ZERO,
                    };
                    instance.update_local_time();
                    if !instance.begin_emitted {
                        instance.begin_emitted = true;
                        self.event_queue.push(EngineEvent::Begin {
                            instance: instance_key,
                        });
                    }
                    fire_crossing_events(instance_key, instance, &mut self.frame_events.deferred);
                    sample_and_emit_update(
                        instance_key,
                        instance,
                        frame,
                        &mut self.event_queue,
                        &mut self.output_slots,
                    );
                    instance.state = PlaybackState::Completed;
                    self.frame_events.deferred.push(EngineEvent::Complete {
                        instance: instance_key,
                    });
                    self.frame_events.deferred.push(EngineEvent::StateChanged {
                        instance: instance_key,
                        state: PlaybackState::Completed,
                    });
                    self.frame_events.deferred.push(EngineEvent::Settled {
                        instance: instance_key,
                        outcome: AnimationOutcome::Completed,
                    });
                }
            },
            EngineCommand::Cancel(_)
                if matches!(
                    instance.state,
                    PlaybackState::Completed | PlaybackState::Cancelled | PlaybackState::Reverted
                ) => {}
            EngineCommand::Cancel(_) => {
                instance.state = PlaybackState::Cancelled;
                instance.last_frame = None;
                self.frame_events.deferred.push(EngineEvent::Cancel {
                    instance: instance_key,
                });
                self.frame_events.deferred.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Cancelled,
                });
                self.frame_events.deferred.push(EngineEvent::Settled {
                    instance: instance_key,
                    outcome: AnimationOutcome::Cancelled,
                });
            }
            EngineCommand::Reset(_) => {
                instance.reset_clock();
                instance.update_local_time();
                instance.state = PlaybackState::Idle;
                sample_and_emit_update(
                    instance_key,
                    instance,
                    frame,
                    &mut self.event_queue,
                    &mut self.output_slots,
                );
                self.event_queue.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Idle,
                });
            }
            EngineCommand::Revert(_) if instance.state == PlaybackState::Reverted => {}
            EngineCommand::Revert(_) => {
                instance.reset_clock();
                instance.state = PlaybackState::Reverted;
                touch_instance_outputs(instance, &mut self.output_slots);
                if !instance.output_slots.is_empty() {
                    instance.pending_render_frame = Some(frame);
                    instance.pending_render_at = TimePoint::ZERO;
                }
                self.frame_events.deferred.push(EngineEvent::Revert {
                    instance: instance_key,
                });
                self.frame_events.deferred.push(EngineEvent::StateChanged {
                    instance: instance_key,
                    state: PlaybackState::Reverted,
                });
                self.frame_events.deferred.push(EngineEvent::Settled {
                    instance: instance_key,
                    outcome: AnimationOutcome::Reverted,
                });
            }
            EngineCommand::Stretch { duration, .. } => {
                instance.stretched_duration = Some(duration);
                instance.update_local_time();
                sample_and_emit_update(
                    instance_key,
                    instance,
                    frame,
                    &mut self.event_queue,
                    &mut self.output_slots,
                );
            }
            EngineCommand::Refresh(_) => {
                instance.last_frame =
                    (instance.state == PlaybackState::Running).then_some(frame_time);
                self.event_queue.push(EngineEvent::RefreshRequested {
                    instance: instance_key,
                    at: instance.local_time,
                });
            }
            EngineCommand::SetPlaybackRate { rate, .. } => {
                let plan_rate = instance.plan.settings().playback_rate.get();
                instance.clock_rate =
                    PlaybackRate::new(rate.get() / plan_rate).unwrap_or(PlaybackRate::NORMAL);
            }
            EngineCommand::Remove(_) => unreachable!("remove commands are handled before lookup"),
            EngineCommand::Pause(_) | EngineCommand::Resume(_) => {}
        }
    }

    fn remove_instance(&mut self, instance: InstanceKey) {
        let instance_id = instance.slot();
        if self.generations.raw.get(instance_id.index()).copied() != Some(instance.generation()) {
            self.event_queue.push(EngineEvent::Error {
                instance,
                error: AnimationRuntimeError::UnknownInstance(instance),
            });
            return;
        }
        let Some(instance_value) = self
            .instances
            .raw
            .get(instance_id.index())
            .and_then(Option::as_ref)
        else {
            self.event_queue.push(EngineEvent::Error {
                instance,
                error: AnimationRuntimeError::UnknownInstance(instance),
            });
            return;
        };
        let output_slots = instance_value.output_slots.clone();
        self.unregister_outputs(instance, &output_slots);
        self.instances[instance_id] = None;
        self.active.retain(|active| *active != instance);
        self.command_queue
            .retain(|command| command_instance(*command) != instance);
        self.event_queue.push(EngineEvent::Removed { instance });
    }
}

struct AnimationInstance {
    plan: Arc<CompiledAnimation>,
    state: PlaybackState,
    direction: PlaybackDirection,
    elapsed: TimePoint,
    local_time: TimePoint,
    last_frame: Option<TimePoint>,
    clock_mode: AnimationClockMode,
    completed_iterations: u64,
    begin_emitted: bool,
    alternate: bool,
    clock_rate: PlaybackRate,
    stretched_duration: Option<TimeSpan>,
    baselines: AnimationBaselineSnapshot,
    cursors: IndexVec<TrackId, TrackCursor>,
    output_slots: IndexVec<OutputId, EngineOutputId>,
    output_values: IndexVec<OutputId, Option<SampledOutput>>,
    activation_sequence: u64,
    pending_render_frame: Option<FrameId>,
    pending_render_at: TimePoint,
    domain_samples: IndexVec<TimeDomainId, TimeDomainSample>,
    previous_domain_samples: IndexVec<TimeDomainId, TimeDomainSample>,
    fired_once: IndexVec<TimelineNodeId, bool>,
}

impl AnimationInstance {
    fn new(
        plan: Arc<CompiledAnimation>,
        baselines: AnimationBaselineSnapshot,
        output_slots: IndexVec<OutputId, EngineOutputId>,
        clock_mode: AnimationClockMode,
    ) -> Self {
        let alternate = plan.settings().alternate;
        let cursors = IndexVec::from_vec(vec![TrackCursor::default(); plan.tracks().len()]);
        let output_values = IndexVec::from_vec(vec![None; plan.outputs().len()]);
        let domain_samples = IndexVec::from_vec(
            plan.domains()
                .iter()
                .map(|_| TimeDomainSample {
                    phase: TimeDomainPhase::BeforeDelay,
                    local_time: TimePoint::ZERO,
                    iteration: 0,
                    direction: PlaybackDirection::Forward,
                })
                .collect(),
        );
        let previous_domain_samples = domain_samples.clone();
        let fired_once = IndexVec::from_vec(vec![false; plan.events().len()]);
        Self {
            plan,
            state: PlaybackState::Idle,
            direction: PlaybackDirection::Forward,
            elapsed: TimePoint::ZERO,
            local_time: TimePoint::ZERO,
            last_frame: None,
            clock_mode,
            completed_iterations: 0,
            begin_emitted: false,
            alternate,
            clock_rate: PlaybackRate::NORMAL,
            stretched_duration: None,
            baselines,
            cursors,
            output_slots,
            output_values,
            activation_sequence: 0,
            pending_render_frame: None,
            pending_render_at: TimePoint::ZERO,
            domain_samples,
            previous_domain_samples,
            fired_once,
        }
    }

    fn snapshot(&self) -> AnimationInstanceSnapshot {
        AnimationInstanceSnapshot {
            state: self.state,
            direction: self.direction,
            elapsed: self.elapsed,
            local_time: self.local_time,
            completed_iterations: self.completed_iterations,
            alternate: self.alternate,
        }
    }

    fn inherit_runtime(&mut self, previous: &Self) {
        self.state = previous.state;
        self.direction = previous.direction;
        self.elapsed = previous.elapsed;
        self.local_time = previous.local_time;
        self.last_frame = previous.last_frame;
        self.clock_mode = previous.clock_mode;
        self.completed_iterations = previous.completed_iterations;
        self.begin_emitted = previous.begin_emitted;
        self.alternate = previous.alternate;
        self.clock_rate = previous.clock_rate;
        self.stretched_duration = previous.stretched_duration;
        self.activation_sequence = previous.activation_sequence;
    }

    fn root_domain(&self) -> &crate::CompiledTimeDomain {
        &self.plan.domains()[TimeDomainId::new(0)]
    }

    fn parent_extent(&self) -> TimeExtent {
        self.stretched_duration
            .map_or_else(|| self.root_domain().parent_extent(), TimeExtent::Finite)
    }

    fn mapped_plan_elapsed(&self) -> TimePoint {
        let Some(stretched) = self.stretched_duration else {
            return self.elapsed;
        };
        let TimeExtent::Finite(original) = self.root_domain().parent_extent() else {
            return self.elapsed;
        };
        if stretched == TimeSpan::ZERO {
            return TimePoint::from_nanos(original.as_nanos());
        }
        let progress = self.elapsed.as_nanos() as f64 / stretched.as_nanos() as f64;
        TimePoint::from_nanos(
            (original.as_nanos() as f64 * progress.clamp(0.0, 1.0)).round() as u64,
        )
    }

    fn update_local_time(&mut self) -> crate::TimeDomainSample {
        self.previous_domain_samples
            .raw
            .copy_from_slice(&self.domain_samples.raw);
        let root_time = self.mapped_plan_elapsed();
        let sample = TimeDomainMapper::sample_with_options(
            self.root_domain(),
            root_time,
            TimeDomainOptions {
                reversed: self.root_domain().settings.reversed,
                alternate: self.alternate,
            },
        );
        self.local_time = sample.local_time;
        self.domain_samples[TimeDomainId::new(0)] = sample;
        for domain_index in 1..self.plan.domains().len() {
            let domain_id = TimeDomainId::new(domain_index);
            let domain = &self.plan.domains()[domain_id];
            if let Some(parent) = domain.parent {
                self.domain_samples[domain_id] =
                    TimeDomainMapper::sample(domain, self.domain_samples[parent].local_time);
            }
        }
        sample
    }

    fn reset_clock(&mut self) {
        self.elapsed = TimePoint::ZERO;
        self.local_time = TimePoint::ZERO;
        self.last_frame = None;
        self.completed_iterations = 0;
        self.begin_emitted = false;
        self.cursors.iter_mut().for_each(TrackCursor::reset);
        for output_index in 0..self.output_values.len() {
            let output_id = OutputId::new(output_index);
            self.output_values[output_id] = Some(SampledOutput::default());
        }
        self.fired_once.iter_mut().for_each(|fired| *fired = false);
    }
}

fn advance_instance(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    frame_time: TimePoint,
    frame: FrameId,
    events: &mut Vec<EngineEvent>,
    frame_events: &mut PendingFrameEvents,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    if instance.state != PlaybackState::Running
        || instance.clock_mode == AnimationClockMode::External
    {
        return;
    }
    let previous_frame = instance
        .last_frame
        .replace(frame_time)
        .unwrap_or(frame_time);
    let elapsed = instance
        .clock_rate
        .scale(frame_time.saturating_duration_since(previous_frame));
    let position = match instance.direction {
        PlaybackDirection::Forward => instance.elapsed + elapsed,
        PlaybackDirection::Reverse => TimePoint::from_nanos(
            instance
                .elapsed
                .as_nanos()
                .saturating_sub(elapsed.as_nanos()),
        ),
    };
    advance_instance_to(
        instance_id,
        instance,
        position,
        frame,
        events,
        frame_events,
        output_slots,
    );
}

fn advance_instance_to(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    position: TimePoint,
    frame: FrameId,
    events: &mut Vec<EngineEvent>,
    frame_events: &mut PendingFrameEvents,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    instance.elapsed = match instance.parent_extent() {
        TimeExtent::Finite(duration) => {
            TimePoint::from_nanos(position.as_nanos().min(duration.as_nanos()))
        }
        TimeExtent::Infinite => position,
    };

    let sample = instance.update_local_time();
    if !instance.begin_emitted && sample.phase == TimeDomainPhase::Active {
        instance.begin_emitted = true;
        events.push(EngineEvent::Begin {
            instance: instance_id,
        });
    }
    sample_and_emit_update(instance_id, instance, frame, events, output_slots);
    fire_crossing_events(instance_id, instance, &mut frame_events.deferred);
    let completed_iterations = if sample.phase == TimeDomainPhase::Complete {
        sample.iteration.saturating_add(1)
    } else {
        sample.iteration
    };
    for completed in instance.completed_iterations + 1..=completed_iterations {
        frame_events.deferred.push(EngineEvent::Loop {
            instance: instance_id,
            completed_iterations: completed.min(u64::from(u32::MAX)) as u32,
        });
    }
    instance.completed_iterations = completed_iterations;

    let reached_terminal = match instance.parent_extent() {
        TimeExtent::Infinite => false,
        TimeExtent::Finite(duration) => match instance.direction {
            PlaybackDirection::Forward => instance.elapsed.as_nanos() >= duration.as_nanos(),
            PlaybackDirection::Reverse => instance.elapsed == TimePoint::ZERO,
        },
    };
    if reached_terminal {
        instance.state = PlaybackState::Completed;
        instance.last_frame = None;
        frame_events.deferred.push(EngineEvent::Complete {
            instance: instance_id,
        });
        frame_events.deferred.push(EngineEvent::StateChanged {
            instance: instance_id,
            state: PlaybackState::Completed,
        });
        frame_events.deferred.push(EngineEvent::Settled {
            instance: instance_id,
            outcome: AnimationOutcome::Completed,
        });
    }
}

fn sample_and_emit_update(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    frame: FrameId,
    events: &mut Vec<EngineEvent>,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    events.push(EngineEvent::BeforeUpdate {
        instance: instance_id,
        at: instance.local_time,
    });
    sample_instance(instance_id, instance, frame, events, output_slots);
    let progress = match instance.plan.extent() {
        TimeExtent::Finite(duration) if duration != TimeSpan::ZERO => {
            instance.local_time.as_nanos() as f32 / duration.as_nanos() as f32
        }
        _ => 0.0,
    };
    events.push(EngineEvent::Update {
        instance: instance_id,
        at: instance.local_time,
        progress,
    });
}

fn sample_output_seeks(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    frame: FrameId,
    first: OutputSeek,
    second: Option<OutputSeek>,
    events: &mut Vec<EngineEvent>,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    events.push(EngineEvent::BeforeUpdate {
        instance: instance_id,
        at: first.position,
    });
    let mut sampled_any = false;
    for seek in [Some(first), second].into_iter().flatten() {
        instance.elapsed = seek.position;
        instance.update_local_time();
        if sample_bound_outputs(instance_id, instance, frame, seek, events, output_slots) {
            sampled_any = true;
        } else {
            events.push(EngineEvent::Error {
                instance: instance_id,
                error: AnimationRuntimeError::UnknownOutput {
                    instance: instance_id,
                    adapter: seek.adapter,
                    target: seek.target,
                    property: seek.property,
                },
            });
        }
    }
    if !sampled_any {
        return;
    }
    let progress = match instance.plan.extent() {
        TimeExtent::Finite(duration) if duration != TimeSpan::ZERO => {
            instance.local_time.as_nanos() as f32 / duration.as_nanos() as f32
        }
        _ => 0.0,
    };
    events.push(EngineEvent::Update {
        instance: instance_id,
        at: instance.local_time,
        progress,
    });
}

fn sample_bound_outputs(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    frame: FrameId,
    seek: OutputSeek,
    events: &mut Vec<EngineEvent>,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) -> bool {
    let mut matched = false;
    for output_index in 0..instance.plan.outputs().len() {
        let output_id = OutputId::new(output_index);
        let matches = {
            let output = &instance.plan.outputs()[output_id];
            let target = &instance.plan.targets()[output.target];
            let property = &instance.plan.properties()[output.property];
            target.adapter == seek.adapter
                && target.adapter_target == seek.target
                && property.adapter == seek.adapter
                && property.adapter_property == seek.property
        };
        if !matches {
            continue;
        }
        matched = true;
        let value = match sample_output(instance, output_id) {
            Ok(value) => value,
            Err(track_id) => {
                events.push(EngineEvent::Error {
                    instance: instance_id,
                    error: AnimationRuntimeError::TrackSamplingFailed(track_id),
                });
                continue;
            }
        };
        let precision = {
            let output = &instance.plan.outputs()[output_id];
            instance.plan.properties()[output.property]
                .descriptor
                .precision
        };
        let unchanged = instance.output_values[output_id]
            .as_ref()
            .is_some_and(|previous| previous.approximately_eq(&value, precision));
        if unchanged {
            continue;
        }
        instance.output_values[output_id] = Some(value);
        output_slots[instance.output_slots[output_id]].touched = true;
    }
    if matched {
        instance.pending_render_frame = Some(frame);
        instance.pending_render_at = instance.local_time;
    }
    matched
}

fn sample_instance(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    frame: FrameId,
    events: &mut Vec<EngineEvent>,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    for output_index in 0..instance.plan.outputs().len() {
        let output_id = OutputId::new(output_index);
        let value = match sample_output(instance, output_id) {
            Ok(value) => value,
            Err(track_id) => {
                events.push(EngineEvent::Error {
                    instance: instance_id,
                    error: AnimationRuntimeError::TrackSamplingFailed(track_id),
                });
                continue;
            }
        };
        let output = &instance.plan.outputs()[output_id];
        let property = &instance.plan.properties()[output.property];
        let unchanged = instance.output_values[output_id]
            .as_ref()
            .is_some_and(|previous| {
                previous.approximately_eq(&value, property.descriptor.precision)
            });
        if unchanged {
            continue;
        }
        instance.output_values[output_id] = Some(value);
        output_slots[instance.output_slots[output_id]].touched = true;
    }
    instance.pending_render_frame = Some(frame);
    instance.pending_render_at = instance.local_time;
}

fn sample_output(
    instance: &mut AnimationInstance,
    output_id: OutputId,
) -> Result<SampledOutput, TrackId> {
    let output = &instance.plan.outputs()[output_id];
    let mut replace: Option<(OutputPrecedence, AnimationValue)> = None;
    let mut additive: Option<AnimationValue> = None;

    for track_id in output.tracks().iter().copied() {
        let track = &instance.plan.tracks()[track_id];
        let domain_sample = instance.domain_samples[track.domain];
        let cursor = &mut instance.cursors[track_id];
        let segment = match domain_sample.direction {
            PlaybackDirection::Forward => cursor.advance(track, domain_sample.local_time),
            PlaybackDirection::Reverse => cursor.seek(track, domain_sample.local_time),
        };
        let Some(segment) = segment else {
            continue;
        };
        let sampled = AnimationSampler::sample_track(
            &instance.plan,
            track,
            segment,
            TrackSampleContext {
                local_time: domain_sample.local_time,
                completed_iterations: domain_sample.iteration,
            },
        )
        .map_err(|_| track_id)?;
        if let Some(sampled_replace) = sampled.replace {
            let tween = &instance.plan.tweens()[sampled_replace.tween];
            let precedence = OutputPrecedence {
                at: tween.start,
                domain: tween.domain,
                priority: tween.priority,
                source_order: tween.source_order,
                kind: 0,
            };
            if replace
                .as_ref()
                .is_none_or(|(winner, _)| precedence > *winner)
            {
                replace = Some((precedence, sampled_replace.value));
            }
        }
        if let Some(contribution) = sampled.additive {
            additive = Some(match additive {
                Some(value) => value.compose_add(&contribution).map_err(|_| track_id)?,
                None => contribution,
            });
        }
    }

    for event_id in output.set_events().iter().copied() {
        let CompiledEvent::Set {
            domain, at, value, ..
        } = &instance.plan.events()[event_id]
        else {
            unreachable!("compiled output contains only set events");
        };
        if instance.domain_samples[*domain].local_time < *at {
            continue;
        }
        let precedence = OutputPrecedence {
            at: *at,
            domain: *domain,
            priority: i32::MAX,
            source_order: event_id.index() as u32,
            kind: 1,
        };
        if replace
            .as_ref()
            .is_none_or(|(winner, _)| precedence > *winner)
        {
            replace = Some((precedence, value.clone()));
        }
    }

    Ok(SampledOutput {
        replace: replace.map(|(_, value)| value),
        additive,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OutputPrecedence {
    at: TimePoint,
    domain: TimeDomainId,
    priority: i32,
    source_order: u32,
    kind: u8,
}

fn touch_instance_outputs(
    instance: &AnimationInstance,
    output_slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
) {
    for slot in instance.output_slots.iter().copied() {
        output_slots[slot].touched = true;
    }
}

fn output_order_key(
    slot: &EngineOutputSlot,
) -> (
    AdapterId,
    AdapterTargetId,
    InvalidationClass,
    AdapterPropertyId,
) {
    (
        slot.key.adapter,
        slot.key.target,
        slot.descriptor.invalidation,
        slot.key.property,
    )
}

fn flush_output_slots(
    instances: &IndexVec<InstanceId, Option<AnimationInstance>>,
    generations: &IndexVec<InstanceId, u64>,
    slots: &mut IndexVec<EngineOutputId, EngineOutputSlot>,
    ordered_outputs: &[EngineOutputId],
    frame_batch: &mut FrameBatch,
    events: &mut Vec<EngineEvent>,
) {
    for slot_id in ordered_outputs.iter().copied() {
        let slot = &mut slots[slot_id];
        if !slot.touched {
            continue;
        }
        slot.touched = false;
        let replace_winner = slot
            .contributors
            .iter()
            .filter_map(|contributor| {
                let instance_slot = contributor.instance.slot();
                if generations.raw.get(instance_slot.index()).copied()?
                    != contributor.instance.generation()
                {
                    return None;
                }
                let instance = instances.raw.get(instance_slot.index())?.as_ref()?;
                let sampled = instance.output_values[contributor.output].as_ref()?;
                let replace = sampled.replace.as_ref()?;
                Some((
                    (instance.activation_sequence, contributor.instance),
                    replace,
                ))
            })
            .max_by_key(|(order, _)| *order);
        let replace_floor = replace_winner.map(|(order, _)| order);
        let mut value = if let Some((_, replace)) = replace_winner {
            replace.clone()
        } else if let Some((_, baseline)) = slot
            .contributors
            .iter()
            .filter_map(|contributor| {
                let instance_slot = contributor.instance.slot();
                if generations.raw.get(instance_slot.index()).copied()?
                    != contributor.instance.generation()
                {
                    return None;
                }
                let instance = instances.raw.get(instance_slot.index())?.as_ref()?;
                instance.output_values[contributor.output].as_ref()?;
                Some((
                    (instance.activation_sequence, contributor.instance),
                    &instance.baselines.values[contributor.output],
                ))
            })
            .min_by_key(|(order, _)| *order)
        {
            baseline.clone()
        } else {
            continue;
        };
        let mut composition_failed = false;
        for contributor in &slot.contributors {
            let instance_slot = contributor.instance.slot();
            if generations.raw.get(instance_slot.index()).copied()
                != Some(contributor.instance.generation())
            {
                continue;
            }
            let Some(instance) = instances
                .raw
                .get(instance_slot.index())
                .and_then(Option::as_ref)
            else {
                continue;
            };
            let Some(sampled) = instance.output_values[contributor.output].as_ref() else {
                continue;
            };
            let order = (instance.activation_sequence, contributor.instance);
            if replace_floor.is_some_and(|floor| order < floor) {
                continue;
            }
            let Some(additive) = sampled.additive.as_ref() else {
                continue;
            };
            match value.compose_add(additive) {
                Ok(composed) => value = composed,
                Err(_) => {
                    events.push(EngineEvent::Error {
                        instance: contributor.instance,
                        error: AnimationRuntimeError::OutputCompositionFailed(slot_id),
                    });
                    composition_failed = true;
                    break;
                }
            }
        }
        if composition_failed {
            continue;
        }
        if slot
            .current
            .as_ref()
            .is_some_and(|current| current.approximately_eq(&value, slot.descriptor.precision))
        {
            continue;
        }
        slot.current = Some(value.clone());
        frame_batch.push(PropertyUpdate {
            adapter: slot.key.adapter,
            target: slot.key.target,
            property: slot.key.property,
            invalidation: slot.descriptor.invalidation,
            value,
        });
    }
}

fn fire_crossing_events(
    instance_id: InstanceKey,
    instance: &mut AnimationInstance,
    events: &mut Vec<EngineEvent>,
) {
    let plan = &instance.plan;
    let fired_once = &mut instance.fired_once;
    for (domain_id, domain) in plan.domains().iter_enumerated() {
        let previous = instance.previous_domain_samples[domain_id];
        let current = instance.domain_samples[domain_id];
        if previous.iteration == current.iteration {
            fire_interval(
                instance_id,
                plan,
                fired_once,
                domain_id,
                previous.local_time,
                current.local_time,
                events,
            );
            continue;
        }

        let terminal = domain.extent.finite().unwrap_or(TimeSpan::ZERO);
        if current.iteration > previous.iteration {
            let previous_end = boundary_for_direction(terminal, previous.direction, true);
            fire_interval(
                instance_id,
                plan,
                fired_once,
                domain_id,
                previous.local_time,
                previous_end,
                events,
            );
            for iteration in previous.iteration + 1..current.iteration {
                let direction = iteration_direction(domain, iteration);
                fire_interval(
                    instance_id,
                    plan,
                    fired_once,
                    domain_id,
                    boundary_for_direction(terminal, direction, false),
                    boundary_for_direction(terminal, direction, true),
                    events,
                );
            }
            let current_start = boundary_for_direction(terminal, current.direction, false);
            fire_interval(
                instance_id,
                plan,
                fired_once,
                domain_id,
                current_start,
                current.local_time,
                events,
            );
        } else {
            let previous_start = boundary_for_direction(terminal, previous.direction, false);
            fire_interval(
                instance_id,
                plan,
                fired_once,
                domain_id,
                previous.local_time,
                previous_start,
                events,
            );
            for iteration in (current.iteration + 1..previous.iteration).rev() {
                let direction = iteration_direction(domain, iteration);
                fire_interval(
                    instance_id,
                    plan,
                    fired_once,
                    domain_id,
                    boundary_for_direction(terminal, direction, true),
                    boundary_for_direction(terminal, direction, false),
                    events,
                );
            }
            let current_end = boundary_for_direction(terminal, current.direction, true);
            fire_interval(
                instance_id,
                plan,
                fired_once,
                domain_id,
                current_end,
                current.local_time,
                events,
            );
        }
    }
}

fn fire_interval(
    instance_id: InstanceKey,
    plan: &CompiledAnimation,
    fired_once: &mut IndexVec<TimelineNodeId, bool>,
    domain_id: TimeDomainId,
    from: TimePoint,
    to: TimePoint,
    events: &mut Vec<EngineEvent>,
) {
    if from == to {
        return;
    }
    let forward = to > from;
    let domain = &plan.domains()[domain_id];
    let range = domain.event_index_range();
    if forward {
        for index in range {
            fire_call_event(
                plan,
                fired_once,
                TimelineNodeId::new(index),
                CallCrossing {
                    instance: instance_id,
                    from,
                    to,
                    forward: true,
                },
                events,
            );
        }
        return;
    }

    let mut group_end = range.end;
    while group_end > range.start {
        let time = compiled_event_time(&plan.events()[TimelineNodeId::new(group_end - 1)]);
        let mut group_start = group_end - 1;
        while group_start > range.start
            && compiled_event_time(&plan.events()[TimelineNodeId::new(group_start - 1)]) == time
        {
            group_start -= 1;
        }
        for index in group_start..group_end {
            fire_call_event(
                plan,
                fired_once,
                TimelineNodeId::new(index),
                CallCrossing {
                    instance: instance_id,
                    from,
                    to,
                    forward: false,
                },
                events,
            );
        }
        group_end = group_start;
    }
}

fn fire_call_event(
    plan: &CompiledAnimation,
    fired_once: &mut IndexVec<TimelineNodeId, bool>,
    event_id: TimelineNodeId,
    crossing: CallCrossing,
    events: &mut Vec<EngineEvent>,
) {
    let CompiledEvent::Call {
        at, call, policy, ..
    } = &plan.events()[event_id]
    else {
        return;
    };
    let crossed = if crossing.forward {
        crossing.from < *at && *at <= crossing.to
    } else {
        crossing.to <= *at && *at < crossing.from
    };
    if !crossed {
        return;
    }
    let allowed = match policy {
        CallPolicy::ForwardOnly => crossing.forward,
        CallPolicy::BothDirections => true,
        CallPolicy::Once => !fired_once[event_id],
    };
    if !allowed {
        return;
    }
    if *policy == CallPolicy::Once {
        fired_once[event_id] = true;
    }
    events.push(EngineEvent::Call {
        instance: crossing.instance,
        call: *call,
    });
}

#[derive(Clone, Copy)]
struct CallCrossing {
    instance: InstanceKey,
    from: TimePoint,
    to: TimePoint,
    forward: bool,
}

const fn compiled_event_time(event: &CompiledEvent) -> TimePoint {
    match event {
        CompiledEvent::Call { at, .. }
        | CompiledEvent::Set { at, .. }
        | CompiledEvent::Barrier { at, .. } => *at,
    }
}

fn iteration_direction(domain: &crate::CompiledTimeDomain, iteration: u64) -> PlaybackDirection {
    let reverse = domain.settings.reversed ^ (domain.settings.alternate && iteration % 2 == 1);
    if reverse {
        PlaybackDirection::Reverse
    } else {
        PlaybackDirection::Forward
    }
}

fn boundary_for_direction(
    duration: TimeSpan,
    direction: PlaybackDirection,
    terminal: bool,
) -> TimePoint {
    let at_end = matches!(direction, PlaybackDirection::Forward) == terminal;
    if at_end {
        TimePoint::from_nanos(duration.as_nanos())
    } else {
        TimePoint::ZERO
    }
}

fn validate_baselines(
    plan: &CompiledAnimation,
    baselines: &AnimationBaselineSnapshot,
) -> Result<(), AnimationRuntimeError> {
    if baselines.values.len() != plan.outputs().len() {
        return Err(AnimationRuntimeError::BaselineCountMismatch {
            expected: plan.outputs().len(),
            actual: baselines.values.len(),
        });
    }
    for (output_id, output) in plan.outputs().iter_enumerated() {
        if baselines.values[output_id].kind()
            != plan.properties()[output.property].descriptor.value_kind
        {
            return Err(AnimationRuntimeError::BaselineKindMismatch(output_id));
        }
    }
    Ok(())
}

fn ensure_active(active: &mut Vec<InstanceKey>, instance: InstanceKey) {
    if !active.contains(&instance) {
        active.push(instance);
    }
}

const fn command_instance(command: EngineCommand) -> InstanceKey {
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        AdapterId, AdapterPropertyId, AdapterTargetId, AnimationCompiler, CallId, CallPolicy,
        Composition, CompositionSupport, Easing, Modifier, Property, PropertyDescriptor,
        ResolvedAnimation, ResolvedEvent, ResolvedProperty, ResolvedTarget, ResolvedTimeDomain,
        ResolvedTween, TargetId, TimeDomainId,
    };

    const OPACITY: Property<f32> = Property::static_name("opacity");
    const SCALE: Property<f32> = Property::static_name("scale");

    fn commit_tick(engine: &mut AnimationEngine, frame_time: TimePoint) -> FrameId {
        let frame = engine.tick(frame_time).unwrap();
        engine.acknowledge_frame(frame).unwrap();
        frame
    }

    fn finite_plan(duration: u64) -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.events.push(ResolvedEvent::Call {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(duration),
            call: CallId::new(0),
            policy: CallPolicy::ForwardOnly,
        });
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn scalar_plan() -> Arc<CompiledAnimation> {
        scalar_plan_to(10.0)
    }

    fn scalar_plan_to(to: f32) -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(0),
            adapter_property: AdapterPropertyId::new(0),
            descriptor: PropertyDescriptor::new(&OPACITY),
        });
        resolved.tweens.push(ResolvedTween {
            domain: TimeDomainId::new(0),
            target: TargetId::new(0),
            property: crate::PropertyId::new(0),
            start: TimePoint::ZERO,
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_nanos(100),
            priority: 0,
            from: AnimationValue::Scalar(0.0),
            to: AnimationValue::Scalar(to),
            easing: Easing::Linear,
            composition: Composition::Replace,
            modifier: Modifier::Identity,
        });
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn two_property_plan() -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        for (property, descriptor) in [
            (AdapterPropertyId::new(0), PropertyDescriptor::new(&OPACITY)),
            (AdapterPropertyId::new(1), PropertyDescriptor::new(&SCALE)),
        ] {
            resolved.properties.push(ResolvedProperty {
                adapter: AdapterId::new(0),
                adapter_property: property,
                descriptor,
            });
        }
        for (property, to) in [
            (crate::PropertyId::new(0), 10.0),
            (crate::PropertyId::new(1), 20.0),
        ] {
            resolved.tweens.push(ResolvedTween {
                domain: TimeDomainId::new(0),
                target: TargetId::new(0),
                property,
                start: TimePoint::ZERO,
                delay: TimeSpan::ZERO,
                duration: TimeSpan::from_nanos(100),
                priority: 0,
                from: AnimationValue::Scalar(0.0),
                to: AnimationValue::Scalar(to),
                easing: Easing::Linear,
                composition: Composition::Replace,
                modifier: Modifier::Identity,
            });
        }
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn scalar_composition_plan(to: f32, composition: Composition) -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        let mut descriptor = PropertyDescriptor::new(&OPACITY);
        descriptor.composition = CompositionSupport::NUMERIC;
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(0),
            adapter_property: AdapterPropertyId::new(0),
            descriptor,
        });
        resolved.tweens.push(ResolvedTween {
            domain: TimeDomainId::new(0),
            target: TargetId::new(0),
            property: crate::PropertyId::new(0),
            start: TimePoint::ZERO,
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_nanos(100),
            priority: 0,
            from: AnimationValue::Scalar(0.0),
            to: AnimationValue::Scalar(to),
            easing: Easing::Linear,
            composition,
            modifier: Modifier::Identity,
        });
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn set_only_plan() -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(0),
            adapter_property: AdapterPropertyId::new(0),
            descriptor: PropertyDescriptor::new(&OPACITY),
        });
        resolved.events.push(ResolvedEvent::Set {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(50),
            target: TargetId::new(0),
            property: crate::PropertyId::new(0),
            value: AnimationValue::Scalar(7.0),
        });
        resolved.events.push(ResolvedEvent::Barrier {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(100),
            participants: NonZeroU32::new(1).unwrap(),
        });
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn cross_domain_composition_plan() -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        let mut descriptor = PropertyDescriptor::new(&OPACITY);
        descriptor.composition = CompositionSupport::NUMERIC;
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(0),
            adapter_property: AdapterPropertyId::new(0),
            descriptor,
        });
        resolved.domains.push(ResolvedTimeDomain {
            parent: Some(TimeDomainId::new(0)),
            offset: TimePoint::ZERO,
            extent: TimeExtent::Finite(TimeSpan::from_nanos(100)),
            settings: Default::default(),
        });
        let tween = |domain, to, composition| ResolvedTween {
            domain,
            target: TargetId::new(0),
            property: crate::PropertyId::new(0),
            start: TimePoint::ZERO,
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_nanos(100),
            priority: 0,
            from: AnimationValue::Scalar(0.0),
            to: AnimationValue::Scalar(to),
            easing: Easing::Linear,
            composition,
            modifier: Modifier::Identity,
        };
        resolved
            .tweens
            .push(tween(TimeDomainId::new(0), 10.0, Composition::Replace));
        resolved
            .tweens
            .push(tween(TimeDomainId::new(1), 2.0, Composition::Add));
        AnimationCompiler.compile(resolved).unwrap()
    }

    fn crossing_plan(iterations: u32) -> Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.settings.iterations = crate::IterationCount::finite(iterations).unwrap();
        resolved.domains[TimeDomainId::new(0)].settings = resolved.settings.clone();
        resolved.events.push(ResolvedEvent::Call {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(25),
            call: CallId::new(1),
            policy: CallPolicy::ForwardOnly,
        });
        resolved.events.push(ResolvedEvent::Call {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(50),
            call: CallId::new(2),
            policy: CallPolicy::BothDirections,
        });
        resolved.events.push(ResolvedEvent::Call {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(75),
            call: CallId::new(3),
            policy: CallPolicy::Once,
        });
        resolved.events.push(ResolvedEvent::Barrier {
            domain: TimeDomainId::new(0),
            at: TimePoint::from_nanos(100),
            participants: NonZeroU32::new(1).unwrap(),
        });
        AnimationCompiler.compile(resolved).unwrap()
    }

    #[test]
    fn engine_drains_commands_and_advances_a_single_owned_clock() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                finite_plan(100),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(10));
        assert_eq!(
            engine.snapshot(instance).unwrap().state,
            PlaybackState::Running
        );
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        assert_eq!(
            engine.snapshot(instance).unwrap().elapsed,
            TimePoint::from_nanos(50)
        );
        engine.enqueue(EngineCommand::Pause(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(70));
        commit_tick(&mut engine, TimePoint::from_nanos(90));
        assert_eq!(
            engine.snapshot(instance).unwrap().elapsed,
            TimePoint::from_nanos(50)
        );
        engine.enqueue(EngineCommand::Resume(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(100));
        commit_tick(&mut engine, TimePoint::from_nanos(150));
        assert_eq!(
            engine.snapshot(instance).unwrap().state,
            PlaybackState::Completed
        );

        assert!(!engine.has_work());
    }

    #[test]
    fn external_clock_advances_only_from_explicit_positions_and_can_handoff() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert_with_clock(
                finite_plan(100),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
                AnimationClockMode::External,
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(10));
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        assert_eq!(engine.snapshot(instance).unwrap().elapsed, TimePoint::ZERO);
        assert!(!engine.has_work());

        engine.enqueue(EngineCommand::AdvanceExternal {
            instance,
            position: TimePoint::from_nanos(50),
        });
        commit_tick(&mut engine, TimePoint::from_nanos(70));
        assert_eq!(
            engine.snapshot(instance).unwrap().elapsed,
            TimePoint::from_nanos(50)
        );

        engine
            .set_clock_mode(instance, AnimationClockMode::Internal)
            .unwrap();
        commit_tick(&mut engine, TimePoint::from_nanos(100));
        commit_tick(&mut engine, TimePoint::from_nanos(150));
        assert_eq!(
            engine.snapshot(instance).unwrap().state,
            PlaybackState::Completed
        );
    }

    #[test]
    fn repeated_instance_mount_and_remove_reuses_dense_storage() {
        let plan = scalar_plan();
        let mut engine = AnimationEngine::new();
        for iteration in 0..1_000_u64 {
            let instance = engine
                .insert(
                    plan.clone(),
                    AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(
                        0.0,
                    )]),
                )
                .unwrap();
            assert_eq!(instance.slot(), InstanceId::new(0));
            assert_eq!(instance.generation(), iteration + 1);
            engine.enqueue(EngineCommand::Remove(instance));
            commit_tick(&mut engine, TimePoint::from_nanos(iteration));
            engine.drain_events().for_each(drop);
        }
        let diagnostics = engine.diagnostics();
        assert_eq!(diagnostics.live_instances, 0);
        assert_eq!(diagnostics.output_slots, 1);
        assert!(!engine.has_work());
    }

    #[test]
    fn target_detach_publishes_removal_before_reusing_the_dense_slot() {
        let plan = scalar_plan();
        let baselines =
            || AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]);
        let mut engine = AnimationEngine::new();
        let removed = engine.insert(plan.clone(), baselines()).unwrap();

        assert_eq!(
            engine.detach_target(AdapterId::new(0), AdapterTargetId::new(0)),
            1
        );
        let events = engine.drain_events().collect::<Vec<_>>();
        assert!(matches!(
            events.last(),
            Some(EngineEvent::Removed { instance }) if *instance == removed
        ));
        assert!(engine.snapshot(removed).is_none());

        let reused = engine.insert(plan, baselines()).unwrap();
        assert_eq!(reused.slot(), removed.slot());
        assert!(reused.generation() > removed.generation());
        assert!(engine.snapshot(reused).is_some());
    }

    #[test]
    fn stale_remove_cannot_delete_a_reused_instance_slot() {
        let plan = scalar_plan();
        let baselines =
            || AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]);
        let mut engine = AnimationEngine::new();
        let retired = engine.insert(plan.clone(), baselines()).unwrap();

        // This is the lifecycle interleaving produced when Controls::drop
        // queues Remove immediately before ArkUI disposes its target.
        engine.enqueue(EngineCommand::Remove(retired));
        assert_eq!(
            engine.detach_target(AdapterId::new(0), AdapterTargetId::new(0)),
            1
        );
        let current = engine.insert(plan, baselines()).unwrap();
        assert_eq!(current.slot(), retired.slot());
        assert_ne!(current, retired);

        // Even a delayed producer that re-enqueues the old command is harmless:
        // generation validation rejects it instead of addressing `current`.
        engine.enqueue(EngineCommand::Remove(retired));
        commit_tick(&mut engine, TimePoint::ZERO);
        assert!(engine.snapshot(current).is_some());
        assert!(engine.events().iter().any(|event| matches!(
            event,
            EngineEvent::Error {
                instance,
                error: AnimationRuntimeError::UnknownInstance(error_instance),
            } if *instance == retired && *error_instance == retired
        )));
    }

    #[test]
    fn complete_reports_a_typed_error_for_infinite_instances() {
        let mut resolved = ResolvedAnimation::default();
        resolved.domains[TimeDomainId::new(0)].extent = TimeExtent::Infinite;
        let plan = AnimationCompiler.compile(resolved).unwrap();
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                plan,
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Complete(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        assert!(engine.events().iter().any(|event| matches!(
            event,
            EngineEvent::Error {
                error: AnimationRuntimeError::InfiniteAnimationCannotComplete(id),
                ..
            } if *id == instance
        )));
    }

    #[test]
    fn engine_samples_tracks_and_skips_unchanged_property_writes() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(0.0)
        );

        commit_tick(&mut engine, TimePoint::from_nanos(50));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(5.0)
        );

        commit_tick(&mut engine, TimePoint::from_nanos(50));
        assert!(engine.frame_batch().is_empty());

        commit_tick(&mut engine, TimePoint::from_nanos(100));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(10.0)
        );
        assert_eq!(
            engine.snapshot(instance).unwrap().state,
            PlaybackState::Completed
        );

        engine.enqueue(EngineCommand::Revert(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(101));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(0.0)
        );
    }

    #[test]
    fn output_seeks_sample_two_properties_at_independent_positions() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                two_property_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![
                    AnimationValue::Scalar(0.0),
                    AnimationValue::Scalar(0.0),
                ]),
            )
            .unwrap();
        let output = |property, position| OutputSeek {
            adapter: AdapterId::new(0),
            target: AdapterTargetId::new(0),
            property: AdapterPropertyId::new(property),
            position: TimePoint::from_nanos(position),
        };
        engine.enqueue(EngineCommand::SeekOutputs {
            instance,
            first: output(0, 25),
            second: Some(output(1, 75)),
        });

        commit_tick(&mut engine, TimePoint::ZERO);

        let updates = engine.frame_batch().as_slice();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].value, AnimationValue::Scalar(2.5));
        assert_eq!(updates[1].value, AnimationValue::Scalar(15.0));
    }

    #[test]
    fn latest_suppressed_seek_replaces_the_pending_sample() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        for position in [10, 40, 80] {
            engine.enqueue(EngineCommand::Seek {
                instance,
                position: TimePoint::from_nanos(position),
                mode: SeekMode::SuppressEvents,
            });
        }
        assert_eq!(engine.diagnostics().pending_commands, 1);

        commit_tick(&mut engine, TimePoint::ZERO);

        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(8.0)
        );
    }

    #[test]
    fn restart_commits_the_initial_value_before_advancing() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        commit_tick(&mut engine, TimePoint::from_nanos(100));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(10.0)
        );

        engine.enqueue(EngineCommand::Restart(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(150));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(0.0)
        );
        assert_eq!(
            engine.snapshot(instance).unwrap().state,
            PlaybackState::Running
        );

        commit_tick(&mut engine, TimePoint::from_nanos(200));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(5.0)
        );
    }

    #[test]
    fn set_events_persist_and_reverse_back_to_the_output_baseline() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                set_only_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(1.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(1.0)
        );

        commit_tick(&mut engine, TimePoint::from_nanos(60));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(7.0)
        );

        engine.enqueue(EngineCommand::Reverse(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(120));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(1.0)
        );

        engine.enqueue(EngineCommand::Reverse(instance));
        engine.enqueue(EngineCommand::Restart(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(120));
        commit_tick(&mut engine, TimePoint::from_nanos(180));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(7.0)
        );
        engine.enqueue(EngineCommand::Pause(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(180));
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::ZERO,
            mode: SeekMode::SuppressEvents,
        });
        commit_tick(&mut engine, TimePoint::from_nanos(180));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(1.0)
        );
    }

    #[test]
    fn additive_contributions_compose_across_nested_time_domains() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                cross_domain_composition_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        commit_tick(&mut engine, TimePoint::from_nanos(50));

        assert_eq!(engine.frame_batch().len(), 1);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(6.0)
        );
    }

    #[test]
    fn engine_validates_baseline_snapshot_shape_and_kind() {
        let mut engine = AnimationEngine::new();
        assert_eq!(
            engine
                .insert(
                    scalar_plan(),
                    AnimationBaselineSnapshot::from_output_values(Vec::new()),
                )
                .unwrap_err(),
            AnimationRuntimeError::BaselineCountMismatch {
                expected: 1,
                actual: 0,
            }
        );
        assert_eq!(
            engine
                .insert(
                    scalar_plan(),
                    AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Length(
                        crate::Length::vp(0.0),
                    )]),
                )
                .unwrap_err(),
            AnimationRuntimeError::BaselineKindMismatch(OutputId::new(0))
        );
    }

    #[test]
    fn engine_honors_call_policies_for_natural_and_seek_crossings() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                crossing_plan(1),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        engine.drain_events().for_each(drop);
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        let calls = engine
            .drain_events()
            .filter_map(|event| match event {
                EngineEvent::Call { call, .. } => Some(call),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(calls, [CallId::new(1), CallId::new(2)]);

        engine.enqueue(EngineCommand::Pause(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        engine.drain_events().for_each(drop);
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(90),
            mode: SeekMode::SuppressEvents,
        });
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        assert!(!engine
            .drain_events()
            .any(|event| matches!(event, EngineEvent::Call { .. })));

        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::ZERO,
            mode: SeekMode::FireCrossingEvents,
        });
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        let reverse_calls = engine
            .drain_events()
            .filter_map(|event| match event {
                EngineEvent::Call { call, .. } => Some(call),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reverse_calls, [CallId::new(3), CallId::new(2)]);

        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(90),
            mode: SeekMode::FireCrossingEvents,
        });
        commit_tick(&mut engine, TimePoint::from_nanos(60));
        let forward_calls = engine
            .drain_events()
            .filter_map(|event| match event {
                EngineEvent::Call { call, .. } => Some(call),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(forward_calls, [CallId::new(1), CallId::new(2)]);
    }

    #[test]
    fn one_frame_crossing_multiple_iterations_emits_all_calls_and_loops() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                crossing_plan(3),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        engine.drain_events().for_each(drop);
        commit_tick(&mut engine, TimePoint::from_nanos(250));
        let events = engine.drain_events().collect::<Vec<_>>();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, EngineEvent::Call { call, .. } if *call == CallId::new(2)))
                .count(),
            3
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, EngineEvent::Loop { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn set_alternate_recomputes_the_current_root_domain_direction() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                crossing_plan(2),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(125),
            mode: SeekMode::SuppressEvents,
        });
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine.snapshot(instance).unwrap().local_time,
            TimePoint::from_nanos(25)
        );
        engine.enqueue(EngineCommand::SetAlternate {
            instance,
            enabled: true,
        });
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine.snapshot(instance).unwrap().local_time,
            TimePoint::from_nanos(75)
        );
    }

    #[test]
    fn reverse_seek_across_multiple_iterations_emits_each_crossed_call() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                crossing_plan(3),
                AnimationBaselineSnapshot::from_output_values(Vec::new()),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(250),
            mode: SeekMode::SuppressEvents,
        });
        commit_tick(&mut engine, TimePoint::ZERO);
        engine.drain_events().for_each(drop);
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(25),
            mode: SeekMode::FireCrossingEvents,
        });
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine
                .drain_events()
                .filter(|event| matches!(
                    event,
                    EngineEvent::Call { call, .. } if *call == CallId::new(2)
                ))
                .count(),
            2
        );
    }

    #[test]
    fn global_output_slot_arbitrates_the_same_adapter_property_once() {
        let mut engine = AnimationEngine::new();
        let first = engine
            .insert(
                scalar_plan_to(10.0),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        let second = engine
            .insert(
                scalar_plan_to(20.0),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(first));
        engine.enqueue(EngineCommand::Play(second));
        commit_tick(&mut engine, TimePoint::ZERO);
        commit_tick(&mut engine, TimePoint::from_nanos(50));
        assert_eq!(engine.frame_batch().len(), 1);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(10.0)
        );
    }

    #[test]
    fn global_output_slot_folds_cross_instance_composition_in_activation_order() {
        let mut engine = AnimationEngine::new();
        let replace = engine
            .insert(
                scalar_composition_plan(10.0, Composition::Replace),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        let additive = engine
            .insert(
                scalar_composition_plan(2.0, Composition::Add),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(replace));
        engine.enqueue(EngineCommand::Play(additive));
        commit_tick(&mut engine, TimePoint::ZERO);
        commit_tick(&mut engine, TimePoint::from_nanos(50));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(6.0)
        );

        engine.enqueue(EngineCommand::Restart(replace));
        commit_tick(&mut engine, TimePoint::from_nanos(50));
        commit_tick(&mut engine, TimePoint::from_nanos(100));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(5.0)
        );

        engine.enqueue(EngineCommand::Restart(additive));
        commit_tick(&mut engine, TimePoint::from_nanos(100));
        commit_tick(&mut engine, TimePoint::from_nanos(150));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(11.0)
        );
    }

    #[test]
    fn engine_rejects_conflicting_contracts_for_one_global_property() {
        let mut engine = AnimationEngine::new();
        engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();

        assert_eq!(
            engine
                .insert(
                    scalar_composition_plan(2.0, Composition::Add),
                    AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(
                        0.0,
                    )]),
                )
                .unwrap_err(),
            AnimationRuntimeError::GlobalPropertyContractMismatch {
                adapter: AdapterId::new(0),
                target: AdapterTargetId::new(0),
                property: AdapterPropertyId::new(0),
            }
        );
    }

    #[test]
    fn render_terminal_and_waiter_events_are_released_only_after_frame_ack() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        engine.drain_events().for_each(drop);

        let frame = engine.tick(TimePoint::from_nanos(100)).unwrap();
        assert_eq!(engine.pending_frame(), Some(frame));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(10.0)
        );
        assert!(!engine.events().iter().any(|event| matches!(
            event,
            EngineEvent::Render { .. }
                | EngineEvent::Loop { .. }
                | EngineEvent::Complete { .. }
                | EngineEvent::Settled { .. }
        )));
        assert_eq!(
            engine.tick(TimePoint::from_nanos(101)).unwrap_err(),
            AnimationRuntimeError::FrameNotAcknowledged(frame)
        );
        let wrong = FrameId::new(frame.sequence() + 1);
        assert_eq!(
            engine.acknowledge_frame(wrong).unwrap_err(),
            AnimationRuntimeError::UnexpectedFrameAcknowledgement {
                expected: frame,
                actual: wrong,
            }
        );

        engine.acknowledge_frame(frame).unwrap();
        let committed = engine
            .drain_events()
            .filter_map(|event| match event {
                EngineEvent::Render { .. } => Some("render"),
                EngineEvent::Loop { .. } => Some("loop"),
                EngineEvent::Complete { .. } => Some("complete"),
                EngineEvent::Settled {
                    outcome: AnimationOutcome::Completed,
                    ..
                } => Some("settled"),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(committed, ["render", "loop", "complete", "settled"]);
        assert_eq!(
            engine.acknowledge_frame(frame).unwrap_err(),
            AnimationRuntimeError::NoFramePending(frame)
        );
        engine.enqueue(EngineCommand::Complete(instance));
        commit_tick(&mut engine, TimePoint::from_nanos(101));
        assert!(!engine.drain_events().any(|event| matches!(
            event,
            EngineEvent::Complete { .. } | EngineEvent::Settled { .. }
        )));
    }

    #[test]
    fn explicit_complete_emits_full_update_before_render_and_settlement() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Complete(instance));
        let frame = engine.tick(TimePoint::ZERO).unwrap();
        let before_ack = engine.drain_events().collect::<Vec<_>>();
        assert!(matches!(before_ack[0], EngineEvent::Begin { .. }));
        assert!(matches!(before_ack[1], EngineEvent::BeforeUpdate { .. }));
        assert!(matches!(before_ack[2], EngineEvent::Update { .. }));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(10.0)
        );

        engine.acknowledge_frame(frame).unwrap();
        let after_ack = engine
            .drain_events()
            .filter_map(|event| match event {
                EngineEvent::Render { .. } => Some("render"),
                EngineEvent::Complete { .. } => Some("complete"),
                EngineEvent::Settled {
                    outcome: AnimationOutcome::Completed,
                    ..
                } => Some("settled"),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(after_ack, ["render", "complete", "settled"]);
    }

    #[test]
    fn explicit_complete_uses_the_current_direction_terminal() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Seek {
            instance,
            position: TimePoint::from_nanos(70),
            mode: SeekMode::SuppressEvents,
        });
        commit_tick(&mut engine, TimePoint::ZERO);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(7.0)
        );

        engine.enqueue(EngineCommand::Reverse(instance));
        engine.enqueue(EngineCommand::Complete(instance));
        let frame = engine.tick(TimePoint::from_nanos(1)).unwrap();
        assert_eq!(engine.snapshot(instance).unwrap().elapsed, TimePoint::ZERO);
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(0.0)
        );
        engine.acknowledge_frame(frame).unwrap();
        assert!(engine.drain_events().any(|event| matches!(
            event,
            EngineEvent::Complete {
                instance: completed
            } if completed == instance
        )));
    }

    #[test]
    fn cancel_settles_after_ack_without_fabricating_a_render() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan(),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        engine.drain_events().for_each(drop);

        engine.enqueue(EngineCommand::Cancel(instance));
        let frame = engine.tick(TimePoint::from_nanos(1)).unwrap();
        assert!(engine.events().is_empty());
        engine.acknowledge_frame(frame).unwrap();
        assert_eq!(
            engine.drain_events().collect::<Vec<_>>(),
            [
                EngineEvent::Cancel { instance },
                EngineEvent::StateChanged {
                    instance,
                    state: PlaybackState::Cancelled,
                },
                EngineEvent::Settled {
                    instance,
                    outcome: AnimationOutcome::Cancelled,
                },
            ]
        );
    }

    #[test]
    fn refresh_replaces_resolution_without_resetting_the_logical_clock() {
        let mut engine = AnimationEngine::new();
        let instance = engine
            .insert(
                scalar_plan_to(10.0),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        engine.enqueue(EngineCommand::Play(instance));
        commit_tick(&mut engine, TimePoint::ZERO);
        commit_tick(&mut engine, TimePoint::from_nanos(50));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(5.0)
        );

        engine.enqueue(EngineCommand::Refresh(instance));
        let frame = engine.tick(TimePoint::from_nanos(50)).unwrap();
        assert!(engine.events().iter().any(|event| matches!(
            event,
            EngineEvent::RefreshRequested { instance: id, at }
                if *id == instance && *at == TimePoint::from_nanos(50)
        )));
        assert_eq!(
            engine
                .replace_resolution(
                    instance,
                    scalar_plan_to(20.0),
                    AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(
                        0.0,
                    )]),
                )
                .unwrap_err(),
            AnimationRuntimeError::FrameNotAcknowledged(frame)
        );
        engine.acknowledge_frame(frame).unwrap();
        engine
            .replace_resolution(
                instance,
                scalar_plan_to(20.0),
                AnimationBaselineSnapshot::from_output_values(vec![AnimationValue::Scalar(0.0)]),
            )
            .unwrap();
        assert_eq!(
            engine.snapshot(instance).unwrap().elapsed,
            TimePoint::from_nanos(50)
        );

        commit_tick(&mut engine, TimePoint::from_nanos(100));
        assert_eq!(
            engine.frame_batch().as_slice()[0].value,
            AnimationValue::Scalar(20.0)
        );
    }
}
