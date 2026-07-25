//! Pull-to-refresh and incremental loading components.
//!
//! [`PullToRefresh`] is a controlled wrapper around ArkUI `Refresh`.
//! [`InfiniteScroll`] owns a regular ArkUI `Scroll` and requests the next page
//! at its native reach-end boundary. Virtual List/WaterFlow callers use the
//! same [`use_load_more`] controller and forward their `on_scroll` index data
//! to `LoadMoreController::on_virtual_scroll`.

use crate::i18n::{use_component_i18n, ComponentI18n};
use crate::theme::*;
use arkit_hooks::use_load_more;
use arkit_prelude::*;

pub use arkit_hooks::LoadMoreState;

/// User-visible copy for the incremental-loading footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadMoreLabels {
    pub idle: String,
    pub loading: String,
    pub failed: String,
    pub no_more: String,
    pub retry: String,
}

impl LoadMoreLabels {
    pub fn new(
        idle: impl Into<String>,
        loading: impl Into<String>,
        failed: impl Into<String>,
        no_more: impl Into<String>,
        retry: impl Into<String>,
    ) -> Self {
        Self {
            idle: idle.into(),
            loading: loading.into(),
            failed: failed.into(),
            no_more: no_more.into(),
            retry: retry.into(),
        }
    }

    pub(crate) fn localized(i18n: ComponentI18n) -> Self {
        Self {
            idle: i18n.load_more_idle(),
            loading: i18n.load_more_loading(),
            failed: i18n.load_more_failed(),
            no_more: i18n.load_more_no_more(),
            retry: i18n.load_more_retry(),
        }
    }
}

/// Props for [`PullToRefresh`].
#[derive(Props, Clone, PartialEq)]
pub struct PullToRefreshProps {
    pub children: Element,
    /// Controlled refreshing state. Set this to `true` before starting async
    /// work and back to `false` after replacing the data.
    #[props(default)]
    pub refreshing: bool,
    /// Enables the native pull gesture.
    #[props(default = true)]
    pub enabled: bool,
    /// Distance in vp at which the native refresh indicator settles.
    #[props(default = 64.0)]
    pub refresh_offset: f32,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default = "100%".to_string())]
    pub height: String,
    #[props(default)]
    pub background_color: Option<u32>,
    #[props(default)]
    pub on_refresh: EventHandler<()>,
}

/// A controlled native pull-to-refresh boundary.
#[component]
pub fn PullToRefresh(props: PullToRefreshProps) -> Element {
    let theme = use_theme();
    let on_refresh = props.on_refresh;
    let enabled = props.enabled;
    let refreshing = props.refreshing;

    rsx! {
        refresh {
            width: props.width,
            height: props.height,
            background_color: props.background_color.unwrap_or(theme.colors.background),
            refreshing,
            refresh_offset: props.refresh_offset.max(0.0),
            refresh_pull_to_refresh: enabled,
            onrefresh: move |_| {
                if enabled && !refreshing {
                    on_refresh.call(());
                }
            },
            {props.children}
        }
    }
}

/// Props for [`LoadMoreIndicator`].
#[derive(Props, Clone, PartialEq)]
pub struct LoadMoreIndicatorProps {
    pub state: LoadMoreState,
    /// Overrides the active component locale.
    #[props(default)]
    pub labels: Option<LoadMoreLabels>,
    /// Whether the idle hint reserves a footer row.
    #[props(default = true)]
    pub show_idle: bool,
    #[props(default)]
    pub on_retry: EventHandler<()>,
}

/// Compact status footer shared by regular and virtual scroll containers.
#[component]
pub fn LoadMoreIndicator(props: LoadMoreIndicatorProps) -> Element {
    let theme = use_theme();
    let labels = props
        .labels
        .unwrap_or_else(|| LoadMoreLabels::localized(use_component_i18n()));
    let show = props.state != LoadMoreState::Idle || props.show_idle;
    if !show {
        return rsx! {};
    }

    let (message, color) = match props.state {
        LoadMoreState::Idle => (labels.idle, theme.colors.muted_foreground),
        LoadMoreState::Loading => (labels.loading, theme.colors.muted_foreground),
        LoadMoreState::Failed => (
            format!("{} · {}", labels.failed, labels.retry),
            theme.colors.destructive,
        ),
        LoadMoreState::NoMore => (labels.no_more, theme.colors.muted_foreground),
    };
    let on_retry = props.on_retry;
    let can_retry = props.state == LoadMoreState::Failed;

    rsx! {
        row {
            width: "100%",
            height: 48.0,
            align_items: "center",
            justify_content: "center",
            hit_test_behavior: if can_retry { "default" } else { "transparent" },
            onclick: move |_| {
                if can_retry {
                    on_retry.call(());
                }
            },
            if props.state == LoadMoreState::Loading {
                loadingprogress {
                    width: 16.0,
                    height: 16.0,
                    margin_right: spacing::SM,
                    loading_progress_color: color,
                    loading_progress_enable_loading: true,
                    hit_test_behavior: "transparent",
                }
            }
            text {
                font_size: typography::SM,
                font_color: color,
                max_lines: 1_i32,
                text_overflow: "ellipsis",
                "{message}"
            }
        }
    }
}

/// Props for [`InfiniteScroll`].
#[derive(Props, Clone, PartialEq)]
pub struct InfiniteScrollProps {
    pub children: Element,
    /// Number of data items currently rendered. It identifies one request
    /// generation and prevents duplicate reach-end callbacks for that page.
    pub item_count: u32,
    /// Increment after replacing the data without changing `item_count` (for
    /// example, a same-size refresh) to re-arm reach-end loading.
    #[props(default)]
    pub data_revision: u64,
    pub state: LoadMoreState,
    #[props(default)]
    pub labels: Option<LoadMoreLabels>,
    #[props(default = true)]
    pub show_idle: bool,
    /// Enables automatic load-more requests without disabling scrolling.
    #[props(default = true)]
    pub enabled: bool,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default = "100%".to_string())]
    pub height: String,
    #[props(default = "auto".to_string())]
    pub scroll_bar: String,
    #[props(default)]
    pub background_color: Option<u32>,
    #[props(default)]
    pub on_load_more: EventHandler<()>,
}

/// A regular scroll container with native reach-end loading and a state footer.
#[component]
pub fn InfiniteScroll(props: InfiniteScrollProps) -> Element {
    let theme = use_theme();
    let controller = use_load_more(props.item_count, props.state, 0, props.on_load_more);
    let reset_controller = controller.clone();
    use_effect(use_reactive((&props.data_revision,), move |(_revision,)| {
        reset_controller.reset()
    }));
    let reach_controller = controller.clone();
    let retry_controller = controller;
    let enabled = props.enabled;

    rsx! {
        scroll {
            width: props.width,
            height: props.height,
            background_color: props.background_color.unwrap_or(theme.colors.background),
            scroll_bar: props.scroll_bar,
            scroll_enabled: true,
            onreachend: move |_| {
                if enabled {
                    reach_controller.reach_end();
                }
            },
            column {
                width: "100%",
                {props.children}
                LoadMoreIndicator {
                    state: props.state,
                    labels: props.labels,
                    show_idle: props.show_idle,
                    on_retry: move |_| retry_controller.retry(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_accept_a_complete_external_locale_snapshot() {
        let labels = LoadMoreLabels::new("ready", "busy", "failed", "done", "again");
        assert_eq!(labels.idle, "ready");
        assert_eq!(labels.loading, "busy");
        assert_eq!(labels.failed, "failed");
        assert_eq!(labels.no_more, "done");
        assert_eq!(labels.retry, "again");
    }
}
