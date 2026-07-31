use arkit_animation_core::{AdapterTargetId, TargetLayoutSnapshot, TargetName};
use arkit_arkui::MountedNodeLease;

#[derive(Clone)]
pub struct AnimationTargetBinding {
    pub id: AdapterTargetId,
    pub name: TargetName,
    pub node: MountedNodeLease,
    pub layout: Option<TargetLayoutSnapshot>,
    pub mounted: bool,
    pub version: u64,
    pub visual: TargetVisualState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetVisualState {
    pub translate: [f32; 3],
    pub scale: [f32; 2],
    pub position: [f32; 2],
    pub rotation_degrees: f32,
}

impl Default for TargetVisualState {
    fn default() -> Self {
        Self {
            translate: [0.0; 3],
            scale: [1.0; 2],
            position: [0.0; 2],
            rotation_degrees: 0.0,
        }
    }
}
