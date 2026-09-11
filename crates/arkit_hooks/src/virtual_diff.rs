//! Pure identity/revision diff for virtual items.
//!
//! This module deliberately has no ArkUI or Dioxus dependencies so the
//! adapter contract can be tested on the host with `rustc --test`.

use std::collections::HashMap;
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
    Id: Eq + Hash,
    Revision: PartialEq,
{
    let previous_indices = id_indices(previous, StampSet::Previous)?;
    let same_ids = previous
        .iter()
        .map(|stamp| &stamp.id)
        .eq(next.iter().map(|stamp| &stamp.id));
    if same_ids {
        // Previous uniqueness proves next uniqueness when identity order is
        // identical, avoiding a second map on unchanged/revision-only renders.
        if previous == next {
            return Ok(Vec::new());
        }
        return Ok(reload_updates(
            previous
                .iter()
                .zip(next)
                .map(|(previous, next)| previous.revision != next.revision),
        ));
    }

    let next_indices = id_indices(next, StampSet::Next)?;
    let mut updates = structural_updates(previous, next, &previous_indices, &next_indices);
    updates.extend(revision_updates(previous, next, &previous_indices));
    Ok(updates)
}

fn validate_ids<Id, Revision>(
    stamps: &[VirtualItemStamp<Id, Revision>],
    set: StampSet,
) -> Result<(), DuplicateVirtualItemId>
where
    Id: Eq + Hash,
{
    id_indices(stamps, set).map(|_| ())
}

fn id_indices<Id, Revision>(
    stamps: &[VirtualItemStamp<Id, Revision>],
    set: StampSet,
) -> Result<HashMap<&Id, usize>, DuplicateVirtualItemId>
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
    Ok(first_indices)
}

fn structural_updates<Id, Revision>(
    previous: &[VirtualItemStamp<Id, Revision>],
    next: &[VirtualItemStamp<Id, Revision>],
    previous_indices: &HashMap<&Id, usize>,
    next_indices: &HashMap<&Id, usize>,
) -> Vec<VirtualItemUpdate>
where
    Id: Eq + Hash,
{
    let mut updates = Vec::new();

    // Remove identities that disappeared, from the back so every range uses
    // its current native index. Survivors remain in their previous order.
    let mut absent_ranges = Vec::new();
    let mut range_start = None;
    for (index, stamp) in previous.iter().enumerate() {
        if !next_indices.contains_key(&stamp.id) {
            range_start.get_or_insert(index);
        } else if let Some(start) = range_start.take() {
            absent_ranges.push((start, index - start));
        }
    }
    if let Some(start) = range_start {
        absent_ranges.push((start, previous.len() - start));
    }
    for (start, count) in absent_ranges.into_iter().rev() {
        updates.push(VirtualItemUpdate::Remove {
            start: start as u32,
            count: count as u32,
        });
    }

    let mut survivor_rank_by_previous_index = vec![usize::MAX; previous.len()];
    let mut survivor_count = 0;
    for (previous_index, stamp) in previous.iter().enumerate() {
        if next_indices.contains_key(&stamp.id) {
            survivor_rank_by_previous_index[previous_index] = survivor_count;
            survivor_count += 1;
        }
    }
    let target_survivor_ranks = next
        .iter()
        .filter_map(|stamp| previous_indices.get(&stamp.id).copied())
        .map(|previous_index| survivor_rank_by_previous_index[previous_index])
        .collect::<Vec<_>>();

    if !target_survivor_ranks
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        if let Some(shift) = cyclic_left_shift(&target_survivor_ranks) {
            updates.extend(rotation_moves(target_survivor_ranks.len(), shift));
        } else {
            updates.extend(survivor_moves(&target_survivor_ranks));
        }
    }

    // Insert only after survivor order is final. The adapter resolves inserted
    // identities from next[start..start + count], so every descriptor must use
    // its final index rather than a temporary permutation index.
    let mut index = 0;
    while index < next.len() {
        if previous_indices.contains_key(&next[index].id) {
            index += 1;
            continue;
        }
        let start = index;
        while index < next.len() && !previous_indices.contains_key(&next[index].id) {
            index += 1;
        }
        updates.push(VirtualItemUpdate::Insert {
            start: start as u32,
            count: (index - start) as u32,
        });
    }

    updates
}

