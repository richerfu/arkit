use crate::prelude::*;
use arkit::router::RouterMessage;
use arkit_icon as lucide;
use arkit_shadcn as shadcn;
use shadcn::theme::{ThemeMode, ThemePreset};

use crate::{Message, Route, ShowcaseState};

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

#[derive(Debug, Clone, Copy)]
pub(crate) struct ThemeMenuState {
    pub(crate) mode: ThemeMode,
    pub(crate) preset: ThemePreset,
    pub(crate) custom: bool,
    pub(crate) open: bool,
}

impl From<&ShowcaseState> for ThemeMenuState {
    fn from(state: &ShowcaseState) -> Self {
        Self {
            mode: state.theme_mode,
            preset: state.theme_preset,
            custom: state.custom_theme,
            open: state.theme_menu_open,
        }
    }
}

fn empty_box(width: f32, height: f32) -> Element {
    arkit::row_component().width(width).height(height).into()
}

fn nav_title_text(title: impl Into<String>, home: bool) -> Element {
    let title = title.into();
    let mut text = arkit::text(title)
        .font_color(shadcn::theme::colors().foreground)
        .text_letter_spacing(TRACKING_TIGHT);

    if home {
        text = text
            .font_size(34.0)
            .font_weight(FontWeight::W700)
            .line_height(40.0);
    } else {
        text = text
            .font_size(17.0)
            .font_weight(FontWeight::W500)
            .line_height(22.0);
    }

    text.into()
}

fn plain_back_button() -> Element {
    shadcn::Button::icon("chevron-left")
        .theme(shadcn::ButtonVariant::Ghost)
        .width(36.0)
        .height(36.0)
        .padding(arkit::Padding::ZERO)
        .on_press(Message::Router(RouterMessage::back()))
        .into()
}

fn theme_menu_icon(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "sun",
        ThemeMode::Dark => "moon",
    }
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

fn theme_menu_button(state: ThemeMenuState) -> Element {
    let selected_preset = if state.custom {
        String::from("custom")
    } else {
        String::from(theme_preset_key(state.preset))
    };

    let mut items = vec![
        shadcn::MenuEntry::label("Appearance"),
        shadcn::MenuEntry::radio(
            "Light",
            theme_mode_key(ThemeMode::Light),
            theme_mode_key(state.mode),
            |_| Message::SetThemeMode(ThemeMode::Light),
        ),
        shadcn::MenuEntry::radio(
            "Dark",
            theme_mode_key(ThemeMode::Dark),
            theme_mode_key(state.mode),
            |_| Message::SetThemeMode(ThemeMode::Dark),
        ),
        shadcn::MenuEntry::separator(),
        shadcn::MenuEntry::label("Theme"),
    ];

    items.extend(THEME_PRESETS.iter().map(|preset| {
        let preset = *preset;
        shadcn::MenuEntry::radio(
            theme_preset_label(preset),
            theme_preset_key(preset),
            selected_preset.clone(),
            move |_| Message::SetThemePreset(preset),
        )
    }));
    items.extend([
        shadcn::MenuEntry::separator(),
        shadcn::MenuEntry::radio("Custom", "custom", selected_preset, |_| {
            Message::SetCustomTheme(true)
        }),
    ]);

    shadcn::DropdownMenu::new(
        shadcn::Button::icon(theme_menu_icon(state.mode))
            .theme(shadcn::ButtonVariant::Ghost)
            .width(36.0)
            .height(36.0)
            .padding(arkit::Padding::ZERO)
            .on_press(Message::SetThemeMenuOpen(!state.open))
            .into(),
        items,
    )
    .open(state.open)
    .on_open_change(Message::SetThemeMenuOpen)
    .align(FloatingAlign::End)
    .into()
}

fn constrained_width(child: Element, width: f32) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_center()
        .children(vec![arkit::row_component()
            .percent_width(1.0)
            .max_width_constraint(width)
            .justify_content_center()
            .children(vec![child])
            .into()])
        .into()
}

fn showcase_horizontal_padding(value: f32) -> f32 {
    value
}

fn canvas_row(content: Element, center_x: bool) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .justify_content(if center_x {
            JustifyContent::Center
        } else {
            JustifyContent::Start
        })
        .children(vec![content])
        .into()
}

pub(crate) fn fixed_width(child: Element, width: f32) -> Element {
    constrained_width(child, width)
}

pub(crate) fn max_width(child: Element, width: f32) -> Element {
    constrained_width(child, width)
}

