//! Platform-independent child-list operations shared by the ArkUI projection.
//!
//! Keep this module `std`-only. Besides making the ordering contract explicit,
//! that lets CI exercise it with `rustc --test` on hosts that do not provide
//! the OpenHarmony native libraries required to link `arkit_arkui` itself.

/// Remove exactly one occurrence of `child`, returning its former index.
pub(crate) fn remove_once<T: PartialEq>(children: &mut Vec<T>, child: &T) -> Option<usize> {
    let index = children.iter().position(|candidate| candidate == child)?;
    children.remove(index);
    Some(index)
}

/// Insert a child that the caller has already detached from its old list.
pub(crate) fn insert_detached_at<T>(children: &mut Vec<T>, index: usize, child: T) {
    children.insert(index, child);
}

/// Resolve an insertion point from live mounted identity.
///
/// `following` contains projected logical siblings at and after the desired
/// logical position. If none is mounted, ordinary root content stops before
/// the first active portal in `trailing`; non-root callers pass an empty
/// `trailing` iterator and append to the mounted list.
pub(crate) fn native_insert_index<T, K, F, P, Key>(
    mounted: &[T],
    following: F,
    trailing: P,
    key: Key,
) -> usize
where
    K: Eq,
    F: IntoIterator<Item = K>,
    P: IntoIterator<Item = K>,
    Key: Fn(&T) -> K,
{
    for sibling in following {
        if let Some(index) = mounted.iter().position(|child| key(child) == sibling) {
            return index;
        }
    }
    trailing
        .into_iter()
        .filter_map(|portal| mounted.iter().position(|child| key(child) == portal))
        .min()
        .unwrap_or(mounted.len())
}

#[cfg(test)]
mod tests {
    use super::{insert_detached_at, native_insert_index, remove_once};

    fn move_before(logical: &mut Vec<char>, native: &mut Vec<char>, child: char, anchor: char) {
        remove_once(logical, &child).expect("moving child must be logical");
        remove_once(native, &child).expect("moving child must be mounted");

        let logical_index = logical
            .iter()
            .position(|candidate| *candidate == anchor)
            .expect("anchor must remain logical");
        let native_index = native_insert_index(
            native,
            logical[logical_index..].iter().copied(),
            std::iter::empty(),
            |child| *child,
        );
        insert_detached_at(logical, logical_index, child);
        insert_detached_at(native, native_index, child);
    }

    #[test]
    fn repeated_keyed_moves_preserve_unique_logical_and_native_order() {
        let mut logical = vec!['A', 'B', 'C'];
        let mut native = logical.clone();

        move_before(&mut logical, &mut native, 'C', 'A');
        assert_eq!(logical, vec!['C', 'A', 'B']);
        assert_eq!(native, logical);

        move_before(&mut logical, &mut native, 'A', 'C');
        move_before(&mut logical, &mut native, 'B', 'C');
        assert_eq!(logical, vec!['A', 'B', 'C']);
        assert_eq!(native, logical);

        move_before(&mut logical, &mut native, 'C', 'A');
        assert_eq!(logical, vec!['C', 'A', 'B']);
        assert_eq!(native, logical);
    }

    #[test]
    fn forward_move_recomputes_anchor_after_detach() {
        let mut logical = vec!['A', 'B', 'C', 'D'];
        let mut native = logical.clone();

        remove_once(&mut logical, &'A').unwrap();
        remove_once(&mut native, &'A').unwrap();
        let after_c = logical.iter().position(|child| *child == 'C').unwrap() + 1;
        let native_index = native_insert_index(
            &native,
            logical[after_c..].iter().copied(),
            std::iter::empty(),
            |child| *child,
        );
        insert_detached_at(&mut logical, after_c, 'A');
        insert_detached_at(&mut native, native_index, 'A');

        assert_eq!(logical, vec!['B', 'C', 'A', 'D']);
        assert_eq!(native, logical);
    }

    #[test]
    fn root_tail_insertion_stays_before_active_portals() {
        let mounted = ['A', 'M', 'F'];
        let index = native_insert_index(
            &mounted,
            std::iter::empty(),
            ['M', 'F'].iter().copied(),
            |child| *child,
        );

        assert_eq!(index, 1);
    }

    #[test]
    fn connected_portal_move_keeps_the_portal_as_the_root_tail_boundary() {
        // The portal has been removed from the logical root for a keyed move,
        // but remains mounted and active while the batch re-inserts it.
        let mut mounted = vec!['B', 'P'];
        let index = native_insert_index(
            &mounted,
            std::iter::empty(),
            ['P'].iter().copied(),
            |child| *child,
        );
        insert_detached_at(&mut mounted, index, 'A');

        assert_eq!(mounted, vec!['B', 'A', 'P']);
    }

    #[test]
    fn removed_portals_do_not_define_the_active_boundary() {
        let mounted = ['A', 'X', 'M'];
        let index = native_insert_index(
            &mounted,
            std::iter::empty(),
            ['M'].iter().copied(),
            |child| *child,
        );

        assert_eq!(index, 2);
    }

    #[test]
    fn removing_merged_text_before_recomputing_observes_final_content() {
        let mut logical = vec!["A", "B"];
        remove_once(&mut logical, &"B").unwrap();

        assert_eq!(logical.concat(), "A");
    }
}
