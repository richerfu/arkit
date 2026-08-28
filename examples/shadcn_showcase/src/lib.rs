//! shadcn showcase aligned with the pre-Dioxus React Native Reusables demo.

use std::{rc::Rc, time::Duration};

use arkit::dioxus_core::{AttributeValue, EventHandler, VNode};
use arkit::dioxus_hooks::use_callback;
use arkit::dioxus_signals::WritableExt;
use arkit::prelude::*;
// The Routable derive emits `::dioxus_router` paths.
use arkit::router::dioxus_router;
use arkit::router::{use_navigator, MemoryRouter, Outlet, Routable, RouteProvider};
use arkit::shadcn as arkit_shadcn;
use arkit::shadcn::components::{
    use_anchor, Accordion, AccordionItemSpec, Alert, AlertDescription, AlertDialog,
    AlertDialogAction, AlertList, AlertTitle, AlertVariant, Anchor, AnchorItem, AnchorSection,
    AspectRatio, Avatar, AvatarFallback, Badge, BadgeVariant, BottomNavigation,
    BottomNavigationItem, BottomSheet, BottomSheetTextInput, Button, ButtonSize, ButtonVariant,
    Calendar, CalendarDayContext, CalendarDayDecoration, CalendarDayEvent, CalendarDayEventKind,
    CalendarDayEventResponse, CalendarDayStyle, CalendarPlugin, CalendarPluginLayout,
    CalendarYearRange, Card, CardContent, CardFooter, CardHeader, Carousel,
    CarouselControlsPlacement, CarouselIndicatorVariant, CarouselStyle, Checkbox, Code,
    Collapsible, ContextMenu, DatePicker, Dialog, DialogFooter, DialogHeader, DropdownMenu, Field,
    FieldContent, FieldDescription, FieldError, FieldGroup, FieldOrientation, FieldSeparator,
    FieldSet, FieldTitle, Form, FormItem, Guide, GuideSide, GuideStep, GuideTarget, HoverCard,
    Index, IndexBarSlot, IndexHeaderContext, IndexItemContext, IndexItemSpec, InfiniteScroll,
    Input, InputMode, InputOtp, InputOtpMode, InputOtpSeparator, Label, LoadMoreIndicator,
    LoadMoreState, Markdown, MenuEntry, Menubar, MenubarMenuSpec, MultiSlider, Popover, Progress,
    PullToRefresh, RadioGroup, RangeSlider, SecureKeyboardMode, SecureKeyboardSheet, Select,
    Separator, Skeleton, Slider, SliderOrientation, SliderStyle, Sonner, SonnerPosition,
    SonnerToast, Spinner, Switch, Table, Tabs, Text, TextVariant, Textarea, TimePicker,
    TimePickerFormat, TimeValue, Timeline, TimelineAlign, TimelineItem, TimelineOrientation,
    ToastAppearance, Toggle, ToggleGroup, ToggleVariant, Tooltip, Watermark, WatermarkBlendMode,
    WatermarkFontStyle, WatermarkShadow, WatermarkSource, WatermarkStroke, WatermarkStyle,
};
use arkit::shadcn::icon::icon_placeholder;
use arkit::shadcn::theme::{
    spacing, typography, ColorTokens, RadiusTokens, Theme, ThemeMode, ThemePreset, ThemeProvider,
};
use arkit_calendar_icu::{use_chinese_lunar_plugin, ChineseLunarOptions};

const HOME_HEADER_HEIGHT: f32 = 80.0;
const DETAIL_HEADER_HEIGHT: f32 = 48.0;
const TRACKING_TIGHT: f32 = -0.35;
const MARKDOWN_STREAM_INTERVAL_MS: u64 = 500;
const WATERMARK_IMAGE_SAMPLE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="840" height="472" viewBox="0 0 840 472">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#0f766e"/>
      <stop offset="1" stop-color="#164e63"/>
    </linearGradient>
    <linearGradient id="ground" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#22c55e"/>
      <stop offset="1" stop-color="#166534"/>
    </linearGradient>
  </defs>
  <rect width="840" height="472" fill="url(#sky)"/>
  <circle cx="664" cy="112" r="58" fill="#fde68a"/>
  <path d="M0 374L194 174L340 323L470 210L720 397L840 304V472H0Z" fill="#d1fae5" opacity=".9"/>
  <path d="M0 418L224 275L351 365L518 250L840 431V472H0Z" fill="url(#ground)"/>
  <path d="M78 90h290" stroke="#ffffff" stroke-opacity=".42" stroke-width="8" stroke-linecap="round"/>
  <path d="M78 118h210" stroke="#ffffff" stroke-opacity=".26" stroke-width="8" stroke-linecap="round"/>
</svg>"##;
const WATERMARK_LOGO_SAMPLE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="512" height="160" viewBox="0 0 512 160">
  <path d="M22 80L58 18h72l36 62-36 62H58L22 80Z" fill="#2563eb"/>
  <path d="M67 106L94 54l27 52M76 88h36" fill="none" stroke="#fff" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M205 112V48h19l28 38 28-38h19v64h-20V79l-24 32h-7l-23-32v33h-20Zm116 0V48h22v64h-22Zm45 0V48h22v24l24-24h27l-29 29 32 35h-29l-25-29v29h-22Z" fill="#0f172a"/>
</svg>"##;

arkit::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "en-US",
        locales: ["en-US", "zh-CN"],
    }
}

/// Static sample for tree-sitter fenced-code highlighting (`markdown` + `code`).
const MARKDOWN_HIGHLIGHT_SAMPLE: &str = r#"## Syntax highlighting

Fenced blocks reuse the standalone `Code` pipeline when the `code` feature is enabled. Unknown languages fall back to plain monospace.

```rust
fn fib(n: u32) -> u32 {
    match n {
        0 | 1 => n,
        _ => fib(n - 1) + fib(n - 2),
    }
}
```

```python
def greet(name: str) -> str:
    # theme-aware token colors
    return f"hello, {name}"
```

```json
{
  "feature": "code",
  "languages": ["rust", "python", "json", "bash"]
}
```

```bash
ohrs build --arch aarch
./app/run.sh shadcn_showcase all
```
"#;

const CODE_STANDALONE_RUST: &str = r#"pub fn answer() -> i32 {
    // standalone Code component
    42
}
"#;

const CODE_STANDALONE_JSON: &str = r#"{
  "component": "Code",
  "feature": "code",
  "highlight": true
}
"#;

const MARKDOWN_STREAM_CHUNKS: &[&str] = &[
    "# Live deployment briefing\n\n",
    "**Chunked",
    " Markdown response**\n\n",
    "Content arrives from the release assistant. Each chunk is reparsed as a compact event stream and rendered directly into native ArkUI nodes.\n\n",
    "> [!NOTE]\n",
    "> Watch incomplete emphasis, tables, and fenced code settle into their final structure ",
    "as later chunks arrive.\n\n",
    "## Rollout status\n\n",
    "| Region | Version | Health |\n",
    "| :-- | --: | :-- |\n",
    "| Shanghai | `2.8.0` | ✅ Healthy |\n",
    "| Singapore | `2.8.0` | 🟡 Observing |\n",
    "| Frankfurt | `2.7.9` | ⏳ Queued |\n\n",
    "## What changed\n\n",
    "- **Parser pipeline**\n  - consumes CommonMark events without HTML\n",
    "  - coalesces adjacent text runs to reduce native nodes\n",
    "- **Renderer**\n  1. maps blocks to ArkUI layout primitives\n",
    "  2. keeps theme and link callbacks independent from parsing\n\n",
    "## Streaming Rust\n\n```rust\n",
    "let mut source = String::new();\n",
    "while let Some(chunk) = response.next().await {\n",
    "    source.push_str(&chunk);\n",
    "    markdown_source.set(source.clone());\n",
    "}\n",
    "```\n\n",
    "## Verification matrix\n\n",
    "- [x] headings, emphasis, and links\n",
    "- [x] nested ordered and unordered lists\n",
    "- [x] tables, task lists, code fences, and footnotes\n",
    "- [ ] production endpoint connected\n\n",
    "> **Result: one component handles partial and complete documents with the same API.**\n\n",
    "The final snapshot is reusable across theme changes.[^snapshot]\n\n",
    "[^snapshot]: Parsing is memoized while `source` and `options` stay unchanged.\n\n",
    "Read the [Markdown component guide](https://example.com/arkit/markdown).",
];

const THEME_PRESETS: [ThemePreset; 7] = [
    ThemePreset::Zinc,
    ThemePreset::Neutral,
    ThemePreset::Stone,
    ThemePreset::Mauve,
    ThemePreset::Olive,
    ThemePreset::Mist,
    ThemePreset::Taupe,
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct ComponentSpec {
    slug: &'static str,
    name: &'static str,
}

const COMPONENTS: &[ComponentSpec] = &[
    ComponentSpec {
        slug: "accordion",
        name: "Accordion",
    },
    ComponentSpec {
        slug: "alert",
        name: "Alert",
    },
    ComponentSpec {
        slug: "alert-dialog",
        name: "Alert Dialog",
    },
    ComponentSpec {
        slug: "anchor",
        name: "Anchor",
    },
    ComponentSpec {
        slug: "aspect-ratio",
        name: "Aspect Ratio",
    },
    ComponentSpec {
        slug: "avatar",
        name: "Avatar",
    },
    ComponentSpec {
        slug: "badge",
        name: "Badge",
    },
    ComponentSpec {
        slug: "bottom-navigation",
        name: "Bottom Navigation",
    },
    ComponentSpec {
        slug: "bottom-sheet",
        name: "Bottom Sheet",
    },
    ComponentSpec {
        slug: "button",
        name: "Button",
    },
    ComponentSpec {
        slug: "calendar",
        name: "Calendar",
    },
    ComponentSpec {
        slug: "card",
        name: "Card",
    },
    ComponentSpec {
        slug: "carousel",
        name: "Carousel",
    },
    ComponentSpec {
        slug: "checkbox",
        name: "Checkbox",
    },
    ComponentSpec {
        slug: "collapsible",
        name: "Collapsible",
    },
    ComponentSpec {
        slug: "context-menu",
        name: "Context Menu",
    },
    ComponentSpec {
        slug: "date-picker",
        name: "Date Picker",
    },
    ComponentSpec {
        slug: "time-picker",
        name: "Time Picker",
    },
    ComponentSpec {
        slug: "timeline",
        name: "Timeline",
    },
    ComponentSpec {
        slug: "index",
        name: "Index",
    },
    ComponentSpec {
        slug: "dialog",
        name: "Dialog",
    },
    ComponentSpec {
        slug: "dropdown-menu",
        name: "Dropdown Menu",
    },
    ComponentSpec {
        slug: "form",
        name: "Form",
    },
    ComponentSpec {
        slug: "guide",
        name: "Guide",
    },
    ComponentSpec {
        slug: "hover-card",
        name: "Hover Card",
    },
    ComponentSpec {
        slug: "icon",
        name: "Icon",
    },
    ComponentSpec {
        slug: "input",
        name: "Input",
    },
    ComponentSpec {
        slug: "input-otp",
        name: "Input OTP",
    },
    ComponentSpec {
        slug: "label",
        name: "Label",
    },
    ComponentSpec {
        slug: "layout",
        name: "Col / Row",
    },
    ComponentSpec {
        slug: "code",
        name: "Code",
    },
    ComponentSpec {
        slug: "markdown",
        name: "Markdown",
    },
    ComponentSpec {
        slug: "menubar",
        name: "Menubar",
    },
    ComponentSpec {
        slug: "popover",
        name: "Popover",
    },
    ComponentSpec {
        slug: "progress",
        name: "Progress",
    },
    ComponentSpec {
        slug: "refresh-load-more",
        name: "Refresh & Load More",
    },
    ComponentSpec {
        slug: "radio-group",
        name: "Radio Group",
    },
    ComponentSpec {
        slug: "select",
        name: "Select",
    },
    ComponentSpec {
        slug: "secure-keyboard",
        name: "Secure Keyboard",
    },
    ComponentSpec {
        slug: "separator",
        name: "Separator",
    },
    ComponentSpec {
        slug: "skeleton",
        name: "Skeleton",
    },
    ComponentSpec {
        slug: "slider",
        name: "Slider",
    },
    ComponentSpec {
        slug: "sonner",
        name: "Sonner",
    },
    ComponentSpec {
        slug: "spinner",
        name: "Spinner",
    },
    ComponentSpec {
        slug: "switch",
        name: "Switch",
    },
    ComponentSpec {
        slug: "tabs",
        name: "Tabs",
    },
    ComponentSpec {
        slug: "text",
        name: "Text",
    },
    ComponentSpec {
        slug: "textarea",
        name: "Textarea",
    },
    ComponentSpec {
        slug: "toggle",
        name: "Toggle",
    },
    ComponentSpec {
        slug: "toggle-group",
        name: "Toggle Group",
    },
    ComponentSpec {
        slug: "tooltip",
        name: "Tooltip",
    },
    ComponentSpec {
        slug: "table",
        name: "Table",
    },
    ComponentSpec {
        slug: "watermark",
        name: "Watermark",
    },
];

#[derive(Routable, Clone, PartialEq, Debug)]
enum Route {
    #[layout(ShowcaseShell)]
    #[route("/")]
    Home {},
    #[route("/components/:slug")]
    Detail { slug: String },
}

#[derive(Clone, Copy)]
struct ShowcaseState {
    mode: Signal<ThemeMode>,
    preset: Signal<ThemePreset>,
    custom: Signal<bool>,
    theme_menu_open: Signal<bool>,
    language_menu_open: Signal<bool>,
    query: Signal<String>,
}

#[component]
pub fn ShadcnShowcasePage() -> Element {
    let _i18n = use_i18n_provider(&tr::CATALOG, tr::FALLBACK_LOCALE.id());
    let state = ShowcaseState {
        mode: use_signal(|| ThemeMode::Light),
        preset: use_signal(|| ThemePreset::Zinc),
        custom: use_signal(|| false),
        theme_menu_open: use_signal(|| false),
        language_menu_open: use_signal(|| false),
        query: use_signal(String::new),
    };
    use_context_provider(|| state);

    let theme = resolve_theme((state.mode)(), (state.preset)(), (state.custom)());

    rsx! {
        ThemeProvider {
            theme,
            MemoryRouter::<Route> {}
        }
    }
}

#[component]
fn ShowcaseShell() -> Element {
    let runtime = arkit::use_runtime_handle();
    let state = use_context::<ShowcaseState>();
    let navigator = use_navigator();
    let mut language_menu_open = state.language_menu_open;
    let mut theme_menu_open = state.theme_menu_open;

    let scoped_back_press = dioxus_hooks::use_callback(move |()| {
        if language_menu_open() {
            language_menu_open.set(false);
            return true;
        }
        if theme_menu_open() {
            theme_menu_open.set(false);
            return true;
        }
        if navigator.can_go_back() {
            navigator.go_back();
            return true;
        }
        false
    });
    let back_press_handler: Rc<dyn Fn() -> bool> = Rc::new(move || scoped_back_press.call(()));
    let _back_press_registration =
        use_hook(move || Rc::new(runtime.register_back_handler(back_press_handler)));

    rsx! { Outlet::<Route> {} }
}

#[component]
fn Home() -> Element {
    let state = use_context::<ShowcaseState>();
    let navigator = use_navigator();
    let mut query = state.query;
    let mut mode = state.mode;
    let mut preset = state.preset;
    let mut custom = state.custom;
    let mut theme_menu_open = state.theme_menu_open;
    let mut language_menu_open = state.language_menu_open;
    let route_key = "home";

    rsx! {
        MountTransition {
            key: "{route_key}",
            preset: TransitionPreset::SlideRight,
            duration_ms: 200,
            fill: true,
            HomeView {
                query: query(),
                mode: mode(),
                preset: preset(),
                custom: custom(),
                theme_menu_open: theme_menu_open(),
                language_menu_open: language_menu_open(),
                on_query: move |value: String| query.set(value),
                on_select: move |slug: &'static str| {
                    navigator.push(Route::Detail {
                        slug: slug.to_string(),
                    });
                },
                on_theme_menu_open: move |value| {
                    theme_menu_open.set(value);
                    if value {
                        language_menu_open.set(false);
                    }
                },
                on_language_menu_open: move |value| {
                    language_menu_open.set(value);
                    if value {
                        theme_menu_open.set(false);
                    }
                },
                on_mode: move |value| {
                    mode.set(value);
                    theme_menu_open.set(false);
                },
                on_preset: move |value| {
                    preset.set(value);
                    custom.set(false);
                    theme_menu_open.set(false);
                },
                on_custom: move |value| {
                    custom.set(value);
                    theme_menu_open.set(false);
                },
            }
        }
    }
}

#[component]
fn Detail(slug: String) -> Element {
    let state = use_context::<ShowcaseState>();
    let navigator = use_navigator();
    let mut mode = state.mode;
    let mut preset = state.preset;
    let mut custom = state.custom;
    let mut theme_menu_open = state.theme_menu_open;
    let mut language_menu_open = state.language_menu_open;
    let slug = COMPONENTS
        .iter()
        .find(|item| item.slug == slug)
        .map(|item| item.slug)
        .unwrap_or("unknown");

    rsx! {
        MountTransition {
            key: "{slug}",
            preset: TransitionPreset::SlideLeft,
            duration_ms: 220,
            fill: true,
            DetailView {
                slug,
                mode: mode(),
                preset: preset(),
                custom: custom(),
                theme_menu_open: theme_menu_open(),
                language_menu_open: language_menu_open(),
                on_back: move |_| navigator.go_back(),
                on_theme_menu_open: move |value| {
                    theme_menu_open.set(value);
                    if value {
                        language_menu_open.set(false);
                    }
                },
                on_language_menu_open: move |value| {
                    language_menu_open.set(value);
                    if value {
                        theme_menu_open.set(false);
                    }
                },
                on_mode: move |value| {
                    mode.set(value);
                    theme_menu_open.set(false);
                },
                on_preset: move |value| {
                    preset.set(value);
                    custom.set(false);
                    theme_menu_open.set(false);
                },
                on_custom: move |value| {
                    custom.set(value);
                    theme_menu_open.set(false);
                },
            }
        }
    }
}

fn resolve_theme(mode: ThemeMode, preset: ThemePreset, custom: bool) -> Theme {
    let theme = if custom {
        Theme::custom(custom_theme_colors(mode))
            .with_mode(mode)
            .with_radius(RadiusTokens::from_base(10.0))
    } else {
        Theme::preset(preset, mode)
    };
    theme.with_colors(theme.colors.with_surface(theme.colors.secondary))
}

fn custom_theme_colors(mode: ThemeMode) -> ColorTokens {
    let mut colors = Theme::preset(ThemePreset::Mist, mode).colors;

    match mode {
        ThemeMode::Light => {
            colors.primary = 0xFF0F766E;
            colors.primary_foreground = 0xFFF0FDFA;
            colors.primary_track = arkit_shadcn::theme::with_alpha(colors.primary, 0x33);
            colors.ring = 0xFF0F766E;
            colors.chart_1 = 0xFF0F766E;
            colors.chart_2 = 0xFF2563EB;
            colors.chart_3 = 0xFF7C3AED;
            colors.sidebar_primary = colors.primary;
            colors.sidebar_primary_foreground = colors.primary_foreground;
        }
        ThemeMode::Dark => {
            colors.primary = 0xFF5EEAD4;
            colors.primary_foreground = 0xFF042F2E;
            colors.primary_track = arkit_shadcn::theme::with_alpha(colors.primary, 0x33);
            colors.ring = 0xFF5EEAD4;
            colors.chart_1 = 0xFF5EEAD4;
            colors.chart_2 = 0xFF60A5FA;
            colors.chart_3 = 0xFFC084FC;
            colors.sidebar_primary = colors.primary;
            colors.sidebar_primary_foreground = colors.primary_foreground;
        }
    }

    colors
}

