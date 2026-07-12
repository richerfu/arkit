//! shadcn showcase aligned with the pre-Dioxus React Native Reusables demo.

use arkit::dioxus_core::EventHandler;
use arkit::dioxus_signals::WritableExt;
use arkit::entry;
use arkit::prelude::*;
use arkit_shadcn::components::{
    Accordion, AccordionItemSpec, Alert, AlertDescription, AlertDialog, AlertList, AlertTitle,
    AlertVariant, AspectRatio, Avatar, AvatarFallback, Badge, BadgeVariant, Button, ButtonSize,
    ButtonVariant, Card, CardContent, CardFooter, CardHeader, Checkbox, Collapsible, ContextMenu,
    Dialog, DialogFooter, DialogHeader, DropdownMenu, HoverCard, Input, Label, MenuEntry, Menubar,
    MenubarMenuSpec, Popover, Progress, RadioGroup, Select, Separator, Skeleton, Switch, Table,
    Tabs, Text, TextVariant, Textarea, Toggle, ToggleGroup, Tooltip,
};
use arkit_shadcn::icon::icon_placeholder;
use arkit_shadcn::theme::{
    spacing, typography, use_theme_provider, ColorTokens, RadiusTokens, Theme, ThemeMode,
    ThemePreset,
};

const HOME_HEADER_HEIGHT: f32 = 80.0;
const DETAIL_HEADER_HEIGHT: f32 = 48.0;
const TRACKING_TIGHT: f32 = -0.35;

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
        slug: "button",
        name: "Button",
    },
    ComponentSpec {
        slug: "card",
        name: "Card",
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
        slug: "dialog",
        name: "Dialog",
    },
    ComponentSpec {
        slug: "dropdown-menu",
        name: "Dropdown Menu",
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
        slug: "label",
        name: "Label",
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
    let mut theme_signal = use_theme_provider(resolve_theme(mode(), preset(), custom()));

    let theme = resolve_theme(mode(), preset(), custom());
    theme_signal.set(theme);

    let selected_slug = selected();
    let home_key = "home";

    rsx! {
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

fn resolve_theme(mode: ThemeMode, preset: ThemePreset, custom: bool) -> Theme {
    if custom {
        return Theme::custom(custom_theme_colors(mode))
            .with_mode(mode)
            .with_radius(RadiusTokens::from_base(10.0));
    }
    Theme::preset(preset, mode)
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
                background_color: theme.colors.background,
            column {
                percent_width: 1.0,
                background_color: theme.colors.background,
                align_items: "center",
                padding_top: spacing::LG,
                padding_right: spacing::LG,
                padding_bottom: spacing::XXL,
                padding_left: spacing::LG,
                column {
                    percent_width: 1.0,
                    max_width_constraint: 512.0,
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
        ),
        MenuEntry::radio(
            "Dark",
            "dark",
            selected_mode,
            EventHandler::new(move |_| on_mode.call(ThemeMode::Dark)),
        ),
        MenuEntry::separator(),
        MenuEntry::label("Theme"),
    ];

    for item in THEME_PRESETS {
        items.push(MenuEntry::radio(
            theme_preset_label(item),
            theme_preset_key(item),
            selected_preset.clone(),
            EventHandler::new(move |_| on_preset.call(item)),
        ));
    }
    items.push(MenuEntry::separator());
    items.push(MenuEntry::radio(
        "Custom",
        "custom",
        selected_preset,
        EventHandler::new(move |_| on_custom.call(true)),
    ));

    rsx! {
        DropdownMenu {
            items,
            open: Some(open),
            default_open: false,
            on_open_change: Some(on_open),
            trigger_capture: Some(false),
            row {
                width: 36.0,
                height: 36.0,
                align_items: "center",
                justify_content: "center",
                border_radius: theme.radii.md,
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
                        padding_bottom: policy.padding[2] + spacing::XXL,
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
                        padding_bottom: policy.padding[2] + spacing::XXL,
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
    let mut alert_open = use_signal(|| false);
    let mut popover_open = use_signal(|| false);
    let mut hover_open = use_signal(|| false);
    let mut tooltip_open = use_signal(|| false);
    let mut menu_open = use_signal(|| false);
    let mut context_open = use_signal(|| false);
    let mut context_bookmarks = use_signal(|| true);
    let mut context_full_urls = use_signal(|| false);
    let mut context_person = use_signal(|| "pedro".to_string());
    let mut menubar_active = use_signal(|| None::<usize>);
    let mut select_open = use_signal(|| false);
    let mut selected_fruit = use_signal(|| "Apple".to_string());
    let mut accordion_value = use_signal(|| Some("item-1".to_string()));
    let mut radio_choice = use_signal(|| "Default".to_string());
    let mut checkbox_first = use_signal(|| false);
    let mut checkbox_second = use_signal(|| false);
    let mut checkbox_card = use_signal(|| false);
    let mut switch_checked = use_signal(|| false);
    let mut toggle_pressed = use_signal(|| false);
    let mut toggle_values = use_signal(|| vec!["bold".to_string()]);

    let theme = arkit_shadcn::theme::use_theme();
    let on_page = EventHandler::new(move |value: i32| page.set(value.max(1)));

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
                            label: None,
                            checked: Some(checkbox_first()),
                            on_change: Some(EventHandler::new(move |value| checkbox_first.set(value))),
                        }
                        h_gap { width: 12.0 }
                        Label { content: "Accept terms and conditions".to_string() }
                    }
                    v_gap { height: spacing::XXL }
                    row {
                        align_self: "start",
                        align_items: "start",
                        justify_content: "start",
                        Checkbox {
                            label: None,
                            checked: Some(checkbox_second()),
                            on_change: Some(EventHandler::new(move |value| checkbox_second.set(value))),
                        }
                        h_gap { width: 12.0 }
                        column {
                            layout_weight: 1.0,
                            align_items: "start",
                            Label { content: "Accept terms and conditions".to_string() }
                            v_gap { height: spacing::SM }
                            Text { content: "By clicking this checkbox, you agree to the terms and conditions.".to_string(), variant: TextVariant::Muted }
                        }
                    }
                    v_gap { height: spacing::XXL }
                    row {
                        align_self: "start",
                        align_items: "start",
                        justify_content: "start",
                        Checkbox {
                            label: None,
                            checked: Some(false),
                            default_checked: Some(false),
                            disabled: Some(true),
                        }
                        h_gap { width: 12.0 }
                        text {
                            content: "Enable notifications".to_string(),
                            font_size: typography::SM,
                            font_color: theme.colors.foreground,
                            opacity: 0.5,
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
                DialogFooter {
                    Button {
                        onclick: move |_| dialog_open.set(false),
                        "OK"
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
                width: 288.0,
                Progress { value: 66.0, total: Some(100.0) }
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
                    column {
                        percent_width: 1.0,
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
            fixed_width {
                width: 320.0,
                row {
                    align_items: "center",
                    justify_content: "start",
                    Skeleton { width: 48.0, height: 48.0 }
                    h_gap { width: spacing::LG }
                    column {
                        Skeleton { width: 250.0, height: 16.0 }
                        v_gap { height: spacing::SM }
                        Skeleton { width: 200.0, height: 16.0 }
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
            Toggle {
                label: "".to_string(),
                icon: Some("bold".to_string()),
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
    rsx! {
        column {
            percent_width: 1.0,
            align_items: "center",
            column {
                percent_width: 1.0,
                max_width_constraint: width,
                align_items: "center",
                {children}
            }
        }
    }
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