pub(crate) fn v_stack(children: Vec<Element>, gap: f32) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .percent_width(1.0)
                            .margin([gap, 0.0, 0.0, 0.0])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub(crate) fn h_stack(children: Vec<Element>, gap: f32) -> Element {
    arkit::row_component()
        .align_items_center()
        .children(
            children
                .into_iter()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component()
                            .margin([0.0, 0.0, 0.0, gap])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

#[allow(dead_code)]
pub(crate) fn page_scroll(children: Vec<Element>) -> Element {
    arkit::scroll_component()
        .percent_width(1.0)
        .layout_weight(1.0_f32)
        .background_color(shadcn::theme::colors().surface)
        .children(vec![arkit::column_component()
            .percent_width(1.0)
            .padding([
                0.0,
                shadcn::theme::spacing::LG,
                shadcn::theme::spacing::XXL,
                shadcn::theme::spacing::LG,
            ])
            .children(vec![v_stack(children, shadcn::theme::spacing::LG)])
            .into()])
        .into()
}

pub(crate) fn component_canvas(content: Element, centered: bool, padding: f32) -> Element {
    component_canvas_with(
        content,
        centered,
        centered,
        true,
        [
            padding,
            showcase_horizontal_padding(padding),
            padding,
            showcase_horizontal_padding(padding),
        ],
    )
}

pub(crate) fn component_canvas_with(
    content: Element,
    center_x: bool,
    center_y: bool,
    fill_height: bool,
    padding: [f32; 4],
) -> Element {
    let container = if fill_height {
        arkit::column_component()
            .percent_width(1.0)
            .percent_height(1.0)
    } else {
        arkit::column_component().percent_width(1.0)
    };

    arkit::scroll_component()
        .percent_width(1.0)
        .layout_weight(1.0_f32)
        .background_color(shadcn::theme::colors().surface)
        .children(vec![container
            .padding([
                padding[0],
                padding[1],
                padding[2] + shadcn::theme::spacing::XXL,
                padding[3],
            ])
            .align_items_start()
            .justify_content(if center_y {
                JustifyContent::Center
            } else {
                JustifyContent::Start
            })
            .children(vec![canvas_row(content, center_x)])
            .into()])
        .into()
}

pub(crate) fn nav_bar(title: impl Into<String>, back: bool, theme: ThemeMenuState) -> Element {
    let title = title.into();

    if !back {
        let left: Element = arkit::row_component()
            .layout_weight(1.0_f32)
            .align_items_bottom()
            .children(vec![nav_title_text(title, true)])
            .into();

        return arkit::row_component()
            .percent_width(1.0)
            .height(HOME_HEADER_HEIGHT)
            .background_color(shadcn::theme::colors().background)
            .padding([
                18.0,
                shadcn::theme::spacing::LG,
                6.0,
                shadcn::theme::spacing::LG,
            ])
            .align_items_bottom()
            .children(vec![left, theme_menu_button(theme)])
            .into();
    }

    let left = arkit::row_component()
        .layout_weight(1.0_f32)
        .align_items_center()
        .justify_content_start()
        .children(vec![
            if back {
                plain_back_button()
            } else {
                empty_box(36.0, 36.0)
            },
            arkit::row_component()
                .margin([0.0, 0.0, 0.0, 8.0])
                .children(vec![nav_title_text(title, false)])
                .into(),
        ])
        .into();

    let right = theme_menu_button(theme);

    arkit::row_component()
        .percent_width(1.0)
        .height(DETAIL_HEADER_HEIGHT)
        .background_color(shadcn::theme::colors().background)
        .padding([
            4.0,
            shadcn::theme::spacing::LG,
            4.0,
            shadcn::theme::spacing::LG,
        ])
        .align_items_center()
        .children(vec![left, right])
        .into()
}

pub(crate) fn component_list_cell(slug: &str, title: &str, first: bool, last: bool) -> Element {
    let slug = slug.to_string();
    let border_width = if last {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        [1.0, 1.0, 0.0, 1.0]
    };
    let radius = shadcn::theme::radii().lg;
    let border_radius = [
        if first { radius } else { 0.0 },
        if first { radius } else { 0.0 },
        if last { radius } else { 0.0 },
        if last { radius } else { 0.0 },
    ];

    arkit::row_component()
        .percent_width(1.0)
        .height(44.0)
        .align_items_center()
        .justify_content(JustifyContent::SpaceBetween)
        .padding([12.0, 16.0, 12.0, 14.0])
        .border_width(border_width)
        .border_color(shadcn::theme::colors().border)
        .border_radius(border_radius)
        .background_color(shadcn::theme::colors().card)
        .on_press(Message::Router(Route::Component { slug }.router_message()))
        .children(vec![
            arkit::text(title)
                .font_size(shadcn::theme::typography::MD)
                .font_color(shadcn::theme::colors().foreground)
                .font_weight(FontWeight::W400)
                .line_height(20.0)
                .into(),
            lucide::icon("chevron-right")
                .size(16.0)
                .stroke_width(1.5)
                .color(shadcn::theme::colors().muted_foreground)
                .render(),
        ])
        .into()
}
