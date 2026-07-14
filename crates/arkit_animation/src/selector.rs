use arkit_animation_core::{SourceTarget, TargetName, TargetSetName};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum AnimationSelector {
    Target(TargetName),
    Set(TargetSetName),
}

impl From<AnimationSelector> for SourceTarget {
    fn from(value: AnimationSelector) -> Self {
        match value {
            AnimationSelector::Target(target) => Self::One(target),
            AnimationSelector::Set(set) => Self::Set(set),
        }
    }
}
