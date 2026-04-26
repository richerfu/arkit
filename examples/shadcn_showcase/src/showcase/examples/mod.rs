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
use crate::prelude::*;
use arkit_shadcn as shadcn;
pub(crate) use shared::DemoContext;

pub(crate) struct ExampleRenderer {
    slug: String,
    ctx: DemoContext,
}

impl ExampleRenderer {
    pub(crate) fn new(slug: impl Into<String>, ctx: DemoContext) -> Self {
        Self {
            slug: slug.into(),
            ctx,
        }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for ExampleRenderer {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some(match self.slug.as_str() {
            "accordion" => Element::new(accordion::AccordionExample::new(ctx)),
            "alert" => Element::new(alert::AlertExample::new(ctx)),
            "alert-dialog" => Element::new(alert_dialog::AlertDialogExample::new(ctx)),
            "aspect-ratio" => Element::new(aspect_ratio::AspectRatioExample::new(ctx)),
            "avatar" => Element::new(avatar::AvatarExample::new(ctx)),
            "badge" => Element::new(badge::BadgeExample::new(ctx)),
            "button" => Element::new(button::ButtonExample::new(ctx)),
            "card" => Element::new(card::CardExample::new(ctx)),
            "checkbox" => Element::new(checkbox::CheckboxExample::new(ctx)),
            "collapsible" => Element::new(collapsible::CollapsibleExample::new(ctx)),
            "context-menu" => Element::new(context_menu::ContextMenuExample::new(ctx)),
            "dialog" => Element::new(dialog::DialogExample::new(ctx)),
            "dropdown-menu" => Element::new(dropdown_menu::DropdownMenuExample::new(ctx)),
            "hover-card" => Element::new(hover_card::HoverCardExample::new(ctx)),
            "icon" => Element::new(icon::IconExample::new(ctx)),
            "input" => Element::new(input::InputExample::new(ctx)),
            "label" => Element::new(label::LabelExample::new(ctx)),
            "menubar" => Element::new(menubar::MenubarExample::new(ctx)),
            "popover" => Element::new(popover::PopoverExample::new(ctx)),
            "progress" => Element::new(progress::ProgressExample::new(ctx)),
            "radio-group" => Element::new(radio_group::RadioGroupExample::new(ctx)),
            "select" => Element::new(select::SelectExample::new(ctx)),
            "separator" => Element::new(separator::SeparatorExample::new(ctx)),
            "skeleton" => Element::new(skeleton::SkeletonExample::new(ctx)),
            "switch" => Element::new(switch::SwitchExample::new(ctx)),
            "tabs" => Element::new(tabs::TabsExample::new(ctx)),
            "text" => Element::new(text::TextExample::new(ctx)),
            "textarea" => Element::new(textarea::TextareaExample::new(ctx)),
            "toggle" => Element::new(toggle::ToggleExample::new(ctx)),
            "toggle-group" => Element::new(toggle_group::ToggleGroupExample::new(ctx)),
            "tooltip" => Element::new(tooltip::TooltipExample::new(ctx)),
            "table" => Element::new(table::TableExample::new(ctx)),
            _ => component_canvas(
                shadcn::Card::new(vec![
                    shadcn::CardTitle::new("Route Not Found").into(),
                    shadcn::CardDescription::new("Please return to list and retry.").into(),
                ])
                .into(),
                true,
                24.0,
            ),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}
