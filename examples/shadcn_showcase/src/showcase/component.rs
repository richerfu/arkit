use crate::prelude::*;

use super::constants::component_title;
use super::examples::{self, DemoContext};
use super::layout::{nav_bar, ThemeMenuState};

pub(crate) fn component_page(name: String, ctx: DemoContext) -> Element {
    let title = component_title(&name);
    let demo_name = name.clone();

    arkit::column(vec![
        nav_bar(
            title,
            true,
            ThemeMenuState {
                mode: ctx.theme_mode,
                preset: ctx.theme_preset,
                custom: ctx.custom_theme,
                open: ctx.theme_menu_open,
            },
        ),
        examples::render(&demo_name, ctx),
    ])
}
