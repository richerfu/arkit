use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

fn repo_row(name: &str) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .padding([8.0, 16.0, 8.0, 16.0])
        .border_width([1.0, 1.0, 1.0, 1.0])
        .border_color(shadcn::theme::colors().border)
        .border_radius([
            shadcn::theme::radii().md,
            shadcn::theme::radii().md,
            shadcn::theme::radii().md,
            shadcn::theme::radii().md,
        ])
        .children(vec![shadcn::Text::small(name).into()])
        .into()
}

pub(crate) struct CollapsibleExample {
    ctx: DemoContext,
}

impl CollapsibleExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for CollapsibleExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::Collapsible::new(
                        "@peduarte starred 3 repositories",
                        vec![
                            repo_row("@radix-ui/primitives"),
                            repo_row("@radix-ui/react"),
                            repo_row("@stitches/core"),
                        ],
                    )
                    .open(ctx.toggle_state)
                    .on_open_change(Message::SetToggleState)
                    .into(),
                    350.0,
                ),
                true,
                32.0,
            )
        })
    }
}

// struct component render
