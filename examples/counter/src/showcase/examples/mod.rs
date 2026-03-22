mod shared;

mod accordion;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod avatar;
mod badge;
mod button;
mod card;
mod checkbox;
mod collapsible;
mod context_menu;
mod dialog;
mod dropdown_menu;
mod hover_card;
mod icon;
mod input;
mod label;
mod menubar;
mod popover;
mod progress;
mod radio_group;
mod select;
mod separator;
mod skeleton;
mod switch;
mod table;
mod tabs;
mod text;
mod textarea;
mod toggle;
mod toggle_group;
mod tooltip;

use super::layout::component_canvas;
use arkit::prelude::*;
use arkit_shadcn as shadcn;
pub(crate) use shared::DemoContext;

pub(crate) fn render(slug: &str, ctx: DemoContext) -> Element {
    match slug {
        "accordion" => accordion::render(ctx),
        "alert" => alert::render(ctx),
        "alert-dialog" => alert_dialog::render(ctx),
        "aspect-ratio" => aspect_ratio::render(ctx),
        "avatar" => avatar::render(ctx),
        "badge" => badge::render(ctx),
        "button" => button::render(ctx),
        "card" => card::render(ctx),
        "checkbox" => checkbox::render(ctx),
        "collapsible" => collapsible::render(ctx),
        "context-menu" => context_menu::render(ctx),
        "dialog" => dialog::render(ctx),
        "dropdown-menu" => dropdown_menu::render(ctx),
        "hover-card" => hover_card::render(ctx),
        "icon" => icon::render(ctx),
        "input" => input::render(ctx),
        "label" => label::render(ctx),
        "menubar" => menubar::render(ctx),
        "popover" => popover::render(ctx),
        "progress" => progress::render(ctx),
        "radio-group" => radio_group::render(ctx),
        "select" => select::render(ctx),
        "separator" => separator::render(ctx),
        "skeleton" => skeleton::render(ctx),
        "switch" => switch::render(ctx),
        "tabs" => tabs::render(ctx),
        "text" => text::render(ctx),
        "textarea" => textarea::render(ctx),
        "toggle" => toggle::render(ctx),
        "toggle-group" => toggle_group::render(ctx),
        "tooltip" => tooltip::render(ctx),
        "table" => table::render(ctx),
        _ => component_canvas(
            shadcn::card(vec![
                shadcn::card_title("Route Not Found"),
                shadcn::card_description("Please return to list and retry."),
            ]),
            true,
            24.0,
        ),
    }
}
