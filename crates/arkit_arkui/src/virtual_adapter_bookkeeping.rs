//! Allocation-reusing storage for virtual-adapter structural bookkeeping.

pub(crate) struct ReusableSnapshot<T> {
    entries: Vec<T>,
}

impl<T> Default for ReusableSnapshot<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T> ReusableSnapshot<T> {
    pub(crate) fn take_from(&mut self, entries: impl IntoIterator<Item = T>) -> Vec<T> {
        let mut snapshot = std::mem::take(&mut self.entries);
        snapshot.clear();
        snapshot.extend(entries);
        snapshot
    }

    pub(crate) fn recycle(&mut self, mut snapshot: Vec<T>) {
        snapshot.clear();
        if snapshot.capacity() > self.entries.capacity() {
            self.entries = snapshot;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReusableSnapshot;

    #[test]
    fn snapshot_storage_reuses_capacity() {
        let mut storage = ReusableSnapshot::default();
        let first = storage.take_from(0..64);
        let capacity = first.capacity();
        storage.recycle(first);

        let second = storage.take_from(100..116);

        assert_eq!(second, (100..116).collect::<Vec<_>>());
        assert_eq!(second.capacity(), capacity);
    }

    #[test]
    fn nested_snapshot_keeps_the_largest_returned_buffer() {
        let mut storage = ReusableSnapshot::default();
        let outer = storage.take_from(0..64);
        let inner = storage.take_from(0..16);

        storage.recycle(inner);
        storage.recycle(outer);

        let reused = storage.take_from(std::iter::empty());
        assert!(reused.capacity() >= 64);
    }
}
