//! Window-metrics and safe-area hooks/components.

use std::ops::{BitOr, BitOrAssign};

use arkit_prelude::*;
use arkit_runtime::{EdgeInsets, SafeAreaPolicy, WindowMetrics, WindowMetricsHandle};

#[derive(Clone, Copy)]
struct WindowMetricsSignal(Signal<WindowMetrics>);

/// Install one reactive metrics signal for the entire Arkit tree.
pub(crate) fn use_window_metrics_provider() -> WindowMetrics {
    let handle = dioxus_core::try_consume_context::<WindowMetricsHandle>();
    let signal = use_signal(|| {
        handle
            .as_ref()
            .map(WindowMetricsHandle::get)
            .unwrap_or_default()
    });
    let _subscription = use_hook(|| {
        let callback_signal = signal;
        handle.clone().map(|handle| {
            handle.subscribe(move |metrics| {
                let mut signal = callback_signal;
                signal.set(metrics);
            })
        })
    });
    use_context_provider(|| WindowMetricsSignal(signal));
    signal()
}

/// Read the current window snapshot.
///
/// The OpenHarmony runtime marks the component tree dirty when this snapshot
/// changes, so callers receive new geometry without subscribing to a second
/// platform-specific event source.
#[track_caller]
pub fn use_window_metrics() -> WindowMetrics {
    if let Some(signal) = dioxus_core::try_consume_context::<WindowMetricsSignal>() {
        return (signal.0)();
    }
    dioxus_core::try_consume_context::<WindowMetricsHandle>()
        .map(|handle| handle.get())
        .unwrap_or_default()
}

/// Read the effective visual safe-area insets in vp.
#[track_caller]
pub fn use_safe_area() -> EdgeInsets {
    use_window_metrics().safe_area
}

/// Read the root safe-area policy selected by the application mount point.
#[track_caller]
pub fn use_safe_area_policy() -> SafeAreaPolicy {
    dioxus_core::try_consume_context::<SafeAreaPolicy>().unwrap_or_default()
}

/// Selected edges consumed by [`SafeArea`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeAreaEdges(u8);

impl SafeAreaEdges {
    pub const NONE: Self = Self(0);
    pub const TOP: Self = Self(1 << 0);
    pub const RIGHT: Self = Self(1 << 1);
    pub const BOTTOM: Self = Self(1 << 2);
    pub const LEFT: Self = Self(1 << 3);
    pub const HORIZONTAL: Self = Self(Self::LEFT.0 | Self::RIGHT.0);
    pub const VERTICAL: Self = Self(Self::TOP.0 | Self::BOTTOM.0);
    pub const ALL: Self = Self(Self::HORIZONTAL.0 | Self::VERTICAL.0);

    pub const fn contains(self, edge: Self) -> bool {
        self.0 & edge.0 == edge.0
    }

    pub fn select(self, insets: EdgeInsets) -> EdgeInsets {
        EdgeInsets {
            top: if self.contains(Self::TOP) {
                insets.top
            } else {
                0.0
            },
            right: if self.contains(Self::RIGHT) {
                insets.right
            } else {
                0.0
            },
            bottom: if self.contains(Self::BOTTOM) {
                insets.bottom
            } else {
                0.0
            },
            left: if self.contains(Self::LEFT) {
                insets.left
            } else {
                0.0
            },
        }
    }
}

impl Default for SafeAreaEdges {
    fn default() -> Self {
        Self::ALL
    }
}

impl BitOr for SafeAreaEdges {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for SafeAreaEdges {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Props, PartialEq)]
pub struct SafeAreaProps {
    #[props(default)]
    pub edges: SafeAreaEdges,
    pub children: Element,
}

/// Full-size content viewport padded by the selected effective safe edges.
#[allow(non_snake_case)]
pub fn SafeArea(props: SafeAreaProps) -> Element {
    let insets = props.edges.select(use_safe_area());
    rsx! {
        stack {
            percent_width: 1.0,
            percent_height: 1.0,
            alignment: 0,
            padding_top: insets.top,
            padding_right: insets.right,
            padding_bottom: insets.bottom,
            padding_left: insets.left,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_selection_only_keeps_requested_sides() {
        let insets = EdgeInsets {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };

        assert_eq!(
            (SafeAreaEdges::TOP | SafeAreaEdges::BOTTOM).select(insets),
            EdgeInsets {
                top: 1.0,
                right: 0.0,
                bottom: 3.0,
                left: 0.0,
            }
        );
    }
}
