use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            h_stack(
                vec![
                    shadcn::skeleton(48.0, 48.0),
                    v_stack(
                        vec![shadcn::skeleton(250.0, 16.0), shadcn::skeleton(200.0, 16.0)],
                        shadcn::theme::spacing::SM,
                    ),
                ],
                shadcn::theme::spacing::LG,
            ),
            320.0,
        ),
        true,
        24.0,
    )
}
