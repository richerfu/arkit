//! Exact-element keyboard focus for roving controls.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::NativeElementRef;
use arkit_prelude::{dioxus_elements, use_hook};
use dioxus_elements::event::{keyboard_navigation_target, KeyboardKey, KeyboardNavigation};

#[derive(Clone)]
pub struct KeyboardFocusList(Rc<RefCell<Vec<NativeElementRef>>>);

impl KeyboardFocusList {
    pub fn native_ref(&self, index: usize) -> NativeElementRef {
        self.0.borrow()[index].clone()
    }

    pub fn focus(&self, index: usize) -> bool {
        self.0
            .borrow()
            .get(index)
            .and_then(NativeElementRef::current)
            .is_some_and(|node| node.request_focus())
    }

    pub fn navigate(
        &self,
        current: usize,
        key: KeyboardKey,
        navigation: KeyboardNavigation,
        enabled: &[bool],
    ) -> Option<usize> {
        let mut enabled = enabled.to_vec();
        for _ in 0..enabled.len() {
            let next = keyboard_navigation_target(current, key, navigation, &enabled)?;
            if self.focus(next) {
                return Some(next);
            }
            enabled[next] = false;
        }
        None
    }
}

#[track_caller]
pub fn use_keyboard_focus_list(count: usize) -> KeyboardFocusList {
    let references = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    references
        .borrow_mut()
        .resize_with(count, NativeElementRef::new);
    KeyboardFocusList(references)
}