fn cyclic_left_shift(target_ranks: &[usize]) -> Option<usize> {
    let count = target_ranks.len();
    if count == 0 {
        return None;
    }
    let shift = target_ranks[0];
    target_ranks
        .iter()
        .enumerate()
        .all(|(index, rank)| *rank == (index + shift) % count)
        .then_some(shift)
}

fn rotation_moves(count: usize, left_shift: usize) -> Vec<VirtualItemUpdate> {
    if left_shift <= count - left_shift {
        (0..left_shift)
            .map(|_| VirtualItemUpdate::Move {
                from: 0,
                to: (count - 1) as u32,
            })
            .collect()
    } else {
        (0..count - left_shift)
            .map(|_| VirtualItemUpdate::Move {
                from: (count - 1) as u32,
                to: 0,
            })
            .collect()
    }
}

/// Return the minimum move sequence for a survivor permutation.
///
/// A longest increasing subsequence stays in its original slots. Processing
/// the target from right to left moves every other identity into the gap before
/// the next retained anchor. A Fenwick tree stores the current population of
/// every original slot plus its leading gap, making both current `from` and
/// post-removal `to` indices O(log n). Exactly `n - LIS` moves are emitted.
fn survivor_moves(target_ranks: &[usize]) -> Vec<VirtualItemUpdate> {
    let retained = longest_increasing_subsequence_members(target_ranks);
    let mut positions = Fenwick::with_ones(target_ranks.len());
    let mut gap_counts = vec![0; target_ranks.len() + 1];
    let mut anchor = target_ranks.len();
    let mut updates = Vec::with_capacity(
        target_ranks.len() - retained.iter().filter(|retained| **retained).count(),
    );

    for (target_index, &original_rank) in target_ranks.iter().enumerate().rev() {
        if retained[target_index] {
            anchor = original_rank;
            continue;
        }

        let from = positions.prefix_sum(original_rank) + gap_counts[original_rank];
        positions.subtract_one(original_rank);
        let to = positions.prefix_sum(anchor);
        positions.add_one(anchor);
        gap_counts[anchor] += 1;
        debug_assert_ne!(from, to, "a maximum LIS never emits a no-op move");
        updates.push(VirtualItemUpdate::Move {
            from: from as u32,
            to: to as u32,
        });
    }

    updates
}

fn longest_increasing_subsequence_members(values: &[usize]) -> Vec<bool> {
    let mut tails = Vec::<usize>::with_capacity(values.len());
    let mut tail_indices = Vec::<usize>::with_capacity(values.len());
    let mut predecessors = vec![usize::MAX; values.len()];

    for (index, &value) in values.iter().enumerate() {
        let position = tails.partition_point(|tail| *tail < value);
        if position > 0 {
            predecessors[index] = tail_indices[position - 1];
        }
        if position == tails.len() {
            tails.push(value);
            tail_indices.push(index);
        } else {
            tails[position] = value;
            tail_indices[position] = index;
        }
    }

    let mut members = vec![false; values.len()];
    let Some(&last) = tail_indices.last() else {
        return members;
    };
    let mut index = last;
    loop {
        members[index] = true;
        let predecessor = predecessors[index];
        if predecessor == usize::MAX {
            break;
        }
        index = predecessor;
    }
    members
}

struct Fenwick {
    tree: Vec<usize>,
}

impl Fenwick {
    fn with_ones(count: usize) -> Self {
        let mut tree = vec![0; count + 2];
        for (tree_index, value) in tree.iter_mut().enumerate().take(count + 1).skip(1) {
            *value = tree_index & tree_index.wrapping_neg();
        }
        let sentinel = count + 1;
        tree[sentinel] = (sentinel & sentinel.wrapping_neg()) - 1;
        Self {
            // One extra logical slot is the gap after the final survivor.
            tree,
        }
    }

    fn add_one(&mut self, index: usize) {
        let mut tree_index = index + 1;
        while tree_index < self.tree.len() {
            self.tree[tree_index] += 1;
            tree_index += tree_index & tree_index.wrapping_neg();
        }
    }

