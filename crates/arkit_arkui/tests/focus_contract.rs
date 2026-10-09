// Run the production focus lifecycle without loading the native renderer.
#[path = "../src/focus_model.rs"]
mod focus_model;

use dioxus_elements::event::{KeyAction, KeyPayload, KeyboardKey, KeyboardNavigation};
use focus_model::{FocusId, FocusModel, FocusScope, FocusTarget};

fn id(value: usize) -> FocusId {
    FocusId::new(value)
}
fn target(value: usize, scopes: &[usize]) -> FocusTarget {
    FocusTarget {
        id: id(value),
        scopes: scopes.iter().map(|value| id(*value)).collect(),
        priority: 0,
        text_input: false,
        tab_stop: true,
    }
}
fn scope(value: usize, trap: bool) -> FocusScope {
    FocusScope {
        id: id(value),
        trap,
        order: value as u64,
        auto_focus: true,
        navigation: None,
    }
}
fn key(key: KeyboardKey) -> KeyPayload {
    KeyPayload {
        key,
        action: KeyAction::Down,
        ..Default::default()
    }
}

#[test]
fn opening_and_closing_a_modal_returns_focus_to_its_trigger() {
    let mut model = FocusModel::default();
    model.synchronize(vec![target(1, &[])], vec![]);
    model.note_focus(id(1));
    assert_eq!(
        model.synchronize(
            vec![target(1, &[]), target(2, &[10])],
            vec![scope(10, true)]
        ),
        Some(id(2))
    );
    assert_eq!(model.synchronize(vec![target(1, &[])], vec![]), Some(id(1)));
}

#[test]
fn nested_popups_restore_the_parent_control_before_the_page_trigger() {
    let mut model = FocusModel::default();
    model.synchronize(vec![target(1, &[])], vec![]);
    model.note_focus(id(1));
    model.synchronize(
        vec![target(1, &[]), target(2, &[10])],
        vec![scope(10, true)],
    );
    assert_eq!(
        model.synchronize(
            vec![target(1, &[]), target(2, &[10]), target(3, &[10, 20])],
            vec![scope(10, true), scope(20, false)]
        ),
        Some(id(3))
    );
    assert_eq!(
        model.synchronize(
            vec![target(1, &[]), target(2, &[10])],
            vec![scope(10, true)]
        ),
        Some(id(2))
    );
    assert_eq!(model.synchronize(vec![target(1, &[])], vec![]), Some(id(1)));
}

#[test]
fn tab_wraps_and_reverse_tab_skips_roving_non_tab_stops() {
    let mut model = FocusModel::default();
    let mut inactive = target(3, &[10]);
    inactive.tab_stop = false;
    model.synchronize(
        vec![target(2, &[10]), inactive, target(4, &[10])],
        vec![scope(10, true)],
    );
    assert_eq!(
        model.handle_key(id(2), &key(KeyboardKey::Tab)),
        (true, Some(id(4)))
    );
    assert_eq!(
        model.handle_key(id(4), &key(KeyboardKey::Tab)),
        (true, Some(id(2)))
    );
    let mut backwards = key(KeyboardKey::Tab);
    backwards.modifiers.shift = true;
    assert_eq!(model.handle_key(id(2), &backwards), (true, Some(id(4))));
    assert_eq!(
        model.handle_key(id(3), &key(KeyboardKey::Enter)),
        (false, None)
    );
}

#[test]
fn disabled_targets_and_background_keys_cannot_escape_a_modal() {
    let mut model = FocusModel::default();
    model.synchronize(
        vec![target(2, &[10]), target(3, &[10])],
        vec![scope(10, true)],
    );
    model.synchronize(vec![target(3, &[10])], vec![scope(10, true)]);
    assert_eq!(
        model.handle_key(id(2), &key(KeyboardKey::ArrowDown)),
        (true, Some(id(3)))
    );
    assert_eq!(
        model.handle_key(id(99), &key(KeyboardKey::Enter)),
        (true, Some(id(3)))
    );
}

#[test]
fn removed_triggers_are_not_replaced_by_unrelated_new_controls() {
    let mut model = FocusModel::default();
    model.synchronize(vec![target(1, &[])], vec![]);
    model.note_focus(id(1));
    model.synchronize(
        vec![target(1, &[]), target(2, &[10])],
        vec![scope(10, true)],
    );
    assert_eq!(model.synchronize(vec![target(99, &[])], vec![]), None);
}

#[test]
fn selected_items_are_preferred_in_popups_but_modals_use_document_order() {
    for (trap, expected) in [(false, 3), (true, 2)] {
        let mut model = FocusModel::default();
        let mut selected = target(3, &[10]);
        selected.priority = 1;
        assert_eq!(
            model.synchronize(vec![target(2, &[10]), selected], vec![scope(10, trap)]),
            Some(id(expected))
        );
    }
}

#[test]
fn search_navigation_preserves_editing_home_end_and_modifier_keys() {
    let mut model = FocusModel::default();
    let mut input = target(1, &[10]);
    input.text_input = true;
    let mut list = scope(10, false);
    list.auto_focus = false;
    list.navigation = Some(KeyboardNavigation::Vertical);
    assert_eq!(
        model.synchronize(vec![input, target(2, &[10])], vec![list]),
        None
    );
    assert_eq!(
        model.handle_key(id(1), &key(KeyboardKey::Home)),
        (false, None)
    );
    assert_eq!(
        model.handle_key(id(1), &key(KeyboardKey::ArrowDown)),
        (true, Some(id(2)))
    );
    let mut shortcut = key(KeyboardKey::ArrowDown);
    shortcut.modifiers.ctrl = true;
    assert_eq!(model.handle_key(id(1), &shortcut), (false, None));
}

#[test]
fn empty_modal_keeps_focus_on_its_escape_handler() {
    let mut model = FocusModel::default();
    assert_eq!(
        model.synchronize(vec![], vec![scope(10, true)]),
        Some(id(10))
    );
    assert_eq!(
        model.handle_key(id(10), &key(KeyboardKey::Tab)),
        (true, Some(id(10)))
    );
    assert_eq!(
        model.handle_key(id(10), &key(KeyboardKey::Escape)),
        (false, None)
    );
}
