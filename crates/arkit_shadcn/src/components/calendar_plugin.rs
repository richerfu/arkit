//! Composable presentation and interaction plugins for the month calendar.
//!
//! The plugin contract is intentionally synchronous. Owners preload external
//! data when the visible month changes, then callbacks perform immutable date
//! lookups while the 42 day cells render. Calendar layout, hit testing, and
//! navigation remain core-owned.

use arkit_prelude::*;

use super::calendar::CalendarDate;

pub(super) const PLUGIN_DAY_SIZE: f32 = 40.0;
pub(super) const PLUGIN_WEEK_ROW_HEIGHT: f32 = 48.0;

/// Stable input supplied to a calendar day plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDayContext {
    pub date: CalendarDate,
    pub selected: bool,
    pub today: bool,
    pub outside_month: bool,
    /// Whether the core range and all earlier plugins allow interaction.
    pub enabled: bool,
    /// Color resolved for the primary Gregorian day number.
    pub primary_color: u32,
    /// Background resolved for the day surface.
    pub background_color: u32,
    /// Color resolved by the core calendar for supporting content.
    pub supporting_color: u32,
}

/// Visual overrides contributed by a calendar plugin.
///
/// Layout and hit testing remain core-owned so independently authored plugins
/// cannot move a day outside its grid cell or shadow another day. Optional
/// fields compose in declaration order; the last non-`None` value wins.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CalendarDayStyle {
    pub primary_color: Option<u32>,
    pub background_color: Option<u32>,
    pub border_color: Option<u32>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub opacity: Option<f32>,
    pub primary_font_weight: Option<i32>,
}

impl CalendarDayStyle {
    pub(super) fn merge(&mut self, next: Self) {
        macro_rules! replace_if_some {
            ($field:ident) => {
                if next.$field.is_some() {
                    self.$field = next.$field;
                }
            };
        }

        replace_if_some!(primary_color);
        replace_if_some!(background_color);
        replace_if_some!(border_color);
        replace_if_some!(border_width);
        replace_if_some!(border_radius);
        replace_if_some!(opacity);
        replace_if_some!(primary_font_weight);
    }
}

/// One plugin's contribution to a rendered day.
///
/// Supporting and overlay nodes are additive across plugins. Replacement
/// content is exclusive and follows last-plugin-wins semantics. `disabled` is
/// monotonic: once any plugin disables a day, a later plugin cannot re-enable
/// it. Interactive business behavior belongs in [`CalendarPlugin::with_day_event`]
/// instead of nested buttons inside these visual slots.
pub struct CalendarDayDecoration {
    pub(super) style: CalendarDayStyle,
    pub(super) supporting: Option<Element>,
    pub(super) overlay: Option<Element>,
    pub(super) replacement: Option<Element>,
    pub(super) disabled: bool,
}

impl CalendarDayDecoration {
    pub const fn new() -> Self {
        Self {
            style: CalendarDayStyle {
                primary_color: None,
                background_color: None,
                border_color: None,
                border_width: None,
                border_radius: None,
                opacity: None,
                primary_font_weight: None,
            },
            supporting: None,
            overlay: None,
            replacement: None,
            disabled: false,
        }
    }

    pub const fn with_style(mut self, style: CalendarDayStyle) -> Self {
        self.style = style;
        self
    }

    /// Append content to the compact row below the primary day number.
    pub fn with_supporting(mut self, content: Element) -> Self {
        self.supporting = Some(content);
        self
    }

    /// Append non-interactive content over the complete day surface.
    pub fn with_overlay(mut self, content: Element) -> Self {
        self.overlay = Some(content);
        self
    }

