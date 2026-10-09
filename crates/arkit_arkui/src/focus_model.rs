//! Native-independent focus lifecycle and keyboard navigation decisions.

use dioxus_elements::event::{
    keyboard_navigation_target, KeyAction, KeyPayload, KeyboardKey, KeyboardNavigation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FocusId(usize);
impl FocusId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
}

pub(crate) struct FocusTarget {
    pub id: FocusId,
    pub scopes: Vec<FocusId>,
    pub priority: u8,
    pub text_input: bool,
    pub tab_stop: bool,
}

pub(crate) struct FocusScope {
    pub id: FocusId,
    pub trap: bool,
    pub order: u64,
    pub auto_focus: bool,
    pub navigation: Option<KeyboardNavigation>,
}

struct ActiveScope {
    id: FocusId,
    return_to: Option<FocusId>,
}

#[derive(Default)]
pub(crate) struct FocusModel {
    targets: Vec<FocusTarget>,
    scopes: Vec<FocusScope>,
    active: Vec<ActiveScope>,
    focused: Option<FocusId>,
}

impl FocusModel {
    pub fn note_focus(&mut self, id: FocusId) {
        self.focused = Some(id);
    }

    pub fn synchronize(
        &mut self,
        targets: Vec<FocusTarget>,
        mut scopes: Vec<FocusScope>,
    ) -> Option<FocusId> {
        scopes.sort_by_key(|scope| scope.order);
        let previous_top = self.active.last().map(|scope| scope.id);
        let next_top = scopes
            .iter()
            .rev()
            .find(|scope| scope.auto_focus)
            .map(|scope| scope.id);
        let return_to = self.active.last().and_then(|scope| scope.return_to);
        let live = |id| {
            targets.iter().any(|target| target.id == id)
                || scopes.iter().any(|scope| scope.id == id)
        };
        let focused = self
            .focused
            .filter(|id| live(*id))
            .or(return_to.filter(|id| live(*id)));
        self.active.retain(|active| {
            scopes
                .iter()
                .any(|scope| scope.auto_focus && scope.id == active.id)
        });
        for scope in scopes.iter().filter(|scope| scope.auto_focus) {
            if !self.active.iter().any(|active| active.id == scope.id) {
                self.active.push(ActiveScope {
                    id: scope.id,
                    return_to: focused,
                });
            }
        }
        self.active
            .sort_by_key(|active| scopes.iter().position(|scope| scope.id == active.id));
        self.targets = targets;
        self.scopes = scopes;
        if previous_top == next_top {
            return None;
        }
        if previous_top.is_some()
            && !self
                .scopes
                .iter()
                .any(|scope| Some(scope.id) == previous_top)
        {
            if let Some(id) = return_to {
                if self.targets.iter().any(|target| {
                    target.id == id && next_top.is_none_or(|scope| target.scopes.contains(&scope))
                }) {
                    return self.request(id);
                }
            }
        }
        next_top.and_then(|scope| self.enter(scope))
    }

    fn request(&mut self, id: FocusId) -> Option<FocusId> {
        self.focused = Some(id);
        Some(id)
    }

    fn enter(&mut self, scope: FocusId) -> Option<FocusId> {
        let trapped = self
            .scopes
            .iter()
            .find(|candidate| candidate.id == scope)
            .is_some_and(|scope| scope.trap);
        let priority = self
            .targets
            .iter()
            .filter(|target| target.scopes.contains(&scope))
            .map(|target| {
                if trapped && target.priority == 1 {
                    0
                } else {
                    target.priority
                }
            })
            .max();
        let target = priority
            .and_then(|priority| {
                self.targets.iter().find(|target| {
                    target.scopes.contains(&scope)
                        && (if trapped && target.priority == 1 {
                            0
                        } else {
                            target.priority
                        }) == priority
                })
            })
            .map(|target| target.id)
            .unwrap_or(scope);
        self.request(target)
    }

    pub fn handle_key(&mut self, origin: FocusId, key: &KeyPayload) -> (bool, Option<FocusId>) {
        if let Some(scope) = self.scopes.iter().rev().find(|scope| scope.trap) {
            let scope_id = scope.id;
            let targets = self
                .targets
                .iter()
                .filter(|target| target.scopes.contains(&scope_id))
                .collect::<Vec<_>>();
            let current = targets.iter().position(|target| target.id == origin);
            if origin != scope_id && current.is_none() {
                return (
                    true,
                    if key.action == KeyAction::Down {
                        self.enter(scope_id)
                    } else {
                        None
                    },
                );
            }
            if key.key == KeyboardKey::Tab && !key.modifiers.ctrl && !key.modifiers.alt {
                if key.action != KeyAction::Down {
                    return (true, None);
                }
                let direction = if key.modifiers.shift {
                    KeyboardKey::ArrowLeft
                } else {
                    KeyboardKey::ArrowRight
                };
                let enabled = targets
                    .iter()
                    .map(|target| target.tab_stop)
                    .collect::<Vec<_>>();
                let next = match current {
                    Some(current) => keyboard_navigation_target(
                        current,
                        direction,
                        KeyboardNavigation::Horizontal,
                        &enabled,
                    ),
                    None if key.modifiers.shift => enabled.iter().rposition(|enabled| *enabled),
                    None => enabled.iter().position(|enabled| *enabled),
                };
                let next = next.map(|index| targets[index].id).unwrap_or(scope_id);
                return (true, self.request(next));
            }
        }
        if !matches!(key.action, KeyAction::Down | KeyAction::Repeat)
            || key.modifiers.ctrl
            || key.modifiers.alt
        {
            return (false, None);
        }
        let Some(target) = self.targets.iter().find(|target| target.id == origin) else {
            return (false, None);
        };
        if target.text_input
            && (key.modifiers.shift || matches!(key.key, KeyboardKey::Home | KeyboardKey::End))
        {
            return (false, None);
        }
        let Some(scope) = self
            .scopes
            .iter()
            .rev()
            .find(|scope| scope.navigation.is_some() && target.scopes.contains(&scope.id))
        else {
            return (false, None);
        };
        let ids = self
            .targets
            .iter()
            .filter(|target| target.scopes.contains(&scope.id))
            .map(|target| target.id)
            .collect::<Vec<_>>();
        let current = ids.iter().position(|id| *id == origin).unwrap_or(0);
        let next = keyboard_navigation_target(
            current,
            key.key,
            scope.navigation.expect("navigation scope"),
            &vec![true; ids.len()],
        );
        if let Some(index) = next {
            return (true, self.request(ids[index]));
        }
        (false, None)
    }
}
