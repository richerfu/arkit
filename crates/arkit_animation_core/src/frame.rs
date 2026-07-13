//! Reusable property update batches emitted in engine slot order.

use smallvec::SmallVec;

use crate::{AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, InvalidationClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameId(u64);

impl FrameId {
    pub const fn new(sequence: u64) -> Self {
        Self(sequence)
    }

    pub const fn sequence(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyUpdate {
    pub adapter: AdapterId,
    pub target: AdapterTargetId,
    pub property: AdapterPropertyId,
    pub invalidation: InvalidationClass,
    pub value: AnimationValue,
}

#[derive(Debug, Default)]
pub struct FrameBatch {
    updates: SmallVec<[PropertyUpdate; 32]>,
}

impl FrameBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            updates: SmallVec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.updates.clear();
    }

    pub fn push(&mut self, update: PropertyUpdate) {
        self.updates.push(update);
    }

    pub fn as_slice(&self) -> &[PropertyUpdate] {
        &self.updates
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_preserves_the_engine_slot_order() {
        let mut batch = FrameBatch::new();
        batch.push(PropertyUpdate {
            adapter: AdapterId::new(0),
            target: AdapterTargetId::new(1),
            property: AdapterPropertyId::new(0),
            invalidation: InvalidationClass::Transform,
            value: AnimationValue::Scalar(1.0),
        });
        batch.push(PropertyUpdate {
            adapter: AdapterId::new(0),
            target: AdapterTargetId::new(0),
            property: AdapterPropertyId::new(1),
            invalidation: InvalidationClass::Paint,
            value: AnimationValue::Scalar(1.0),
        });
        batch.push(PropertyUpdate {
            adapter: AdapterId::new(0),
            target: AdapterTargetId::new(0),
            property: AdapterPropertyId::new(0),
            invalidation: InvalidationClass::Transform,
            value: AnimationValue::Scalar(1.0),
        });
        let ids = batch
            .as_slice()
            .iter()
            .map(|update| (update.target.index(), update.property.index()))
            .collect::<Vec<_>>();
        assert_eq!(ids, [(1, 0), (0, 1), (0, 0)]);
    }
}
