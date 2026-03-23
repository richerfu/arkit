use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        h_stack(
            vec![
                shadcn::avatar_ring(
                    Some(String::from("https://github.com/mrzachnugent.png")),
                    "ZN",
                ),
                shadcn::avatar_ring_with_radius(
                    Some(String::from("https://github.com/shadcn.png")),
                    "CN",
                    shadcn::theme::radius::LG,
                ),
                h_stack(
                    vec![
                        shadcn::avatar_ring(
                            Some(String::from("https://github.com/mrzachnugent.png")),
                            "ZN",
                        ),
                        shadcn::avatar_ring(
                            Some(String::from("https://github.com/leerob.png")),
                            "LR",
                        ),
                        shadcn::avatar_ring(
                            Some(String::from("https://github.com/evilrabbit.png")),
                            "ER",
                        ),
                    ],
                    -8.0,
                ),
            ],
            48.0,
        ),
        true,
        24.0,
    )
}
