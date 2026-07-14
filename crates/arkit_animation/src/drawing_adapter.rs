use std::cell::{Cell, RefCell};

use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use arkit_animation_core::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, PropertyDescriptor,
    PropertyName, PropertyUpdate, ResolutionTarget, ResolvedProperty, ResolvedTarget, SourceTarget,
    TargetName, TargetSetName,
};

use crate::{AnimationAdapterError, PropertySchema, TargetAdapter};

struct DrawingTarget {
    name: TargetName,
    values: IndexVec<AdapterPropertyId, Option<AnimationValue>>,
    version: u64,
}

type DrawingInvalidator = Box<dyn Fn(AdapterTargetId)>;

pub struct DrawingAdapter {
    id: AdapterId,
    schema: PropertySchema,
    targets: RefCell<IndexVec<AdapterTargetId, Option<DrawingTarget>>>,
    names: RefCell<FxHashMap<TargetName, AdapterTargetId>>,
    sets: RefCell<FxHashMap<TargetSetName, Vec<AdapterTargetId>>>,
    invalidator: RefCell<Option<DrawingInvalidator>>,
    next_version: Cell<u64>,
}

impl DrawingAdapter {
    pub fn new(id: AdapterId, descriptors: impl IntoIterator<Item = PropertyDescriptor>) -> Self {
        let mut schema = PropertySchema::default();
        for descriptor in descriptors {
            schema.insert(descriptor);
        }
        Self {
            id,
            schema,
            targets: RefCell::new(IndexVec::new()),
            names: RefCell::new(FxHashMap::default()),
            sets: RefCell::new(FxHashMap::default()),
            invalidator: RefCell::new(None),
            next_version: Cell::new(0),
        }
    }

    pub fn set_invalidator(&self, invalidator: impl Fn(AdapterTargetId) + 'static) {
        *self.invalidator.borrow_mut() = Some(Box::new(invalidator));
    }

    pub fn register_target(
        &self,
        name: TargetName,
        baselines: impl IntoIterator<Item = (PropertyName, AnimationValue)>,
    ) -> Result<AdapterTargetId, AnimationAdapterError> {
        if self.names.borrow().contains_key(&name) {
            return Err(AnimationAdapterError::DuplicateTarget(name));
        }
        let mut values = IndexVec::new();
        for (property, value) in baselines {
            let (property, descriptor) = self.schema.resolve(&property)?;
            descriptor.validate_value(&value)?;
            while values.len() <= property.index() {
                values.push(None);
            }
            values[property] = Some(value);
        }
        let mut targets = self.targets.borrow_mut();
        let version = self.next_version.get();
        self.next_version.set(
            version
                .checked_add(1)
                .expect("drawing target lifecycle version exhausted"),
        );
        let target = Some(DrawingTarget {
            name: name.clone(),
            values,
            version,
        });
        let id = targets
            .iter_enumerated()
            .find_map(|(id, target)| target.is_none().then_some(id))
            .unwrap_or_else(|| AdapterTargetId::new(targets.len()));
        if id.index() == targets.len() {
            targets.push(target);
        } else {
            targets[id] = target;
        }
        self.names.borrow_mut().insert(name, id);
        Ok(id)
    }

    pub fn unregister_target(&self, id: AdapterTargetId) -> bool {
        let Some(target) = self
            .targets
            .borrow_mut()
            .raw
            .get_mut(id.index())
            .and_then(Option::take)
        else {
            return false;
        };
        self.names.borrow_mut().remove(&target.name);
        for members in self.sets.borrow_mut().values_mut() {
            members.retain(|member| *member != id);
        }
        true
    }

    pub fn value(
        &self,
        target_id: AdapterTargetId,
        property: &PropertyName,
    ) -> Option<AnimationValue> {
        let (property, _) = self.schema.resolve(property).ok()?;
        self.targets
            .borrow()
            .raw
            .get(target_id.index())?
            .as_ref()?
            .values
            .raw
            .get(property.index())?
            .clone()
    }