#[component]
fn HomeView(
    query: String,
    mode: ThemeMode,
    preset: ThemePreset,
    custom: bool,
    theme_menu_open: bool,
    language_menu_open: bool,
    on_query: EventHandler<String>,
    on_select: EventHandler<&'static str>,
    on_theme_menu_open: EventHandler<bool>,
    on_language_menu_open: EventHandler<bool>,
    on_mode: EventHandler<ThemeMode>,
    on_preset: EventHandler<ThemePreset>,
    on_custom: EventHandler<bool>,
) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let q = query.to_ascii_lowercase();
    let items = COMPONENTS
        .iter()
        .copied()
        .filter(|item| q.is_empty() || item.name.to_ascii_lowercase().contains(&q))
        .collect::<Vec<_>>();

    rsx! {
        NavBar {
            title: "Arkit".to_string(),
            back: false,
            mode,
            preset,
            custom,
            open: theme_menu_open,
            language_open: language_menu_open,
            on_back: move |_| {},
            on_open: on_theme_menu_open,
            on_language_open: on_language_menu_open,
            on_mode,
            on_preset,
            on_custom,
        }
        RouteProvider {
            column {
                width: "100%",
                background_color: theme.colors.background,
                align_items: "center",
                justify_content: "start",
                padding_top: spacing::LG,
                padding_right: spacing::LG,
                padding_bottom: spacing::XXL,
                padding_left: spacing::LG,
                column {
                    width: "100%",
                    max_width_constraint: 512.0,
                    align_items: "start",
                    justify_content: "start",
                    Input {
                        placeholder: Some("Search UI...".to_string()),
                        value: Some(query),
                        width: "100%",
                        on_change: move |value| on_query.call(value),
                    }
                    row { height: spacing::LG }
                    if items.is_empty() {
                        Card {
                            CardHeader {
                                title: "No component found".to_string(),
                                description: "Try a different keyword".to_string(),
                            }
                        }
                    } else {
                        column {
                            width: "100%",
                            align_items: "start",
                            justify_content: "start",
                            for (index, item) in items.iter().enumerate() {
                                ComponentListItem {
                                    spec: *item,
                                    first: index == 0,
                                    last: index + 1 == items.len(),
                                    on_select,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DetailView(
    slug: &'static str,
    mode: ThemeMode,
    preset: ThemePreset,
    custom: bool,
    theme_menu_open: bool,
    language_menu_open: bool,
    on_back: EventHandler<()>,
    on_theme_menu_open: EventHandler<bool>,
    on_language_menu_open: EventHandler<bool>,
    on_mode: EventHandler<ThemeMode>,
    on_preset: EventHandler<ThemePreset>,
    on_custom: EventHandler<bool>,
) -> Element {
    rsx! {
        NavBar {
            title: component_title(slug),
            back: true,
            mode,
            preset,
            custom,
            open: theme_menu_open,
            language_open: language_menu_open,
            on_back,
            on_open: on_theme_menu_open,
            on_language_open: on_language_menu_open,
            on_mode,
            on_preset,
            on_custom,
        }
        DemoCanvas {
            slug,
        }
    }
}

#[component]
fn NavBar(
    title: String,
    back: bool,
    mode: ThemeMode,
    preset: ThemePreset,
    custom: bool,
    open: bool,
    language_open: bool,
    on_back: EventHandler<()>,
    on_open: EventHandler<bool>,
    on_language_open: EventHandler<bool>,
    on_mode: EventHandler<ThemeMode>,
    on_preset: EventHandler<ThemePreset>,
    on_custom: EventHandler<bool>,
) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let title_size = if back { 17.0 } else { 34.0 };
    let title_weight = if back { 500 } else { 700 };
    let title_line_height = if back { 22.0 } else { 40.0 };
    let header_height = if back {
        DETAIL_HEADER_HEIGHT
    } else {
        HOME_HEADER_HEIGHT
    };

    rsx! {
        row {
            width: "100%",
            height: header_height,
            background_color: theme.colors.background,
            padding_top: if back { 4.0 } else { 18.0 },
            padding_right: spacing::LG,
            padding_bottom: if back { 4.0 } else { 6.0 },
            padding_left: spacing::LG,
            align_items: if back { "center" } else { "bottom" },
            row {
                layout_weight: 1.0,
                align_items: if back { "center" } else { "bottom" },
                clip: true,
                if back {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        onclick: move |_| on_back.call(()),
                        {icon_placeholder("chevron-left", 20.0, theme.colors.foreground)}
                    }
                    row { width: spacing::SM }
                }
                row {
                    layout_weight: 1.0,
                    clip: true,
                    text {
                        width: "100%",
                        content: title,
                        font_size: title_size,
                        font_weight: title_weight,
                        line_height: title_line_height,
                        font_color: theme.colors.foreground,
                        text_letter_spacing: TRACKING_TIGHT,
                        max_lines: 1_i32,
                        text_overflow: "ellipsis",
                    }
                }
            }
            ThemeMenu {
                mode,
                preset,
                custom,
                open,
                on_open,
                on_mode,
                on_preset,
                on_custom,
            }
            row { width: spacing::SM }
            LanguageMenu {
                open: language_open,
                on_open: on_language_open,
            }
        }
    }
}

#[component]
fn LanguageMenu(open: bool, on_open: EventHandler<bool>) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let i18n = use_i18n();
    let selected = i18n.locale_id();
    let items = vec![
        MenuEntry::radio(
            "简体中文",
            "zh-CN",
            selected.clone(),
            EventHandler::new(move |_| i18n.set_locale_id("zh-CN")),
        )
        .close_on_select(),
        MenuEntry::radio(
            "English",
            "en-US",
            selected,
            EventHandler::new(move |_| i18n.set_locale_id("en-US")),
        )
        .close_on_select(),
    ];

    rsx! {
        DropdownMenu {
            items,
            open: Some(open),
            default_open: false,
            on_open_change: Some(on_open),
            trigger_capture: Some(false),
            width: Some(176.0),
            row {
                width: 40.0,
                height: 40.0,
                align_items: "center",
                justify_content: "center",
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                {icon_placeholder("languages", 18.0, theme.colors.foreground)}
            }
        }
    }
}

#[component]
fn ThemeMenu(
    mode: ThemeMode,
    preset: ThemePreset,
    custom: bool,
    open: bool,
    on_open: EventHandler<bool>,
    on_mode: EventHandler<ThemeMode>,
    on_preset: EventHandler<ThemePreset>,
    on_custom: EventHandler<bool>,
) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let selected_preset = if custom {
        String::from("custom")
    } else {
        theme_preset_key(preset).to_string()
    };
    let selected_mode = theme_mode_key(mode).to_string();
    let icon = match mode {
        ThemeMode::Light => "sun",
        ThemeMode::Dark => "moon",
    };

    let mut items = vec![
        MenuEntry::label("Appearance"),
        MenuEntry::radio(
            "Light",
            "light",
            selected_mode.clone(),
            EventHandler::new(move |_| on_mode.call(ThemeMode::Light)),
        )
        .close_on_select(),
        MenuEntry::radio(
            "Dark",
            "dark",
            selected_mode,
            EventHandler::new(move |_| on_mode.call(ThemeMode::Dark)),
        )
        .close_on_select(),
        MenuEntry::separator(),
        MenuEntry::label("Theme"),
    ];

    for item in THEME_PRESETS {
        items.push(
            MenuEntry::radio(
                theme_preset_label(item),
                theme_preset_key(item),
                selected_preset.clone(),
                EventHandler::new(move |_| on_preset.call(item)),
            )
            .close_on_select(),
        );
    }
    items.push(MenuEntry::separator());
    items.push(
        MenuEntry::radio(
            "Custom",
            "custom",
            selected_preset,
            EventHandler::new(move |_| on_custom.call(true)),
        )
        .close_on_select(),
    );

    rsx! {
        DropdownMenu {
            items,
            open: Some(open),
            default_open: false,
            on_open_change: Some(on_open),
            trigger_capture: Some(false),
            row {
                width: 40.0,
                height: 40.0,
                align_items: "center",
                justify_content: "center",
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                {icon_placeholder(icon, 18.0, theme.colors.foreground)}
            }
        }
    }
}

#[component]
fn ComponentListItem(
    spec: ComponentSpec,
    first: bool,
    last: bool,
    on_select: EventHandler<&'static str>,
) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let radius = theme.radii.lg;
    let top_radius = if first { radius } else { 0.0 };
    let bottom_radius = if last { radius } else { 0.0 };
    let radius_value = format!("{top_radius},{top_radius},{bottom_radius},{bottom_radius}");
    let bottom_border = if last { 1.0 } else { 0.0 };
    let border_width = format!("1,1,{bottom_border},1");
    let row_background = arkit_shadcn::theme::with_alpha(theme.colors.secondary, 0x66);
    let row_border = arkit_shadcn::theme::with_alpha(theme.colors.foreground, 0x0D);
    let icon_color = arkit_shadcn::theme::with_alpha(theme.colors.foreground, 0x80);

    rsx! {
        row {
            width: "100%",
            height: 56.0,
            align_items: "center",
            justify_content: "space_between",
            padding_top: 0.0,
            padding_right: 6.0,
            padding_bottom: 0.0,
            padding_left: spacing::LG,
            background_color: row_background,
            border_width: border_width,
            border_color: row_border,
            border_style: "solid",
            border_radius: radius_value,
            clip: true,
            onclick: move |_| on_select.call(spec.slug),
            text {
                content: spec.name.to_string(),
                font_size: typography::XL,
                font_weight: 400_i32,
                font_color: theme.colors.foreground,
                line_height: 24.0,
            }
            row {
                layout_weight: 1.0,
            }
            {icon_placeholder("chevron-right", 24.0, icon_color)}
        }
    }
}

#[component]
fn DemoCanvas(slug: &'static str) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    if slug == "refresh-load-more" {
        return rsx! {
            column {
                width: "100%",
                layout_weight: 1.0,
                background_color: theme.colors.surface,
                ComponentDemo { slug }
            }
        };
    }
    let policy = demo_canvas_policy(slug);
    let bottom_padding = if slug == "bottom-navigation" {
        policy.padding[2]
    } else {
        policy.padding[2] + spacing::XXL
    };

    if policy.fill_height {
        rsx! {
            RouteProvider {
                column {
                    width: "100%",
                    height: "100%",
                    background_color: theme.colors.surface,
                    align_items: if policy.center_x { "center" } else { "start" },
                    justify_content: if policy.center_y { "center" } else { "start" },
                    padding_top: policy.padding[0],
                    padding_right: policy.padding[1],
                    padding_bottom: bottom_padding,
                    padding_left: policy.padding[3],
                    ComponentDemo { slug }
                }
            }
        }
    } else {
        rsx! {
            RouteProvider {
                column {
                    width: "100%",
                    background_color: theme.colors.surface,
                    align_items: if policy.center_x { "center" } else { "start" },
                    justify_content: if policy.center_y { "center" } else { "start" },
                    padding_top: policy.padding[0],
                    padding_right: policy.padding[1],
                    padding_bottom: bottom_padding,
                    padding_left: policy.padding[3],
                    ComponentDemo { slug }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct DemoCanvasPolicy {
    center_x: bool,
    center_y: bool,
    fill_height: bool,
    padding: [f32; 4],
}

fn demo_canvas_policy(slug: &str) -> DemoCanvasPolicy {
    match slug {
        "button" | "select" | "text" => DemoCanvasPolicy {
            center_x: true,
            center_y: true,
            fill_height: true,
            padding: [0.0, 0.0, 0.0, 0.0],
        },
        "aspect-ratio" => DemoCanvasPolicy {
            center_x: false,
            center_y: true,
            fill_height: true,
            padding: [0.0, spacing::LG, 0.0, spacing::LG],
        },
        "accordion" | "timeline" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: false,
            padding: [0.0, spacing::LG, 0.0, spacing::LG],
        },
        "calendar" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: false,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "carousel" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: false,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "bottom-navigation" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [0.0, 0.0, 0.0, 0.0],
        },
        "context-menu" | "dropdown-menu" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: true,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "menubar" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: true,
            padding: [spacing::MD, spacing::MD, spacing::MD, spacing::MD],
        },
        "alert" | "tabs" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "slider" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: false,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "form" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: false,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "guide" => DemoCanvasPolicy {
            center_x: true,
            center_y: true,
            fill_height: true,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "anchor" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: false,
            padding: [spacing::LG, spacing::LG, spacing::XXL, spacing::LG],
        },
        "index" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
        "refresh-load-more" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [0.0, 0.0, 0.0, 0.0],
        },
        "sonner" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "input-otp" | "secure-keyboard" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "code" | "markdown" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: false,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "watermark" => DemoCanvasPolicy {
            center_x: true,
            center_y: false,
            fill_height: false,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        _ => DemoCanvasPolicy {
            center_x: true,
            center_y: true,
            fill_height: true,
            padding: [spacing::LG, spacing::LG, spacing::LG, spacing::LG],
        },
    }
}

#[component]
fn ComponentDemo(slug: &'static str) -> Element {
    let lunar_calendar_plugin = use_chinese_lunar_plugin(ChineseLunarOptions::default());
    let calendar_memos = use_signal(Vec::<String>::new);
    let calendar_plugin_status =
        use_signal(|| "Long-press a date to add or remove a memo marker.".to_string());
    let memo_dates_for_render = calendar_memos;
    let memo_renderer = use_callback(move |context: CalendarDayContext| {
        let date = context.date.to_string();
        let has_memo = memo_dates_for_render
            .read()
            .iter()
            .any(|memo_date| memo_date == &date);
        let mut decoration = CalendarDayDecoration::new().with_style(CalendarDayStyle {
            border_color: has_memo.then_some(0xFFF59E0Bu32),
            border_width: has_memo.then_some(1.5),
            ..CalendarDayStyle::default()
        });
        if has_memo {
            decoration = decoration.with_overlay(rsx! {
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "end",
                    justify_content: "start",
                    hit_test_behavior: "none",
                    row {
                        width: 6.0,
                        height: 6.0,
                        border_radius: 3.0,
                        background_color: 0xFFF59E0Bu32,
                        hit_test_behavior: "none",
                    }
                }
            });
        }
        decoration
    });
    let mut memo_dates_for_event = calendar_memos;
    let mut memo_status_for_event = calendar_plugin_status;
    let memo_event = use_callback(move |event: CalendarDayEvent| {
        if event.kind != CalendarDayEventKind::LongPress {
            return CalendarDayEventResponse::continue_default();
        }
        let date = event.context.date.to_string();
        let mut dates = memo_dates_for_event();
        if let Some(index) = dates.iter().position(|memo_date| memo_date == &date) {
            dates.remove(index);
            memo_status_for_event.set(format!("Removed memo marker from {date}"));
        } else {
            dates.push(date.clone());
            dates.sort();
            memo_status_for_event.set(format!("Added memo marker to {date}"));
        }
        memo_dates_for_event.set(dates);
        CalendarDayEventResponse::prevent_default()
    });
    let memo_calendar_plugin = CalendarPlugin::decorator(memo_renderer)
        .with_day_event(memo_event)
        .with_layout(CalendarPluginLayout::default());
    let mut page = use_signal(|| 1_i32);
    let mut dialog_open = use_signal(|| false);
    let mut dialog_name = use_signal(|| "Pedro Duarte".to_string());
    let mut dialog_username = use_signal(|| "@peduarte".to_string());
    let mut alert_open = use_signal(|| false);
    let mut anchor_last_jump =
        use_signal(|| "Click an anchor item to jump to a section.".to_string());
    let mut bottom_navigation_selected = use_signal(|| 0_usize);
    let mut bottom_sheet_open = use_signal(|| false);
    let mut bottom_sheet_name = use_signal(|| "Pedro Duarte".to_string());
    let mut bottom_sheet_username = use_signal(|| "@peduarte".to_string());
    let mut calendar_selected = use_signal(|| None::<String>);
    let mut calendar_selected_dates = use_signal(Vec::<String>::new);
    let mut carousel_index = use_signal(|| 0_usize);
    let mut carousel_overlay_index = use_signal(|| 0_usize);
    let mut date_picker_selected = use_signal(|| None::<String>);
    let mut date_picker_open = use_signal(|| false);
    let mut time_picker_selected = use_signal(|| TimeValue::new(9, 30));
    let mut time_picker_open = use_signal(|| false);
    let mut timeline_step = use_signal(|| 3_i32);
    let mut timeline_uc_note = use_signal(|| "default_value = 2".to_string());
    let mut timeline_orientation = use_signal(|| TimelineOrientation::Vertical);
    let mut timeline_align = use_signal(|| TimelineAlign::Right);
    let mut index_large = use_signal(|| false);
    let mut index_show_empty = use_signal(|| false);
    let mut index_custom = use_signal(|| false);
    let mut index_note = use_signal(|| "cities · hide empty · default UI".to_string());
    let index_render_item =
        use_callback(move |ctx: IndexItemContext| rsx! { IndexCustomItem { ctx } });
    let index_render_header =
        use_callback(move |ctx: IndexHeaderContext| rsx! { IndexCustomHeader { ctx } });
    let index_render_bar = use_callback(move |slot: IndexBarSlot| rsx! { IndexCustomBar { slot } });
    let mut popover_open = use_signal(|| false);
    let mut hover_open = use_signal(|| false);
    let mut tooltip_open = use_signal(|| false);
    let mut menu_open = use_signal(|| false);
    let mut context_open = use_signal(|| false);
    let mut context_bookmarks = use_signal(|| true);
    let mut context_full_urls = use_signal(|| false);
    let mut context_person = use_signal(|| "pedro".to_string());
    let mut context_outside_clicks = use_signal(|| 0_u32);
    let mut menubar_active = use_signal(|| None::<usize>);
    let mut select_open = use_signal(|| false);
    let mut selected_fruit = use_signal(|| "Apple".to_string());
    let mut accordion_value = use_signal(|| Some("item-1".to_string()));
    let mut upload_progress = use_signal(|| 13.0_f32);
    let mut radio_choice = use_signal(|| "Default".to_string());
    let mut checkbox_first = use_signal(|| false);
    let mut checkbox_second = use_signal(|| false);
    let mut checkbox_card = use_signal(|| false);
    let mut switch_checked = use_signal(|| false);
    let mut toggle_pressed = use_signal(|| false);
    let mut toggle_values = use_signal(|| vec!["bold".to_string()]);
    // Dual-mode demo signals (controlled half). Uncontrolled halves omit value/open.
    let mut accordion_uc_note = use_signal(|| "Uses default_value only.".to_string());
    let mut checkbox_uc_label = use_signal(|| false);
    let mut switch_uc_note = use_signal(|| "default_checked=false".to_string());
    let mut toggle_uc_note = use_signal(|| "default_checked=false".to_string());
    let mut toggle_group_uc_note = use_signal(|| "default: bold".to_string());
    let mut radio_uc_note = use_signal(|| "default: Comfortable".to_string());
    let mut tabs_active = use_signal(|| 0_usize);
    let mut collapsible_open = use_signal(|| true);
    let mut collapsible_uc_note = use_signal(|| "default_open=true".to_string());
    let mut select_uc_note = use_signal(|| "default: Apple".to_string());
    let mut select_open_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut carousel_uc_note = use_signal(|| "default_index=0".to_string());
    let mut date_picker_uc_selected = use_signal(|| None::<String>);
    let mut date_picker_uc_note = use_signal(|| "selection/open unmanaged".to_string());
    let mut time_picker_uc_note = use_signal(|| "default: 03:30 PM".to_string());
    let mut dialog_uc_gen = use_signal(|| 0_u64);
    let mut alert_uc_gen = use_signal(|| 0_u64);
    let mut sheet_uc_gen = use_signal(|| 0_u64);
    let mut popover_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut hover_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut tooltip_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut menu_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut context_uc_note = use_signal(|| "open unmanaged".to_string());
    let mut menubar_uc_note = use_signal(|| "active unmanaged".to_string());
    let mut input_controlled = use_signal(|| "hello".to_string());
    let mut input_password = use_signal(|| "secret".to_string());
    let mut input_number = use_signal(|| "2026".to_string());
    let mut textarea_controlled = use_signal(|| "Draft notes…".to_string());
    let mut otp_value = use_signal(String::new);
    let mut otp_invalid = use_signal(|| false);
    let mut otp_status = use_signal(|| "Enter the six-digit code.".to_string());
    let mut invite_code = use_signal(|| "A7".to_string());
    let mut secure_pin = use_signal(String::new);
    let mut secure_keyboard_open = use_signal(|| false);
    let mut secure_keyboard_status = use_signal(|| "No PIN has been submitted.".to_string());
    let mut secure_text = use_signal(String::new);
    let mut secure_text_open = use_signal(|| false);
    let mut secure_text_status =
        use_signal(|| "Letters, numbers, spaces, and symbols are accepted.".to_string());
    let mut form_name = use_signal(|| "Avery Stone".to_string());
    let mut form_email = use_signal(String::new);
    let mut form_bio = use_signal(|| "Product designer and weekend cyclist.".to_string());
    let mut form_product_updates = use_signal(|| true);
    let mut form_terms_accepted = use_signal(|| false);
    let mut form_attempted = use_signal(|| false);
    let mut form_status = use_signal(|| None::<bool>);
    let mut guide_open = use_signal(|| false);
    let mut guide_step = use_signal(|| 0_usize);
    let mut guide_status = use_signal(|| "Start the guide to preview all steps.".to_string());
    let mut playback_position = use_signal(|| 92.0_f32);
    let mut media_volume = use_signal(|| 68.0_f32);
    let mut notification_strength = use_signal(|| 6.0_f32);
    let mut listening_range = use_signal(|| [24.0_f32, 78.0_f32]);
    let mut equalizer_points = use_signal(|| vec![18.0_f32, 50.0_f32, 84.0_f32]);
    let mut left_channel = use_signal(|| 76.0_f32);
    let mut right_channel = use_signal(|| 58.0_f32);
    let sonner_toasts = use_signal(Vec::<SonnerToast>::new);
    let sonner_next_id = use_signal(|| 1_u64);
    let sonner_status = use_signal(|| "Tap a type to show a toast.".to_string());
    let mut sonner_background_clicks = use_signal(|| 0_u32);
    let mut markdown_link =
        use_signal(|| "Links become active as soon as their chunk arrives.".to_string());
    let mut markdown_source = use_signal(String::new);
    let mut markdown_chunk_index = use_signal(|| 0_usize);
    let mut markdown_streaming = use_signal(|| true);

    let async_runtime = arkit::use_runtime_handle().tokio();
    let markdown_stream_task = use_future(move || {
        let async_runtime = async_runtime.clone();
        async move {
            if slug != "markdown" {
                return;
            }

            loop {
                let timer = async_runtime.spawn(async {
                    tokio::time::sleep(Duration::from_millis(MARKDOWN_STREAM_INTERVAL_MS)).await;
                });
                if timer.await.is_err() {
                    markdown_streaming.set(false);
                    return;
                }

                let index = markdown_chunk_index();
                let Some(chunk) = MARKDOWN_STREAM_CHUNKS.get(index) else {
                    markdown_streaming.set(false);
                    return;
                };

                markdown_source.write().push_str(chunk);
                let next = index + 1;
                markdown_chunk_index.set(next);
                if next == MARKDOWN_STREAM_CHUNKS.len() {
                    markdown_streaming.set(false);
                    return;
                }
            }
        }
    });

    let theme = arkit_shadcn::theme::use_theme();
    let on_page = EventHandler::new(move |value: i32| page.set(value.max(1)));
    let form_name_invalid = form_attempted() && form_name().trim().chars().count() < 2;
    let form_email_invalid = form_attempted() && !is_valid_email(form_email().as_str());
    let form_terms_invalid = form_attempted() && !form_terms_accepted();

    match slug {
        "accordion" => rsx! {
            fixed_width {
                width: 512.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("value = {:?}", accordion_value())),
                    }
                    Accordion {
                        items: accordion_demo_items(),
                        value: Some(accordion_value()),
                        collapsible: true,
                        on_value_change: move |value| accordion_value.set(value),
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(accordion_uc_note()),
                    }
                    Accordion {
                        items: accordion_demo_items(),
                        default_value: Some("item-1".to_string()),
                        collapsible: true,
                        on_value_change: move |value| {
                            accordion_uc_note.set(format!("on_value_change = {:?}", value));
                        },
                    }
                }
            }
        },
        "alert" => rsx! {
            fixed_width {
                width: 576.0,
                column {
                    width: "100%",
                    Alert {
                        icon: "circle-check".to_string(),
                        AlertTitle { content: "Success! Your changes have been saved".to_string() }
                        AlertDescription { content: "This is an alert with icon, title and description.".to_string() }
                    }
                    v_gap { height: spacing::LG }
                    Alert {
                        icon: "terminal".to_string(),
                        AlertTitle { content: "This Alert has no description.".to_string() }
                    }
                    v_gap { height: spacing::LG }
                    Alert {
                        icon: "circle-alert".to_string(),
                        variant: AlertVariant::Destructive,
                        AlertTitle { content: "Unable to process your payment.".to_string(), variant: AlertVariant::Destructive }
                        AlertDescription { content: "Please verify your billing information and try again.".to_string(), variant: AlertVariant::Destructive }
                        AlertList {
                            items: vec![
                                "Check your card details".to_string(),
                                "Ensure sufficient funds".to_string(),
                                "Verify billing address".to_string(),
                            ],
                            variant: AlertVariant::Destructive,
                        }
                    }
                }
            }
        },
        "alert-dialog" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("open = {}", alert_open())),
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| alert_open.set(true),
                    "Show Alert Dialog"
                }
            }
            AlertDialog {
                title: "Are you absolutely sure?".to_string(),
                description: "This action cannot be undone. This will permanently delete your account and remove your data from our servers.".to_string(),
                open: Some(alert_open()),
                default_open: Some(false),
                on_close: move |_| alert_open.set(false),
                cancel: rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        width: "100%",
                        onclick: move |_| alert_open.set(false),
                        "Cancel"
                    }
                },
                action: rsx! {
                    Button {
                        width: "100%",
                        onclick: move |_| alert_open.set(false),
                        "Continue"
                    }
                },
            }
            {demo_mode_divider()}
            demo_mode_label {
                title: "Uncontrolled".to_string(),
                detail: Some("default_open + internal state; actions call use_dialog_close".to_string()),
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| alert_uc_gen.set(alert_uc_gen() + 1),
                "Show Uncontrolled Alert"
            }
            if alert_uc_gen() > 0 {
                AlertDialog {
                    key: "{alert_uc_gen()}",
                    title: "Uncontrolled dialog".to_string(),
                    description: "Opened with default_open. Cancel/OK and backdrop dismiss via internal close.".to_string(),
                    default_open: Some(true),
                    on_close: move |_| {},
                    cancel: rsx! {
                        AlertDialogAction {
                            width: "100%",
                            variant: ButtonVariant::Outline,
                            "Cancel"
                        }
                    },
                    action: rsx! {
                        AlertDialogAction {
                            width: "100%",
                            "OK"
                        }
                    },
                }
            }
        },
        "anchor" => rsx! {
            column {
                width: "100%",
                align_items: "start",
                demo_mode_label {
                    title: "Anchor".to_string(),
                    detail: Some(anchor_last_jump()),
                }
                row {
                    width: "100%",
                    height: 520.0,
                    Anchor {
                        scroll_duration: 300,
                        active_threshold: 8.0,
                        nav: rsx! {
                            column {
                                width: 176.0,
                                background_color: theme.colors.popover,
                                border_width: 1.0,
                                border_color: theme.colors.border,
                                border_radius: theme.radii.md,
                                padding_top: spacing::SM,
                                padding_right: spacing::SM,
                                padding_bottom: spacing::SM,
                                padding_left: spacing::SM,
                                AnchorActiveLabel {}
                                v_gap { height: spacing::SM }
                                AnchorItem {
                                    id: "intro".to_string(),
                                    title: "Introduction".to_string(),
                                    onclick: move |_| {
                                        anchor_last_jump.set("Jump to Introduction".to_string());
                                    },
                                }
                                v_gap { height: 2.0 }
                                AnchorItem {
                                    id: "install".to_string(),
                                    title: "Installation".to_string(),
                                    onclick: move |_| {
                                        anchor_last_jump.set("Jump to Installation".to_string());
                                    },
                                }
                                v_gap { height: 2.0 }
                                AnchorItem {
                                    id: "usage".to_string(),
                                    title: "Usage".to_string(),
                                    onclick: move |_| {
                                        anchor_last_jump.set("Jump to Usage".to_string());
                                    },
                                }
                                v_gap { height: 2.0 }
                                AnchorItem {
                                    id: "api".to_string(),
                                    title: "API Reference".to_string(),
                                    onclick: move |_| {
                                        anchor_last_jump.set("Jump to API Reference".to_string());
                                    },
                                }
                            }
                        },
                        children: rsx! {
                            column {
                                width: "100%",
                                AnchorSection {
                                    id: "intro".to_string(),
                                    children: anchor_section_card(
                                        "Introduction",
                                        "Anchor 提供页内锚点导航：左侧导航项对应右侧内容区块，点击后滚动到对应位置，滚动时高亮自动跟随当前可见区块。",
                                    ),
                                }
                                AnchorSection {
                                    id: "install".to_string(),
                                    children: anchor_section_card(
                                        "Installation",
                                        "使用 Anchor 时，先声明滚动容器和区块：Anchor 渲染 row { nav, scroll }，children 里的 AnchorSection 会被测量并注册为可跳转锚点。",
                                    ),
                                }
                                AnchorSection {
                                    id: "usage".to_string(),
                                    children: anchor_section_card(
                                        "Usage",
                                        "导航项用 AnchorItem 声明，点击后调用上下文 jump(id) 发起一次 scroll_offset 命令；滚动位置由 onscroll 增量累积，与区块帧比对得出激活项。",
                                    ),
                                }
                                AnchorSection {
                                    id: "api".to_string(),
                                    children: anchor_section_card(
                                        "API Reference",
                                        "use_anchor() 返回 AnchorContext，提供 jump(id)、active_id()、scroll_position()；scroll_duration 控制跳转动画时长，active_threshold 调节区块进入视口的判定阈值。",
                                    ),
                                }
                            }
                        },
                    }
                }
            }
        },
        "aspect-ratio" => rsx! {
            column {
                width: "100%",
                AspectRatio {
                    ratio: 16.0 / 9.0,
                    image {
                        src: "https://images.unsplash.com/photo-1672758247442-82df22f5899e".to_string(),
                        width: "100%",
                        height: "100%",
                        object_fit: "cover",
                        border_radius: theme.radii.md,
                        clip: true,
                    }
                }
            }
        },
        "avatar" => rsx! {
            row {
                align_items: "center",
                justify_content: "start",
                {demo_avatar("https://github.com/mrzachnugent.png", "ZN", true, None)}
                h_gap { width: 48.0 }
                {demo_avatar("https://github.com/shadcn.png", "CN", true, Some(theme.radii.lg))}
                h_gap { width: 48.0 }
                row {
                    align_items: "center",
                    justify_content: "start",
                    {demo_avatar("https://github.com/mrzachnugent.png", "ZN", true, None)}
                    row { width: -8.0 }
                    {demo_avatar("https://github.com/leerob.png", "LR", true, None)}
                    row { width: -8.0 }
                    {demo_avatar("https://github.com/evilrabbit.png", "ER", true, None)}
                }
            }
        },
        "badge" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: 384.0,
                    align_items: "start",
                    row {
                        align_self: "start",
                        align_items: "center",
                        justify_content: "start",
                        Badge { content: "Badge".to_string() }
                        h_gap { width: spacing::SM }
                        Badge { content: "Secondary".to_string(), variant: BadgeVariant::Secondary }
                        h_gap { width: spacing::SM }
                        Badge { content: "Destructive".to_string(), variant: BadgeVariant::Destructive }
                        h_gap { width: spacing::SM }
                        Badge { content: "Outline".to_string(), variant: BadgeVariant::Outline }
                    }
                    v_gap { height: spacing::SM }
                    row {
                        align_self: "start",
                        align_items: "center",
                        justify_content: "start",
                        Badge { content: "Verified".to_string(), icon: Some("badge-check".to_string()), icon_colors: Some((0xFF3B82F6u32, 0xFFFFFFFFu32)) }
                        h_gap { width: spacing::SM }
                        Badge { content: "8".to_string(), pill: Some(true) }
                        h_gap { width: spacing::SM }
                        Badge { content: "99".to_string(), variant: BadgeVariant::Destructive, pill: Some(true) }
                        h_gap { width: spacing::SM }
                        Badge { content: "20+".to_string(), variant: BadgeVariant::Outline, pill: Some(true) }
                    }
                }
            }
        },
        "bottom-navigation" => {
            let (page_title, page_description, page_icon) = match bottom_navigation_selected() {
                1 => ("Explore", "Discover something new today.", "compass"),
                2 => ("Alerts", "You are all caught up.", "bell"),
                3 => ("Profile", "Manage your account and preferences.", "user"),
                _ => ("Home", "Your latest activity is ready.", "house"),
            };

            rsx! {
                column {
                    width: "100%",
                    height: "100%",
                    background_color: theme.colors.card,
                    column {
                        width: "100%",
                        layout_weight: 1.0,
                        align_items: "center",
                        justify_content: "center",
                        {icon_placeholder(page_icon, 42.0, theme.colors.primary)}
                        v_gap { height: spacing::LG }
                        text {
                            content: page_title.to_string(),
                            font_size: typography::XXL,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 32.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: page_description.to_string(),
                            font_size: typography::SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                    BottomNavigation {
                        items: vec![
                            BottomNavigationItem::new("Home", "house"),
                            BottomNavigationItem::new("Explore", "compass"),
                            BottomNavigationItem::new("Alerts", "bell"),
                            BottomNavigationItem::new("Profile", "user"),
                        ],
                        selected: Some(bottom_navigation_selected()),
                        on_select: move |index| bottom_navigation_selected.set(index),
                    }
                }
            }
        }
        "bottom-sheet" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("open = {}", bottom_sheet_open())),
                }
                Button {
                    onclick: move |_| bottom_sheet_open.set(true),
                    "Open"
                }
            }
            BottomSheet {
                title: "Edit your profile".to_string(),
                open: Some(bottom_sheet_open()),
                default_open: Some(false),
                on_close: move |_| bottom_sheet_open.set(false),
                column {
                    width: "100%",
                    Label { content: "Name".to_string() }
                    v_gap { height: 10.0 }
                    BottomSheetTextInput {
                        value: Some(bottom_sheet_name()),
                        on_change: move |value| bottom_sheet_name.set(value),
                    }
                    v_gap { height: spacing::XXL }
                    Label { content: "Username".to_string() }
                    v_gap { height: 10.0 }
                    BottomSheetTextInput {
                        value: Some(bottom_sheet_username()),
                        on_change: move |value| bottom_sheet_username.set(value),
                    }
                    v_gap { height: spacing::XL }
                    Button {
                        width: "100%",
                        onclick: move |_| bottom_sheet_open.set(false),
                        "Save Changes"
                    }
                }
            }
            {demo_mode_divider()}
            demo_mode_label {
                title: "Uncontrolled".to_string(),
                detail: Some("default_open via remount key".to_string()),
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| sheet_uc_gen.set(sheet_uc_gen() + 1),
                "Open Uncontrolled Sheet"
            }
            if sheet_uc_gen() > 0 {
                BottomSheet {
                    key: "{sheet_uc_gen()}",
                    title: "Uncontrolled sheet".to_string(),
                    default_open: Some(true),
                    on_close: move |_| {},
                    Text {
                        content: "Dismiss uses internal open state.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            }
        },
        "button" => rsx! {
            column {
                width: "100%",
                height: "100%",
                align_items: "center",
                justify_content: "center",
                Button { onclick: move |_| {}, "Default" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Destructive, onclick: move |_| {}, "Destructive" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Destructive, disabled: Some(true), onclick: move |_| {}, "Destructive disabled" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| {}, "Secondary" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Lg, onclick: move |_| {}, "Outline lg" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| {}, "Outline sm" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Ghost, onclick: move |_| {}, "Ghost" }
                v_gap { height: spacing::XL }
                Button { variant: ButtonVariant::Link, size: ButtonSize::Sm, onclick: move |_| {}, "Link sm" }
            }
        },
        "calendar" => rsx! {
            column {
                width: "100%",
                Calendar {
                    selected: calendar_selected(),
                    year_range: CalendarYearRange::new(1900, 2100),
                    plugins: vec![lunar_calendar_plugin, memo_calendar_plugin],
                    on_day_press: move |date| calendar_selected.set(Some(date)),
                }
                v_gap { height: spacing::SM }
                text {
                    content: calendar_plugin_status(),
                    font_size: typography::XS,
                    font_color: theme.colors.muted_foreground,
                    line_height: 16.0,
                }
                v_gap { height: spacing::XXL }
                Calendar {
                    selected_dates: calendar_selected_dates(),
                    selection_color: Some(0xFFF97316u32),
                    today_color: Some(0xFFF97316u32),
                    on_day_press: move |date: String| {
                        let mut dates = calendar_selected_dates();
                        if let Some(index) = dates.iter().position(|selected| selected == &date) {
                            dates.remove(index);
                        } else {
                            dates.push(date);
                        }
                        calendar_selected_dates.set(dates);
                    },
                }
            }
        },
        "card" => rsx! {
            fixed_width {
                width: 384.0,
                Card {
                    CardHeader {
                        title: "Card Title".to_string(),
                        description: "Card Description".to_string(),
                    }
                    CardContent {
                        Text { content: "Card Content".to_string() }
                    }
                    CardFooter {
                        Text { content: "Card Footer".to_string() }
                    }
                }
            }
        },
        "carousel" => {
            let slides = [
                (
                    "mountain",
                    "Plan your next escape",
                    "Save places and build a trip that fits your pace.",
                ),
                (
                    "compass",
                    "Find something nearby",
                    "Explore hand-picked places, food, and experiences.",
                ),
                (
                    "calendar",
                    "Keep plans together",
                    "Dates, bookings, and reminders stay in one place.",
                ),
                (
                    "map-pin",
                    "Navigate with confidence",
                    "Get the details you need before heading out.",
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (icon, title, description))| {
                rsx! {
                    column {
                        width: "100%",
                        height: "100%",
                        align_items: "center",
                        justify_content: "center",
                        padding_top: spacing::XXL,
                        padding_right: spacing::XXL,
                        padding_bottom: spacing::XXL,
                        padding_left: spacing::XXL,
                        background_color: theme.colors.card,
                        {icon_placeholder(icon, 42.0, theme.colors.primary)}
                        v_gap { height: spacing::XL }
                        text {
                            content: title,
                            font_size: typography::XL,
                            font_weight: 600_i32,
                            font_color: theme.colors.card_foreground,
                            line_height: 28.0,
                            text_align: "center",
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: description,
                            font_size: typography::SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                            text_align: "center",
                        }
                        v_gap { height: spacing::XL }
                        text {
                            content: format!("{} / 4", index + 1),
                            font_size: typography::XS,
                            font_weight: 500_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 16.0,
                        }
                    }
                }
            })
            .collect();
            let make_overlay_slides = || {
                ["First", "Second", "Third", "Fourth"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        rsx! {
                            column {
                                width: "100%",
                                height: "100%",
                                align_items: "center",
                                justify_content: "center",
                                background_color: theme.colors.muted,
                                text {
                                    content: (index + 1).to_string(),
                                    font_size: 64.0,
                                    font_weight: 700_i32,
                                    font_color: theme.colors.foreground,
                                    line_height: 72.0,
                                }
                                v_gap { height: spacing::SM }
                                text {
                                    content: label,
                                    font_size: typography::SM,
                                    font_weight: 500_i32,
                                    font_color: theme.colors.muted_foreground,
                                    line_height: 20.0,
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            };
            let overlay_slides = make_overlay_slides();
            let overlay_slides_uc = make_overlay_slides();

            rsx! {
                fixed_width {
                    width: 336.0,
                    column {
                        width: "100%",
                        demo_mode_label {
                            title: "Controlled".to_string(),
                            detail: Some(format!(
                                "index = {}, overlay = {}",
                                carousel_index(),
                                carousel_overlay_index()
                            )),
                        }
                        Carousel {
                            slides,
                            index: Some(carousel_index()),
                            height: 280.0,
                            indicator_variant: CarouselIndicatorVariant::Pill,
                            style: CarouselStyle {
                                viewport_radius: Some(theme.radii.xxl),
                                navigation_background: Some(theme.colors.primary),
                                navigation_foreground: Some(theme.colors.primary_foreground),
                                indicator_active_color: Some(theme.colors.primary),
                                ..CarouselStyle::default()
                            },
                            on_change: move |index| carousel_index.set(index),
                        }
                        v_gap { height: spacing::LG }
                        Carousel {
                            slides: overlay_slides,
                            index: Some(carousel_overlay_index()),
                            height: 180.0,
                            show_indicators: false,
                            controls_placement: CarouselControlsPlacement::OverlayCenter,
                            style: CarouselStyle {
                                viewport_radius: Some(theme.radii.xxl),
                                viewport_border_width: 1.0,
                                navigation_background: Some(theme.colors.background),
                                navigation_foreground: Some(theme.colors.foreground),
                                ..CarouselStyle::default()
                            },
                            on_change: move |index| carousel_overlay_index.set(index),
                        }
                        {demo_mode_divider()}
                        demo_mode_label {
                            title: "Uncontrolled".to_string(),
                            detail: Some(carousel_uc_note()),
                        }
                        Carousel {
                            slides: overlay_slides_uc,
                            default_index: 0,
                            height: 160.0,
                            style: CarouselStyle {
                                viewport_radius: Some(theme.radii.xxl),
                                ..CarouselStyle::default()
                            },
                            on_change: move |index| {
                                carousel_uc_note.set(format!("on_change = {index}"));
                            },
                        }
                    }
                }
            }
        }
        "checkbox" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!(
                            "a={}, b={}, card={}",
                            checkbox_first(),
                            checkbox_second(),
                            checkbox_card()
                        )),
                    }
                    Checkbox {
                        label: Some("Accept terms and conditions".to_string()),
                        checked: Some(checkbox_first()),
                        on_change: Some(EventHandler::new(move |value| checkbox_first.set(value))),
                    }
                    v_gap { height: spacing::LG }
                    Checkbox {
                        label: Some("Marketing emails".to_string()),
                        checked: Some(checkbox_second()),
                        on_change: Some(EventHandler::new(move |value| checkbox_second.set(value))),
                    }
                    v_gap { height: spacing::LG }
                    row {
                        width: "100%",
                        align_items: "start",
                        padding: 12.0,
                        border_width: 1.0,
                        border_color: if checkbox_card() { 0xFF2563EBu32 } else { theme.colors.border },
                        border_radius: theme.radii.lg,
                        background_color: if checkbox_card() { 0xFFEFF6FFu32 } else { theme.colors.background },
                        onclick: move |_| checkbox_card.set(!checkbox_card()),
                        Checkbox {
                            label: None,
                            checked: Some(checkbox_card()),
                            checked_color: Some(0xFF2563EBu32),
                            on_change: Some(EventHandler::new(move |value| checkbox_card.set(value))),
                        }
                        h_gap { width: 12.0 }
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            Text { content: "Enable notifications".to_string(), variant: TextVariant::Small }
                            Text { content: "You can enable or disable anytime.".to_string(), variant: TextVariant::Muted }
                        }
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some("default_checked=true; parent only observes on_change".to_string()),
                    }
                    Checkbox {
                        label: Some("Remember me".to_string()),
                        default_checked: Some(true),
                        on_change: Some(EventHandler::new(move |value| checkbox_uc_label.set(value))),
                    }
                    v_gap { height: spacing::SM }
                    Text {
                        content: format!("last on_change = {}", checkbox_uc_label()),
                        variant: TextVariant::Muted,
                    }
                    v_gap { height: spacing::LG }
                    Checkbox {
                        label: Some("Disabled".to_string()),
                        default_checked: Some(false),
                        disabled: Some(true),
                    }
                }
            }
        },
        "collapsible" => rsx! {
            fixed_width {
                width: 350.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("open = {}", collapsible_open())),
                    }
                    Collapsible {
                        title: "@peduarte starred 3 repositories".to_string(),
                        open: Some(collapsible_open()),
                        on_open_change: EventHandler::new(move |value| collapsible_open.set(value)),
                        column {
                            width: "100%",
                            repo_row { name: "@radix-ui/primitives".to_string() }
                            v_gap { height: spacing::SM }
                            repo_row { name: "@radix-ui/react".to_string() }
                            v_gap { height: spacing::SM }
                            repo_row { name: "@stitches/core".to_string() }
                        }
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(collapsible_uc_note()),
                    }
                    Collapsible {
                        title: "Uncontrolled collapsible".to_string(),
                        default_open: true,
                        on_open_change: EventHandler::new(move |value| {
                            collapsible_uc_note.set(format!("on_open_change = {value}"));
                        }),
                        column {
                            width: "100%",
                            repo_row { name: "@radix-ui/primitives".to_string() }
                            v_gap { height: spacing::SM }
                            repo_row { name: "@radix-ui/react".to_string() }
                            v_gap { height: spacing::SM }
                            repo_row { name: "@stitches/core".to_string() }
                        }
                    }
                }
            }
        },
        "context-menu" => rsx! {
            fixed_width {
                width: 300.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("open = {}", context_open())),
                    }
                    ContextMenu {
                        open: Some(context_open()),
                        default_open: false,
                        on_open_change: move |value| context_open.set(value),
                        width: Some(288.0),
                        items: context_menu_items(
                            context_bookmarks(),
                            context_full_urls(),
                            context_person(),
                            EventHandler::new(move |value| context_bookmarks.set(value)),
                            EventHandler::new(move |value| context_full_urls.set(value)),
                            EventHandler::new(move |value| context_person.set(value)),
                        ),
                        stack {
                            width: 300.0,
                            height: 120.0,
                            alignment: "center",
                            border_width: 1.0,
                            border_color: theme.colors.foreground,
                            border_radius: theme.radii.md,
                            border_style: "dashed",
                            clip: true,
                            text {
                                content: "Long press (controlled)".to_string(),
                                font_size: typography::LG,
                                font_color: theme.colors.foreground,
                                line_height: 22.0,
                            }
                        }
                    }
                    v_gap { height: spacing::MD }
                    Button {
                        variant: ButtonVariant::Outline,
                        width: "100%",
                        onclick: move |_| context_outside_clicks += 1,
                        "Outside click · {context_outside_clicks()}"
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(context_uc_note()),
                    }
                    ContextMenu {
                        default_open: false,
                        on_open_change: move |value| {
                            context_uc_note.set(format!("on_open_change = {value}"));
                        },
                        width: Some(288.0),
                        items: context_menu_items(
                            context_bookmarks(),
                            context_full_urls(),
                            context_person(),
                            EventHandler::new(move |value| context_bookmarks.set(value)),
                            EventHandler::new(move |value| context_full_urls.set(value)),
                            EventHandler::new(move |value| context_person.set(value)),
                        ),
                        stack {
                            width: 300.0,
                            height: 100.0,
                            alignment: "center",
                            border_width: 1.0,
                            border_color: theme.colors.border,
                            border_radius: theme.radii.md,
                            border_style: "dashed",
                            clip: true,
                            text {
                                content: "Long press (uncontrolled)".to_string(),
                                font_size: typography::MD,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                        }
                    }
                }
            }
        },
        "date-picker" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!(
                        "selected = {:?}, open = {}",
                        date_picker_selected(),
                        date_picker_open()
                    )),
                }
                DatePicker {
                    selected: date_picker_selected(),
                    open: Some(date_picker_open()),
                    calendar_year_range: CalendarYearRange::new(1900, 2100),
                    calendar_plugins: vec![lunar_calendar_plugin],
                    on_change: move |date| date_picker_selected.set(date),
                    on_open_change: move |open| date_picker_open.set(open),
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled open".to_string(),
                    detail: Some(date_picker_uc_note()),
                }
                DatePicker {
                    selected: date_picker_uc_selected(),
                    default_open: false,
                    calendar_year_range: CalendarYearRange::new(1900, 2100),
                    calendar_plugins: vec![lunar_calendar_plugin],
                    on_change: move |date: Option<String>| {
                        date_picker_uc_selected.set(date.clone());
                        date_picker_uc_note.set(format!("on_change = {:?}", date));
                    },
                    on_open_change: move |open| {
                        date_picker_uc_note.set(format!("on_open_change = {open}"));
                    },
                }
            }
        },
        "time-picker" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled · 24 hour".to_string(),
                    detail: Some(format!(
                        "selected = {:?}, open = {}",
                        time_picker_selected(),
                        time_picker_open()
                    )),
                }
                TimePicker {
                    selected: time_picker_selected(),
                    open: Some(time_picker_open()),
                    minute_step: 5,
                    on_change: move |time| time_picker_selected.set(time),
                    on_open_change: move |open| time_picker_open.set(open),
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled · 12 hour".to_string(),
                    detail: Some(time_picker_uc_note()),
                }
                TimePicker {
                    default_selected: TimeValue::new(15, 30),
                    format: TimePickerFormat::TwelveHour,
                    minute_step: 15,
                    default_open: false,
                    on_change: move |time| {
                        time_picker_uc_note.set(format!("on_change = {:?}", time));
                    },
                    on_open_change: move |open| {
                        time_picker_uc_note.set(format!("on_open_change = {open}"));
                    },
                }
            }
        },
        "timeline" => {
            let orientation = timeline_orientation();
            let align = timeline_align();
            let orientation_label = match orientation {
                TimelineOrientation::Vertical => "Vertical",
                TimelineOrientation::Horizontal => "Horizontal",
            };
            let align_label = match align {
                TimelineAlign::Right => "Right",
                TimelineAlign::Left => "Left",
                TimelineAlign::Alternate => "Alternate",
            };
            let item_min_width = match orientation {
                TimelineOrientation::Horizontal => Some(148.0),
                TimelineOrientation::Vertical => None,
            };
            rsx! {
                fixed_width {
                    width: 360.0,
                    column {
                        width: "100%",
                        align_items: "start",
                        demo_mode_label {
                            title: "Orientation".to_string(),
                            detail: Some(format!("orientation = {orientation_label}")),
                        }
                        ToggleGroup {
                            options: vec!["Vertical".to_string(), "Horizontal".to_string()],
                            selected: Some(vec![orientation_label.to_string()]),
                            width: Some("100%".to_string()),
                            on_change: move |values: Vec<String>| {
                                let horizontal = values.iter().any(|value| value == "Horizontal");
                                timeline_orientation.set(if horizontal {
                                    TimelineOrientation::Horizontal
                                } else {
                                    TimelineOrientation::Vertical
                                });
                            },
                        }
                        v_gap { height: spacing::LG }
                        demo_mode_label {
                            title: "Align".to_string(),
                            detail: Some(format!("align = {align_label}")),
                        }
                        ToggleGroup {
                            options: vec![
                                "Right".to_string(),
                                "Left".to_string(),
                                "Alternate".to_string(),
                            ],
                            selected: Some(vec![align_label.to_string()]),
                            width: Some("100%".to_string()),
                            on_change: move |values: Vec<String>| {
                                let next = if values.iter().any(|value| value == "Left") {
                                    TimelineAlign::Left
                                } else if values.iter().any(|value| value == "Alternate") {
                                    TimelineAlign::Alternate
                                } else {
                                    TimelineAlign::Right
                                };
                                timeline_align.set(next);
                            },
                        }
                        {demo_mode_divider()}
                        demo_mode_label {
                            title: "Controlled".to_string(),
                            detail: Some(format!("value = {}", timeline_step())),
                        }
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "start",
                            for step in 0..=4 {
                                {
                                    let selected = timeline_step() == step;
                                    rsx! {
                                        row {
                                            margin_right: spacing::SM,
                                            Button {
                                                variant: if selected {
                                                    ButtonVariant::Default
                                                } else {
                                                    ButtonVariant::Outline
                                                },
                                                size: ButtonSize::Sm,
                                                onclick: move |_| timeline_step.set(step),
                                                "{step}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        v_gap { height: spacing::LG }
                        Timeline {
                            value: Some(timeline_step()),
                            orientation,
                            align,
                            item_min_width,
                            interactive: true,
                            on_value_change: move |step| timeline_step.set(step),
                            TimelineItem {
                                step: 1,
                                date: Some("Mar 15, 2024".to_string()),
                                title: Some("Project Kickoff".to_string()),
                                description: Some("Initial team meeting and project scope definition.".to_string()),
                            }
                            TimelineItem {
                                step: 2,
                                date: Some("Mar 22, 2024".to_string()),
                                title: Some("Design Phase".to_string()),
                                description: Some("Wireframes, mockups, and stakeholder review.".to_string()),
                            }
                            TimelineItem {
                                step: 3,
                                date: Some("Apr 5, 2024".to_string()),
                                title: Some("Development Sprint".to_string()),
                                description: Some("API implementation and frontend components.".to_string()),
                            }
                            TimelineItem {
                                step: 4,
                                last: true,
                                date: Some("Apr 19, 2024".to_string()),
                                title: Some("Testing & Deployment".to_string()),
                                description: Some("QA, performance passes, and production rollout.".to_string()),
                            }
                        }
                        {demo_mode_divider()}
                        demo_mode_label {
                            title: "Uncontrolled · icons".to_string(),
                            detail: Some(timeline_uc_note()),
                        }
                        Timeline {
                            default_value: Some(2),
                            orientation,
                            align,
                            item_min_width,
                            interactive: true,
                            on_value_change: move |step| {
                                timeline_uc_note.set(format!("on_value_change = {step}"));
                            },
                            TimelineItem {
                                step: 1,
                                icon: Some("rocket".to_string()),
                                title: Some("Repository created".to_string()),
                                description: Some("Scaffolded the app and CI pipeline.".to_string()),
                            }
                            TimelineItem {
                                step: 2,
                                icon: Some("palette".to_string()),
                                title: Some("Design system".to_string()),
                                description: Some("Tokens, radii, and component primitives.".to_string()),
                            }
                            TimelineItem {
                                step: 3,
                                icon: Some("code".to_string()),
                                title: Some("Feature work".to_string()),
                                description: Some("Timeline rail, indicators, and orientations.".to_string()),
                            }
                            TimelineItem {
                                step: 4,
                                last: true,
                                icon: Some("circle-check".to_string()),
                                title: Some("Shipped".to_string()),
                                description: Some("Released in the shadcn crate.".to_string()),
                            }
                        }
                    }
                }
            }
        }
        "index" => {
            let large = index_large();
            let show_empty = index_show_empty();
            let custom = index_custom();
            let items = if large {
                index_contact_items(800)
            } else {
                index_city_items()
            };
            let count = items.len();
            rsx! {
                column {
                    width: "100%",
                    height: "100%",
                    align_items: "start",
                    demo_mode_label {
                        title: "Index".to_string(),
                        detail: Some(index_note()),
                    }
                    ToggleGroup {
                        options: vec!["Cities".to_string(), "800 contacts".to_string()],
                        selected: Some(vec![if large {
                            "800 contacts".to_string()
                        } else {
                            "Cities".to_string()
                        }]),
                        width: Some("100%".to_string()),
                        on_change: move |values: Vec<String>| {
                            let next_large = values.iter().any(|value| value == "800 contacts");
                            index_large.set(next_large);
                            index_note.set(index_demo_note(next_large, index_show_empty(), index_custom()));
                        },
                    }
                    v_gap { height: spacing::SM }
                    ToggleGroup {
                        options: vec!["Hide empty".to_string(), "Show empty".to_string()],
                        selected: Some(vec![if show_empty {
                            "Show empty".to_string()
                        } else {
                            "Hide empty".to_string()
                        }]),
                        width: Some("100%".to_string()),
                        on_change: move |values: Vec<String>| {
                            let next = values.iter().any(|value| value == "Show empty");
                            index_show_empty.set(next);
                            index_note.set(index_demo_note(index_large(), next, index_custom()));
                        },
                    }
                    v_gap { height: spacing::SM }
                    ToggleGroup {
                        options: vec!["Default UI".to_string(), "Custom UI".to_string()],
                        selected: Some(vec![if custom {
                            "Custom UI".to_string()
                        } else {
                            "Default UI".to_string()
                        }]),
                        width: Some("100%".to_string()),
                        on_change: move |values: Vec<String>| {
                            let next = values.iter().any(|value| value == "Custom UI");
                            index_custom.set(next);
                            index_note.set(index_demo_note(index_large(), index_show_empty(), next));
                        },
                    }
                    v_gap { height: spacing::SM }
                    column {
                        width: "100%",
                        layout_weight: 1.0,
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.md,
                        clip: true,
                        Index {
                            items,
                            show_empty_indexes: show_empty,
                            render_item: custom.then_some(index_render_item),
                            render_header: custom.then_some(index_render_header),
                            render_bar: custom.then_some(index_render_bar),
                            on_select: move |item_index| {
                                index_note.set(format!("on_select = {item_index} / {count}"));
                            },
                            on_index_change: move |letter| {
                                index_note.set(format!("index = {letter}"));
                            },
                        }
                    }
                }
            }
        }
        "dialog" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("open = {}", dialog_open())),
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| dialog_open.set(true),
                    "Edit Profile"
                }
            }
            Dialog {
                open: Some(dialog_open()),
                default_open: Some(false),
                on_close: move |_| dialog_open.set(false),
                DialogHeader {
                    title: "Edit profile and manage account preferences".to_string(),
                    description: Some("Make changes to your profile here. Click save when you're done.".to_string()),
                }
                column {
                    width: "100%",
                    align_items: "start",
                    margin_top: spacing::XL,
                    Label { content: "Name".to_string() }
                    v_gap { height: spacing::SM }
                    Input {
                        value: Some(dialog_name()),
                        placeholder: Some("Your name".to_string()),
                        width: "100%",
                        on_change: move |value| dialog_name.set(value),
                    }
                    v_gap { height: spacing::LG }
                    Label { content: "Username".to_string() }
                    v_gap { height: spacing::SM }
                    Input {
                        value: Some(dialog_username()),
                        placeholder: Some("@username".to_string()),
                        width: "100%",
                        on_change: move |value| dialog_username.set(value),
                    }
                }
                DialogFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| dialog_open.set(false),
                        "Cancel"
                    }
                    row { width: spacing::SM }
                    Button {
                        onclick: move |_| dialog_open.set(false),
                        "Save changes"
                    }
                }
            }
            {demo_mode_divider()}
            demo_mode_label {
                title: "Uncontrolled".to_string(),
                detail: Some("default_open via remount key".to_string()),
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| dialog_uc_gen.set(dialog_uc_gen() + 1),
                "Open Uncontrolled Dialog"
            }
            if dialog_uc_gen() > 0 {
                Dialog {
                    key: "{dialog_uc_gen()}",
                    default_open: Some(true),
                    on_close: move |_| {},
                    DialogHeader {
                        title: "Uncontrolled".to_string(),
                        description: Some("Opened with default_open; dismiss uses internal state.".to_string()),
                    }
                    DialogFooter {
                        Button {
                            onclick: move |_| {},
                            "Close"
                        }
                    }
                }
            }
        },
        "dropdown-menu" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("open = {}", menu_open())),
                    }
                    DropdownMenu {
                        open: Some(menu_open()),
                        default_open: false,
                        on_open_change: Some(EventHandler::new(move |value| menu_open.set(value))),
                        width: Some(288.0),
                        items: dropdown_menu_items(),
                        Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Open" }
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(menu_uc_note()),
                    }
                    DropdownMenu {
                        default_open: false,
                        on_open_change: Some(EventHandler::new(move |value| {
                            menu_uc_note.set(format!("on_open_change = {value}"));
                        })),
                        width: Some(288.0),
                        items: dropdown_menu_items(),
                        Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Open (uncontrolled)" }
                    }
                }
            }
        },
        "form" => rsx! {
            fixed_width {
                width: 440.0,
                column {
                    width: "100%",
                    height: 1110.0,
                    align_items: "start",
                    text {
                        content: "Account settings".to_string(),
                        width: "100%",
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                        text_align: "start",
                    }
                    v_gap { height: spacing::SM }
                    text {
                        content: "Update your public profile and communication preferences.".to_string(),
                        width: "100%",
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: "start",
                    }
                    v_gap { height: spacing::XXL }
                    Form {
                        submit_label: "Save changes".to_string(),
                        on_submit: move |_| {
                            form_attempted.set(true);
                            let valid = form_name().trim().chars().count() >= 2
                                && is_valid_email(form_email().as_str())
                                && form_terms_accepted();
                            form_status.set(Some(valid));
                        },
                        FieldSet {
                            arkit_shadcn::components::FieldLegend {
                                content: "Profile".to_string(),
                            }
                            FieldDescription {
                                content: "This information is visible to people you collaborate with.".to_string(),
                                inset: false,
                            }
                            v_gap { height: spacing::XL }
                            FieldGroup {
                                FormItem {
                                    label: "Display name".to_string(),
                                    required: true,
                                    description: Some("Use the name your teammates know you by.".to_string()),
                                    error: if form_name_invalid { Some("Enter at least two characters.".to_string()) } else { None },
                                    Input {
                                        value: Some(form_name()),
                                        placeholder: Some("Your name".to_string()),
                                        width: "100%",
                                        invalid: form_name_invalid,
                                        on_change: move |value| {
                                            form_name.set(value);
                                            form_status.set(None);
                                        },
                                    }
                                }
                                FormItem {
                                    label: "Email address".to_string(),
                                    required: true,
                                    description: Some("We only use this for account and security updates.".to_string()),
                                    error: if form_email_invalid { Some("Enter a valid email address.".to_string()) } else { None },
                                    Input {
                                        value: Some(form_email()),
                                        placeholder: Some("name@example.com".to_string()),
                                        width: "100%",
                                        invalid: form_email_invalid,
                                        on_change: move |value| {
                                            form_email.set(value);
                                            form_status.set(None);
                                        },
                                    }
                                }
                                FormItem {
                                    label: "Bio".to_string(),
                                    description: Some("Keep it short. You can change this anytime.".to_string()),
                                    Textarea {
                                        value: Some(form_bio()),
                                        placeholder: Some("Tell people a little about yourself.".to_string()),
                                        height: Some(96.0),
                                        width: "100%",
                                        on_change: move |value| {
                                            form_bio.set(value);
                                            form_status.set(None);
                                        },
                                    }
                                }
                            }
                        }
                        FieldSeparator { label: Some("Preferences".to_string()) }
                        Field {
                            orientation: FieldOrientation::Horizontal,
                            FieldContent {
                                FieldTitle { content: "Product updates".to_string() }
                                FieldDescription {
                                    content: "Receive occasional release notes and feature tips.".to_string(),
                                }
                            }
                            Switch {
                                checked: Some(form_product_updates()),
                                on_change: move |value| {
                                    form_product_updates.set(value);
                                    form_status.set(None);
                                },
                            }
                        }
                        Field {
                            invalid: form_terms_invalid,
                            Checkbox {
                                label: Some("Terms and privacy".to_string()),
                                checked: Some(form_terms_accepted()),
                                checked_color: if form_terms_invalid { Some(theme.colors.destructive) } else { None },
                                on_change: move |value| {
                                    form_terms_accepted.set(value);
                                    form_status.set(None);
                                },
                            }
                            row {
                                width: "100%",
                                margin_top: spacing::XS,
                                padding_left: spacing::XXL,
                                FieldDescription {
                                    content: "I agree to the terms of service and privacy policy.".to_string(),
                                    inset: false,
                                }
                            }
                        }
                        if form_terms_invalid {
                            FieldError { message: Some("Accept the terms before saving.".to_string()) }
                            v_gap { height: spacing::LG }
                        }
                        if let Some(success) = form_status() {
                            row {
                                width: "100%",
                                align_items: "center",
                                justify_content: "start",
                                margin_bottom: spacing::LG,
                                padding_top: spacing::MD,
                                padding_right: spacing::MD,
                                padding_bottom: spacing::MD,
                                padding_left: spacing::MD,
                                background_color: arkit_shadcn::theme::with_alpha(
                                    if success { theme.colors.chart_2 } else { theme.colors.destructive },
                                    0x18,
                                ),
                                border_radius: theme.radii.md,
                                {icon_placeholder(
                                    if success { "circle-check" } else { "circle-alert" },
                                    18.0,
                                    if success { theme.colors.chart_2 } else { theme.colors.destructive },
                                )}
                                h_gap { width: spacing::SM }
                                row {
                                    layout_weight: 1.0,
                                    text {
                                        content: if success {
                                            "Your account settings were saved.".to_string()
                                        } else {
                                            "Review the highlighted fields and try again.".to_string()
                                        },
                                        width: "100%",
                                        font_size: typography::XS,
                                        font_weight: 500_i32,
                                        font_color: if success { theme.colors.chart_2 } else { theme.colors.destructive },
                                        line_height: 18.0,
                                        text_align: "start",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "guide" => rsx! {
            fixed_width {
                width: 420.0,
                Guide {
                    steps: vec![
                        GuideStep::new(
                            "guide-profile",
                            "Your workspace",
                            "This summary keeps the active project and its recent activity in one place.",
                        )
                        .side(GuideSide::Bottom),
                        GuideStep::new(
                            "guide-search",
                            "Find anything",
                            "Search across components, examples, and documentation without leaving the page.",
                        )
                        .side(GuideSide::Bottom),
                        GuideStep::new(
                            "guide-settings",
                            "Tune the experience",
                            "Open preferences to change appearance, notifications, and workspace defaults.",
                        )
                        .side(GuideSide::Top),
                    ],
                    open: Some(guide_open()),
                    step: Some(guide_step()),
                    on_open_change: move |open| guide_open.set(open),
                    on_step_change: move |step| {
                        guide_step.set(step);
                        guide_status.set(format!("Showing step {} of 3.", step + 1));
                    },
                    on_skip: move |_| {
                        guide_status.set("Guide skipped. You can restart it anytime.".to_string());
                    },
                    on_finish: move |_| {
                        guide_status.set("Guide completed.".to_string());
                    },
                    column {
                        width: "100%",
                        align_items: "start",
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "space_between",
                            column {
                                align_items: "start",
                                text {
                                    content: "Product tour".to_string(),
                                    font_size: typography::XXL,
                                    font_weight: 700_i32,
                                    font_color: theme.colors.foreground,
                                    line_height: 32.0,
                                }
                                text {
                                    content: guide_status(),
                                    font_size: typography::XS,
                                    font_color: theme.colors.muted_foreground,
                                    line_height: 18.0,
                                }
                            }
                            Button {
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    guide_step.set(0);
                                    guide_status.set("Showing step 1 of 3.".to_string());
                                    guide_open.set(true);
                                },
                                "Start guide"
                            }
                        }
                        v_gap { height: spacing::XXL }
                        GuideTarget {
                            id: "guide-profile".to_string(),
                            render: move |target_ref| rsx! {
                                Card {
                                    native_ref: target_ref,
                                    CardHeader {
                                        title: "Arkit workspace".to_string(),
                                        description: "12 components updated this week".to_string(),
                                    }
                                    CardContent {
                                        row {
                                            width: "100%",
                                            align_items: "center",
                                            justify_content: "space_between",
                                            Badge {
                                                content: "Active".to_string(),
                                                variant: BadgeVariant::Secondary,
                                            }
                                            text {
                                                content: "Last opened today".to_string(),
                                                font_size: typography::XS,
                                                font_color: theme.colors.muted_foreground,
                                                line_height: 18.0,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        v_gap { height: spacing::XL }
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "space_between",
                            GuideTarget {
                                id: "guide-search".to_string(),
                                render: move |target_ref| rsx! {
                                    Button {
                                        native_ref: target_ref,
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {},
                                        row {
                                            align_items: "center",
                                            {icon_placeholder("search", 18.0, theme.colors.foreground)}
                                            h_gap { width: spacing::SM }
                                            "Search"
                                        }
                                    }
                                }
                            }
                            GuideTarget {
                                id: "guide-settings".to_string(),
                                render: move |target_ref| rsx! {
                                    Button {
                                        native_ref: target_ref,
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {},
                                        row {
                                            align_items: "center",
                                            {icon_placeholder("settings-2", 18.0, theme.colors.foreground)}
                                            h_gap { width: spacing::SM }
                                            "Preferences"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "hover-card" => rsx! {
            fixed_width {
                width: 320.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("open = {}", hover_open())),
                    }
                    HoverCard {
                        open: Some(hover_open()),
                        default_open: Some(false),
                        on_close: move |_| hover_open.set(false),
                        on_open_change: move |value| hover_open.set(value),
                        width: Some(320.0),
                        trigger: rsx! { Button { variant: ButtonVariant::Link, onclick: move |_| {}, "@expo" } },
                        row {
                            width: "100%",
                            align_items: "start",
                            {demo_avatar("https://github.com/expo.png", "E", false, None)}
                            h_gap { width: spacing::LG }
                            column {
                                layout_weight: 1.0,
                                align_items: "start",
                                text {
                                    content: "@expo".to_string(),
                                    font_size: typography::SM,
                                    font_weight: 600_i32,
                                    font_color: theme.colors.foreground,
                                    line_height: 20.0,
                                }
                                Text {
                                    content: "Framework and tools for creating native apps with React.".to_string(),
                                    variant: TextVariant::Muted,
                                }
                            }
                        }
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(hover_uc_note()),
                    }
                    HoverCard {
                        default_open: Some(false),
                        on_open_change: move |value| {
                            hover_uc_note.set(format!("on_open_change = {value}"));
                        },
                        width: Some(280.0),
                        trigger: rsx! { Button { variant: ButtonVariant::Link, onclick: move |_| {}, "@shadcn" } },
                        Text {
                            content: "Hover card with internal open state.".to_string(),
                            variant: TextVariant::Muted,
                        }
                    }
                }
            }
        },
        "icon" => rsx! {
            fixed_width {
                width: 352.0,
                column {
                    width: 352.0,
                    align_items: "start",
                    row {
                        align_self: "start",
                        justify_content: "start",
                        IconTile { name: "mail".to_string(), color: theme.colors.foreground }
                        h_gap { width: spacing::MD }
                        IconTile { name: "chevron-right".to_string(), color: theme.colors.foreground }
                        h_gap { width: spacing::MD }
                        IconTile { name: "search".to_string(), color: theme.colors.foreground }
                    }
                    v_gap { height: spacing::LG }
                    row {
                        align_self: "start",
                        justify_content: "start",
                        IconTile { name: "bell-off".to_string(), color: 0xFFEF4444u32 }
                        h_gap { width: spacing::MD }
                        IconTile { name: "star".to_string(), color: 0xFFF59E0Bu32, size: Some(24.0) }
                        h_gap { width: spacing::MD }
                        IconTile { name: "settings-2".to_string(), color: theme.colors.foreground }
                    }
                }
            }
        },
        "input" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("value = {:?}", input_controlled())),
                    }
                    Input {
                        placeholder: Some("Email".to_string()),
                        value: Some(input_controlled()),
                        width: "100%",
                        on_change: Some(EventHandler::new(move |value| input_controlled.set(value))),
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some("omit value — native field owns text".to_string()),
                    }
                    Input {
                        placeholder: Some("Uncontrolled input".to_string()),
                        width: "100%",
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Password".to_string(),
                        detail: Some(format!("{} characters", input_password().chars().count())),
                    }
                    Input {
                        mode: InputMode::Password,
                        placeholder: Some("Password".to_string()),
                        value: Some(input_password()),
                        width: "100%",
                        on_change: move |value| input_password.set(value),
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Number".to_string(),
                        detail: Some(format!("value = {:?}", input_number())),
                    }
                    Input {
                        mode: InputMode::Number,
                        placeholder: Some("Digits only".to_string()),
                        value: Some(input_number()),
                        width: "100%",
                        on_change: move |value| input_number.set(value),
                    }
                }
            }
        },
        "input-otp" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    width: "100%",
                    align_items: "start",
                    text {
                        width: "100%",
                        content: "Verify your email".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "Enter the verification code sent to m@example.com.".to_string(),
                        font_size: typography::SM,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        content: "Verification code".to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::SM }
                    InputOtp {
                        value: otp_value(),
                        digits: 6,
                        invalid: otp_invalid(),
                        on_change: move |value: String| {
                            otp_invalid.set(false);
                            otp_status.set(format!("{} of 6 digits entered.", value.chars().count()));
                            otp_value.set(value);
                        },
                        on_complete: move |_: String| {
                            otp_status.set("Code complete. Ready to verify.".to_string());
                        },
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: otp_status(),
                        font_size: typography::XS,
                        font_weight: 400_i32,
                        font_color: if otp_invalid() {
                            theme.colors.destructive
                        } else {
                            theme.colors.muted_foreground
                        },
                        line_height: 18.0,
                    }
                    v_gap { height: spacing::XL }
                    Button {
                        width: "100%",
                        disabled: Some(otp_value().chars().count() != 6),
                        onclick: move |_| {
                            if otp_value() == "246810" {
                                otp_status.set("Code verified successfully.".to_string());
                            } else {
                                otp_invalid.set(true);
                                otp_status.set("That code does not match. Try 246810.".to_string());
                            }
                        },
                        "Verify code"
                    }
                    Button {
                        variant: ButtonVariant::Link,
                        width: "100%",
                        onclick: move |_| {
                            otp_value.set(String::new());
                            otp_invalid.set(false);
                            otp_status.set("A new code was sent.".to_string());
                        },
                        "Resend code"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        content: "Alphanumeric code".to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    text {
                        content: "Visual caret hidden".to_string(),
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                    v_gap { height: spacing::SM }
                    InputOtp {
                        value: invite_code(),
                        digits: 4,
                        mode: InputOtpMode::Alphanumeric,
                        group_size: 0,
                        separator: InputOtpSeparator::None,
                        show_caret: false,
                        on_change: move |value: String| invite_code.set(value),
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        content: "Disabled".to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::SM }
                    InputOtp {
                        value: "123456".to_string(),
                        digits: 6,
                        disabled: true,
                    }
                }
            }
        },
        "secure-keyboard" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    width: "100%",
                    align_items: "start",
                    text {
                        width: "100%",
                        content: "Confirm payment PIN".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "This keypad is rendered inside the app and never opens the system input method.".to_string(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::XXL }
                    Input {
                        value: Some("•".repeat(secure_pin().chars().count())),
                        placeholder: Some("Tap to enter payment PIN".to_string()),
                        width: "100%",
                        read_only: true,
                        on_click: move |_| secure_keyboard_open.set(true),
                    }
                    SecureKeyboardSheet {
                        value: Some(secure_pin()),
                        open: Some(secure_keyboard_open()),
                        max_length: 6,
                        randomized: true,
                        on_change: move |value: String| {
                            secure_keyboard_status.set(format!(
                                "{} of 6 digits entered.",
                                value.chars().count(),
                            ));
                            secure_pin.set(value);
                        },
                        on_complete: move |_: String| {
                            secure_keyboard_status.set(
                                "Six digits entered. Press Done to submit.".to_string(),
                            );
                        },
                        on_confirm: move |value: String| {
                            secure_keyboard_status.set(format!(
                                "Submitted a {}-digit PIN without displaying it.",
                                value.chars().count(),
                            ));
                        },
                        on_open_change: move |open| secure_keyboard_open.set(open),
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "start",
                        {icon_placeholder("shield-check", 16.0, theme.colors.primary)}
                        h_gap { width: spacing::SM }
                        row {
                            layout_weight: 1.0,
                            text {
                                width: "100%",
                                content: secure_keyboard_status(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                    }
                    v_gap { height: spacing::MD }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: Some(secure_pin().is_empty()),
                        onclick: move |_| {
                            secure_pin.set(String::new());
                            secure_keyboard_status.set(
                                "PIN cleared from the controlled value.".to_string(),
                            );
                        },
                        "Clear from parent"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        width: "100%",
                        content: "Full secure text input".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "Use a familiar QWERTY layout with a digit row and a dedicated symbol page, without invoking the system input method.".to_string(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::LG }
                    Input {
                        value: Some(secure_text()),
                        placeholder: Some("Tap to enter secure text".to_string()),
                        width: "100%",
                        read_only: true,
                        on_click: move |_| secure_text_open.set(true),
                    }
                    SecureKeyboardSheet {
                        value: Some(secure_text()),
                        open: Some(secure_text_open()),
                        mode: SecureKeyboardMode::Full,
                        max_length: 20,
                        randomized: false,
                        confirm_requires_complete: false,
                        on_change: move |value: String| {
                            secure_text_status.set(format!(
                                "{} of 20 characters entered.",
                                value.chars().count(),
                            ));
                            secure_text.set(value);
                        },
                        on_complete: move |_: String| {
                            secure_text_status.set(
                                "The 20-character limit has been reached.".to_string(),
                            );
                        },
                        on_confirm: move |value: String| {
                            secure_text_status.set(format!(
                                "Submitted a {}-character value.",
                                value.chars().count(),
                            ));
                        },
                        on_open_change: move |open| secure_text_open.set(open),
                    }
                    v_gap { height: spacing::MD }
                    text {
                        width: "100%",
                        content: secure_text_status(),
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                    v_gap { height: spacing::MD }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: Some(secure_text().is_empty()),
                        onclick: move |_| {
                            secure_text.set(String::new());
                            secure_text_status.set(
                                "Secure text cleared from the controlled value.".to_string(),
                            );
                        },
                        "Clear secure text"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        width: "100%",
                        content: "Security boundary".to_string(),
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    text {
                        width: "100%",
                        content: "Randomized keys and IME avoidance reduce exposure, but this is not a hardware-backed trusted keyboard.".to_string(),
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                }
            }
        },
        "label" => rsx! {
            row {
                align_items: "center",
                justify_content: "start",
                onclick: move |_| toggle_pressed.set(!toggle_pressed()),
                Checkbox {
                    label: None,
                    checked: Some(toggle_pressed()),
                    on_change: Some(EventHandler::new(move |value| toggle_pressed.set(value))),
                }
                h_gap { width: spacing::SM }
                Label { content: "Accept terms and conditions".to_string() }
            }
        },
        "layout" => rsx! {
            fixed_width {
                width: 560.0,
                Col {
                    width: "100%",
                    demo_mode_label {
                        title: "Col defaults to top-left".to_string(),
                        detail: Some("No alignment attributes are required".to_string()),
                    }
                    Col {
                        width: "100%",
                        height: 144.0,
                        padding: spacing::MD,
                        background_color: arkit_shadcn::theme::with_alpha(
                            theme.colors.secondary,
                            0x80,
                        ),
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.lg,
                        {layout_demo_item("First", 88.0, 32.0, theme)}
                        v_gap { height: spacing::SM }
                        {layout_demo_item("Second", 120.0, 32.0, theme)}
                    }
                    v_gap { height: spacing::XXL }
                    demo_mode_label {
                        title: "Row defaults to top-left".to_string(),
                        detail: Some("Items start on both axes".to_string()),
                    }
                    Row {
                        width: "100%",
                        height: 104.0,
                        padding: spacing::MD,
                        background_color: arkit_shadcn::theme::with_alpha(
                            theme.colors.secondary,
                            0x80,
                        ),
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.lg,
                        {layout_demo_item("One", 72.0, 32.0, theme)}
                        h_gap { width: spacing::SM }
                        {layout_demo_item("Two", 72.0, 48.0, theme)}
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Native attributes still override defaults".to_string(),
                        detail: Some(
                            "align_items = center · justify_content = center".to_string(),
                        ),
                    }
                    Row {
                        width: "100%",
                        height: 88.0,
                        align_items: "center",
                        justify_content: "center",
                        background_color: arkit_shadcn::theme::with_alpha(
                            theme.colors.secondary,
                            0x80,
                        ),
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.lg,
                        {layout_demo_item("Centered", 104.0, 36.0, theme)}
                    }
                }
            }
        },
        "code" => rsx! {
            fixed_width {
                width: 640.0,
                column {
                    width: "100%",
                    align_items: "start",
                    Text {
                        content: "Standalone Code (feature = code)".to_string(),
                        variant: TextVariant::Small,
                    }
                    v_gap { height: spacing::SM }
                    Text {
                        content: "No Markdown required. Toggle theme to compare palettes.".to_string(),
                        variant: TextVariant::Muted,
                    }
                    v_gap { height: spacing::MD }
                    Code {
                        source: CODE_STANDALONE_RUST.to_string(),
                        language: Some("rust".to_string()),
                    }
                    v_gap { height: spacing::LG }
                    Code {
                        source: CODE_STANDALONE_JSON.to_string(),
                        language: Some("json".to_string()),
                    }
                    v_gap { height: spacing::LG }
                    Code {
                        source: "plain monospace without highlighting".to_string(),
                        language: None,
                        highlight: false,
                    }
                }
            }
        },
        "markdown" => {
            let chunk_index = markdown_chunk_index();
            let complete = chunk_index == MARKDOWN_STREAM_CHUNKS.len();
            let streaming = markdown_streaming();
            let document = markdown_source();
            let status = if complete {
                format!(
                    "Complete · {} chunks · {} bytes",
                    MARKDOWN_STREAM_CHUNKS.len(),
                    document.len()
                )
            } else if streaming {
                format!(
                    "Receiving chunk {}/{} · {} bytes",
                    chunk_index + 1,
                    MARKDOWN_STREAM_CHUNKS.len(),
                    document.len()
                )
            } else {
                format!(
                    "Paused at chunk {}/{} · {} bytes",
                    chunk_index,
                    MARKDOWN_STREAM_CHUNKS.len(),
                    document.len()
                )
            };
            let stream_action = if streaming { "Pause" } else { "Continue" };
            let mut stream_control = markdown_stream_task;
            let mut stream_replay = markdown_stream_task;

            rsx! {
                fixed_width {
                    width: 640.0,
                    column {
                        width: "100%",
                        align_items: "start",
                        Text {
                            content: "Tree-sitter highlight".to_string(),
                            variant: TextVariant::Small,
                        }
                        v_gap { height: spacing::SM }
                        Text {
                            content: "Static fences (rust / python / json / bash). Toggle theme to compare palettes.".to_string(),
                            variant: TextVariant::Muted,
                        }
                        v_gap { height: spacing::MD }
                        Markdown {
                            source: MARKDOWN_HIGHLIGHT_SAMPLE.to_string(),
                        }
                        v_gap { height: spacing::XL }
                        Separator {}
                        v_gap { height: spacing::XL }
                        Text {
                            content: "Streaming document".to_string(),
                            variant: TextVariant::Small,
                        }
                        v_gap { height: spacing::SM }
                        Text { content: status, variant: TextVariant::Muted }
                        v_gap { height: spacing::SM }
                        Progress {
                            value: chunk_index as f32,
                            total: Some(MARKDOWN_STREAM_CHUNKS.len() as f32),
                            height: Some(4.0),
                            animation_duration_ms: 120,
                        }
                        v_gap { height: spacing::MD }
                        row {
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: Some(complete),
                                onclick: move |_| {
                                    if markdown_streaming() {
                                        markdown_streaming.set(false);
                                        stream_control.pause();
                                    } else {
                                        markdown_streaming.set(true);
                                        stream_control.resume();
                                    }
                                },
                                "{stream_action}"
                            }
                            h_gap { width: spacing::SM }
                            Button {
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    markdown_source.set(String::new());
                                    markdown_chunk_index.set(0);
                                    markdown_streaming.set(true);
                                    markdown_link.set("Links become active as soon as their chunk arrives.".to_string());
                                    stream_replay.restart();
                                },
                                "Replay"
                            }
                        }
                        v_gap { height: spacing::XL }
                        Markdown {
                            source: document,
                            on_link_click: Some(EventHandler::new(move |url: String| {
                                markdown_link.set(format!("Activated: {url}"));
                            })),
                        }
                        v_gap { height: spacing::LG }
                        Text { content: markdown_link(), variant: TextVariant::Muted }
                    }
                }
            }
        }
        "menubar" => rsx! {
            column {
                width: "100%",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("active = {:?}", menubar_active())),
                }
                Menubar {
                    active: Some(menubar_active()),
                    default_active: None,
                    on_active_change: move |value| menubar_active.set(value),
                    menus: menubar_menus(
                        context_bookmarks(),
                        context_full_urls(),
                        context_person(),
                        EventHandler::new(move |value| context_bookmarks.set(value)),
                        EventHandler::new(move |value| context_full_urls.set(value)),
                        EventHandler::new(move |value| context_person.set(value)),
                    ),
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(menubar_uc_note()),
                }
                Menubar {
                    default_active: None,
                    on_active_change: move |value| {
                        menubar_uc_note.set(format!("on_active_change = {:?}", value));
                    },
                    menus: menubar_menus(
                        context_bookmarks(),
                        context_full_urls(),
                        context_person(),
                        EventHandler::new(move |value| context_bookmarks.set(value)),
                        EventHandler::new(move |value| context_full_urls.set(value)),
                        EventHandler::new(move |value| context_person.set(value)),
                    ),
                }
            }
        },
        "popover" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("open = {}", popover_open())),
                }
                Popover {
                    open: Some(popover_open()),
                    default_open: Some(false),
                    on_close: move |_| popover_open.set(false),
                    on_open_change: move |value| popover_open.set(value),
                    width: Some(320.0),
                    trigger: rsx! { Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Open popover" } },
                    column {
                        width: "100%",
                        text {
                            content: "Dimensions".to_string(),
                            font_size: typography::MD,
                            font_weight: 500_i32,
                            font_color: theme.colors.foreground,
                            line_height: 16.0,
                        }
                        v_gap { height: spacing::SM }
                        Text { content: "Set the dimensions for the layer.".to_string(), variant: TextVariant::Muted }
                        v_gap { height: spacing::LG }
                        popover_form_row {
                            label: "Width".to_string(),
                            value: "100%".to_string(),
                        }
                        v_gap { height: spacing::SM }
                        popover_form_row {
                            label: "Max. width".to_string(),
                            value: "300px".to_string(),
                        }
                        v_gap { height: spacing::SM }
                        popover_form_row {
                            label: "Height".to_string(),
                            value: "25px".to_string(),
                        }
                        v_gap { height: spacing::SM }
                        popover_form_row {
                            label: "Max. height".to_string(),
                            value: "none".to_string(),
                        }
                    }
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(popover_uc_note()),
                }
                Popover {
                    default_open: Some(false),
                    on_open_change: move |value| {
                        popover_uc_note.set(format!("on_open_change = {value}"));
                    },
                    width: Some(280.0),
                    trigger: rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {},
                            "Open (uncontrolled)"
                        }
                    },
                    Text {
                        content: "Popover manages its own open state.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            }
        },
        "progress" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    width: "100%",
                    align_items: "start",
                    padding_top: spacing::XL,
                    padding_right: spacing::XL,
                    padding_bottom: spacing::XL,
                    padding_left: spacing::XL,
                    background_color: theme.colors.card,
                    border_width: 1.0,
                    border_color: theme.colors.border,
                    border_radius: theme.radii.xl,
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "start",
                        row {
                            width: 44.0,
                            height: 44.0,
                            align_items: "center",
                            justify_content: "center",
                            background_color: theme.colors.secondary,
                            border_radius: theme.radii.lg,
                            {icon_placeholder("file-up", 20.0, theme.colors.secondary_foreground)}
                        }
                        h_gap { width: spacing::MD }
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            text {
                                content: if upload_progress() >= 100.0 { "Upload complete".to_string() } else { "Uploading assets".to_string() },
                                font_size: typography::MD,
                                font_weight: 600_i32,
                                font_color: theme.colors.card_foreground,
                                line_height: 22.0,
                            }
                            text {
                                content: "design-system.fig · 100 MB".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                    }
                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        text {
                            content: "Upload progress".to_string(),
                            font_size: typography::SM,
                            font_weight: 500_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                        text {
                            content: format!("{:.0}%", upload_progress()),
                            font_size: typography::SM,
                            font_weight: 500_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                    }
                    v_gap { height: spacing::SM }
                    Progress {
                        value: upload_progress(),
                        total: Some(100.0),
                    }
                    v_gap { height: spacing::SM }
                    text {
                        content: format!("{:.0} MB of 100 MB", upload_progress()),
                        width: "100%",
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                        text_align: "start",
                    }
                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "end",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            onclick: move |_| upload_progress.set(13.0),
                            "Restart"
                        }
                        h_gap { width: spacing::SM }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                upload_progress.set(if upload_progress() < 66.0 {
                                    66.0
                                } else if upload_progress() < 100.0 {
                                    100.0
                                } else {
                                    13.0
                                });
                            },
                            if upload_progress() >= 100.0 { "Upload again" } else { "Continue" }
                        }
                    }
                }
            }
        },
        "refresh-load-more" => rsx! {
            RefreshLoadMoreDemo {}
        },
        "radio-group" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("selected = {}", radio_choice())),
                    }
                    RadioGroup {
                        options: vec!["Default".to_string(), "Comfortable".to_string(), "Compact".to_string()],
                        selected: Some(radio_choice()),
                        on_select: move |value| radio_choice.set(value),
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some(radio_uc_note()),
                    }
                    RadioGroup {
                        options: vec!["Default".to_string(), "Comfortable".to_string(), "Compact".to_string()],
                        default_selected: "Comfortable".to_string(),
                        on_select: move |value| {
                            radio_uc_note.set(format!("on_select = {value}"));
                        },
                    }
                }
            }
        },
        "select" => rsx! {
            column {
                width: "100%",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!(
                        "selected = {:?}, open = {}",
                        selected_fruit(),
                        select_open()
                    )),
                }
                {select_carousel(
                    page(),
                    selected_fruit(),
                    select_open(),
                    on_page,
                    EventHandler::new(move |value| selected_fruit.set(value)),
                    EventHandler::new(move |value| select_open.set(value)),
                )}
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(format!("{} | {}", select_uc_note(), select_open_uc_note())),
                }
                fixed_width {
                    width: 180.0,
                    Select {
                        options: vec![
                            "Apple".to_string(),
                            "Banana".to_string(),
                            "Blueberry".to_string(),
                            "Grapes".to_string(),
                            "Pineapple".to_string(),
                        ],
                        default_selected: "Apple".to_string(),
                        default_open: false,
                        on_select: Some(EventHandler::new(move |value| {
                            select_uc_note.set(format!("on_select = {value}"));
                        })),
                        on_open_change: Some(EventHandler::new(move |open| {
                            select_open_uc_note.set(format!("on_open_change = {open}"));
                        })),
                    }
                }
            }
        },
        "separator" => rsx! {
            fixed_width {
                width: 320.0,
                column {
                    width: "100%",
                    align_items: "start",
                    column {
                        width: "100%",
                        align_items: "start",
                        Text { content: "Radix Primitives".to_string(), variant: TextVariant::Small }
                        v_gap { height: 4.0 }
                        Text { content: "An open-source UI component library.".to_string(), variant: TextVariant::Muted }
                    }
                    v_gap { height: spacing::MD }
                    Separator {}
                    v_gap { height: spacing::MD }
                    row {
                        align_self: "start",
                        align_items: "center",
                        justify_content: "start",
                        Text { content: "Blog".to_string(), variant: TextVariant::Small }
                        h_gap { width: spacing::MD }
                        Separator { vertical_height: Some(20.0) }
                        h_gap { width: spacing::MD }
                        Text { content: "Docs".to_string(), variant: TextVariant::Small }
                        h_gap { width: spacing::MD }
                        Separator { vertical_height: Some(20.0) }
                        h_gap { width: spacing::MD }
                        Text { content: "Source".to_string(), variant: TextVariant::Small }
                    }
                }
            }
        },
        "skeleton" => rsx! {
            // Card-like frame so the avatar + text lines read as a real loading
            // block (geometry is from Skeleton; contrast is fixed in the component).
            fixed_width {
                width: 320.0,
                column {
                    width: "100%",
                    align_items: "start",
                    padding: spacing::LG,
                    background_color: theme.colors.background,
                    border_radius: theme.radii.lg,
                    border_width: 1.0,
                    border_color: theme.colors.border,
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "start",
                        Skeleton { width: 48.0, height: 48.0 }
                        h_gap { width: spacing::LG }
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            Skeleton { width: 220.0, height: 16.0 }
                            v_gap { height: spacing::SM }
                            Skeleton { width: 180.0, height: 16.0 }
                        }
                    }
                }
            }
        },
        "slider" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    width: "100%",
                    height: 1064.0,
                    align_items: "start",
                    text {
                        width: "100%",
                        content: "Sound & haptics".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "Tune playback, output levels, and channel balance.".to_string(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::XXL }

                    column {
                        width: "100%",
                        align_items: "start",
                        padding: spacing::LG,
                        background_color: theme.colors.card,
                        border_style: "solid",
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.xl,
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "start",
                            row {
                                width: 48.0,
                                height: 48.0,
                                align_items: "center",
                                justify_content: "center",
                                background_color: theme.colors.primary,
                                border_radius: theme.radii.lg,
                                {icon_placeholder("music-2", 22.0, theme.colors.primary_foreground)}
                            }
                            h_gap { width: spacing::MD }
                            column {
                                layout_weight: 1.0,
                                align_items: "start",
                                text {
                                    content: "Midnight Drive".to_string(),
                                    font_size: typography::MD,
                                    font_weight: 600_i32,
                                    font_color: theme.colors.card_foreground,
                                    line_height: 22.0,
                                }
                                text {
                                    content: "Neon Avenue".to_string(),
                                    font_size: typography::SM,
                                    font_color: theme.colors.muted_foreground,
                                    line_height: 20.0,
                                }
                            }
                            {icon_placeholder("volume-2", 20.0, theme.colors.muted_foreground)}
                        }
                        v_gap { height: spacing::LG }
                        Slider {
                            value: playback_position(),
                            min: Some(0.0),
                            max: Some(240.0),
                            step: Some(1.0),
                            on_change: move |value| playback_position.set(value),
                        }
                        row {
                            width: "100%",
                            align_items: "center",
                            justify_content: "space_between",
                            text {
                                content: format_media_time(playback_position()),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                            text {
                                content: "4:00".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                    }

                    v_gap { height: spacing::XXL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        text {
                            content: "Media volume".to_string(),
                            font_size: typography::SM,
                            font_weight: 500_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                        text {
                            content: format!("{:.0}%", media_volume()),
                            font_size: typography::SM,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                    }
                    Slider {
                        value: media_volume(),
                        min: Some(0.0),
                        max: Some(100.0),
                        on_change: move |value| media_volume.set(value),
                    }

                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        column {
                            align_items: "start",
                            text {
                                content: "Safe listening range".to_string(),
                                font_size: typography::SM,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                            text {
                                content: "Drag either edge to set the comfort zone.".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                        text {
                            content: format!(
                                "{:.0}–{:.0}%",
                                listening_range()[0],
                                listening_range()[1],
                            ),
                            font_size: typography::SM,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                    }
                    RangeSlider {
                        value: listening_range(),
                        min: Some(0.0),
                        max: Some(100.0),
                        step: Some(1.0),
                        on_change: move |value| listening_range.set(value),
                    }

                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        column {
                            align_items: "start",
                            text {
                                content: "Equalizer crossover points".to_string(),
                                font_size: typography::SM,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                            text {
                                content: "Three thumbs split low, mid, and high bands.".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                        text {
                            content: equalizer_points()
                                .iter()
                                .map(|value| format!("{value:.0}"))
                                .collect::<Vec<_>>()
                                .join(" · "),
                            font_size: typography::SM,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                    }
                    MultiSlider {
                        values: equalizer_points(),
                        min: Some(0.0),
                        max: Some(100.0),
                        step: Some(1.0),
                        on_change: move |values| equalizer_points.set(values),
                    }

                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        text {
                            content: "Notification strength".to_string(),
                            font_size: typography::SM,
                            font_weight: 500_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                        text {
                            content: format!("{:.0} / 10", notification_strength()),
                            font_size: typography::SM,
                            font_weight: 600_i32,
                            font_color: theme.colors.foreground,
                            line_height: 20.0,
                        }
                    }
                    Slider {
                        value: notification_strength(),
                        min: Some(0.0),
                        max: Some(10.0),
                        step: Some(1.0),
                        show_steps: true,
                        on_change: move |value| notification_strength.set(value),
                    }

                    v_gap { height: spacing::XXL }
                    text {
                        content: "Channel balance".to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    text {
                        width: "100%",
                        content: "Vertical controls keep minimum at the bottom.".to_string(),
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        height: 220.0,
                        align_items: "center",
                        justify_content: "center",
                        column {
                            width: 120.0,
                            align_items: "center",
                            Slider {
                                value: left_channel(),
                                min: Some(0.0),
                                max: Some(100.0),
                                orientation: SliderOrientation::Vertical,
                                reversed: true,
                                height: Some(160.0),
                                style: SliderStyle {
                                    thumb_color: Some(theme.colors.chart_1),
                                    thumb_border_color: Some(theme.colors.chart_1),
                                    selected_color: Some(theme.colors.chart_1),
                                    track_color: Some(arkit_shadcn::theme::with_alpha(theme.colors.chart_1, 0x33)),
                                    ..SliderStyle::default()
                                },
                                on_change: move |value| left_channel.set(value),
                            }
                            v_gap { height: spacing::SM }
                            text {
                                content: format!("Left · {:.0}%", left_channel()),
                                font_size: typography::XS,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 18.0,
                            }
                        }
                        h_gap { width: spacing::XXL }
                        column {
                            width: 120.0,
                            align_items: "center",
                            Slider {
                                value: right_channel(),
                                min: Some(0.0),
                                max: Some(100.0),
                                orientation: SliderOrientation::Vertical,
                                reversed: true,
                                height: Some(160.0),
                                style: SliderStyle {
                                    thumb_color: Some(theme.colors.chart_2),
                                    thumb_border_color: Some(theme.colors.chart_2),
                                    selected_color: Some(theme.colors.chart_2),
                                    track_color: Some(arkit_shadcn::theme::with_alpha(theme.colors.chart_2, 0x33)),
                                    ..SliderStyle::default()
                                },
                                on_change: move |value| right_channel.set(value),
                            }
                            v_gap { height: spacing::SM }
                            text {
                                content: format!("Right · {:.0}%", right_channel()),
                                font_size: typography::XS,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 18.0,
                            }
                        }
                    }

                    v_gap { height: spacing::XL }
                    row {
                        width: "100%",
                        align_items: "center",
                        justify_content: "space_between",
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            text {
                                content: "System limit".to_string(),
                                font_size: typography::SM,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                            }
                            text {
                                content: "Managed by your device administrator.".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 18.0,
                            }
                        }
                        text {
                            content: "35%".to_string(),
                            font_size: typography::SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                    Slider {
                        value: 35.0,
                        min: Some(0.0),
                        max: Some(100.0),
                        disabled: true,
                    }
                }
            }
        },
        "sonner" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    width: "100%",
                    align_items: "start",
                    text {
                        width: "100%",
                        content: "Notifications".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "Bottom-center Sonner stack (official peeks). Swipe up to expand, down to collapse/dismiss. Minimal is a compact chip.".to_string(),
                        font_size: typography::SM,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::LG }
                    Button {
                        variant: ButtonVariant::Outline,
                        width: "100%",
                        onclick: move |_| sonner_background_clicks += 1,
                        "Background click test · {sonner_background_clicks()}"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        width: "100%",
                        content: "Notification".to_string(),
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        Button {
                            variant: ButtonVariant::Outline,
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                move |id| SonnerToast::new(id, "Event created")
                                    .description("Mon, Jan 3 · 6:00pm")
                                    .action("Undo")
                                    .duration_ms(0)
                                    .on_action(move || {
                                        let mut status = sonner_status;
                                        status.set(format!("Undo #{id}"));
                                    }),
                            ),
                            "Default"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::success(id, "Changes saved")
                                    .description("Profile is up to date")
                                    .duration_ms(0),
                            ),
                            "Success"
                        }
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        Button {
                            variant: ButtonVariant::Secondary,
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::info(id, "Update available")
                                    .description("Install when ready")
                                    .duration_ms(0),
                            ),
                            "Info"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            variant: ButtonVariant::Outline,
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::warning(id, "Storage low")
                                    .description("Free up space")
                                    .duration_ms(0),
                            ),
                            "Warning"
                        }
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        Button {
                            variant: ButtonVariant::Destructive,
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::error(id, "Upload failed")
                                    .description("Check connection")
                                    .duration_ms(0),
                            ),
                            "Error"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            variant: ButtonVariant::Outline,
                            width: "48%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::loading(id, "Uploading…")
                                    .description("Stays until dismissed"),
                            ),
                            "Loading"
                        }
                    }
                    v_gap { height: spacing::MD }
                    Button {
                        variant: ButtonVariant::Secondary,
                        width: "100%",
                        onclick: move |_| {
                            for (title, description) in [
                                ("Alice", "Free this afternoon?"),
                                ("Reminder", "Review in 10 min"),
                                ("Payment", "¥128 received"),
                            ] {
                                enqueue_sonner_toast(
                                    sonner_toasts,
                                    sonner_next_id,
                                    move |id| SonnerToast::info(id, title)
                                        .description(description)
                                        .duration_ms(0),
                                );
                            }
                            let mut status = sonner_status;
                            status.set("Stack ready — swipe up to expand, down to dismiss.".into());
                        },
                        "Stack 3 · swipe up"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        width: "100%",
                        content: "Minimal".to_string(),
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        width: "100%",
                        content: "Compact chip — short copy only.".to_string(),
                        font_size: typography::XS,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 16.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        width: "100%",
                        Button {
                            variant: ButtonVariant::Outline,
                            width: "31%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::minimal(id, "Copied"),
                            ),
                            "Copy"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            width: "31%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::success(id, "Saved")
                                    .appearance(ToastAppearance::Minimal)
                                    .dismissible(false)
                                    .duration_ms(2_000),
                            ),
                            "Saved"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            variant: ButtonVariant::Destructive,
                            width: "31%",
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::error(id, "Failed")
                                    .appearance(ToastAppearance::Minimal)
                                    .dismissible(false)
                                    .duration_ms(2_500),
                            ),
                            "Fail"
                        }
                    }
                    v_gap { height: spacing::XL }
                    text {
                        width: "100%",
                        content: sonner_status(),
                        font_size: typography::XS,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                }
            }
            Sonner {
                toasts: sonner_toasts(),
                position: SonnerPosition::BottomCenter,
                visible_toasts: 3,
                rich_colors: true,
            }
        },
        "spinner" => rsx! {
            fixed_width {
                width: 320.0,
                column {
                    width: "100%",
                    align_items: "center",
                    text {
                        content: "Sizes".to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::LG }
                    row {
                        align_items: "center",
                        justify_content: "center",
                        Spinner {}
                        h_gap { width: spacing::XXL }
                        Spinner {
                            size: 24.0,
                            color: Some(theme.colors.primary),
                            icon: Some("refresh-cw".to_string()),
                        }
                        h_gap { width: spacing::XXL }
                        Spinner { size: 32.0, color: Some(theme.colors.destructive) }
                    }
                    v_gap { height: spacing::XXL }
                    Button {
                        disabled: Some(true),
                        onclick: move |_| {},
                        Spinner { color: Some(theme.colors.primary_foreground) }
                        h_gap { width: spacing::SM }
                        text {
                            content: "Please wait".to_string(),
                            font_size: typography::MD,
                            font_weight: 500_i32,
                            font_color: theme.colors.primary_foreground,
                            line_height: 20.0,
                        }
                    }
                }
            }
        },
        "switch" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("checked = {}", switch_checked())),
                }
                row {
                    align_items: "center",
                    Switch {
                        checked: Some(switch_checked()),
                        on_change: Some(EventHandler::new(move |value| switch_checked.set(value))),
                    }
                    h_gap { width: spacing::SM }
                    Label { content: "Airplane Mode".to_string() }
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(switch_uc_note()),
                }
                row {
                    align_items: "center",
                    Switch {
                        default_checked: Some(false),
                        on_change: Some(EventHandler::new(move |value| {
                            switch_uc_note.set(format!("on_change = {value}"));
                        })),
                    }
                    h_gap { width: spacing::SM }
                    Label { content: "Wi‑Fi".to_string() }
                }
            }
        },
        "tabs" => rsx! {
            column {
                width: "100%",
                max_width_constraint: 384.0,
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("active = {}", tabs_active())),
                }
                Tabs {
                    labels: vec!["Account".to_string(), "Password".to_string()],
                    active: Some(tabs_active()),
                    on_change: move |index| tabs_active.set(index),
                    panels: vec![
                        rsx! {
                            Card {
                                CardHeader {
                                    title: "Account".to_string(),
                                    description: "Controlled tab panel.".to_string(),
                                }
                                CardContent {
                                    Text { content: "Parent owns active index.".to_string(), variant: TextVariant::Muted }
                                }
                            }
                        },
                        rsx! {
                            Card {
                                CardHeader {
                                    title: "Password".to_string(),
                                    description: "Second controlled panel.".to_string(),
                                }
                                CardContent {
                                    Text { content: "Switch tabs via triggers.".to_string(), variant: TextVariant::Muted }
                                }
                            }
                        },
                    ],
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some("default_active = 0".to_string()),
                }
                Tabs {
                    labels: vec!["One".to_string(), "Two".to_string()],
                    default_active: 0,
                    on_change: move |_| {},
                    panels: vec![
                        rsx! {
                            Text { content: "Uncontrolled panel A".to_string(), variant: TextVariant::Muted }
                        },
                        rsx! {
                            Text { content: "Uncontrolled panel B".to_string(), variant: TextVariant::Muted }
                        },
                    ],
                }
            }
        },
        "text" => rsx! {
            {text_carousel(page(), on_page)}
        },
        "textarea" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    width: "100%",
                    demo_mode_label {
                        title: "Controlled".to_string(),
                        detail: Some(format!("value.len = {}", textarea_controlled().chars().count())),
                    }
                    Textarea {
                        placeholder: Some("Type your message here.".to_string()),
                        value: Some(textarea_controlled()),
                        width: "100%",
                        on_change: Some(EventHandler::new(move |value| textarea_controlled.set(value))),
                    }
                    {demo_mode_divider()}
                    demo_mode_label {
                        title: "Uncontrolled".to_string(),
                        detail: Some("omit value — native field owns text".to_string()),
                    }
                    Textarea {
                        placeholder: Some("Uncontrolled textarea".to_string()),
                        width: "100%",
                    }
                }
            }
        },
        "toggle" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("checked = {}", toggle_pressed())),
                }
                Toggle {
                    label: "".to_string(),
                    icon: Some("bold".to_string()),
                    variant: ToggleVariant::Outline,
                    checked: Some(toggle_pressed()),
                    on_change: EventHandler::new(move |value| toggle_pressed.set(value)),
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(toggle_uc_note()),
                }
                Toggle {
                    label: "".to_string(),
                    icon: Some("italic".to_string()),
                    variant: ToggleVariant::Outline,
                    default_checked: false,
                    on_change: EventHandler::new(move |value| {
                        toggle_uc_note.set(format!("on_change = {value}"));
                    }),
                }
            }
        },
        "toggle-group" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("selected = {:?}", toggle_values())),
                }
                ToggleGroup {
                    options: vec!["bold".to_string(), "italic".to_string(), "underline".to_string()],
                    selected: Some(toggle_values()),
                    icons: true,
                    multi: true,
                    on_change: move |values| toggle_values.set(values),
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(toggle_group_uc_note()),
                }
                ToggleGroup {
                    options: vec!["bold".to_string(), "italic".to_string(), "underline".to_string()],
                    default_selected: vec!["bold".to_string()],
                    icons: true,
                    multi: true,
                    on_change: move |values| {
                        toggle_group_uc_note.set(format!("on_change = {:?}", values));
                    },
                }
            }
        },
        "tooltip" => rsx! {
            column {
                align_items: "start",
                demo_mode_label {
                    title: "Controlled".to_string(),
                    detail: Some(format!("open = {}", tooltip_open())),
                }
                Tooltip {
                    open: Some(tooltip_open()),
                    default_open: Some(false),
                    on_close: move |_| tooltip_open.set(false),
                    on_open_change: move |value| tooltip_open.set(value),
                    content: "Add to library".to_string(),
                    trigger: rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {},
                            "Press"
                        }
                    },
                }
                {demo_mode_divider()}
                demo_mode_label {
                    title: "Uncontrolled".to_string(),
                    detail: Some(tooltip_uc_note()),
                }
                Tooltip {
                    default_open: Some(false),
                    on_open_change: move |value| {
                        tooltip_uc_note.set(format!("on_open_change = {value}"));
                    },
                    content: "Uncontrolled tooltip".to_string(),
                    trigger: rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {},
                            "Hover/Press"
                        }
                    },
                }
            }
        },
        "table" => rsx! {
            fixed_width {
                width: 560.0,
                Table {
                    headers: vec!["Invoice".to_string(), "Status".to_string(), "Method".to_string(), "Amount".to_string()],
                    rows: vec![
                        vec!["INV001".to_string(), "Paid".to_string(), "Credit Card".to_string(), "$250.00".to_string()],
                        vec!["INV002".to_string(), "Pending".to_string(), "PayPal".to_string(), "$150.00".to_string()],
                        vec!["INV003".to_string(), "Unpaid".to_string(), "Bank Transfer".to_string(), "$350.00".to_string()],
                        vec!["INV004".to_string(), "Paid".to_string(), "Credit Card".to_string(), "$450.00".to_string()],
                    ],
                }
            }
        },
        "watermark" => rsx! { WatermarkDemo {} },
        _ => rsx! {
            Text { content: "Component not found".to_string(), variant: TextVariant::Muted }
        },
    }
}

