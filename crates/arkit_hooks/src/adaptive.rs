//! Window-width driven Phone/PC layout selection.
//!
//! The effective style follows the current application window, not the
//! physical device category. This is required on PC/2-in-1 where a freely
//! resizable application window can cross responsive breakpoints at runtime.

use arkit_prelude::*;
use arkit_runtime::WindowMetrics;

/// HarmonyOS large-window breakpoint in virtual pixels.
///
/// Windows at or above this width use the PC component style by default.
pub const DEFAULT_PC_MIN_WIDTH: f32 = 840.0;

/// Requested component style policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdaptiveMode {
    /// Resolve from the current application-window width.
    #[default]
    Auto,
    /// Always use touch-oriented Phone layouts.
    Phone,
    /// Always use pointer-oriented PC layouts.
    Pc,
}

/// Effective component style after resolving [`AdaptiveMode::Auto`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdaptiveStyle {
    #[default]
    Phone,
    Pc,
}

/// Configuration inherited by adaptive components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveConfig {
    pub mode: AdaptiveMode,
    /// Minimum application-window width that resolves Auto to PC style.
    pub pc_min_width: f32,
}

impl AdaptiveConfig {
    pub const fn new(mode: AdaptiveMode) -> Self {
        Self {
            mode,
            pc_min_width: DEFAULT_PC_MIN_WIDTH,
        }
    }

    pub const fn with_pc_min_width(mut self, width: f32) -> Self {
        self.pc_min_width = width;
        self
    }

    fn effective_pc_min_width(self) -> f32 {
        if self.pc_min_width.is_finite() && self.pc_min_width > 0.0 {
            self.pc_min_width
        } else {
            DEFAULT_PC_MIN_WIDTH
        }
    }

    pub fn resolve(self, width_vp: f32) -> AdaptiveStyle {
        match self.mode {
            AdaptiveMode::Phone => AdaptiveStyle::Phone,
            AdaptiveMode::Pc => AdaptiveStyle::Pc,
            AdaptiveMode::Auto => {
                if width_vp.is_finite() && width_vp >= self.effective_pc_min_width() {
                    AdaptiveStyle::Pc
                } else {
                    AdaptiveStyle::Phone
                }
            }
        }
    }
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self::new(AdaptiveMode::Auto)
    }
}

/// Resolved responsive state for the current application window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveLayout {
    pub mode: AdaptiveMode,
    pub style: AdaptiveStyle,
    pub width_vp: f32,
    pub height_vp: f32,
    pub pc_min_width: f32,
}

impl AdaptiveLayout {
    pub fn resolve(config: AdaptiveConfig, metrics: WindowMetrics) -> Self {
        let (width_vp, height_vp) = metrics.content_size_vp();
        Self {
            mode: config.mode,
            style: config.resolve(width_vp),
            width_vp,
            height_vp,
            pc_min_width: config.effective_pc_min_width(),
        }
    }

    pub const fn is_phone(self) -> bool {
        matches!(self.style, AdaptiveStyle::Phone)
    }

    pub const fn is_pc(self) -> bool {
        matches!(self.style, AdaptiveStyle::Pc)
    }

    /// Select a value from the resolved style without repeating a match.
    pub fn select<T>(self, phone: T, pc: T) -> T {
        match self.style {
            AdaptiveStyle::Phone => phone,
            AdaptiveStyle::Pc => pc,
        }
    }
}

#[derive(Clone, Copy)]
struct AdaptiveConfigSignal(Signal<AdaptiveConfig>);

/// Provide a mutable adaptive configuration to the current Dioxus subtree.
///
/// Prefer [`AdaptiveProvider`] for prop-controlled configuration. This hook is
/// useful when an application settings surface owns the selected mode.
pub fn use_adaptive_config_provider(initial: AdaptiveConfig) -> Signal<AdaptiveConfig> {
    let signal = use_signal(|| initial);
    use_context_provider(|| AdaptiveConfigSignal(signal));
    signal
}

/// Resolve the active Phone/PC style reactively from window metrics.
///
/// Without a provider this uses [`AdaptiveConfig::default`], so every Arkit
/// component remains responsive by default.
#[track_caller]
pub fn use_adaptive_layout() -> AdaptiveLayout {
    let config = dioxus_core::try_consume_context::<AdaptiveConfigSignal>()
        .map(|signal| (signal.0)())
        .unwrap_or_default();
    AdaptiveLayout::resolve(config, crate::use_window_metrics())
}

/// Inherit one adaptive policy through a component subtree.
#[component]
pub fn AdaptiveProvider(config: AdaptiveConfig, children: Element) -> Element {
    let mut provided = use_adaptive_config_provider(config);
    use_effect(use_reactive((&config,), move |(config,)| {
        if *provided.peek() != config {
            provided.set(config);
        }
    }));
    rsx! { {children} }
}

/// Render one of two component trees from the effective Phone/PC style.
///
/// This is the escape hatch for structural changes such as bottom navigation
/// becoming a sidebar. Nested [`AdaptiveProvider`] values can force either
/// branch for previews and tests.
#[component]
pub fn AdaptiveView(phone: Element, pc: Element) -> Element {
    if use_adaptive_layout().is_pc() {
        pc
    } else {
        phone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkit_runtime::PhysicalRect;

    fn metrics(width_vp: f32) -> WindowMetrics {
        let mut metrics = WindowMetrics::default();
        metrics.content_rect = PhysicalRect {
            left: 0,
            top: 0,
            width: (width_vp * 2.0) as i32,
            height: 1200,
        };
        metrics.scale = 2.0;
        metrics
    }

    #[test]
    fn auto_switches_at_harmony_large_window_breakpoint() {
        let config = AdaptiveConfig::default();
        assert_eq!(
            AdaptiveLayout::resolve(config, metrics(839.5)).style,
            AdaptiveStyle::Phone
        );
        assert_eq!(
            AdaptiveLayout::resolve(config, metrics(840.0)).style,
            AdaptiveStyle::Pc
        );
    }

    #[test]
    fn explicit_modes_override_window_width() {
        assert_eq!(
            AdaptiveLayout::resolve(AdaptiveConfig::new(AdaptiveMode::Phone), metrics(1440.0),)
                .style,
            AdaptiveStyle::Phone
        );
        assert_eq!(
            AdaptiveLayout::resolve(AdaptiveConfig::new(AdaptiveMode::Pc), metrics(360.0)).style,
            AdaptiveStyle::Pc
        );
    }

    #[test]
    fn custom_and_invalid_breakpoints_are_normalized() {
        let custom = AdaptiveConfig::default().with_pc_min_width(720.0);
        assert_eq!(custom.resolve(720.0), AdaptiveStyle::Pc);

        let invalid = AdaptiveConfig::default().with_pc_min_width(f32::NAN);
        assert_eq!(invalid.resolve(839.0), AdaptiveStyle::Phone);
        assert_eq!(invalid.resolve(840.0), AdaptiveStyle::Pc);
    }
}
