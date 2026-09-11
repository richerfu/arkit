//! Pure ownership bookkeeping for renderer attributes driven by animations.

use std::collections::HashMap;

/// One animation host and one host-local claim generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AttributeOwner {
    pub(crate) host: u64,
    pub(crate) claim: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OwnershipClaimError<K> {
    ConflictingHost(K),
    DuplicateOwner(K),
}

#[derive(Debug)]
struct AttributeOwners {
    host: u64,
    claims: HashMap<u64, bool>,
}

/// Per-element ownership. Different instances from one host may compose, but
/// different hosts cannot independently arbitrate the same native property.
#[derive(Debug)]
pub(crate) struct AttributeOwnership<K> {
    attrs: Vec<(K, AttributeOwners)>,
}

impl<K> Default for AttributeOwnership<K> {
    fn default() -> Self {
        Self { attrs: Vec::new() }
    }
}

impl<K: Clone + PartialEq> AttributeOwnership<K> {
    pub(crate) fn claim(
        &mut self,
        owner: AttributeOwner,
        attrs: &[K],
    ) -> Result<(), OwnershipClaimError<K>> {
        for (index, key) in attrs.iter().enumerate() {
            if attrs[..index].contains(key) {
                return Err(OwnershipClaimError::DuplicateOwner(key.clone()));
            }
            let Some((_, owners)) = self.attrs.iter().find(|(candidate, _)| candidate == key)
            else {
                continue;
            };
            if owners.host != owner.host {
                return Err(OwnershipClaimError::ConflictingHost(key.clone()));
            }
            if owners.claims.contains_key(&owner.claim) {
                return Err(OwnershipClaimError::DuplicateOwner(key.clone()));
            }
        }

        for key in attrs {
            let owners = if let Some((_, owners)) = self
                .attrs
                .iter_mut()
                .find(|(candidate, _)| candidate == key)
            {
                owners
            } else {
                self.attrs.push((
                    key.clone(),
                    AttributeOwners {
                        host: owner.host,
                        claims: HashMap::new(),
                    },
                ));
                &mut self.attrs.last_mut().expect("just inserted").1
            };
            owners.claims.insert(owner.claim, false);
        }
        Ok(())
    }

    pub(crate) fn activate(&mut self, owner: AttributeOwner, attrs: &[K]) {
        for key in attrs {
            let Some((_, owners)) = self
                .attrs
                .iter_mut()
                .find(|(candidate, _)| candidate == key)
            else {
                continue;
            };
            if owners.host == owner.host {
                if let Some(active) = owners.claims.get_mut(&owner.claim) {
                    *active = true;
                }
            }
        }
    }

    /// Remove an instance reservation, returning attributes whose last active
    /// driver disappeared as part of the removal.
    pub(crate) fn remove(&mut self, owner: AttributeOwner, attrs: &[K]) -> Vec<K> {
        let mut released = Vec::new();
        let mut empty = Vec::new();
        for key in attrs {
            let Some((_, owners)) = self
                .attrs
                .iter_mut()
                .find(|(candidate, _)| candidate == key)
            else {
                continue;
            };
            if owners.host != owner.host {
                continue;
            }
            let was_active = owners.claims.remove(&owner.claim).unwrap_or(false);
            if was_active && !owners.claims.values().any(|active| *active) {
                released.push(key.clone());
            }
            if owners.claims.is_empty() {
                empty.push(key.clone());
            }
        }
        for key in empty {
            if let Some(index) = self
                .attrs
                .iter()
                .position(|(candidate, _)| candidate == &key)
            {
                self.attrs.swap_remove(index);
            }
        }
        released
    }

    #[cfg(test)]
    pub(crate) fn is_active(&self, key: &K) -> bool {
        self.attrs
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, owners)| owners)
            .is_some_and(|owners| owners.claims.values().any(|active| *active))
    }

    pub(crate) fn active_attrs(&self) -> impl Iterator<Item = &K> {
        self.attrs
            .iter()
            .filter_map(|(key, owners)| owners.claims.values().any(|active| *active).then_some(key))
    }
}

