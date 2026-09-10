//! Pure identity/revision diff for virtual items.
//!
//! This module deliberately has no ArkUI or Dioxus dependencies so the
//! adapter contract can be tested on the host with `rustc --test`.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

/// Stable identity plus the revision of every visual input for one item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualItemStamp<Id, Revision> {
    pub id: Id,
    pub revision: Revision,
}

impl<Id, Revision> VirtualItemStamp<Id, Revision> {
    pub const fn new(id: Id, revision: Revision) -> Self {
        Self { id, revision }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VirtualItemUpdate {
    Insert { start: u32, count: u32 },
    Remove { start: u32, count: u32 },
    Move { from: u32, to: u32 },
    Reload { start: u32, count: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StampSet {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DuplicateVirtualItemId {
    pub set: StampSet,
    pub first_index: usize,
    pub duplicate_index: usize,
}

impl fmt::Display for DuplicateVirtualItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let set = match self.set {
            StampSet::Previous => "previous",
            StampSet::Next => "next",
        };
        write!(
            formatter,
            "duplicate id in {set} virtual items at indices {} and {}",
            self.first_index, self.duplicate_index
        )
    }
}

pub(crate) fn validate_virtual_item_ids<Id, Revision>(
    stamps: &[VirtualItemStamp<Id, Revision>],
) -> Result<(), DuplicateVirtualItemId>
where
    Id: Eq + Hash,
{
    validate_ids(stamps, StampSet::Next)
}

pub(crate) fn virtual_item_updates<Id, Revision>(
    previous: &[VirtualItemStamp<Id, Revision>],
    next: &[VirtualItemStamp<Id, Revision>],
) -> Result<Vec<VirtualItemUpdate>, DuplicateVirtualItemId>
where
    Id: Clone + Eq + Hash,
    Revision: PartialEq,
{
    validate_ids(previous, StampSet::Previous)?;
    validate_ids(next, StampSet::Next)?;
    if previous == next {
        return Ok(Vec::new());
    }

    let mut updates = if previous
        .iter()
        .map(|stamp| &stamp.id)
        .eq(next.iter().map(|stamp| &stamp.id))
    {
        Vec::new()
    } else {
        structural_updates(previous, next)
    };
    updates.extend(revision_updates(previous, next));
    Ok(updates)
}

fn validate_ids<Id, Revision>(
    stamps: &[VirtualItemStamp<Id, Revision>],
    set: StampSet,
) -> Result<(), DuplicateVirtualItemId>
where
    Id: Eq + Hash,
{
    let mut first_indices = HashMap::with_capacity(stamps.len());
    for (duplicate_index, stamp) in stamps.iter().enumerate() {
        if let Some(first_index) = first_indices.insert(&stamp.id, duplicate_index) {
            return Err(DuplicateVirtualItemId {
                set,
                first_index,
                duplicate_index,
            });
        }
    }
    Ok(())
}

fn structural_updates<Id, Revision>(
    previous: &[VirtualItemStamp<Id, Revision>],
    next: &[VirtualItemStamp<Id, Revision>],
) -> Vec<VirtualItemUpdate>
where
    Id: Clone + Eq + Hash,
{
    let next_ids = next.iter().map(|stamp| &stamp.id).collect::<HashSet<_>>();
    let mut current = previous
        .iter()
        .map(|stamp| stamp.id.clone())
        .collect::<Vec<_>>();
    let mut updates = Vec::new();

    let mut absent_ranges = Vec::new();
    let mut range_start = None;
    for (index, id) in current.iter().enumerate() {
        if !next_ids.contains(id) {
            range_start.get_or_insert(index);
        } else if let Some(start) = range_start.take() {
            absent_ranges.push((start, index - start));
        }
    }
    if let Some(start) = range_start {
        absent_ranges.push((start, current.len() - start));
    }
    for (start, count) in absent_ranges.into_iter().rev() {
        current.drain(start..start + count);
        updates.push(VirtualItemUpdate::Remove {
            start: start as u32,
            count: count as u32,
        });
    }

    let mut target = 0;
    while target < next.len() {
        if current.get(target) == Some(&next[target].id) {
            target += 1;
            continue;
        }
        if let Some(offset) = current[target..]
            .iter()
            .position(|id| id == &next[target].id)
        {
            let from = target + offset;
            let id = current.remove(from);
            current.insert(target, id);
            updates.push(VirtualItemUpdate::Move {
                from: from as u32,
                to: target as u32,
            });
            target += 1;
            continue;
        }

        let start = target;
        while target < next.len()
            && !current[target.min(current.len())..].contains(&next[target].id)
        {
            current.insert(target, next[target].id.clone());
            target += 1;
        }
        updates.push(VirtualItemUpdate::Insert {
            start: start as u32,
            count: (target - start) as u32,
        });
    }

    if current.len() > next.len() {
        let start = next.len();
        let count = current.len() - start;
        current.truncate(start);
        updates.push(VirtualItemUpdate::Remove {
            start: start as u32,
            count: count as u32,
        });
    }
    debug_assert!(current.iter().eq(next.iter().map(|stamp| &stamp.id)));
    updates
}

fn revision_updates<Id, Revision>(
    previous: &[VirtualItemStamp<Id, Revision>],
    next: &[VirtualItemStamp<Id, Revision>],
) -> Vec<VirtualItemUpdate>
where
    Id: Eq + Hash,
    Revision: PartialEq,
{
    let previous_revisions = previous
        .iter()
        .map(|stamp| (&stamp.id, &stamp.revision))
        .collect::<HashMap<_, _>>();
    let mut updates = Vec::new();
    let mut range_start = None;

    for (index, stamp) in next.iter().enumerate() {
        let changed = previous_revisions
            .get(&stamp.id)
            .is_some_and(|revision| *revision != &stamp.revision);
        if changed {
            range_start.get_or_insert(index);
        } else if let Some(start) = range_start.take() {
            updates.push(VirtualItemUpdate::Reload {
                start: start as u32,
                count: (index - start) as u32,
            });
        }
    }
    if let Some(start) = range_start {
        updates.push(VirtualItemUpdate::Reload {
            start: start as u32,
            count: (next.len() - start) as u32,
        });
    }
    updates
}

#[cfg(test)]
mod tests {
    use super::{virtual_item_updates, StampSet, VirtualItemStamp, VirtualItemUpdate};

    fn stamp(id: char, revision: u8) -> VirtualItemStamp<char, u8> {
        VirtualItemStamp::new(id, revision)
    }

    #[test]
    fn move_and_revision_reload_the_moved_identity_at_its_final_index() {
        let previous = [stamp('a', 0), stamp('b', 0), stamp('c', 0)];
        let next = [stamp('b', 1), stamp('a', 0), stamp('c', 0)];

        assert_eq!(
            virtual_item_updates(&previous, &next),
            Ok(vec![
                VirtualItemUpdate::Move { from: 1, to: 0 },
                VirtualItemUpdate::Reload { start: 0, count: 1 },
            ])
        );
    }

    #[test]
    fn unchanged_and_unaffected_items_emit_no_reload() {
        let previous = [stamp('a', 0), stamp('b', 0), stamp('c', 0), stamp('d', 0)];
        let next = [stamp('a', 0), stamp('b', 1), stamp('c', 0), stamp('d', 0)];

        assert_eq!(
            virtual_item_updates(&previous, &next),
            Ok(vec![VirtualItemUpdate::Reload { start: 1, count: 1 }])
        );
        assert_eq!(virtual_item_updates(&next, &next), Ok(Vec::new()));
    }

    #[test]
    fn duplicate_ids_are_rejected_in_both_snapshots() {
        let duplicate = [stamp('a', 0), stamp('a', 1)];
        let unique = [stamp('a', 0), stamp('b', 0)];

        let previous_error = virtual_item_updates(&duplicate, &unique).unwrap_err();
        assert_eq!(previous_error.set, StampSet::Previous);
        assert_eq!(
            (previous_error.first_index, previous_error.duplicate_index),
            (0, 1)
        );

        let next_error = virtual_item_updates(&unique, &duplicate).unwrap_err();
        assert_eq!(next_error.set, StampSet::Next);
        assert_eq!((next_error.first_index, next_error.duplicate_index), (0, 1));
    }

    #[test]
    fn insertions_do_not_reload_new_items() {
        let previous = [stamp('a', 0), stamp('c', 0)];
        let next = [stamp('a', 0), stamp('b', 7), stamp('c', 0)];

        assert_eq!(
            virtual_item_updates(&previous, &next),
            Ok(vec![VirtualItemUpdate::Insert { start: 1, count: 1 }])
        );
    }
}
