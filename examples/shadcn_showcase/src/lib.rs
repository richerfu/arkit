//! shadcn showcase aligned with the pre-Dioxus React Native Reusables demo.

use std::time::Duration;

use arkit::dioxus_core::EventHandler;
use arkit::dioxus_signals::WritableExt;
use arkit::entry;
use arkit::prelude::*;
use arkit::shadcn as arkit_shadcn;
use arkit::shadcn::components::{
    Accordion, AccordionItemSpec, Alert, AlertDescription, AlertDialog, AlertList, AlertTitle,
    AlertVariant, AspectRatio, Avatar, AvatarFallback, Badge, BadgeVariant, BottomNavigation,
    BottomNavigationItem, BottomSheet, BottomSheetTextInput, Button, ButtonSize, ButtonVariant,
    Calendar, Card, CardContent, CardFooter, CardHeader, Carousel, CarouselControlsPlacement,
    CarouselIndicatorVariant, CarouselStyle, Checkbox, Collapsible, ContextMenu, DatePicker,
    Dialog, DialogFooter, DialogHeader, DropdownMenu, Field, FieldContent, FieldDescription,
    FieldError, FieldGroup, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, Form, FormItem,
    HoverCard, Input, InputOtp, InputOtpMode, InputOtpSeparator, Label, Markdown, MenuEntry,
    Menubar, MenubarMenuSpec, MultiSlider, Popover, Progress, RadioGroup, RangeSlider, Select,
    Separator, Skeleton, Slider, SliderOrientation, SliderStyle, Sonner, SonnerPosition,
    SonnerToast, Spinner, Switch, Table, Tabs, Text, TextVariant, Textarea, ToastAppearance,
    Toggle, ToggleGroup, ToggleVariant, Tooltip,
};
use arkit::shadcn::icon::icon_placeholder;
use arkit::shadcn::theme::{
    spacing, typography, ColorTokens, RadiusTokens, Theme, ThemeMode, ThemePreset, ThemeProvider,
};

const HOME_HEADER_HEIGHT: f32 = 80.0;
const DETAIL_HEADER_HEIGHT: f32 = 48.0;
const TRACKING_TIGHT: f32 = -0.35;
const MARKDOWN_STREAM_INTERVAL_MS: u64 = 500;

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
        slug: "radio-group",
        name: "Radio Group",
    },
    ComponentSpec {
        slug: "select",
        name: "Select",
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
];

