use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_animation_core::{
    Easing, LayoutId, LayoutNodeId, Length, TargetName, TimeSpan, TimelinePosition, TransformValue,
    Vec2, Vec3, WindowMetrics,
};
use arkit_hooks::LayoutFrame;
use arkit_prelude::*;
use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::properties::{SCALE_X, SCALE_Y, TRANSLATE_X, TRANSLATE_Y};
use crate::{Animation, AnimationSelector, Timeline};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutMountState {
    Mounted,
    Entering,
    Leaving,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutNode {
    pub id: LayoutId,
    pub parent: Option<LayoutNodeId>,
    pub frame: LayoutFrame,
    pub transform: TransformValue,
    pub visible: bool,
    pub clip: Option<LayoutFrame>,
    pub z_order: i32,
    pub mount_state: LayoutMountState,
}

#[derive(Debug, Clone)]
pub struct LayoutSnapshot {
    pub root: Option<LayoutNodeId>,
    pub nodes: IndexVec<LayoutNodeId, LayoutNode>,
    pub window_metrics: WindowMetrics,
    pub scroll_offset: Vec2,
    pub generation: u64,
}

impl LayoutSnapshot {
    pub fn new(window_metrics: WindowMetrics, generation: u64) -> Self {
        Self {
            root: None,
            nodes: IndexVec::new(),
            window_metrics,
            scroll_offset: Vec2::default(),
            generation,
        }
    }

    pub fn push(&mut self, node: LayoutNode) -> LayoutNodeId {
        let id = self.nodes.push(node);
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    fn by_layout_id(&self) -> FxHashMap<LayoutId, LayoutNodeId> {
        self.nodes
            .iter_enumerated()
            .map(|(id, node)| (node.id.clone(), id))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutChangeKind {
    Enter,
    Exit,
    Move,
    Resize,
    Reparent,
    Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutAnimationMode {
    Position,
    Size,
    PositionAndSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDelta {
    pub id: LayoutId,
    pub old: Option<LayoutNode>,
    pub new: Option<LayoutNode>,
    pub kind: LayoutChangeKind,
    pub inverse: TransformValue,
}

impl LayoutDelta {
    pub fn timeline(
        &self,
        target: TargetName,
        mode: LayoutAnimationMode,
        duration: TimeSpan,
        easing: Easing,
    ) -> Option<Timeline> {
        let new = self.new.as_ref()?;
        let mut inverse = self.inverse.clone();
        match mode {
            LayoutAnimationMode::Position => inverse.scale = Vec3::new(1.0, 1.0, 1.0),
            LayoutAnimationMode::Size => inverse.translation = [Length::vp(0.0); 3],
            LayoutAnimationMode::PositionAndSize => {}
        }
        let animation = Animation::new(AnimationSelector::Target(target))
            .tween(
                &TRANSLATE_X,
                inverse.translation[0],
                new.transform.translation[0],
                duration,
            )
            .configure_last(
                easing.clone(),
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            )
            .tween(
                &TRANSLATE_Y,
                inverse.translation[1],
                new.transform.translation[1],
                duration,
            )
            .configure_last(
                easing.clone(),
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            )
            .tween(&SCALE_X, inverse.scale.x, new.transform.scale.x, duration)
            .configure_last(
                easing.clone(),
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            )
            .tween(&SCALE_Y, inverse.scale.y, new.transform.scale.y, duration)
            .configure_last(
                easing,
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            );
        Some(Timeline::new().add(animation, TimelinePosition::START))
    }
}

#[derive(Debug, Clone)]
pub struct LayoutAnimation {
    pub mode: LayoutAnimationMode,
    pub delta: LayoutDelta,
}

impl LayoutAnimation {
    pub fn flip(&self) -> &TransformValue {
        &self.delta.inverse
    }
}

#[derive(Default)]
pub struct LayoutEngine {
    old: Option<LayoutSnapshot>,
}

impl LayoutEngine {
    pub fn record_old(&mut self, snapshot: LayoutSnapshot) {
        self.old = Some(snapshot);
    }

    pub fn record_new(&mut self, snapshot: LayoutSnapshot) -> Vec<LayoutDelta> {
        let Some(old) = self.old.replace(snapshot.clone()) else {
            return snapshot
                .nodes
                .iter()
                .map(|node| LayoutDelta {
                    id: node.id.clone(),
                    old: None,
                    new: Some(node.clone()),
                    kind: LayoutChangeKind::Enter,
                    inverse: TransformValue::default(),
                })
                .collect();
        };
        compute_deltas(&old, &snapshot)
    }

    pub fn clear(&mut self) {
        self.old = None;
    }
}

pub struct SharedElementProjection {
    id: LayoutId,
    cleanup: Option<Rc<dyn Fn()>>,
}

#[derive(Clone)]
struct LayoutRegistryContext {
    nodes: Rc<RefCell<FxHashMap<LayoutId, RegisteredLayoutNode>>>,
    generation: Rc<Cell<u64>>,
}

#[derive(Debug, Clone)]
struct RegisteredLayoutNode {
    parent: Option<LayoutId>,
    frame: LayoutFrame,
    visible: bool,
    z_order: i32,
}

impl LayoutRegistryContext {
    fn snapshot(&self, metrics: WindowMetrics) -> LayoutSnapshot {
        let nodes = self.nodes.borrow();
        let mut ordered = nodes.iter().collect::<Vec<_>>();
        ordered.sort_by(|(left, _), (right, _)| left.cmp(right));
        let ids = ordered
            .iter()
            .enumerate()
            .map(|(index, (id, _))| ((*id).clone(), LayoutNodeId::new(index)))
            .collect::<FxHashMap<_, _>>();
        let mut snapshot = LayoutSnapshot::new(metrics, self.generation.get());
        for (id, node) in ordered {
            snapshot.push(LayoutNode {
                id: id.clone(),
                parent: node
                    .parent
                    .as_ref()
                    .and_then(|parent| ids.get(parent).copied()),
                frame: node.frame,
                transform: TransformValue::default(),
                visible: node.visible,
                clip: None,
                z_order: node.z_order,
                mount_state: LayoutMountState::Mounted,
            });
        }
        snapshot
    }

    fn bump_generation(&self) {
        self.generation.set(
            self.generation
                .get()
                .checked_add(1)
                .expect("layout generation exhausted"),
        );
    }
}

#[track_caller]
pub(crate) fn use_layout_registry_provider() {
    use_context_provider(|| LayoutRegistryContext {
        nodes: Rc::new(RefCell::new(FxHashMap::default())),
        generation: Rc::new(Cell::new(0)),
    });
}

#[track_caller]
pub fn use_animation_layout(
    reference: arkit_arkui::NativeElementRef,
    id: LayoutId,
    parent: Option<LayoutId>,
    visible: bool,
    z_order: i32,
) {
    let registry = use_context::<LayoutRegistryContext>();
    let layout_id = use_hook(|| id);
    let observed_registry = registry.clone();
    let observed_id = layout_id.clone();
    arkit_hooks::use_layout_frame(reference.clone(), move |frame| {
        observed_registry.nodes.borrow_mut().insert(
            observed_id.clone(),
            RegisteredLayoutNode {
                parent: parent.clone(),
                frame,
                visible,
                z_order,
            },
        );
        observed_registry.bump_generation();
    });
    let drop_registry = registry;
    use_drop(move || {
        if drop_registry
            .nodes
            .borrow_mut()
            .remove(&layout_id)
            .is_some()
        {
            drop_registry.bump_generation();
        }
    });
}

#[track_caller]
pub fn use_layout_snapshot(metrics: WindowMetrics) -> LayoutSnapshot {
    use_context::<LayoutRegistryContext>().snapshot(metrics)
}

impl SharedElementProjection {
    pub fn new(id: LayoutId, cleanup: impl Fn() + 'static) -> Self {
        Self {
            id,
            cleanup: Some(Rc::new(cleanup)),
        }
    }

    pub fn id(&self) -> &LayoutId {
        &self.id
    }

    pub fn settle(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

impl Drop for SharedElementProjection {
    fn drop(&mut self) {
        self.settle();
    }
}

fn compute_deltas(old: &LayoutSnapshot, new: &LayoutSnapshot) -> Vec<LayoutDelta> {
    let old_ids = old.by_layout_id();
    let new_ids = new.by_layout_id();
    let mut deltas = Vec::new();
    for (new_id, new_node) in new.nodes.iter_enumerated() {
        let Some(old_id) = old_ids.get(&new_node.id).copied() else {
            deltas.push(LayoutDelta {
                id: new_node.id.clone(),
                old: None,
                new: Some(new_node.clone()),
                kind: LayoutChangeKind::Enter,
                inverse: TransformValue::default(),
            });
            continue;
        };
        let old_node = &old.nodes[old_id];
        let old_parent = old_node.parent.map(|id| old.nodes[id].id.clone());
        let new_parent = new_node.parent.map(|id| new.nodes[id].id.clone());
        let moved = old_node.frame.x != new_node.frame.x || old_node.frame.y != new_node.frame.y;
        let resized = old_node.frame.width != new_node.frame.width
            || old_node.frame.height != new_node.frame.height;
        let kind = if old_parent != new_parent {
            Some(LayoutChangeKind::Reparent)
        } else if old_node.visible != new_node.visible {
            Some(LayoutChangeKind::Visibility)
        } else if resized {
            Some(LayoutChangeKind::Resize)
        } else if moved {
            Some(LayoutChangeKind::Move)
        } else {
            None
        };
        if let Some(kind) = kind {
            deltas.push(LayoutDelta {
                id: new_node.id.clone(),
                old: Some(old_node.clone()),
                new: Some(new_node.clone()),
                kind,
                inverse: inverse_transform(old_node.frame, new_node.frame),
            });
        }
        let _ = new_id;
    }
    for old_node in old.nodes.iter() {
        if !new_ids.contains_key(&old_node.id) {
            deltas.push(LayoutDelta {
                id: old_node.id.clone(),
                old: Some(old_node.clone()),
                new: None,
                kind: LayoutChangeKind::Exit,
                inverse: TransformValue::default(),
            });
        }
    }
    deltas
}

fn inverse_transform(old: LayoutFrame, new: LayoutFrame) -> TransformValue {
    let scale_x = if new.width.abs() <= f32::EPSILON {
        1.0
    } else {
        old.width / new.width
    };
    let scale_y = if new.height.abs() <= f32::EPSILON {
        1.0
    } else {
        old.height / new.height
    };
    TransformValue {
        translation: [
            Length::vp(old.x - new.x),
            Length::vp(old.y - new.y),
            Length::vp(0.0),
        ],
        scale: Vec3::new(scale_x, scale_y, 1.0),
        ..TransformValue::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &'static str, parent: Option<LayoutNodeId>, frame: LayoutFrame) -> LayoutNode {
        LayoutNode {
            id: LayoutId::owned(id),
            parent,
            frame,
            transform: TransformValue::default(),
            visible: true,
            clip: None,
            z_order: 0,
            mount_state: LayoutMountState::Mounted,
        }
    }

    #[test]
    fn detects_reorder_resize_enter_exit_and_reparent() {
        let mut old = LayoutSnapshot::new(WindowMetrics::default(), 1);
        let root = old.push(node("root", None, LayoutFrame::default()));
        old.push(node(
            "moving",
            Some(root),
            LayoutFrame {
                x: 10.0,
                y: 20.0,
                width: 50.0,
                height: 40.0,
            },
        ));
        old.push(node("leaving", Some(root), LayoutFrame::default()));
        let mut new = LayoutSnapshot::new(WindowMetrics::default(), 2);
        let new_root = new.push(node("root", None, LayoutFrame::default()));
        let parent = new.push(node("parent", Some(new_root), LayoutFrame::default()));
        new.push(node(
            "moving",
            Some(parent),
            LayoutFrame {
                x: 30.0,
                y: 10.0,
                width: 100.0,
                height: 20.0,
            },
        ));
        new.push(node("entering", Some(new_root), LayoutFrame::default()));
        let deltas = compute_deltas(&old, &new);
        assert!(deltas
            .iter()
            .any(|delta| delta.kind == LayoutChangeKind::Reparent));
        assert!(deltas
            .iter()
            .any(|delta| delta.kind == LayoutChangeKind::Enter));
        assert!(deltas
            .iter()
            .any(|delta| delta.kind == LayoutChangeKind::Exit));
    }
}
