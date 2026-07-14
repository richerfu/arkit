use arkit_animation_core::{
    AnimationResolveError, AnimationValue, PropertyName, ResolutionContext, ResolutionTarget,
    ResolvedProperty, ResolvedTarget, SourceTarget, TargetContext, ValueFunctionName,
    WindowMetrics,
};

use crate::AdapterRegistry;

pub struct AdapterResolutionSnapshot<'registry> {
    registry: &'registry AdapterRegistry,
    window_metrics: WindowMetrics,
}

impl<'registry> AdapterResolutionSnapshot<'registry> {
    pub const fn new(registry: &'registry AdapterRegistry, window_metrics: WindowMetrics) -> Self {
        Self {
            registry,
            window_metrics,
        }
    }

    fn adapter(
        &self,
        target: &ResolvedTarget,
    ) -> Result<&dyn crate::TargetAdapter, AnimationResolveError> {
        self.registry
            .get(target.adapter)
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }
}

impl ResolutionContext for AdapterResolutionSnapshot<'_> {
    fn resolve_targets(
        &self,
        target: &SourceTarget,
    ) -> Result<Box<[ResolutionTarget]>, AnimationResolveError> {
        let mut resolved = Vec::new();
        for adapter in self.registry.iter() {
            match adapter.resolve_targets(target) {
                Ok(mut targets) => resolved.append(&mut targets),
                Err(crate::AnimationAdapterError::UnknownTarget(_)) => {}
                Err(error) => return Err(AnimationResolveError::context(error.to_string())),
            }
        }
        if resolved.is_empty() {
            return Err(AnimationResolveError::EmptyTargetSelection);
        }
        Ok(resolved.into_boxed_slice())
    }

    fn resolve_property(
        &self,
        target: &ResolvedTarget,
        property: &PropertyName,
    ) -> Result<ResolvedProperty, AnimationResolveError> {
        self.adapter(target)?
            .resolve_property(target.adapter_target, property)
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }

    fn read_baseline(
        &self,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
    ) -> Result<AnimationValue, AnimationResolveError> {
        self.adapter(target)?
            .read_baseline(target.adapter_target, property.adapter_property)
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }

    fn resolve_value(
        &self,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
        value: &AnimationValue,
    ) -> Result<AnimationValue, AnimationResolveError> {
        self.adapter(target)?
            .resolve_value(target.adapter_target, property.adapter_property, value)
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }

    fn resolve_relative(
        &self,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
        baseline: &AnimationValue,
        delta: &AnimationValue,
    ) -> Result<AnimationValue, AnimationResolveError> {
        self.adapter(target)?
            .resolve_relative(
                target.adapter_target,
                property.adapter_property,
                baseline,
                delta,
            )
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }

    fn resolve_function(
        &self,
        function: &ValueFunctionName,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
        context: TargetContext<'_>,
    ) -> Result<AnimationValue, AnimationResolveError> {
        self.adapter(target)?
            .resolve_function(
                function,
                target.adapter_target,
                property.adapter_property,
                context,
            )
            .map_err(|error| AnimationResolveError::context(error.to_string()))
    }

    fn window_metrics(&self) -> WindowMetrics {
        self.window_metrics
    }
}
