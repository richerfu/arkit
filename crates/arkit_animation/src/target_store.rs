use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use arkit_animation_core::{
    AdapterTargetId, SourceTarget, TargetLayoutSnapshot, TargetName, TargetSetName,
};
use arkit_arkui::MountedNodeLease;

use crate::{AnimationAdapterError, AnimationTargetBinding, TargetVisualState};

#[derive(Default)]
pub struct TargetStore {
    targets: IndexVec<AdapterTargetId, Option<AnimationTargetBinding>>,
    names: FxHashMap<TargetName, AdapterTargetId>,
    sets: FxHashMap<TargetSetName, Vec<AdapterTargetId>>,
    next_version: u64,
}

impl TargetStore {
    pub fn register(
        &mut self,
        name: TargetName,
        node: MountedNodeLease,
        layout: Option<TargetLayoutSnapshot>,
    ) -> Result<AdapterTargetId, AnimationAdapterError> {
        if self.names.contains_key(&name) {
            return Err(AnimationAdapterError::DuplicateTarget(name));
        }
        let id = self
            .targets
            .iter_enumerated()
            .find_map(|(id, target)| target.is_none().then_some(id))
            .unwrap_or_else(|| self.targets.push(None));
        let version = self.next_version;
        self.next_version = self
            .next_version
            .checked_add(1)
            .expect("target lifecycle version exhausted");
        self.targets[id] = Some(AnimationTargetBinding {
            id,
            name: name.clone(),
            node,
            layout,
            mounted: true,
            version,
            visual: TargetVisualState::default(),
        });
        self.names.insert(name, id);
        Ok(id)
    }

    pub fn unregister(&mut self, id: AdapterTargetId) -> bool {
        let Some(binding) = self.targets.raw.get_mut(id.index()).and_then(Option::take) else {
            return false;
        };
        self.names.remove(&binding.name);
        for members in self.sets.values_mut() {
            members.retain(|member| *member != id);
        }
        true
    }

    pub fn set_members(&mut self, set: TargetSetName, members: Vec<AdapterTargetId>) {
        self.sets.insert(set, members);
    }

    pub fn resolve(
        &self,
        target: &SourceTarget,
    ) -> Result<Vec<AdapterTargetId>, AnimationAdapterError> {
        match target {
            SourceTarget::One(name) => self
                .names
                .get(name)
                .copied()
                .map(|id| vec![id])
                .ok_or_else(|| AnimationAdapterError::UnknownTarget(target.clone())),
            SourceTarget::Set(set) => self
                .sets
                .get(set)
                .cloned()
                .ok_or_else(|| AnimationAdapterError::UnknownTarget(target.clone())),
        }
    }

    pub fn get(&self, id: AdapterTargetId) -> Option<&AnimationTargetBinding> {
        self.targets.raw.get(id.index()).and_then(Option::as_ref)
    }

    pub fn get_mut(&mut self, id: AdapterTargetId) -> Option<&mut AnimationTargetBinding> {
        self.targets
            .raw
            .get_mut(id.index())
            .and_then(Option::as_mut)
    }

    pub fn id_for_name(&self, name: &TargetName) -> Option<AdapterTargetId> {
        self.names.get(name).copied()
    }

    /// Iterate every currently registered target binding.
    pub fn iter(&self) -> impl Iterator<Item = &AnimationTargetBinding> {
        self.targets.raw.iter().flatten()
    }
}