#[entry]
fn app() -> Element {
    let mut mode = use_signal(|| ThemeMode::Light);
    let mut preset = use_signal(|| ThemePreset::Zinc);
    let mut custom = use_signal(|| false);
    let mut theme_menu_open = use_signal(|| false);
    let mut selected = use_signal(|| None::<&'static str>);
    let mut query = use_signal(String::new);

    let theme = resolve_theme(mode(), preset(), custom());

    let selected_slug = selected();
    let home_key = "home";

    rsx! {
        ThemeProvider {
            theme,
            column {
                percent_width: 1.0,
                percent_height: 1.0,
                background_color: theme.colors.background,
                if let Some(slug) = selected_slug {
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
                            on_back: move |_| selected.set(None),
                            on_theme_menu_open: move |value| theme_menu_open.set(value),
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
                } else {
                    MountTransition {
                        key: "{home_key}",
                        preset: TransitionPreset::SlideRight,
                        duration_ms: 200,
                        fill: true,
                        HomeView {
                            query: query(),
                            mode: mode(),
                            preset: preset(),
                            custom: custom(),
                            theme_menu_open: theme_menu_open(),
                            on_query: move |value: String| query.set(value),
                            on_select: move |slug: &'static str| selected.set(Some(slug)),
                            on_theme_menu_open: move |value| theme_menu_open.set(value),
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
    on_query: EventHandler<String>,
    on_select: EventHandler<&'static str>,
    on_theme_menu_open: EventHandler<bool>,
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
            title: "Showcase".to_string(),
            back: false,
            mode,
            preset,
            custom,
            open: theme_menu_open,
            on_back: move |_| {},
            on_open: on_theme_menu_open,
            on_mode,
            on_preset,
            on_custom,
        }
        column {
            percent_width: 1.0,
            layout_weight: 1.0,
            background_color: theme.colors.background,
            scroll {
                percent_width: 1.0,
                percent_height: 1.0,
                alignment: 0_i32,
                background_color: theme.colors.background,
            column {
                percent_width: 1.0,
                background_color: theme.colors.background,
                align_items: "center",
                justify_content: "start",
                padding_top: spacing::LG,
                padding_right: spacing::LG,
                padding_bottom: spacing::XXL,
                padding_left: spacing::LG,
                column {
                    percent_width: 1.0,
                    max_width_constraint: 512.0,
                    align_items: "start",
                    justify_content: "start",
                    Input {
                        placeholder: Some("Search UI...".to_string()),
                        value: Some(query),
                        percent_width: Some(1.0),
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
                            percent_width: 1.0,
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
}

#[component]
fn DetailView(
    slug: &'static str,
    mode: ThemeMode,
    preset: ThemePreset,
    custom: bool,
    theme_menu_open: bool,
    on_back: EventHandler<()>,
    on_theme_menu_open: EventHandler<bool>,
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
            on_back,
            on_open: on_theme_menu_open,
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
    on_back: EventHandler<()>,
    on_open: EventHandler<bool>,
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
            percent_width: 1.0,
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
                if back {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        onclick: move |_| on_back.call(()),
                        {icon_placeholder("chevron-left", 20.0, theme.colors.foreground)}
                    }
                    row { width: spacing::SM }
                }
                text {
                    content: title,
                    font_size: title_size,
                    font_weight: title_weight,
                    line_height: title_line_height,
                    font_color: theme.colors.foreground,
                    text_letter_spacing: TRACKING_TIGHT,
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
    let active_theme_label = if custom {
        "Custom"
    } else {
        theme_preset_label(preset)
    };
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
                width: 120.0,
                height: 36.0,
                align_items: "center",
                padding_right: spacing::SM,
                padding_left: spacing::SM,
                border_radius: theme.radii.md,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.secondary,
                row {
                    width: 12.0,
                    height: 12.0,
                    border_radius: theme.radii.full,
                    background_color: theme.colors.primary,
                }
                row { width: spacing::SM }
                row {
                    layout_weight: 1.0,
                    clip: true,
                    text {
                        percent_width: 1.0,
                        content: active_theme_label.to_string(),
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.secondary_foreground,
                        line_height: 18.0,
                        max_lines: 1_i32,
                        text_overflow: 2_i32,
                    }
                }
                row { width: spacing::XS }
                {icon_placeholder(icon, 16.0, theme.colors.secondary_foreground)}
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
            percent_width: 1.0,
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
            border_style: 0_i32,
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
    let policy = demo_canvas_policy(slug);
    let bottom_padding = if slug == "bottom-navigation" {
        policy.padding[2]
    } else {
        policy.padding[2] + spacing::XXL
    };

    if policy.fill_height {
        rsx! {
            column {
                percent_width: 1.0,
                layout_weight: 1.0,
                background_color: theme.colors.surface,
                scroll {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    background_color: theme.colors.surface,
                    scroll_enabled: true,
                    column {
                        percent_width: 1.0,
                        percent_height: 1.0,
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
    } else {
        rsx! {
            column {
                percent_width: 1.0,
                layout_weight: 1.0,
                background_color: theme.colors.surface,
                scroll {
                    percent_width: 1.0,
                    percent_height: 1.0,
                    background_color: theme.colors.surface,
                    scroll_enabled: true,
                    column {
                        percent_width: 1.0,
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
        "accordion" => DemoCanvasPolicy {
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
        "sonner" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "input-otp" => DemoCanvasPolicy {
            center_x: false,
            center_y: false,
            fill_height: true,
            padding: [spacing::XXL, spacing::LG, spacing::XXL, spacing::LG],
        },
        "markdown" => DemoCanvasPolicy {
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
    let mut page = use_signal(|| 1_i32);
    let mut dialog_open = use_signal(|| false);
    let mut dialog_name = use_signal(|| "Pedro Duarte".to_string());
    let mut dialog_username = use_signal(|| "@peduarte".to_string());
    let mut alert_open = use_signal(|| false);
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
    let mut otp_value = use_signal(String::new);
    let mut otp_invalid = use_signal(|| false);
    let mut otp_status = use_signal(|| "Enter the six-digit code.".to_string());
    let mut invite_code = use_signal(|| "A7".to_string());
    let mut form_name = use_signal(|| "Avery Stone".to_string());
    let mut form_email = use_signal(String::new);
    let mut form_bio = use_signal(|| "Product designer and weekend cyclist.".to_string());
    let mut form_product_updates = use_signal(|| true);
    let mut form_terms_accepted = use_signal(|| false);
    let mut form_attempted = use_signal(|| false);
    let mut form_status = use_signal(|| None::<bool>);
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

    let async_runtime = arkit::tokio_handle();
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
                Accordion {
                    items: vec![
                        AccordionItemSpec::new(
                            "Product Information",
                            "item-1",
                            rsx! {
                                column {
                                    percent_width: 1.0,
                                    Text { content: "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.".to_string(), variant: TextVariant::Muted }
                                    v_gap { height: spacing::LG }
                                    Text { content: "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.".to_string(), variant: TextVariant::Muted }
                                }
                            },
                        ),
                        AccordionItemSpec::new(
                            "Shipping Details",
                            "item-2",
                            rsx! {
                                column {
                                    percent_width: 1.0,
                                    Text { content: "We offer worldwide shipping through trusted courier partners. Standard delivery takes 3-5 business days, while express shipping ensures delivery within 1-2 business days.".to_string(), variant: TextVariant::Muted }
                                    v_gap { height: spacing::LG }
                                    Text { content: "All orders are carefully packaged and fully insured. Track your shipment in real-time through our dedicated tracking portal.".to_string(), variant: TextVariant::Muted }
                                }
                            },
                        ),
                        AccordionItemSpec::new(
                            "Return Policy",
                            "item-3",
                            rsx! {
                                column {
                                    percent_width: 1.0,
                                    Text { content: "We stand behind our products with a comprehensive 30-day return policy. If you're not completely satisfied, simply return the item in its original condition.".to_string(), variant: TextVariant::Muted }
                                    v_gap { height: spacing::LG }
                                    Text { content: "Our hassle-free return process includes free return shipping and full refunds processed within 48 hours of receiving the returned item.".to_string(), variant: TextVariant::Muted }
                                }
                            },
                        ),
                    ],
                    value: Some(accordion_value()),
                    default_value: Some("item-1".to_string()),
                    collapsible: true,
                    on_value_change: move |value| accordion_value.set(value),
                }
            }
        },
        "alert" => rsx! {
            fixed_width {
                width: 576.0,
                column {
                    percent_width: 1.0,
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
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| alert_open.set(true),
                "Show Alert Dialog"
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
                        percent_width: Some(1.0),
                        onclick: move |_| alert_open.set(false),
                        "Cancel"
                    }
                },
                action: rsx! {
                    Button {
                        percent_width: Some(1.0),
                        onclick: move |_| alert_open.set(false),
                        "Continue"
                    }
                },
            }
        },
        "aspect-ratio" => rsx! {
            column {
                percent_width: 1.0,
                AspectRatio {
                    ratio: 16.0 / 9.0,
                    image {
                        src: "https://images.unsplash.com/photo-1672758247442-82df22f5899e".to_string(),
                        percent_width: 1.0,
                        percent_height: 1.0,
                        object_fit: 1_i32,
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
                    percent_width: 1.0,
                    percent_height: 1.0,
                    background_color: theme.colors.card,
                    column {
                        percent_width: 1.0,
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
            Button {
                onclick: move |_| bottom_sheet_open.set(true),
                "Open"
            }
            BottomSheet {
                title: "Edit your profile".to_string(),
                open: Some(bottom_sheet_open()),
                default_open: Some(false),
                on_close: move |_| bottom_sheet_open.set(false),
                column {
                    percent_width: 1.0,
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
                        percent_width: Some(1.0),
                        onclick: move |_| bottom_sheet_open.set(false),
                        "Save Changes"
                    }
                }
            }
        },
        "button" => rsx! {
            column {
                percent_width: 1.0,
                percent_height: 1.0,
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
                percent_width: 1.0,
                Calendar {
                    selected: calendar_selected(),
                    on_day_press: move |date| calendar_selected.set(Some(date)),
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
                        percent_width: 1.0,
                        percent_height: 1.0,
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
                            text_align: 1_i32,
                        }
                        v_gap { height: spacing::SM }
                        text {
                            content: description,
                            font_size: typography::SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                            text_align: 1_i32,
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
            let overlay_slides = ["First", "Second", "Third", "Fourth"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    rsx! {
                        column {
                            percent_width: 1.0,
                            percent_height: 1.0,
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
                .collect();

            rsx! {
                fixed_width {
                    width: 336.0,
                    column {
                        percent_width: 1.0,
                        Carousel {
                            slides,
                            index: Some(carousel_index()),
                            height: 300.0,
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
                        v_gap { height: spacing::XXL }
                        Carousel {
                            slides: overlay_slides,
                            index: Some(carousel_overlay_index()),
                            height: 220.0,
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
                    }
                }
            }
        }
        "checkbox" => rsx! {
            fixed_width {
                width: 384.0,
                column {
                    percent_width: 1.0,
                    row {
                        align_self: "start",
                        align_items: "center",
                        justify_content: "start",
                        Checkbox {
                            label: Some("Accept terms and conditions".to_string()),
                            checked: Some(checkbox_first()),
                            on_change: Some(EventHandler::new(move |value| checkbox_first.set(value))),
                        }
                    }
                    v_gap { height: spacing::XXL }
                    column {
                        align_self: "start",
                        align_items: "start",
                        Checkbox {
                            label: Some("Accept terms and conditions".to_string()),
                            checked: Some(checkbox_second()),
                            on_change: Some(EventHandler::new(move |value| checkbox_second.set(value))),
                        }
                        row {
                            margin_top: spacing::SM,
                            margin_left: spacing::XXL,
                            Text { content: "By clicking this checkbox, you agree to the terms and conditions.".to_string(), variant: TextVariant::Muted }
                        }
                    }
                    v_gap { height: spacing::XXL }
                    row {
                        align_self: "start",
                        align_items: "center",
                        justify_content: "start",
                        Checkbox {
                            label: Some("Enable notifications".to_string()),
                            checked: Some(false),
                            default_checked: Some(false),
                            disabled: Some(true),
                        }
                    }
                    v_gap { height: spacing::XXL }
                    row {
                        percent_width: 1.0,
                        align_self: "start",
                        align_items: "start",
                        justify_content: "start",
                        padding_top: 12.0,
                        padding_right: 12.0,
                        padding_bottom: 12.0,
                        padding_left: 12.0,
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
                            text {
                                content: "Enable notifications".to_string(),
                                percent_width: 1.0,
                                font_size: typography::SM,
                                font_weight: 500_i32,
                                font_color: theme.colors.foreground,
                                line_height: 14.0,
                            }
                            v_gap { height: spacing::SM }
                            Text { content: "You can enable or disable notifications at any time.".to_string(), variant: TextVariant::Muted }
                        }
                    }
                }
            }
        },
        "collapsible" => rsx! {
            fixed_width {
                width: 350.0,
                Collapsible {
                    title: "@peduarte starred 3 repositories".to_string(),
                    open: Some(toggle_pressed()),
                    default_open: true,
                    on_open_change: EventHandler::new(move |value| toggle_pressed.set(value)),
                    column {
                        percent_width: 1.0,
                        repo_row { name: "@radix-ui/primitives".to_string() }
                        v_gap { height: spacing::SM }
                        repo_row { name: "@radix-ui/react".to_string() }
                        v_gap { height: spacing::SM }
                        repo_row { name: "@stitches/core".to_string() }
                    }
                }
            }
        },
        "context-menu" => rsx! {
            fixed_width {
                width: 300.0,
                column {
                    percent_width: 1.0,
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
                            height: 150.0,
                            alignment: 4_i32,
                            border_width: 1.0,
                            border_color: theme.colors.foreground,
                            border_radius: theme.radii.md,
                            border_style: 1_i32,
                            clip: true,
                            text {
                                content: "Long press here".to_string(),
                                font_size: typography::LG,
                                font_color: theme.colors.foreground,
                                line_height: 22.0,
                            }
                        }
                    }
                    v_gap { height: spacing::LG }
                    Button {
                        variant: ButtonVariant::Outline,
                        percent_width: Some(1.0),
                        onclick: move |_| context_outside_clicks += 1,
                        "Outside click · {context_outside_clicks()}"
                    }
                    v_gap { height: spacing::SM }
                    Text {
                        content: "With the menu open, the first click closes it without activating the button. The next click increments the count.".to_string(),
                        variant: TextVariant::Muted,
                    }
                }
            }
        },
        "date-picker" => rsx! {
            DatePicker {
                selected: date_picker_selected(),
                open: Some(date_picker_open()),
                on_change: move |date| date_picker_selected.set(date),
                on_open_change: move |open| date_picker_open.set(open),
            }
        },
        "dialog" => rsx! {
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| dialog_open.set(true),
                "Edit Profile"
            }
            Dialog {
                open: Some(dialog_open()),
                default_open: Some(false),
                on_close: move |_| dialog_open.set(false),
                DialogHeader {
                    title: "Edit profile".to_string(),
                    description: Some("Make changes to your profile here. Click save when you're done.".to_string()),
                }
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    margin_top: spacing::XL,
                    Label { content: "Name".to_string() }
                    v_gap { height: spacing::SM }
                    Input {
                        value: Some(dialog_name()),
                        placeholder: Some("Your name".to_string()),
                        percent_width: Some(1.0),
                        on_change: move |value| dialog_name.set(value),
                    }
                    v_gap { height: spacing::LG }
                    Label { content: "Username".to_string() }
                    v_gap { height: spacing::SM }
                    Input {
                        value: Some(dialog_username()),
                        placeholder: Some("@username".to_string()),
                        percent_width: Some(1.0),
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
        },
        "dropdown-menu" => rsx! {
            fixed_width {
                width: 384.0,
                DropdownMenu {
                    open: Some(menu_open()),
                    default_open: false,
                    on_open_change: Some(EventHandler::new(move |value| menu_open.set(value))),
                    width: Some(288.0),
                    items: dropdown_menu_items(),
                    Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Open" }
                }
            }
        },
        "form" => rsx! {
            fixed_width {
                width: 440.0,
                column {
                    percent_width: 1.0,
                    height: 1110.0,
                    align_items: "start",
                    text {
                        content: "Account settings".to_string(),
                        percent_width: 1.0,
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                        text_align: 0_i32,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        content: "Update your public profile and communication preferences.".to_string(),
                        percent_width: 1.0,
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: 0_i32,
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
                                        percent_width: Some(1.0),
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
                                        percent_width: Some(1.0),
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
                                        percent_width: Some(1.0),
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
                                percent_width: 1.0,
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
                                percent_width: 1.0,
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
                                        percent_width: 1.0,
                                        font_size: typography::XS,
                                        font_weight: 500_i32,
                                        font_color: if success { theme.colors.chart_2 } else { theme.colors.destructive },
                                        line_height: 18.0,
                                        text_align: 0_i32,
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
                HoverCard {
                    open: Some(hover_open()),
                    default_open: Some(false),
                    on_close: move |_| hover_open.set(false),
                    on_open_change: move |value| hover_open.set(value),
                    width: Some(320.0),
                    trigger: rsx! { Button { variant: ButtonVariant::Link, onclick: move |_| {}, "@expo" } },
                    row {
                        percent_width: 1.0,
                        align_items: "start",
                        justify_content: "start",
                        {demo_avatar("https://github.com/expo.png", "E", false, None)}
                        h_gap { width: spacing::LG }
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            justify_content: "start",
                            text {
                                content: "@expo".to_string(),
                                font_size: typography::SM,
                                font_weight: 600_i32,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                                text_align: 0_i32,
                            }
                            v_gap { height: 4.0 }
                            text {
                                content: "Framework and tools for creating native apps with React.".to_string(),
                                font_size: typography::SM,
                                font_color: theme.colors.foreground,
                                line_height: 20.0,
                                text_align: 0_i32,
                            }
                            v_gap { height: 4.0 }
                            text {
                                content: "Joined December 2021".to_string(),
                                font_size: typography::XS,
                                font_color: theme.colors.muted_foreground,
                                line_height: 16.0,
                                text_align: 0_i32,
                            }
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
                Input { placeholder: Some("Email".to_string()), percent_width: Some(1.0) }
            }
        },
        "input-otp" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    text {
                        percent_width: 1.0,
                        content: "Verify your email".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: Some(1.0),
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
                        percent_width: Some(1.0),
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
                        percent_width: 1.0,
                        align_items: "start",
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
        },
        "popover" => rsx! {
            Popover {
                open: Some(popover_open()),
                default_open: Some(false),
                on_close: move |_| popover_open.set(false),
                on_open_change: move |value| popover_open.set(value),
                width: Some(320.0),
                trigger: rsx! { Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Open popover" } },
                column {
                    percent_width: 1.0,
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
        },
        "progress" => rsx! {
            fixed_width {
                width: 420.0,
                column {
                    percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                        text_align: 0_i32,
                    }
                    v_gap { height: spacing::XL }
                    row {
                        percent_width: 1.0,
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
        "radio-group" => rsx! {
            fixed_width {
                width: 384.0,
                RadioGroup {
                    options: vec!["Default".to_string(), "Comfortable".to_string(), "Compact".to_string()],
                    selected: Some(radio_choice()),
                    default_selected: "Default".to_string(),
                    on_select: move |value| radio_choice.set(value),
                }
            }
        },
        "select" => rsx! {
            {select_carousel(page(), selected_fruit(), select_open(), on_page, EventHandler::new(move |value| selected_fruit.set(value)), EventHandler::new(move |value| select_open.set(value)))}
        },
        "separator" => rsx! {
            fixed_width {
                width: 320.0,
                column {
                    percent_width: 1.0,
                    align_items: "start",
                    column {
                        percent_width: 1.0,
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
                    percent_width: 1.0,
                    padding: spacing::LG,
                    background_color: theme.colors.background,
                    border_radius: theme.radii.lg,
                    border_width: 1.0,
                    border_color: theme.colors.border,
                    row {
                        align_items: "center",
                        justify_content: "start",
                        Skeleton { width: 48.0, height: 48.0 }
                        h_gap { width: spacing::LG }
                        column {
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
                    percent_width: 1.0,
                    height: 1064.0,
                    align_items: "start",
                    text {
                        percent_width: 1.0,
                        content: "Sound & haptics".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        percent_width: 1.0,
                        content: "Tune playback, output levels, and channel balance.".to_string(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::XXL }

                    column {
                        percent_width: 1.0,
                        align_items: "start",
                        padding: spacing::LG,
                        background_color: theme.colors.card,
                        border_style: 0_i32,
                        border_width: 1.0,
                        border_color: theme.colors.border,
                        border_radius: theme.radii.xl,
                        row {
                            percent_width: 1.0,
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
                            percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                        percent_width: 1.0,
                        content: "Vertical controls keep minimum at the bottom.".to_string(),
                        font_size: typography::XS,
                        font_color: theme.colors.muted_foreground,
                        line_height: 18.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        percent_width: 1.0,
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
                        percent_width: 1.0,
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
                    percent_width: 1.0,
                    align_items: "start",
                    text {
                        percent_width: 1.0,
                        content: "Notifications".to_string(),
                        font_size: typography::XXL,
                        font_weight: 700_i32,
                        font_color: theme.colors.foreground,
                        line_height: 32.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        percent_width: 1.0,
                        content: "Bottom-center Sonner stack (official peeks). Swipe up to expand, down to collapse/dismiss. Minimal is a compact chip.".to_string(),
                        font_size: typography::SM,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::LG }
                    Button {
                        variant: ButtonVariant::Outline,
                        percent_width: Some(1.0),
                        onclick: move |_| sonner_background_clicks += 1,
                        "Background click test · {sonner_background_clicks()}"
                    }
                    v_gap { height: spacing::XXL }
                    text {
                        percent_width: 1.0,
                        content: "Notification".to_string(),
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        percent_width: 1.0,
                        Button {
                            variant: ButtonVariant::Outline,
                            percent_width: Some(0.48),
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
                            percent_width: Some(0.48),
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
                        percent_width: 1.0,
                        Button {
                            variant: ButtonVariant::Secondary,
                            percent_width: Some(0.48),
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
                            percent_width: Some(0.48),
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
                        percent_width: 1.0,
                        Button {
                            variant: ButtonVariant::Destructive,
                            percent_width: Some(0.48),
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
                            percent_width: Some(0.48),
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
                        percent_width: Some(1.0),
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
                        percent_width: 1.0,
                        content: "Minimal".to_string(),
                        font_size: typography::SM,
                        font_weight: 600_i32,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                    v_gap { height: spacing::SM }
                    text {
                        percent_width: 1.0,
                        content: "Compact chip — short copy only.".to_string(),
                        font_size: typography::XS,
                        font_weight: 400_i32,
                        font_color: theme.colors.muted_foreground,
                        line_height: 16.0,
                    }
                    v_gap { height: spacing::MD }
                    row {
                        percent_width: 1.0,
                        Button {
                            variant: ButtonVariant::Outline,
                            percent_width: Some(0.31),
                            onclick: move |_| enqueue_sonner_toast(
                                sonner_toasts,
                                sonner_next_id,
                                |id| SonnerToast::minimal(id, "Copied"),
                            ),
                            "Copy"
                        }
                        row { layout_weight: 1.0 }
                        Button {
                            percent_width: Some(0.31),
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
                            percent_width: Some(0.31),
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
                        percent_width: 1.0,
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
                    percent_width: 1.0,
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
            row {
                align_items: "center",
                justify_content: "start",
                Switch {
                    checked: Some(switch_checked()),
                    on_change: Some(EventHandler::new(move |value| switch_checked.set(value))),
                }
                h_gap { width: spacing::SM }
                Label { content: "Airplane Mode".to_string() }
            }
        },
        "tabs" => rsx! {
            row {
                percent_width: 1.0,
                max_width_constraint: 384.0,
                Tabs {
                    labels: vec!["Account".to_string(), "Password".to_string()],
                    panels: vec![
                        rsx! {
                            Card {
                                CardHeader {
                                    title: "Account".to_string(),
                                    description: "Make changes to your account here. Click save when you're done.".to_string(),
                                }
                                CardContent {
                                    column {
                                        percent_width: 1.0,
                                        column {
                                            percent_width: 1.0,
                                            Label { content: "Name".to_string() }
                                            v_gap { height: spacing::XXS }
                                            Input {
                                                placeholder: Some("Pedro Duarte".to_string()),
                                                value: Some("Pedro Duarte".to_string()),
                                                percent_width: Some(1.0),
                                            }
                                        }
                                        v_gap { height: spacing::SM }
                                        column {
                                            percent_width: 1.0,
                                            Label { content: "Username".to_string() }
                                            v_gap { height: spacing::XXS }
                                            Input {
                                                placeholder: Some("@peduarte".to_string()),
                                                value: Some("@peduarte".to_string()),
                                                percent_width: Some(1.0),
                                            }
                                        }
                                    }
                                }
                                CardFooter {
                                    Button { onclick: move |_| {}, "Save changes" }
                                }
                            }
                        },
                        rsx! {
                            Card {
                                CardHeader {
                                    title: "Password".to_string(),
                                    description: "Change your password here. After saving, you'll be logged out.".to_string(),
                                }
                                CardContent {
                                    column {
                                        percent_width: 1.0,
                                        column {
                                            percent_width: 1.0,
                                            Label { content: "Current password".to_string() }
                                            v_gap { height: spacing::XXS }
                                            Input {
                                                placeholder: Some("********".to_string()),
                                                percent_width: Some(1.0),
                                            }
                                        }
                                        v_gap { height: spacing::SM }
                                        column {
                                            percent_width: 1.0,
                                            Label { content: "New password".to_string() }
                                            v_gap { height: spacing::XXS }
                                            Input {
                                                placeholder: Some("********".to_string()),
                                                percent_width: Some(1.0),
                                            }
                                        }
                                    }
                                }
                                CardFooter {
                                    Button { onclick: move |_| {}, "Save password" }
                                }
                            }
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
                Textarea { placeholder: Some("Type your message here.".to_string()), percent_width: Some(1.0) }
            }
        },
        "toggle" => rsx! {
            // Outline so pressed/unpressed stay visible on light Zinc surfaces
            // (Default accent fill is nearly the same as the canvas).
            Toggle {
                label: "".to_string(),
                icon: Some("bold".to_string()),
                variant: ToggleVariant::Outline,
                checked: Some(toggle_pressed()),
                on_change: EventHandler::new(move |value| toggle_pressed.set(value)),
            }
        },
        "toggle-group" => rsx! {
            ToggleGroup {
                options: vec!["bold".to_string(), "italic".to_string(), "underline".to_string()],
                selected: Some(toggle_values()),
                default_selected: vec!["bold".to_string()],
                icons: true,
                multi: true,
                on_change: move |values| toggle_values.set(values),
            }
        },
        "tooltip" => rsx! {
            Tooltip {
                open: Some(tooltip_open()),
                default_open: Some(false),
                on_close: move |_| tooltip_open.set(false),
                on_open_change: move |value| tooltip_open.set(value),
                content: "Add to library".to_string(),
                trigger: rsx! { Button { variant: ButtonVariant::Outline, onclick: move |_| {}, "Press" } },
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
        _ => rsx! {
            Text { content: "Component not found".to_string(), variant: TextVariant::Muted }
        },
    }
}

#[component]
fn fixed_width(width: f32, children: Element) -> Element {
    // shadcn-style max-width cap: fill the parent up to `width`, never force a
    // hard width that can overflow narrow screens (512vp ≈ 1664px @3.25x).
    // Select/Popover still measure the painted control; anchor geometry no
    // longer depends on this wrapper using an absolute width.
    rsx! {
        column {
            percent_width: 1.0,
            align_items: "center",
            column {
                percent_width: 1.0,
                max_width_constraint: width,
                align_items: "stretch",
                {children}
            }
        }
    }
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
                alignment: 4_i32,
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
            percent_width: 1.0,
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
            percent_width: 1.0,
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
                    percent_width: Some(1.0),
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
            percent_width: 1.0,
            percent_height: 1.0,
            row {
                percent_width: 1.0,
                percent_height: 1.0,
                align_items: "center",
                justify_content: "center",
                padding_bottom: if reserve_bottom_controls { 48.0 + spacing::LG } else { 0.0 },
                {preview}
            }
            column {
                percent_width: 1.0,
                percent_height: 1.0,
                align_items: "center",
                justify_content: "end",
                hit_test_behavior: 2_i32,
                row {
                    percent_width: 1.0,
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
            border_style: 0_i32,
            clip: true,
            opacity: if disabled { 0.5 } else { 1.0 },
            shadow: 1_i32,
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
                    percent_width: 1.0,
                    percent_height: 1.0,
                    column {
                        percent_width: 1.0,
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