    pub fn set_value(
        &self,
        target_id: AdapterTargetId,
        property: &PropertyName,
        value: AnimationValue,
    ) -> Result<(), AnimationAdapterError> {
        let (property, descriptor) = self.schema.resolve(property)?;
        descriptor.validate_value(&value)?;
        let mut targets = self.targets.borrow_mut();
        let target = targets
            .raw
            .get_mut(target_id.index())
            .and_then(Option::as_mut)
            .ok_or(AnimationAdapterError::UnknownTargetId(target_id))?;
        while target.values.len() <= property.index() {
            target.values.push(None);
        }
        target.values[property] = Some(value);
        drop(targets);
        if let Some(invalidator) = self.invalidator.borrow().as_ref() {
            invalidator(target_id);
        }
        Ok(())
    }
}

impl TargetAdapter for DrawingAdapter {
    fn id(&self) -> AdapterId {
        self.id
    }

    fn diagnostic_name(&self) -> &str {
        "drawing"
    }

    fn target_lifecycle(&self, target: AdapterTargetId) -> Option<crate::TargetLifecycle> {
        self.targets
            .borrow()
            .raw
            .get(target.index())
            .and_then(Option::as_ref)
            .map(|target| crate::TargetLifecycle {
                version: target.version,
                mounted: true,
            })
    }

    fn property_descriptor(&self, property: AdapterPropertyId) -> Option<&PropertyDescriptor> {
        self.schema.get(property)
    }

    fn resolve_targets(
        &self,
        target: &SourceTarget,
    ) -> Result<Vec<ResolutionTarget>, AnimationAdapterError> {
        let ids = match target {
            SourceTarget::One(name) => self.names.borrow().get(name).copied().map(|id| vec![id]),
            SourceTarget::Set(set) => self.sets.borrow().get(set).cloned(),
        }
        .ok_or_else(|| AnimationAdapterError::UnknownTarget(target.clone()))?;
        let targets = self.targets.borrow();
        ids.into_iter()
            .map(|id| {
                let target = targets
                    .raw
                    .get(id.index())
                    .and_then(Option::as_ref)
                    .ok_or(AnimationAdapterError::UnknownTargetId(id))?;
                Ok(ResolutionTarget {
                    name: target.name.clone(),
                    target: ResolvedTarget {
                        adapter: self.id,
                        adapter_target: id,
                    },
                    layout: None,
                })
            })
            .collect()
    }

    fn resolve_property(
        &self,
        _target: AdapterTargetId,
        property: &PropertyName,
    ) -> Result<ResolvedProperty, AnimationAdapterError> {
        let (adapter_property, descriptor) = self.schema.resolve(property)?;
        Ok(ResolvedProperty {
            adapter: self.id,
            adapter_property,
            descriptor,
        })
    }

    fn read_baseline(
        &self,
        target: AdapterTargetId,
        property: AdapterPropertyId,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        self.targets
            .borrow()
            .raw
            .get(target.index())
            .and_then(Option::as_ref)
            .and_then(|target| target.values.raw.get(property.index()))
            .and_then(Clone::clone)
            .ok_or(AnimationAdapterError::NativeRead { target, property })
    }

    fn apply(&self, update: &PropertyUpdate) -> Result<(), AnimationAdapterError> {
        let mut targets = self.targets.borrow_mut();
        let target = targets
            .raw
            .get_mut(update.target.index())
            .and_then(Option::as_mut)
            .ok_or(AnimationAdapterError::UnknownTargetId(update.target))?;
        while target.values.len() <= update.property.index() {
            target.values.push(None);
        }
        target.values[update.property] = Some(update.value.clone());
        drop(targets);
        if let Some(invalidator) = self.invalidator.borrow().as_ref() {
            invalidator(update.target);
        }
        Ok(())
    }
}