#[component]
fn WatermarkDemo() -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let mut clicks = use_signal(|| 0_u32);
    let sample_image =
        ArkImageSource::svg("watermark-demo-landscape", WATERMARK_IMAGE_SAMPLE, 840, 472);
    let logo_image = ArkImageSource::encoded(
        "watermark-demo-logo",
        WATERMARK_LOGO_SAMPLE.as_bytes().to_vec(),
        512,
        160,
    );

    rsx! {
        fixed_width {
            width: 420.0,
            column {
                width: "100%",
                align_items: "start",

                demo_mode_label {
                    title: "Basic text watermark".to_string(),
                    detail: Some("Theme-aware defaults over ordinary content.".to_string()),
                }
                Watermark {
                    source: WatermarkSource::text("ARKIT · INTERNAL"),
                    width: "100%".to_string(),
                    height: "220".to_string(),
                    column {
                        width: "100%",
                        height: 220.0,
                        align_items: "start",
                        padding_top: spacing::LG,
                        padding_right: spacing::LG,
                        padding_bottom: spacing::LG,
                        padding_left: spacing::LG,
                        background_color: theme.colors.card,
                        border_radius: theme.radii.xl,
                        text {
                            content: "Quarterly report".to_string(),
                            font_size: typography::XXL,
                            font_weight: 700_i32,
                            font_color: theme.colors.foreground,
                            line_height: 30.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: "Revenue and retention remained above the target range.".to_string(),
                            font_size: typography::SM,
                            font_weight: 400_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                }

                {demo_mode_divider()}
                demo_mode_label {
                    title: "Custom multiline style".to_string(),
                    detail: Some(
                        "Offset, repeat origin, blend, outline, shadow, font, and spacing are configurable."
                            .to_string(),
                    ),
                }
                Watermark {
                    source: WatermarkSource::text("ARKIT\nCONFIDENTIAL"),
                    width: "100%".to_string(),
                    height: "260".to_string(),
                    style: WatermarkStyle {
                        color: Some(theme.colors.primary),
                        font_size: 15.0,
                        font_weight: 700,
                        font_style: WatermarkFontStyle::Italic,
                        font_family: Some("HarmonyOS Sans".to_string()),
                        opacity: 0.24,
                        rotation_degrees: -18.0,
                        gap_x: 72.0,
                        gap_y: 56.0,
                        offset_x: 18.0,
                        offset_y: -8.0,
                        repeat_origin_x: 28.0,
                        repeat_origin_y: 20.0,
                        blend_mode: WatermarkBlendMode::Multiply,
                        stroke: Some(WatermarkStroke::new(0xCCFFFFFF, 1.2)),
                        shadow: Some(WatermarkShadow::new(0x66000000, 4.0, 3.0, 4.0)),
                    },
                    column {
                        width: "100%",
                        height: 260.0,
                        align_items: "start",
                        padding_top: spacing::LG,
                        padding_right: spacing::LG,
                        padding_bottom: spacing::LG,
                        padding_left: spacing::LG,
                        background_color: theme.colors.card,
                        border_radius: theme.radii.xl,
                        text {
                            content: "Architecture decision".to_string(),
                            font_size: typography::XL,
                            font_weight: 700_i32,
                            font_color: theme.colors.foreground,
                            line_height: 26.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: "The document can contain multiple paragraphs while the watermark itself uses multiple lines.".to_string(),
                            font_size: typography::SM,
                            font_weight: 400_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: "Only the watermark tile is rasterized; document text remains native ArkUI content.".to_string(),
                            font_size: typography::SM,
                            font_weight: 400_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                }

                {demo_mode_divider()}
                demo_mode_label {
                    title: "Watermark over image content".to_string(),
                    detail: Some("The child may be an image while the watermark remains independently styled.".to_string()),
                }
                Watermark {
                    source: WatermarkSource::text("PREVIEW"),
                    width: "100%".to_string(),
                    height: "236".to_string(),
                    style: WatermarkStyle {
                        color: Some(0xFFFFFFFF),
                        font_size: 16.0,
                        font_weight: 700,
                        opacity: 0.6,
                        rotation_degrees: -24.0,
                        gap_x: 84.0,
                        gap_y: 64.0,
                        blend_mode: WatermarkBlendMode::Difference,
                        ..WatermarkStyle::default()
                    },
                    image {
                        src: AttributeValue::any_value(sample_image),
                        width: "100%",
                        height: 236.0,
                        object_fit: "cover",
                        border_radius: theme.radii.xl,
                        clip: true,
                    }
                }

                {demo_mode_divider()}
                demo_mode_label {
                    title: "Image watermark source".to_string(),
                    detail: Some("An embedded image is decoded once, cached, and repeated by one shader.".to_string()),
                }
                Watermark {
                    source: WatermarkSource::image(logo_image, 128.0, 40.0),
                    width: "100%".to_string(),
                    height: "240".to_string(),
                    style: WatermarkStyle {
                        opacity: 0.2,
                        rotation_degrees: -16.0,
                        gap_x: 72.0,
                        gap_y: 58.0,
                        shadow: Some(WatermarkShadow::new(0x55000000, 5.0, 2.0, 3.0)),
                        ..WatermarkStyle::default()
                    },
                    column {
                        width: "100%",
                        height: 240.0,
                        align_items: "start",
                        padding_top: spacing::LG,
                        padding_right: spacing::LG,
                        padding_bottom: spacing::LG,
                        padding_left: spacing::LG,
                        background_color: theme.colors.card,
                        border_radius: theme.radii.xl,
                        text {
                            content: "Brand asset review".to_string(),
                            font_size: typography::XL,
                            font_weight: 700_i32,
                            font_color: theme.colors.foreground,
                            line_height: 26.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: "The watermark source is an embedded SVG image rather than text.".to_string(),
                            font_size: typography::SM,
                            font_weight: 400_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                }

                {demo_mode_divider()}
                demo_mode_label {
                    title: "Long document stress sample".to_string(),
                    detail: Some(
                        "1,440vp content; the watermark remains one cached repeating texture."
                            .to_string(),
                    ),
                }
                Watermark {
                    source: WatermarkSource::text("ARKIT · INTERNAL"),
                    width: "100%".to_string(),
                    height: "1440".to_string(),
                    style: WatermarkStyle {
                        opacity: 0.16,
                        gap_x: 96.0,
                        gap_y: 80.0,
                        ..WatermarkStyle::default()
                    },
                    column {
                        width: "100%",
                        height: 1440.0,
                        align_items: "start",
                        padding_top: spacing::LG,
                        padding_right: spacing::LG,
                        padding_bottom: spacing::LG,
                        padding_left: spacing::LG,
                        background_color: theme.colors.card,
                        border_radius: theme.radii.xl,
                        text {
                            content: "Deployment audit".to_string(),
                            font_size: typography::XXL,
                            font_weight: 700_i32,
                            font_color: theme.colors.foreground,
                            line_height: 30.0,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: "The overlay does not intercept the button or scrolling.".to_string(),
                            font_size: typography::SM,
                            font_weight: 400_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                        v_gap { height: spacing::LG }
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| clicks += 1,
                            "Button clicks: {clicks}"
                        }
                        v_gap { height: spacing::LG }
                        for index in 1..=24 {
                            row {
                                width: "100%",
                                height: 44.0,
                                align_items: "center",
                                text {
                                    content: format!("Audit event #{index:02}"),
                                    font_size: typography::SM,
                                    font_weight: 500_i32,
                                    font_color: theme.colors.foreground,
                                    line_height: 20.0,
                                }
                                row { layout_weight: 1.0 }
                                text {
                                    content: "verified".to_string(),
                                    font_size: typography::XS,
                                    font_weight: 500_i32,
                                    font_color: theme.colors.muted_foreground,
                                    line_height: 18.0,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn layout_demo_item(label: &'static str, width: f32, height: f32, theme: Theme) -> Element {
    rsx! {
        row {
            width,
            height,
            align_items: "center",
            justify_content: "center",
            background_color: theme.colors.primary,
            border_radius: theme.radii.md,
            text {
                content: label,
                font_size: typography::SM,
                font_weight: 600_i32,
                font_color: theme.colors.primary_foreground,
                line_height: 18.0,
            }
        }
    }
}

#[component]
fn RefreshLoadMoreDemo() -> Element {
    const INITIAL_ITEMS: u32 = 24;
    const PAGE_SIZE: u32 = 12;
    const MAX_ITEMS: u32 = 60;

    let theme = arkit_shadcn::theme::use_theme();
    let mut virtual_mode = use_signal(|| true);
    let mut item_count = use_signal(|| INITIAL_ITEMS);
    let mut data_revision = use_signal(|| 0_u64);
    let mut refreshing = use_signal(|| false);
    let mut load_state = use_signal(LoadMoreState::default);
    let mut operation_epoch = use_signal(|| 0_u64);
    let mut refresh_request = use_signal(|| 0_u64);
    let mut load_request = use_signal(|| 0_u64);
    let async_runtime = arkit::use_runtime_handle().tokio();

    let refresh_runtime = async_runtime.clone();
    let _refresh_task = use_resource(move || {
        let refresh_runtime = refresh_runtime.clone();
        let request = refresh_request();
        async move {
            if request == 0 {
                return;
            }
            let _ = refresh_runtime
                .spawn(async {
                    tokio::time::sleep(Duration::from_millis(700)).await;
                })
                .await;
            if *operation_epoch.peek() != request {
                return;
            }
            item_count.set(INITIAL_ITEMS);
            data_revision += 1;
            load_state.set(LoadMoreState::Idle);
            refreshing.set(false);
        }
    });

    let load_runtime = async_runtime;
    let _load_task = use_resource(move || {
        let load_runtime = load_runtime.clone();
        let request = load_request();
        async move {
            if request == 0 {
                return;
            }
            let _ = load_runtime
                .spawn(async {
                    tokio::time::sleep(Duration::from_millis(650)).await;
                })
                .await;
            if *operation_epoch.peek() != request {
                return;
            }
            let next = (*item_count.peek())
                .saturating_add(PAGE_SIZE)
                .min(MAX_ITEMS);
            item_count.set(next);
            load_state.set(if next >= MAX_ITEMS {
                LoadMoreState::NoMore
            } else {
                LoadMoreState::Idle
            });
        }
    });

    let begin_refresh = EventHandler::new(move |()| {
        if refreshing() {
            return;
        }
        let request = operation_epoch().saturating_add(1);
        operation_epoch.set(request);
        refresh_request.set(request);
        refreshing.set(true);
        load_state.set(LoadMoreState::Idle);
    });
    let begin_load = EventHandler::new(move |()| {
        if matches!(load_state(), LoadMoreState::Loading | LoadMoreState::NoMore) {
            return;
        }
        let request = operation_epoch().saturating_add(1);
        operation_epoch.set(request);
        load_request.set(request);
        load_state.set(LoadMoreState::Loading);
    });

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: theme.colors.background,
            column {
                width: "100%",
                padding_top: spacing::SM,
                padding_right: spacing::MD,
                padding_bottom: spacing::SM,
                padding_left: spacing::MD,
                background_color: theme.colors.card,
                border_width: 1.0,
                border_color: theme.colors.border,
                row {
                    width: "100%",
                    align_items: "center",
                    Button {
                        size: ButtonSize::Sm,
                        variant: if virtual_mode() { ButtonVariant::Default } else { ButtonVariant::Outline },
                        onclick: move |_| virtual_mode.set(true),
                        "Virtual List"
                    }
                    h_gap { width: spacing::SM }
                    Button {
                        size: ButtonSize::Sm,
                        variant: if virtual_mode() { ButtonVariant::Outline } else { ButtonVariant::Default },
                        onclick: move |_| virtual_mode.set(false),
                        "Regular Scroll"
                    }
                    row {
                        layout_weight: 1.0,
                        justify_content: "end",
                        Button {
                            size: ButtonSize::Sm,
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| {
                                operation_epoch += 1;
                                refreshing.set(false);
                                load_state.set(LoadMoreState::Failed);
                            },
                            "Test error"
                        }
                    }
                }
                text {
                    margin_top: spacing::XS,
                    font_size: typography::XS,
                    font_color: theme.colors.muted_foreground,
                    max_lines: 1_i32,
                    text_overflow: "ellipsis",
                    if virtual_mode() {
                        "NodeAdapter · {item_count} items · pull down / scroll to bottom"
                    } else {
                        "Native Scroll · {item_count} items · pull down / scroll to bottom"
                    }
                }
            }
            column {
                width: "100%",
                layout_weight: 1.0,
                if virtual_mode() {
                    RefreshVirtualListDemo {
                        item_count: item_count(),
                        state: load_state(),
                        data_revision: data_revision(),
                        refreshing: refreshing(),
                        on_refresh: begin_refresh,
                        on_load_more: begin_load,
                    }
                } else {
                    PullToRefresh {
                        refreshing: refreshing(),
                        on_refresh: move |_| begin_refresh.call(()),
                        InfiniteScroll {
                            item_count: item_count(),
                            data_revision: data_revision(),
                            state: load_state(),
                            scroll_bar: "off".to_string(),
                            on_load_more: move |_| begin_load.call(()),
                            for index in 0..item_count() {
                                {refresh_demo_row(index, theme)}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RefreshVirtualListDemo(
    item_count: u32,
    state: LoadMoreState,
    data_revision: u64,
    refreshing: bool,
    on_refresh: EventHandler<()>,
    on_load_more: EventHandler<()>,
) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let controller = use_load_more(item_count, state, 3, on_load_more);
    let reset_controller = controller.clone();
    use_effect(use_reactive((&data_revision,), move |(_revision,)| {
        reset_controller.reset()
    }));

    let mut item_keys: Vec<u64> = (0..u64::from(item_count)).collect();
    item_keys.push(u64::MAX - load_more_state_key(state));
    let item_controller = controller.clone();
    let footer_controller = controller.clone();
    let runtime = arkit::use_runtime_handle();
    let source = use_virtual_source_items_keyed(VirtualKind::List, item_keys, move |index| {
        if index < item_count {
            let visible_controller = item_controller.clone();
            runtime.queue_ui(move || visible_controller.on_virtual_item(index));
            refresh_demo_row(index, theme)
        } else {
            let retry_controller = footer_controller.clone();
            rsx! {
                LoadMoreIndicator {
                    state,
                    on_retry: move |_| retry_controller.retry(),
                }
            }
        }
    });
    let scroll_controller = controller.clone();
    let refresh_controller = controller;

    rsx! {
        PullToRefresh {
            refreshing,
            on_refresh: move |_| {
                refresh_controller.reset();
                on_refresh.call(());
            },
            ShowcaseVirtualListHost {
                source,
                on_scroll: move |data| scroll_controller.on_virtual_scroll(data),
            }
        }
    }
}

#[component]
fn ShowcaseVirtualListHost(
    source: VirtualSource,
    on_scroll: EventHandler<dioxus_elements::event::ScrollData>,
) -> Element {
    rsx! {
        list {
            virtual_source: source,
            width: "100%",
            height: "100%",
            scroll_bar: "off",
            list_cached_count: 6_i32,
            onscroll: move |event| on_scroll.call(*event.data()),
        }
    }
}

fn load_more_state_key(state: LoadMoreState) -> u64 {
    match state {
        LoadMoreState::Idle => 0,
        LoadMoreState::Loading => 1,
        LoadMoreState::Failed => 2,
        LoadMoreState::NoMore => 3,
    }
}

fn refresh_demo_row(index: u32, theme: Theme) -> Element {
    rsx! {
        row {
            width: "100%",
            height: 58.0,
            padding_left: spacing::MD,
            padding_right: spacing::MD,
            align_items: "center",
            background_color: theme.colors.background,
            border_width: 1.0,
            border_color: theme.colors.border,
            text {
                font_size: typography::SM,
                font_weight: 500_i32,
                font_color: theme.colors.foreground,
                "Activity #{index}"
            }
            row {
                layout_weight: 1.0,
                justify_content: "end",
                text {
                    font_size: typography::XS,
                    font_color: theme.colors.muted_foreground,
                    "virtual-ready"
                }
            }
        }
    }
}

fn index_demo_note(large: bool, show_empty: bool, custom: bool) -> String {
    let data = if large { "800 contacts" } else { "cities" };
    let empty = if show_empty {
        "show empty"
    } else {
        "hide empty"
    };
    let ui = if custom { "custom UI" } else { "default UI" };
    format!("{data} · {empty} · {ui}")
}

#[component]
fn IndexCustomItem(ctx: IndexItemContext) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let mark = ctx
        .item
        .title
        .chars()
        .next()
        .map(|ch| ch.to_string())
        .unwrap_or_else(|| ctx.index.clone());
    rsx! {
        row {
            width: "100%",
            height: 56.0,
            padding_left: spacing::MD,
            padding_right: spacing::XXL,
            align_items: "center",
            column {
                width: 32.0,
                height: 32.0,
                border_radius: theme.radii.full,
                background_color: theme.colors.secondary,
                align_items: "center",
                justify_content: "center",
                text {
                    content: mark,
                    font_size: typography::XS,
                    font_weight: 600,
                    font_color: theme.colors.secondary_foreground,
                }
            }
            column {
                layout_weight: 1.0,
                margin_left: spacing::MD,
                align_items: "start",
                text {
                    content: ctx.item.title.clone(),
                    width: "100%",
                    font_size: typography::SM,
                    font_weight: 500,
                    font_color: theme.colors.foreground,
                    text_align: "start",
                    max_lines: 1,
                    text_overflow: "ellipsis",
                }
                if let Some(description) = ctx.item.description.clone() {
                    text {
                        content: description,
                        width: "100%",
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        text_align: "start",
                        max_lines: 1,
                        text_overflow: "ellipsis",
                    }
                }
            }
        }
    }
}

#[component]
fn IndexCustomHeader(ctx: IndexHeaderContext) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    rsx! {
        row {
            width: "100%",
            height: 32.0,
            padding_left: spacing::MD,
            align_items: "center",
            background_color: theme.colors.accent,
            text {
                content: ctx.index,
                font_size: typography::XS,
                font_weight: 700,
                font_color: theme.colors.accent_foreground,
            }
        }
    }
}

#[component]
fn IndexCustomBar(slot: IndexBarSlot) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let color = if slot.empty {
        theme.colors.border
    } else if slot.active {
        theme.colors.primary_foreground
    } else {
        theme.colors.muted_foreground
    };
    rsx! {
        column {
            width: if slot.active { 18.0 } else { 14.0 },
            height: 18.0,
            border_radius: theme.radii.full,
            background_color: if slot.active {
                theme.colors.primary
            } else {
                0x0000_0000
            },
            align_items: "center",
            justify_content: "center",
            hit_test_behavior: "none",
            text {
                content: slot.index,
                font_size: 9.0,
                font_weight: if slot.active { 700 } else { 500 },
                font_color: color,
                hit_test_behavior: "none",
            }
        }
    }
}

fn index_city_items() -> Vec<IndexItemSpec> {
    [
        ("#", "*客服", "symbol"),
        ("#", "@通知", "mention"),
        ("", "12306", "rail"),
        ("", "10086", "mobile"),
        ("B", "北京", "Jing"),
        ("C", "成都", "Chuan"),
        ("C", "重庆", "Yu"),
        ("C", "长沙", "Xiang"),
        ("C", "长春", "Ji"),
        ("D", "大连", "Liao"),
        ("F", "福州", "Min"),
        ("G", "广州", "Yue"),
        ("H", "杭州", "Zhe"),
        ("H", "合肥", "Wan"),
        ("H", "哈尔滨", "Hei"),
        ("H", "呼和浩特", "Meng"),
        ("H", "海口", "Qiong"),
        ("J", "济南", "Lu"),
        ("K", "昆明", "Dian"),
        ("L", "拉萨", "Zang"),
        ("N", "南京", "Su"),
        ("N", "宁波", "Zhe"),
        ("N", "南宁", "Gui"),
        ("Q", "青岛", "Lu"),
        ("S", "上海", "Hu"),
        ("S", "深圳", "Yue"),
        ("S", "苏州", "Su"),
        ("S", "沈阳", "Liao"),
        ("T", "天津", "Jin"),
        ("W", "武汉", "E"),
        ("W", "乌鲁木齐", "Xin"),
        ("X", "西安", "Shaan"),
        ("X", "厦门", "Min"),
        ("X", "西宁", "Qing"),
        ("Y", "银川", "Ning"),
        ("Z", "郑州", "Yu"),
    ]
    .into_iter()
    .map(|(index, title, description)| {
        IndexItemSpec::new(index, title).with_description(description)
    })
    .collect()
}

fn index_contact_items(count: usize) -> Vec<IndexItemSpec> {
    const LETTERS: &[char] = &[
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    let mut items = Vec::with_capacity(count);
    items.push(IndexItemSpec::new("#", "*Starred").with_description("favorites"));
    items.push(IndexItemSpec::new("", "@support").with_description("symbol"));
    items.push(IndexItemSpec::new("", "10086").with_description("hotline"));
    items.push(IndexItemSpec::new("", "12306").with_description("rail"));
    let remaining = count.saturating_sub(items.len());
    let per = (remaining / LETTERS.len()).max(1);
    for (letter_index, letter) in LETTERS.iter().enumerate() {
        for n in 0..per {
            if items.len() >= count {
                break;
            }
            let serial = letter_index * per + n;
            items.push(
                IndexItemSpec::new(letter.to_string(), format!("{letter}lex {serial:03}"))
                    .with_description(format!("+86 138 {serial:04}")),
            );
        }
    }
    items
}

#[component]
fn fixed_width(width: f32, children: Element) -> Element {
    // shadcn-style max-width cap: fill the parent up to `width`, never force a
    // hard width that can overflow narrow screens (512vp ≈ 1664px @3.25x).
    // Select/Popover still measure the painted control; anchor geometry no
    // longer depends on this wrapper using an absolute width.
    rsx! {
        column {
            width: "100%",
            align_items: "center",
            column {
                width: "100%",
                max_width_constraint: width,
                align_items: "stretch",
                {children}
            }
        }
    }
}

/// Section header for controlled / uncontrolled showcase pairs.
#[component]
fn demo_mode_label(title: String, detail: Option<String>) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            text {
                content: title,
                font_size: typography::SM,
                font_weight: 600_i32,
                font_color: theme.colors.foreground,
                line_height: 20.0,
            }
            if let Some(text) = detail {
                text {
                    content: text,
                    font_size: typography::XS,
                    font_color: theme.colors.muted_foreground,
                    line_height: 16.0,
                    margin_top: 2.0,
                }
            }
            v_gap { height: spacing::SM }
        }
    }
}

fn demo_mode_divider() -> Element {
    rsx! {
        column {
            width: "100%",
            v_gap { height: spacing::XL }
            Separator {}
            v_gap { height: spacing::XL }
        }
    }
}

/// 显示当前 Anchor 激活区块（演示 `use_anchor` 公共 hook，必须在 Anchor 子树内）。
#[component]
fn AnchorActiveLabel() -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let active = use_anchor()
        .and_then(|context| context.active_id())
        .unwrap_or_else(|| "none".to_string());
    rsx! {
        text {
            content: format!("Active: {active}"),
            font_size: typography::XS,
            font_color: theme.colors.muted_foreground,
            line_height: 16.0,
        }
    }
}

/// Anchor demo 的一个内容区块：Card + 标题 + 正文。
fn anchor_section_card(title: &'static str, body: &'static str) -> Element {
    rsx! {
        Card {
            CardHeader {
                title: title.to_string(),
                description: "Scroll to this section from the nav.".to_string(),
            }
            CardContent {
                Text { content: body.to_string() }
            }
        }
        v_gap { height: spacing::MD }
    }
}

fn accordion_demo_items() -> Vec<AccordionItemSpec> {
    vec![
        AccordionItemSpec::new(
            "Product Information",
            "item-1",
            rsx! {
                column {
                    width: "100%",
                    Text {
                        content: "Our flagship product combines cutting-edge technology with sleek design.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            },
        ),
        AccordionItemSpec::new(
            "Shipping Details",
            "item-2",
            rsx! {
                column {
                    width: "100%",
                    Text {
                        content: "Standard delivery takes 3-5 business days worldwide.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            },
        ),
        AccordionItemSpec::new(
            "Return Policy",
            "item-3",
            rsx! {
                column {
                    width: "100%",
                    Text {
                        content: "30-day returns with free shipping on eligible orders.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            },
        ),
    ]
}

fn enqueue_sonner_toast(
    mut toasts: Signal<Vec<SonnerToast>>,
    mut next_id: Signal<u64>,
    build: impl FnOnce(u64) -> SonnerToast,
) {
    let id = next_id();
    next_id.set(
        id.checked_add(1)
            .expect("showcase toast id space exhausted"),
    );
    let dismiss_toasts = toasts;
    let toast = build(id).on_dismiss(move || {
        let mut toasts = dismiss_toasts;
        toasts.with_mut(|items| toast_retain_without_id(items, id));
    });
    toasts.with_mut(|items| {
        if items.len() >= 8 {
            items.remove(0);
        }
        items.push(toast);
    });
}

fn toast_retain_without_id(toasts: &mut Vec<SonnerToast>, dismissed_id: u64) {
    toasts.retain(|toast| toast.id != dismissed_id);
}

#[component]
fn v_gap(height: f32) -> Element {
    rsx! { row { height } }
}

#[component]
fn h_gap(width: f32) -> Element {
    rsx! { row { width } }
}

fn demo_avatar(src: &str, fallback: &str, ring: bool, radius: Option<f32>) -> Element {
    let src = src.to_string();
    let fallback = fallback.to_string();

    rsx! {
        Avatar {
            src: Some(src),
            fallback: rsx! {
                AvatarFallback { content: fallback }
            },
            ring: Some(ring),
            radius,
        }
    }
}

#[component]
fn IconTile(name: String, color: u32, size: Option<f32>) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    let icon_size = size.unwrap_or(20.0);
    rsx! {
        column {
            width: 96.0,
            align_items: "center",
            stack {
                width: 48.0,
                height: 48.0,
                alignment: "center",
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                {icon_placeholder(name.as_str(), icon_size, color)}
            }
            row { height: spacing::SM }
            Text { content: name, variant: TextVariant::Small }
        }
    }
}

#[component]
fn repo_row(name: String) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    rsx! {
        row {
            width: "100%",
            align_self: "start",
            padding_top: spacing::SM,
            padding_right: spacing::LG,
            padding_bottom: spacing::SM,
            padding_left: spacing::LG,
            justify_content: "start",
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            Text { content: name, variant: TextVariant::Small }
        }
    }
}

#[component]
fn popover_form_row(label: String, value: String) -> Element {
    rsx! {
        row {
            width: "100%",
            align_self: "start",
            align_items: "center",
            justify_content: "start",
            row {
                width: 96.0,
                Label { content: label }
            }
            row {
                layout_weight: 1.0,
                margin_left: spacing::LG,
                Input {
                    placeholder: Some(value.clone()),
                    value: Some(value),
                    height: Some(32.0),
                    width: "100%",
                }
            }
        }
    }
}

fn carousel_frame(
    page: i32,
    count: i32,
    preview: Element,
    on_page: EventHandler<i32>,
    reserve_bottom_controls: bool,
) -> Element {
    let current = page.clamp(1, count);
    let prev_disabled = current == 1;
    let next_disabled = current == count;

    rsx! {
        stack {
            width: "100%",
            height: "100%",
            row {
                width: "100%",
                height: "100%",
                align_items: "center",
                justify_content: "center",
                padding_bottom: if reserve_bottom_controls { 48.0 + spacing::LG } else { 0.0 },
                {preview}
            }
            column {
                width: "100%",
                height: "100%",
                align_items: "center",
                justify_content: "end",
                hit_test_behavior: "transparent",
                row {
                    width: "100%",
                    height: 48.0,
                    align_items: "center",
                    justify_content: "center",
                    margin_bottom: spacing::LG,
                    padding_left: spacing::LG,
                    padding_right: spacing::LG,
                    carousel_button {
                        icon: "chevron-left".to_string(),
                        disabled: prev_disabled,
                        onclick: move |_| on_page.call((current - 1).max(1)),
                    }
                    h_gap { width: spacing::SM }
                    carousel_button {
                        icon: "chevron-right".to_string(),
                        disabled: next_disabled,
                        onclick: move |_| on_page.call((current + 1).min(count)),
                    }
                }
            }
        }
    }
}

#[component]
fn carousel_button(icon: String, disabled: bool, onclick: EventHandler<()>) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    rsx! {
        row {
            width: 40.0,
            height: 40.0,
            align_items: "center",
            justify_content: "center",
            background_color: theme.colors.background,
            border_radius: theme.radii.md,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_style: "solid",
            clip: true,
            opacity: if disabled { 0.5 } else { 1.0 },
            shadow: "sm",
            onclick: move |_| {
                if !disabled {
                    onclick.call(());
                }
            },
            {icon_placeholder(icon.as_str(), 18.0, theme.colors.foreground)}
        }
    }
}

fn select_carousel(
    page: i32,
    selected: String,
    open: bool,
    on_page: EventHandler<i32>,
    on_select: EventHandler<String>,
    on_open: EventHandler<bool>,
) -> Element {
    let default_items = vec!["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"];
    let scrollable_items = vec![
        "Apple",
        "Banana",
        "Blueberry",
        "Grapes",
        "Pineapple",
        "Cherry",
        "Strawberry",
        "Orange",
        "Lemon",
        "Kiwi",
        "Mango",
        "Pomegranate",
        "Watermelon",
        "Peach",
        "Pear",
        "Plum",
        "Raspberry",
        "Tangerine",
    ];

    let count = 2;
    let options = if page.clamp(1, count) == 2 {
        scrollable_items
    } else {
        default_items
    }
    .into_iter()
    .map(ToString::to_string)
    .collect::<Vec<_>>();

    carousel_frame(
        page,
        count,
        rsx! {
            fixed_width {
                width: 180.0,
                Select {
                    options,
                    selected: Some(selected),
                    default_selected: "Apple".to_string(),
                    open: Some(open),
                    default_open: false,
                    on_open_change: Some(on_open),
                    on_select: Some(on_select),
                }
            }
        },
        on_page,
        true,
    )
}

fn text_carousel(page: i32, on_page: EventHandler<i32>) -> Element {
    let current = page.clamp(1, 3);
    let preview = match current {
        2 => rsx! {
            fixed_width {
                width: 512.0,
                scroll {
                    width: "100%",
                    height: "100%",
                    column {
                        width: "100%",
                        padding_top: spacing::XXL,
                        padding_right: spacing::XXL,
                        padding_bottom: 72.0,
                        padding_left: spacing::XXL,
                        Text { content: "The Rainbow Forest Adventure".to_string(), variant: TextVariant::H1 }
                        v_gap { height: 12.0 }
                        Text { content: "Once upon a time, in a magical forest, there lived a curious rabbit named Whiskers. Whiskers loved exploring and discovering new things every day.".to_string(), variant: TextVariant::P }
                        v_gap { height: spacing::XXL }
                        Text { content: "Whiskers' Discovery".to_string(), variant: TextVariant::H2 }
                        Text { content: "One day, while hopping through the forest, Whiskers stumbled upon a mysterious rainbow-colored flower. The flower had the power to make the forest come alive with vibrant colors and happy creatures.".to_string(), variant: TextVariant::P }
                        Text { content: "\"Oh, what a wonderful discovery!\" exclaimed Whiskers. \"I must share this magic with all my forest friends!\"".to_string(), variant: TextVariant::Blockquote }
                        v_gap { height: 32.0 }
                        Text { content: "The Colorful Transformation".to_string(), variant: TextVariant::H3 }
                        v_gap { height: 4.0 }
                        Text { content: "Whiskers excitedly gathered all the animals in the forest and showed them the magical rainbow flower. The animals were amazed and decided to plant more of these flowers to make their home even more magical.".to_string(), variant: TextVariant::P }
                        v_gap { height: 12.0 }
                        Text { content: "As the rainbow flowers bloomed, the entire forest transformed into a kaleidoscope of colors. Birds chirped in harmony, butterflies danced in the air, and even the trees swayed to the rhythm of the wind.".to_string(), variant: TextVariant::P }
                        v_gap { height: spacing::XXL }
                        Text { content: "The Enchanted Celebration".to_string(), variant: TextVariant::H3 }
                        v_gap { height: 4.0 }
                        Text { content: "The animals decided to celebrate their enchanted forest with a grand feast. They gathered nuts, berries, and fruits from the colorful trees and shared stories of their adventures. The joyous laughter echoed through the Rainbow Forest.".to_string(), variant: TextVariant::P }
                        v_gap { height: 12.0 }
                        Text { content: "And so, the Rainbow Forest became a place of wonder and happiness, where Whiskers and all the animals lived together in harmony.".to_string(), variant: TextVariant::Lead }
                        v_gap { height: spacing::XXL }
                        Text { content: "The Never-ending Magic".to_string(), variant: TextVariant::H3 }
                        v_gap { height: 4.0 }
                        Text { content: "The magic of the rainbow flowers continued to spread, reaching other parts of the world. Soon, forests everywhere became vibrant and alive, thanks to the discovery of Whiskers and the enchanted Rainbow Forest.".to_string(), variant: TextVariant::P }
                        v_gap { height: 12.0 }
                        Text { content: "The moral of the story is: embrace the magic of discovery, share joy with others, and watch as the world transforms into a colorful and beautiful place.".to_string(), variant: TextVariant::Large }
                        v_gap { height: spacing::XXL }
                    }
                }
            }
        },
        3 => rsx! {
            fixed_width {
                width: 352.0,
                column {
                    align_items: "center",
                    row {
                        Text { content: "Default:".to_string() }
                        h_gap { width: 4.0 }
                        Text { content: "text-foreground".to_string(), variant: TextVariant::Code }
                    }
                    v_gap { height: spacing::SM }
                    colored_text_row { label: "Inherited from Parent:".to_string(), chip: "text-emerald-500".to_string(), color: 0xFF10B981u32 }
                    v_gap { height: spacing::SM }
                    colored_text_row { label: "Overridden:".to_string(), chip: "text-purple-500".to_string(), color: 0xFFA855F7u32 }
                    v_gap { height: spacing::SM }
                    colored_text_row { label: "Inherited from NestedParent:".to_string(), chip: "text-sky-500".to_string(), color: 0xFF0EA5E9u32 }
                }
            }
        },
        _ => rsx! {
            Text { content: "Hello, world!".to_string() }
        },
    };

    carousel_frame(page, 3, preview, on_page, false)
}

#[component]
fn colored_text_row(label: String, chip: String, color: u32) -> Element {
    let theme = arkit_shadcn::theme::use_theme();
    rsx! {
        row {
            align_self: "start",
            align_items: "center",
            justify_content: "start",
            text {
                content: label,
                font_size: typography::MD,
                font_color: color,
                line_height: 24.0,
            }
            h_gap { width: 4.0 }
            row {
                background_color: theme.colors.muted,
                border_radius: theme.radii.sm,
                padding_top: 3.0,
                padding_right: 5.0,
                padding_bottom: 3.0,
                padding_left: 5.0,
                text {
                    content: chip,
                    font_size: typography::SM,
                    font_family: "monospace",
                    font_weight: 600_i32,
                    font_color: color,
                    line_height: 18.0,
                }
            }
        }
    }
}

fn dropdown_menu_items() -> Vec<MenuEntry> {
    vec![
        MenuEntry::label("My Account"),
        MenuEntry::separator(),
        MenuEntry::action("Team").icon("users"),
        MenuEntry::submenu(
            "Invite users",
            vec![
                MenuEntry::action("Email").icon("mail"),
                MenuEntry::action("Message").icon("message-square"),
                MenuEntry::separator(),
                MenuEntry::action("More...").icon("circle-plus"),
            ],
        )
        .icon("user-plus"),
        MenuEntry::action("New Team").icon("plus").shortcut("⌘+T"),
        MenuEntry::separator(),
        MenuEntry::action("GitHub").icon("github"),
        MenuEntry::action("Support").icon("life-buoy"),
        MenuEntry::action("API").icon("cloud").disabled(),
        MenuEntry::separator(),
        MenuEntry::action("Log out").icon("log-out").shortcut("⇧⌘Q"),
    ]
}

fn context_menu_items(
    bookmarks: bool,
    full_urls: bool,
    person: String,
    on_bookmarks: EventHandler<bool>,
    on_full_urls: EventHandler<bool>,
    on_person: EventHandler<String>,
) -> Vec<MenuEntry> {
    vec![
        MenuEntry::action("Back").inset().shortcut("⌘["),
        MenuEntry::action("Forward")
            .inset()
            .shortcut("⌘]")
            .disabled(),
        MenuEntry::action("Reload").inset().shortcut("⌘R"),
        MenuEntry::submenu(
            "More Tools",
            vec![
                MenuEntry::action("Save Page As...").shortcut("⇧⌘S"),
                MenuEntry::action("Create Shortcut..."),
                MenuEntry::separator(),
                MenuEntry::action("Developer Tools"),
            ],
        )
        .inset(),
        MenuEntry::separator(),
        MenuEntry::checkbox("Show Bookmarks Bar", bookmarks, on_bookmarks).shortcut("⌘⇧B"),
        MenuEntry::checkbox("Show Full URLs", full_urls, on_full_urls),
        MenuEntry::separator(),
        MenuEntry::label("People").inset(),
        MenuEntry::separator(),
        MenuEntry::radio("Elmer Fudd", "pedro", person.clone(), on_person),
        MenuEntry::radio("Foghorn Leghorn", "colm", person, on_person),
    ]
}

fn menubar_menus(
    bookmarks: bool,
    full_urls: bool,
    person: String,
    on_bookmarks: EventHandler<bool>,
    on_full_urls: EventHandler<bool>,
    on_person: EventHandler<String>,
) -> Vec<MenubarMenuSpec> {
    vec![
        MenubarMenuSpec::new(
            "File",
            vec![
                MenuEntry::action("New Tab").shortcut("⌘T"),
                MenuEntry::action("New Window").shortcut("⌘N"),
                MenuEntry::action("New Incognito Window"),
                MenuEntry::separator(),
                MenuEntry::submenu(
                    "Share",
                    vec![
                        MenuEntry::action("Email link"),
                        MenuEntry::action("Messages"),
                        MenuEntry::action("Notes"),
                    ],
                ),
                MenuEntry::separator(),
                MenuEntry::action("Print...").shortcut("⌘P"),
            ],
        ),
        MenubarMenuSpec::new(
            "Edit",
            vec![
                MenuEntry::action("Undo").shortcut("⌘Z"),
                MenuEntry::action("Redo").shortcut("⇧⌘Z"),
                MenuEntry::separator(),
                MenuEntry::submenu(
                    "Find",
                    vec![
                        MenuEntry::action("Search the web"),
                        MenuEntry::separator(),
                        MenuEntry::action("Find..."),
                        MenuEntry::action("Find Next"),
                        MenuEntry::action("Find Previous"),
                    ],
                ),
                MenuEntry::separator(),
                MenuEntry::action("Cut"),
                MenuEntry::action("Copy"),
                MenuEntry::action("Paste"),
            ],
        ),
        MenubarMenuSpec::new(
            "View",
            vec![
                MenuEntry::checkbox("Always Show Bookmarks Bar", bookmarks, on_bookmarks),
                MenuEntry::checkbox("Always Show Full URLs", full_urls, on_full_urls),
                MenuEntry::separator(),
                MenuEntry::action("Reload").shortcut("⌘R"),
                MenuEntry::separator(),
                MenuEntry::action("Toggle Fullscreen").shortcut(""),
                MenuEntry::separator(),
                MenuEntry::action("Hide Sidebar"),
            ],
        ),
        MenubarMenuSpec::new(
            "Profiles",
            vec![
                MenuEntry::radio("Andy", "andy", person.clone(), on_person),
                MenuEntry::radio("Benoit", "benoit", person.clone(), on_person),
                MenuEntry::radio("Luis", "luis", person, on_person),
                MenuEntry::separator(),
                MenuEntry::action("Edit..."),
                MenuEntry::separator(),
                MenuEntry::action("Add Profile..."),
            ],
        ),
    ]
}

fn format_media_time(seconds: f32) -> String {
    let total_seconds = if seconds.is_finite() {
        seconds.max(0.0).round() as u32
    } else {
        0
    };
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}

fn is_valid_email(value: &str) -> bool {
    let value = value.trim();
    if value.contains(char::is_whitespace) {
        return false;
    }

    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    let Some((domain_name, suffix)) = domain.rsplit_once('.') else {
        return false;
    };

    !local.is_empty() && !domain_name.is_empty() && !suffix.is_empty()
}

fn component_title(slug: &str) -> String {
    COMPONENTS
        .iter()
        .find_map(|item| {
            if item.slug == slug {
                Some(item.name.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn theme_mode_key(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

fn theme_preset_key(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Zinc => "zinc",
        ThemePreset::Neutral => "neutral",
        ThemePreset::Stone => "stone",
        ThemePreset::Mauve => "mauve",
        ThemePreset::Olive => "olive",
        ThemePreset::Mist => "mist",
        ThemePreset::Taupe => "taupe",
    }
}

fn theme_preset_label(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Zinc => "Zinc",
        ThemePreset::Neutral => "Neutral",
        ThemePreset::Stone => "Stone",
        ThemePreset::Mauve => "Mauve",
        ThemePreset::Olive => "Olive",
        ThemePreset::Mist => "Mist",
        ThemePreset::Taupe => "Taupe",
    }
}