/// Execute a queued restore only for the mount generation that requested it.
pub(crate) fn restore_if_current<T: PartialEq>(
    requested: &T,
    current: &T,
    restore: impl FnOnce(),
) -> bool {
    if requested != current {
        return false;
    }
    restore();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn short_instance_cannot_release_long_instance_property() {
        let opacity = [1];
        let short = AttributeOwner { host: 1, claim: 1 };
        let long = AttributeOwner { host: 1, claim: 2 };
        let mut ownership = AttributeOwnership::default();
        ownership.claim(short, &opacity).unwrap();
        ownership.claim(long, &opacity).unwrap();
        ownership.activate(short, &opacity);
        ownership.activate(long, &opacity);

        assert!(ownership.remove(short, &opacity).is_empty());
        assert!(ownership.is_active(&1));
        assert_eq!(ownership.remove(long, &opacity), opacity);
        assert!(!ownership.is_active(&1));
    }

    #[test]
    fn cancelling_one_instance_releases_only_its_attribute() {
        let opacity = [1];
        let width = [2];
        let first = AttributeOwner { host: 1, claim: 1 };
        let second = AttributeOwner { host: 1, claim: 2 };
        let mut ownership = AttributeOwnership::default();
        ownership.claim(first, &opacity).unwrap();
        ownership.claim(second, &width).unwrap();
        ownership.activate(first, &opacity);
        ownership.activate(second, &width);

        assert_eq!(ownership.remove(first, &opacity), opacity);
        assert!(!ownership.is_active(&1));
        assert!(ownership.is_active(&2));
    }

    #[test]
    fn cross_host_same_attribute_is_an_explicit_conflict() {
        let opacity = [1];
        let mut ownership = AttributeOwnership::default();
        ownership
            .claim(AttributeOwner { host: 1, claim: 1 }, &opacity)
            .unwrap();

        assert_eq!(
            ownership.claim(AttributeOwner { host: 2, claim: 1 }, &opacity),
            Err(OwnershipClaimError::ConflictingHost(1))
        );
    }

    #[test]
    fn inactive_claim_does_not_hide_declarative_state() {
        let opacity = [1];
        let owner = AttributeOwner { host: 1, claim: 1 };
        let mut ownership = AttributeOwnership::default();
        ownership.claim(owner, &opacity).unwrap();

        assert!(!ownership.is_active(&1));
        assert!(ownership.active_attrs().next().is_none());
    }

    #[test]
    fn duplicate_live_owner_is_rejected_without_changing_activity() {
        let opacity = [1];
        let owner = AttributeOwner { host: 1, claim: 1 };
        let mut ownership = AttributeOwnership::default();
        ownership.claim(owner, &opacity).unwrap();
        ownership.activate(owner, &opacity);

        assert_eq!(
            ownership.claim(owner, &opacity),
            Err(OwnershipClaimError::DuplicateOwner(1))
        );
        assert!(ownership.is_active(&1));
    }

    #[test]
    fn duplicate_attribute_in_one_claim_is_rejected_atomically() {
        let owner = AttributeOwner { host: 1, claim: 1 };
        let mut ownership = AttributeOwnership::default();

        assert_eq!(
            ownership.claim(owner, &[1, 1]),
            Err(OwnershipClaimError::DuplicateOwner(1))
        );
        assert!(ownership.active_attrs().next().is_none());
    }

    #[test]
    fn removal_allows_a_later_host_to_claim_and_restart() {
        let opacity = [1];
        let first = AttributeOwner { host: 1, claim: 1 };
        let restarted = AttributeOwner { host: 2, claim: 1 };
        let mut ownership = AttributeOwnership::default();
        ownership.claim(first, &opacity).unwrap();
        ownership.activate(first, &opacity);
        assert_eq!(ownership.remove(first, &opacity), opacity);

        ownership.claim(restarted, &opacity).unwrap();
        ownership.activate(restarted, &opacity);
        assert!(ownership.is_active(&1));
    }

    #[test]
    fn queued_restore_reads_latest_desired_value() {
        let desired = Cell::new(0);
        let restored = Cell::new(None);
        let request_epoch = 7;
        desired.set(2);

        assert!(restore_if_current(&request_epoch, &7, || {
            restored.set(Some(desired.get()));
        }));
        assert_eq!(restored.get(), Some(2));
    }

    #[test]
    fn stale_mount_restore_is_ignored() {
        let restored = Cell::new(false);
        assert!(!restore_if_current(&7, &8, || restored.set(true)));
        assert!(!restored.get());
    }

    #[test]
    fn same_epoch_on_a_different_reference_is_stale() {
        let restored = Cell::new(false);
        assert!(!restore_if_current(&(1, 7), &(2, 7), || {
            restored.set(true);
        }));
        assert!(!restored.get());
    }
}
