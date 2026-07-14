//! Symbolic animation source types produced before adapter resolution.

use std::sync::Arc;

use crate::{PropertyName, SymbolName, TimelinePosition, TweenSpec};

macro_rules! source_name {
    ($name:ident) => {
        #[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(SymbolName);

        impl $name {
            pub const fn static_name(value: &'static str) -> Self {
                Self(SymbolName::static_name(value))
            }

            pub fn owned(value: impl Into<Arc<str>>) -> Self {
                Self(SymbolName::owned(value))
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

source_name!(TargetName);
source_name!(TargetSetName);
source_name!(LabelName);
source_name!(LayoutId);
source_name!(ScopeMethodName);
source_name!(ValueFunctionName);

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SourceTarget {
    One(TargetName),
    Set(TargetSetName),
}

#[derive(Debug, Clone)]
pub struct SourceAnimation {
    pub target: SourceTarget,
    pub tweens: Vec<TweenSpec>,
}

impl SourceAnimation {
    pub fn new(target: SourceTarget) -> Self {
        Self {
            target,
            tweens: Vec::new(),
        }
    }

    pub fn push(&mut self, tween: TweenSpec) {
        self.tweens.push(tween);
    }
}

#[derive(Debug, Clone)]
pub struct SourceSet {
    pub target: SourceTarget,
    pub property: PropertyName,
    pub value: crate::AnimationValue,
    pub position: TimelinePosition,
}
