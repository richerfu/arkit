use super::shared::{button_carousel, no_padding_center_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct ButtonExample {
    ctx: DemoContext,
}

impl ButtonExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for ButtonExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            let mut children = vec![button_carousel(ctx.page).into()];
            if let Some(feedback) = ctx.button_preview_feedback {
                children.push(
                    arkit::row_component()
                        .margin_top(16.0)
                        .children(vec![shadcn::Text::with_variant(
                            feedback,
                            shadcn::TextVariant::Muted,
                        )
                        .into()])
                        .into(),
                );
            }

            no_padding_center_canvas(
                arkit::column_component()
                    .percent_width(1.0)
                    .percent_height(1.0)
                    .align_items_center()
                    .justify_content_center()
                    .children(children)
                    .into(),
            )
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// struct component render