    /// Replace the core day number and supporting-content column.
    ///
    /// The outer button, selection callback, long-press dispatch, disabled
    /// state, and grid sizing remain core-owned.
    pub fn with_replacement(mut self, content: Element) -> Self {
        self.replacement = Some(content);
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Default for CalendarDayDecoration {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable input supplied to month-level calendar plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarMonthContext {
    pub year: i32,
    pub month: u8,
    pub first_visible_date: CalendarDate,
    pub last_visible_date: CalendarDate,
    /// Title resolved by the core and every earlier plugin.
    pub title: String,
}

/// One plugin's contribution to the visible month header.
///
/// A title replaces the previously resolved title. Supporting nodes append
/// below the navigation row in plugin declaration order.
pub struct CalendarMonthDecoration {
    pub(super) title: Option<String>,
    pub(super) supporting: Option<Element>,
}

impl CalendarMonthDecoration {
    pub const fn new() -> Self {
        Self {
            title: None,
            supporting: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_supporting(mut self, content: Element) -> Self {
        self.supporting = Some(content);
        self
    }
}

impl Default for CalendarMonthDecoration {
    fn default() -> Self {
        Self::new()
    }
}

/// User interaction dispatched to each plugin for a day cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDayEvent {
    pub context: CalendarDayContext,
    pub kind: CalendarDayEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarDayEventKind {
    Press,
    LongPress,
}

/// Result returned by one plugin event handler.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDayEventResponse {
    pub prevent_default: bool,
}

impl CalendarDayEventResponse {
    pub const fn continue_default() -> Self {
        Self {
            prevent_default: false,
        }
    }

    pub const fn prevent_default() -> Self {
        Self {
            prevent_default: true,
        }
    }
}

/// Static footprint requested by a plugin for every day in the month grid.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CalendarPluginLayout {
    pub minimum_day_size: Option<f32>,
    pub minimum_week_row_height: Option<f32>,
}

impl CalendarPluginLayout {
    pub const fn supporting_content() -> Self {
        Self {
            minimum_day_size: Some(PLUGIN_DAY_SIZE),
            minimum_week_row_height: Some(PLUGIN_WEEK_ROW_HEIGHT),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CalendarDayRenderer {
    Supporting(dioxus_core::Callback<CalendarDayContext, Element>),
    Decoration(dioxus_core::Callback<CalendarDayContext, CalendarDayDecoration>),
}

/// Composable extension point for calendar presentation and interaction.
///
/// Plugins are synchronous render-time adapters. They must consume data that
/// the owner has already loaded; filesystem, database, or network work is
/// forbidden inside render callbacks. Use the calendar's month-change event to
/// preload business data, then capture an immutable lookup snapshot here.
#[derive(Clone, Copy, PartialEq)]
pub struct CalendarPlugin {
    day_renderer: Option<CalendarDayRenderer>,
    month_renderer: Option<dioxus_core::Callback<CalendarMonthContext, CalendarMonthDecoration>>,
    day_event: Option<dioxus_core::Callback<CalendarDayEvent, CalendarDayEventResponse>>,
    layout: CalendarPluginLayout,
}

impl CalendarPlugin {
    /// Create a compact supporting-content plugin.
    ///
    /// This retains the original `CalendarDayPlugin` constructor contract.
    pub const fn new(renderer: dioxus_core::Callback<CalendarDayContext, Element>) -> Self {
        Self {
            day_renderer: Some(CalendarDayRenderer::Supporting(renderer)),
            month_renderer: None,
            day_event: None,
            layout: CalendarPluginLayout::supporting_content(),
        }
    }

    /// Create a plugin that returns semantic style and content contributions.
    pub const fn decorator(
        renderer: dioxus_core::Callback<CalendarDayContext, CalendarDayDecoration>,
    ) -> Self {
        Self {
            day_renderer: Some(CalendarDayRenderer::Decoration(renderer)),
            month_renderer: None,
            day_event: None,
            layout: CalendarPluginLayout::supporting_content(),
        }
    }

    /// Create a plugin with no day renderer, ready for event or month hooks.
    pub const fn empty() -> Self {
        Self {
            day_renderer: None,
            month_renderer: None,
            day_event: None,
            layout: CalendarPluginLayout {
                minimum_day_size: None,
                minimum_week_row_height: None,
            },
        }
    }

    pub const fn with_month_renderer(
        mut self,
        renderer: dioxus_core::Callback<CalendarMonthContext, CalendarMonthDecoration>,
    ) -> Self {
        self.month_renderer = Some(renderer);
        self
    }

    pub const fn with_day_event(
        mut self,
        handler: dioxus_core::Callback<CalendarDayEvent, CalendarDayEventResponse>,
    ) -> Self {
        self.day_event = Some(handler);
        self
    }

    pub const fn with_layout(mut self, layout: CalendarPluginLayout) -> Self {
        self.layout = layout;
        self
    }

    pub(super) const fn layout(self) -> CalendarPluginLayout {
        self.layout
    }

    pub(super) fn decorate_day(self, context: CalendarDayContext) -> CalendarDayDecoration {
        match self.day_renderer {
            Some(CalendarDayRenderer::Supporting(renderer)) => {
                CalendarDayDecoration::new().with_supporting(renderer.call(context))
            }
            Some(CalendarDayRenderer::Decoration(renderer)) => renderer.call(context),
            None => CalendarDayDecoration::new(),
        }
    }

    pub(super) fn decorate_month(self, context: CalendarMonthContext) -> CalendarMonthDecoration {
        self.month_renderer
            .map_or_else(CalendarMonthDecoration::new, |renderer| {
                renderer.call(context)
            })
    }

    pub(super) fn dispatch_day_event(self, event: CalendarDayEvent) -> CalendarDayEventResponse {
        self.day_event
            .map_or_else(CalendarDayEventResponse::continue_default, |handler| {
                handler.call(event)
            })
    }
}

impl Default for CalendarPlugin {
    fn default() -> Self {
        Self::empty()
    }
}

/// Compatibility name for the original supporting-content plugin API.
pub type CalendarDayPlugin = CalendarPlugin;
