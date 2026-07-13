use arkit_animation_core::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, PropertyDescriptor,
    PropertyName, PropertyUpdate, ResolutionTarget, ResolvedProperty, SourceTarget, TargetContext,
    ValueFunctionName,
};

use crate::AnimationAdapterError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetLifecycle {
    pub version: u64,
    pub mounted: bool,
}

pub trait TargetAdapter {
    fn id(&self) -> AdapterId;

    fn diagnostic_name(&self) -> &str;

    fn target_lifecycle(&self, target: AdapterTargetId) -> Option<TargetLifecycle>;

    fn property_descriptor(&self, property: AdapterPropertyId) -> Option<&PropertyDescriptor>;

    fn resolve_targets(
        &self,
        target: &SourceTarget,
    ) -> Result<Vec<ResolutionTarget>, AnimationAdapterError>;

    fn resolve_property(
        &self,
        target: AdapterTargetId,
        property: &PropertyName,
    ) -> Result<ResolvedProperty, AnimationAdapterError>;

    fn read_baseline(
        &self,
        target: AdapterTargetId,
        property: AdapterPropertyId,
    ) -> Result<AnimationValue, AnimationAdapterError>;

    fn resolve_value(
        &self,
        _target: AdapterTargetId,
        _property: AdapterPropertyId,
        value: &AnimationValue,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        Ok(value.clone())
    }

    fn resolve_relative(
        &self,
        _target: AdapterTargetId,
        _property: AdapterPropertyId,
        baseline: &AnimationValue,
        delta: &AnimationValue,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        baseline.compose_add(delta).map_err(Into::into)
    }

    fn resolve_function(
        &self,
        function: &ValueFunctionName,
        _target: AdapterTargetId,
        _property: AdapterPropertyId,
        _context: TargetContext<'_>,
    ) -> Result<AnimationValue, AnimationAdapterError> {
        Err(AnimationAdapterError::UnknownProperty(PropertyName::owned(
            function.as_str(),
        )))
    }

    fn apply(&self, update: &PropertyUpdate) -> Result<(), AnimationAdapterError>;

    fn validate_update(&self, update: &PropertyUpdate) -> Result<(), AnimationAdapterError> {
        let lifecycle = self
            .target_lifecycle(update.target)
            .ok_or(AnimationAdapterError::UnknownTargetId(update.target))?;
        if !lifecycle.mounted {
            return Err(AnimationAdapterError::DisposedTarget(update.target));
        }
        self.property_descriptor(update.property)
            .ok_or(AnimationAdapterError::UnknownPropertyId(update.property))?
            .validate_value(&update.value)?;
        Ok(())
    }

    fn apply_batch(&self, updates: &[PropertyUpdate]) -> Result<(), AnimationAdapterError> {
        for update in updates {
            self.apply(update)?;
        }
        Ok(())
    }
}
