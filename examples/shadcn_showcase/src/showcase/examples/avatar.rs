use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct AvatarExample {
    ctx: DemoContext,
}

impl AvatarExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for AvatarExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas(
                h_stack(
                    vec![
                        shadcn::Avatar::new(
                            Some(String::from("https://github.com/mrzachnugent.png")),
                            "ZN",
                        )
                        .ring(true)
                        .into(),
                        shadcn::Avatar::new(
                            Some(String::from("https://github.com/shadcn.png")),
                            "CN",
                        )
                        .ring(true)
                        .radius(shadcn::theme::radii().lg)
                        .into(),
                        h_stack(
                            vec![
                                shadcn::Avatar::new(
                                    Some(String::from("https://github.com/mrzachnugent.png")),
                                    "ZN",
                                )
                                .ring(true)
                                .into(),
                                shadcn::Avatar::new(
                                    Some(String::from("https://github.com/leerob.png")),
                                    "LR",
                                )
                                .ring(true)
                                .into(),
                                shadcn::Avatar::new(
                                    Some(String::from("https://github.com/evilrabbit.png")),
                                    "ER",
                                )
                                .ring(true)
                                .into(),
                            ],
                            -8.0,
                        ),
                    ],
                    48.0,
                ),
                true,
                24.0,
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
