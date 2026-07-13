use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use arkit_prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::Stagger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresenceMode {
    Sync,
    Wait,
    PopLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresencePhase {
    Entering,
    Present,
    Leaving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCancelPolicy {
    CompleteExit,
    Reenter,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct PresenceKey(Arc<str>);

impl PresenceKey {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct PresenceEntry<T> {
    pub key: PresenceKey,
    pub value: T,
    pub phase: PresencePhase,
    pub popped_from_layout: bool,
    pub stagger_delay_ms: u32,
}

pub struct AnimatePresence<T> {
    mode: PresenceMode,
    entries: Vec<PresenceEntry<T>>,
    positions: FxHashMap<PresenceKey, usize>,
    waiting: Vec<(PresenceKey, T)>,
    enter_stagger: Option<Stagger>,
    exit_stagger: Option<Stagger>,
}

impl<T> AnimatePresence<T> {
    pub fn new(mode: PresenceMode) -> Self {
        Self {
            mode,
            entries: Vec::new(),
            positions: FxHashMap::default(),
            waiting: Vec::new(),
            enter_stagger: None,
            exit_stagger: None,
        }
    }

    pub fn with_stagger(mut self, enter: Option<Stagger>, exit: Option<Stagger>) -> Self {
        self.enter_stagger = enter;
        self.exit_stagger = exit;
        self
    }

    pub fn entries(&self) -> &[PresenceEntry<T>] {
        &self.entries
    }

    pub fn phase(&self, key: &PresenceKey) -> Option<PresencePhase> {
        self.positions
            .get(key)
            .map(|index| self.entries[*index].phase)
    }

    pub fn update(&mut self, children: impl IntoIterator<Item = (PresenceKey, T)>) {
        let next = children.into_iter().collect::<Vec<_>>();
        let next_keys = next
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<FxHashSet<_>>();
        let leaving_total = self
            .entries
            .iter()
            .filter(|entry| !next_keys.contains(&entry.key))
            .count();
        let mut leaving_index = 0;
        for entry in &mut self.entries {
            if !next_keys.contains(&entry.key) && entry.phase != PresencePhase::Leaving {
                entry.phase = PresencePhase::Leaving;
                entry.popped_from_layout = self.mode == PresenceMode::PopLayout;
                entry.stagger_delay_ms = self
                    .exit_stagger
                    .as_ref()
                    .map_or(0, |stagger| stagger.delay(leaving_index, leaving_total));
                leaving_index += 1;
            }
        }
        let has_leaving = self
            .entries
            .iter()
            .any(|entry| entry.phase == PresencePhase::Leaving);
        if self.mode == PresenceMode::Wait && has_leaving {
            self.waiting = next
                .into_iter()
                .filter(|(key, _)| !self.positions.contains_key(key))
                .collect();
            return;
        }
        self.insert_new(next);
    }

    pub fn mark_present(&mut self, key: &PresenceKey) -> bool {
        let Some(index) = self.positions.get(key).copied() else {
            return false;
        };
        let entry = &mut self.entries[index];
        if entry.phase != PresencePhase::Entering {
            return false;
        }
        entry.phase = PresencePhase::Present;
        true
    }

    pub fn settle_exit(&mut self, key: &PresenceKey) -> bool {
        let Some(index) = self.positions.get(key).copied() else {
            return false;
        };
        if self.entries[index].phase != PresencePhase::Leaving {
            return false;
        }
        self.entries.remove(index);
        self.reindex();
        self.flush_waiting();
        true
    }

    pub fn cancel_exit(&mut self, key: &PresenceKey, policy: ExitCancelPolicy) -> bool {
        match policy {
            ExitCancelPolicy::CompleteExit => self.settle_exit(key),
            ExitCancelPolicy::Reenter => {
                let Some(index) = self.positions.get(key).copied() else {
                    return false;
                };
                let entry = &mut self.entries[index];
                if entry.phase != PresencePhase::Leaving {
                    return false;
                }
                entry.phase = PresencePhase::Entering;
                entry.popped_from_layout = false;
                true
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.positions.clear();
        self.waiting.clear();
    }

    fn insert_new(&mut self, children: Vec<(PresenceKey, T)>) {
        let total = children.len();
        for (index, (key, value)) in children.into_iter().enumerate() {
            if let Some(position) = self.positions.get(&key).copied() {
                let entry = &mut self.entries[position];
                entry.value = value;
                if entry.phase == PresencePhase::Leaving {
                    entry.phase = PresencePhase::Entering;
                    entry.popped_from_layout = false;
                }
                continue;
            }
            let stagger_delay_ms = self
                .enter_stagger
                .as_ref()
                .map_or(0, |stagger| stagger.delay(index, total));
            self.entries.push(PresenceEntry {
                key,
                value,
                phase: PresencePhase::Entering,
                popped_from_layout: false,
                stagger_delay_ms,
            });
        }
        self.reindex();
    }

    fn flush_waiting(&mut self) {
        if self.mode == PresenceMode::Wait
            && !self
                .entries
                .iter()
                .any(|entry| entry.phase == PresencePhase::Leaving)
        {
            let waiting = std::mem::take(&mut self.waiting);
            self.insert_new(waiting);
        }
    }

    fn reindex(&mut self) {
        self.positions.clear();
        self.positions.extend(
            self.entries
                .iter()
                .enumerate()
                .map(|(index, entry)| (entry.key.clone(), index)),
        );
    }
}

impl<T> Drop for AnimatePresence<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[derive(Clone)]
pub struct PresenceHandle<T> {
    inner: Rc<RefCell<AnimatePresence<T>>>,
    version: Signal<u64>,
}

impl<T: Clone> PresenceHandle<T> {
    pub fn entries(&self) -> Vec<PresenceEntry<T>> {
        self.inner.borrow().entries().to_vec()
    }

    pub fn mark_present(&self, key: &PresenceKey) -> bool {
        let changed = self.inner.borrow_mut().mark_present(key);
        self.invalidate_if(changed);
        changed
    }

    pub fn settle_exit(&self, key: &PresenceKey) -> bool {
        let changed = self.inner.borrow_mut().settle_exit(key);
        self.invalidate_if(changed);
        changed
    }

    pub fn cancel_exit(&self, key: &PresenceKey, policy: ExitCancelPolicy) -> bool {
        let changed = self.inner.borrow_mut().cancel_exit(key, policy);
        self.invalidate_if(changed);
        changed
    }

    fn invalidate_if(&self, changed: bool) {
        if changed {
            let mut version = self.version;
            version += 1;
        }
    }
}

#[track_caller]
pub fn use_animate_presence<T: Clone + 'static>(
    mode: PresenceMode,
    children: impl IntoIterator<Item = (PresenceKey, T)>,
) -> PresenceHandle<T> {
    let inner = use_hook(|| Rc::new(RefCell::new(AnimatePresence::new(mode))));
    let version = use_signal(|| 0_u64);
    let _ = version();
    inner.borrow_mut().update(children);
    PresenceHandle { inner, version }
}

pub struct PresenceStore {
    inner: AnimatePresence<()>,
}

impl Default for PresenceStore {
    fn default() -> Self {
        Self {
            inner: AnimatePresence::new(PresenceMode::Sync),
        }
    }
}

impl PresenceStore {
    pub fn enter(&mut self, key: impl Into<Box<str>>) {
        let key: Box<str> = key.into();
        let key = PresenceKey::new(Arc::<str>::from(key));
        let mut children = self
            .inner
            .entries()
            .iter()
            .filter(|entry| entry.phase != PresencePhase::Leaving)
            .map(|entry| (entry.key.clone(), ()))
            .collect::<Vec<_>>();
        children.push((key, ()));
        self.inner.update(children);
    }

    pub fn mark_present(&mut self, key: &str) {
        self.inner.mark_present(&PresenceKey::new(key));
    }

    pub fn leave(&mut self, key: &str) -> bool {
        let key = PresenceKey::new(key);
        if self.inner.phase(&key).is_none() {
            return false;
        }
        let children = self
            .inner
            .entries()
            .iter()
            .filter(|entry| entry.key != key && entry.phase != PresencePhase::Leaving)
            .map(|entry| (entry.key.clone(), ()))
            .collect::<Vec<_>>();
        self.inner.update(children);
        true
    }

    pub fn settle_exit(&mut self, key: &str) -> bool {
        self.inner.settle_exit(&PresenceKey::new(key))
    }

    pub fn phase(&self, key: &str) -> Option<PresencePhase> {
        self.inner.phase(&PresenceKey::new(key))
    }

    pub fn can_enter(&self, mode: PresenceMode) -> bool {
        mode != PresenceMode::Wait
            || !self
                .inner
                .entries()
                .iter()
                .any(|entry| entry.phase == PresencePhase::Leaving)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_mode_defers_enter_until_terminal_exit() {
        let a = PresenceKey::new("a");
        let b = PresenceKey::new("b");
        let mut presence = AnimatePresence::new(PresenceMode::Wait);
        presence.update([(a.clone(), 1)]);
        presence.mark_present(&a);
        presence.update([(b.clone(), 2)]);
        assert_eq!(presence.phase(&a), Some(PresencePhase::Leaving));
        assert_eq!(presence.phase(&b), None);
        presence.settle_exit(&a);
        assert_eq!(presence.phase(&b), Some(PresencePhase::Entering));
    }

    #[test]
    fn pop_layout_retains_leaving_representation() {
        let key = PresenceKey::new("item");
        let mut presence = AnimatePresence::new(PresenceMode::PopLayout);
        presence.update([(key.clone(), 1)]);
        presence.update([]);
        assert!(presence.entries()[0].popped_from_layout);
        assert_eq!(presence.phase(&key), Some(PresencePhase::Leaving));
    }
}
