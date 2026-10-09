//! Native leases around the independent focus lifecycle model.

use crate::element_ref::SharedNativeNode;
use crate::focus_model::{self, FocusId, FocusModel};
use dioxus_elements::event::{KeyPayload, KeyboardNavigation};
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Weak;

pub(crate) struct FocusTarget {
    pub id: FocusId,
    pub node: Weak<RefCell<ArkUINode>>,
    pub scopes: Vec<FocusId>,
    pub priority: u8,
    pub text_input: bool,
    pub tab_stop: bool,
}
pub(crate) struct FocusScope {
    pub id: FocusId,
    pub node: Weak<RefCell<ArkUINode>>,
    pub trap: bool,
    pub order: u64,
    pub auto_focus: bool,
    pub navigation: Option<KeyboardNavigation>,
}

#[derive(Default)]
pub(crate) struct FocusRegistry {
    model: FocusModel,
    nodes: FxHashMap<FocusId, Weak<RefCell<ArkUINode>>>,
}
impl FocusRegistry {
    pub fn note_focus(&mut self, id: FocusId) {
        self.model.note_focus(id);
    }
    pub fn synchronize(
        &mut self,
        targets: Vec<FocusTarget>,
        scopes: Vec<FocusScope>,
    ) -> Option<SharedNativeNode> {
        self.nodes.clear();
        let targets = targets
            .into_iter()
            .map(|target| {
                self.nodes.insert(target.id, target.node);
                focus_model::FocusTarget {
                    id: target.id,
                    scopes: target.scopes,
                    priority: target.priority,
                    text_input: target.text_input,
                    tab_stop: target.tab_stop,
                }
            })
            .collect();
        let scopes = scopes
            .into_iter()
            .map(|scope| {
                self.nodes.insert(scope.id, scope.node);
                focus_model::FocusScope {
                    id: scope.id,
                    trap: scope.trap,
                    order: scope.order,
                    auto_focus: scope.auto_focus,
                    navigation: scope.navigation,
                }
            })
            .collect();
        self.model
            .synchronize(targets, scopes)
            .and_then(|id| self.nodes.get(&id).and_then(Weak::upgrade))
    }
    pub fn handle_key(
        &mut self,
        origin: FocusId,
        key: &KeyPayload,
    ) -> (bool, Option<SharedNativeNode>) {
        let (handled, focus) = self.model.handle_key(origin, key);
        (
            handled,
            focus.and_then(|id| self.nodes.get(&id).and_then(Weak::upgrade)),
        )
    }
}
pub(crate) fn request_native_focus(node: &SharedNativeNode) {
    crate::log_arkui_result(
        "request keyboard focus",
        node.borrow()
            .set_attribute(ArkUINodeAttributeType::FocusStatus, 1_i32.into()),
    );
}
