use std::rc::Rc;

use oxc_index::IndexVec;

use arkit_animation_core::{AdapterId, FrameBatch};

use crate::{AnimationAdapterError, TargetAdapter};

#[derive(Default)]
pub struct AdapterRegistry {
    adapters: IndexVec<AdapterId, Option<Rc<dyn TargetAdapter>>>,
}

impl AdapterRegistry {
    pub fn next_id(&self) -> AdapterId {
        self.adapters
            .iter_enumerated()
            .find_map(|(id, adapter)| adapter.is_none().then_some(id))
            .unwrap_or_else(|| AdapterId::new(self.adapters.len()))
    }

    pub fn register(
        &mut self,
        adapter: Rc<dyn TargetAdapter>,
    ) -> Result<AdapterId, AnimationAdapterError> {
        let expected = self.next_id();
        if adapter.id() != expected {
            return Err(AnimationAdapterError::DuplicateAdapter(adapter.id()));
        }
        if expected.index() == self.adapters.len() {
            Ok(self.adapters.push(Some(adapter)))
        } else {
            self.adapters[expected] = Some(adapter);
            Ok(expected)
        }
    }

    pub fn get(&self, id: AdapterId) -> Result<&dyn TargetAdapter, AnimationAdapterError> {
        self.adapters
            .raw
            .get(id.index())
            .and_then(Option::as_ref)
            .map(Rc::as_ref)
            .ok_or(AnimationAdapterError::UnknownAdapter(id))
    }

    pub fn unregister(&mut self, id: AdapterId) -> Option<Rc<dyn TargetAdapter>> {
        self.adapters.raw.get_mut(id.index()).and_then(Option::take)
    }

    pub fn apply(&self, batch: &FrameBatch) -> Result<(), AnimationAdapterError> {
        for update in batch.as_slice() {
            self.get(update.adapter)?.validate_update(update)?;
        }
        let updates = batch.as_slice();
        let mut start = 0;
        while start < updates.len() {
            let adapter = updates[start].adapter;
            let mut end = start + 1;
            while end < updates.len() && updates[end].adapter == adapter {
                end += 1;
            }
            self.get(adapter)?.apply_batch(&updates[start..end])?;
            start = end;
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn TargetAdapter> {
        self.adapters.iter().flatten().map(Rc::as_ref)
    }
}