    fn subtract_one(&mut self, index: usize) {
        let mut tree_index = index + 1;
        while tree_index < self.tree.len() {
            self.tree[tree_index] -= 1;
            tree_index += tree_index & tree_index.wrapping_neg();
        }
    }

    /// Sum logical slots in `0..end`.
    fn prefix_sum(&self, end: usize) -> usize {
        let mut tree_index = end;
        let mut sum = 0;
        while tree_index > 0 {
            sum += self.tree[tree_index];
            tree_index &= tree_index - 1;
        }
        sum
    }
}

fn revision_updates<Id, Revision>(
    previous: &[VirtualItemStamp<Id, Revision>],
    next: &[VirtualItemStamp<Id, Revision>],
    previous_indices: &HashMap<&Id, usize>,
) -> Vec<VirtualItemUpdate>
where
    Id: Eq + Hash,
    Revision: PartialEq,
{
    reload_updates(next.iter().map(|stamp| {
        previous_indices
            .get(&stamp.id)
            .is_some_and(|previous_index| previous[*previous_index].revision != stamp.revision)
    }))
}

fn reload_updates(changed_items: impl IntoIterator<Item = bool>) -> Vec<VirtualItemUpdate> {
    let mut updates = Vec::new();
    let mut range_start = None;
    let mut len = 0;

    for (index, changed) in changed_items.into_iter().enumerate() {
        len = index + 1;
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
            count: (len - start) as u32,
        });
    }
    updates
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        longest_increasing_subsequence_members, virtual_item_updates, StampSet, VirtualItemStamp,
        VirtualItemUpdate,
    };

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

    fn assert_diff_contract(
        previous: &[VirtualItemStamp<u16, u8>],
        next: &[VirtualItemStamp<u16, u8>],
    ) {
        let updates = virtual_item_updates(previous, next).unwrap();
        let previous_ids = previous
            .iter()
            .map(|stamp| stamp.id)
            .collect::<HashSet<_>>();
        let next_ids = next.iter().map(|stamp| stamp.id).collect::<HashSet<_>>();
        let mut current = previous
            .iter()
            .map(|stamp| (stamp.id, true))
            .collect::<Vec<_>>();
        let mut reloaded = vec![false; next.len()];
        let mut move_count = 0;
        let mut reached_reloads = false;

        for update in updates {
            match update {
                VirtualItemUpdate::Remove { start, count } => {
                    assert!(!reached_reloads, "structural update after reload");
                    let start = start as usize;
                    let count = count as usize;
                    assert!(count > 0 && start + count <= current.len());
                    for (id, was_previous) in current.drain(start..start + count) {
                        assert!(was_previous);
                        assert!(!next_ids.contains(&id), "removed surviving identity {id}");
                    }
                }
                VirtualItemUpdate::Insert { start, count } => {
                    assert!(!reached_reloads, "structural update after reload");
                    let start = start as usize;
                    let count = count as usize;
                    assert!(count > 0 && start <= current.len());
                    assert!(start + count <= next.len());
                    for offset in 0..count {
                        let id = next[start + offset].id;
                        assert!(
                            !previous_ids.contains(&id),
                            "reinserted surviving identity {id}"
                        );
                        current.insert(start + offset, (id, false));
                    }
                }
                VirtualItemUpdate::Move { from, to } => {
                    assert!(!reached_reloads, "structural update after reload");
                    let from = from as usize;
                    let to = to as usize;
                    assert!(from < current.len() && to < current.len());
                    assert_ne!(from, to);
                    let item = current.remove(from);
                    assert!(item.1, "new identities must never be moved");
                    current.insert(to, item);
                    move_count += 1;
                }
                VirtualItemUpdate::Reload { start, count } => {
                    reached_reloads = true;
                    let start = start as usize;
                    let count = count as usize;
                    assert!(count > 0 && start + count <= next.len());
                    for changed in &mut reloaded[start..start + count] {
                        assert!(!*changed, "overlapping reload ranges");
                        *changed = true;
                    }
                }
            }
        }

        assert_eq!(
            current.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            next.iter().map(|stamp| stamp.id).collect::<Vec<_>>()
        );
        for (id, retained_previous_identity) in current {
            assert_eq!(retained_previous_identity, previous_ids.contains(&id));
        }

        let previous_survivors = previous
            .iter()
            .filter(|stamp| next_ids.contains(&stamp.id))
            .map(|stamp| stamp.id)
            .collect::<Vec<_>>();
        let target_ranks = next
            .iter()
            .filter_map(|stamp| previous_survivors.iter().position(|id| *id == stamp.id))
            .collect::<Vec<_>>();
        let lis_length = longest_increasing_subsequence_members(&target_ranks)
            .into_iter()
            .filter(|member| *member)
            .count();
        assert_eq!(move_count, target_ranks.len() - lis_length);

        let expected_reloads = next
            .iter()
            .map(|next_stamp| {
                previous
                    .iter()
                    .find(|previous_stamp| previous_stamp.id == next_stamp.id)
                    .is_some_and(|previous_stamp| previous_stamp.revision != next_stamp.revision)
            })
            .collect::<Vec<_>>();
        assert_eq!(reloaded, expected_reloads);
    }

    fn unique_sequences(ids: &[u16]) -> Vec<Vec<u16>> {
        fn visit(ids: &[u16], current: &mut Vec<u16>, sequences: &mut Vec<Vec<u16>>) {
            sequences.push(current.clone());
            for &id in ids {
                if current.contains(&id) {
                    continue;
                }
                current.push(id);
                visit(ids, current, sequences);
                current.pop();
            }
        }

        let mut sequences = Vec::new();
        visit(ids, &mut Vec::new(), &mut sequences);
        sequences
    }

    #[test]
    fn exhaustive_unique_orders_preserve_identity_use_minimum_moves_and_reload_exactly() {
        let sequences = unique_sequences(&[0, 1, 2, 3]);
        for previous_ids in &sequences {
            for next_ids in &sequences {
                let common = next_ids
                    .iter()
                    .filter(|id| previous_ids.contains(id))
                    .copied()
                    .collect::<Vec<_>>();
                for changed_mask in 0..(1usize << common.len()) {
                    let previous = previous_ids
                        .iter()
                        .map(|id| VirtualItemStamp::new(*id, 0))
                        .collect::<Vec<_>>();
                    let next = next_ids
                        .iter()
                        .map(|id| {
                            let revision = common
                                .iter()
                                .position(|common_id| common_id == id)
                                .map_or(7, |index| ((changed_mask >> index) & 1) as u8);
                            VirtualItemStamp::new(*id, revision)
                        })
                        .collect::<Vec<_>>();
                    assert_diff_contract(&previous, &next);
                }
            }
        }
    }

    struct TestRng(u64);

    impl TestRng {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0
        }

        fn shuffle<T>(&mut self, values: &mut [T]) {
            for index in (1..values.len()).rev() {
                values.swap(index, self.next() as usize % (index + 1));
            }
        }
    }

    #[test]
    fn randomized_mixed_diffs_match_the_model() {
        let mut rng = TestRng(0x8fd5_1a2b_7c39_0041);
        for case in 0..5_000u16 {
            let previous_len = rng.next() as usize % 48;
            let mut previous_ids = (0..previous_len as u16).collect::<Vec<_>>();
            rng.shuffle(&mut previous_ids);
            let mut next_ids = previous_ids
                .iter()
                .copied()
                .filter(|_| !rng.next().is_multiple_of(5))
                .collect::<Vec<_>>();
            let new_count = rng.next() as usize % 9;
            next_ids.extend((0..new_count).map(|offset| 128 + case * 8 + offset as u16));
            rng.shuffle(&mut next_ids);

            let previous = previous_ids
                .iter()
                .map(|id| VirtualItemStamp::new(*id, (rng.next() & 3) as u8))
                .collect::<Vec<_>>();
            let next = next_ids
                .iter()
                .map(|id| VirtualItemStamp::new(*id, (rng.next() & 3) as u8))
                .collect::<Vec<_>>();
            assert_diff_contract(&previous, &next);
        }
    }
}
