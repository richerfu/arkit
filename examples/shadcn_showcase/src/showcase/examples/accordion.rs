use super::super::layout::{max_width, v_stack};
use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct AccordionExample {
    ctx: DemoContext,
}

impl AccordionExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for AccordionExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            top_center_canvas(
        max_width(
            shadcn::Accordion::single(
                vec![
                    shadcn::AccordionItemSpec::new(
                        "Product Information",
                        "item-1",
                        vec![v_stack(
                            vec![
                                shadcn::Text::small(
                                    "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                                ).into(),
                                shadcn::Text::small(
                                    "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                                ).into(),
                            ],
                            16.0,
                        )],
                    ),
                    shadcn::AccordionItemSpec::new(
                        "Shipping Details",
                        "item-2",
                        vec![v_stack(
                            vec![
                                shadcn::Text::small(
                                    "We offer worldwide shipping through trusted courier partners. Standard delivery takes 3-5 business days, while express shipping ensures delivery within 1-2 business days.",
                                ).into(),
                                shadcn::Text::small(
                                    "All orders are carefully packaged and fully insured. Track your shipment in real-time through our dedicated tracking portal.",
                                ).into(),
                            ],
                            16.0,
                        )],
                    ),
                    shadcn::AccordionItemSpec::new(
                        "Return Policy",
                        "item-3",
                        vec![v_stack(
                            vec![
                                shadcn::Text::small(
                                    "We stand behind our products with a comprehensive 30-day return policy. If you're not completely satisfied, simply return the item in its original condition.",
                                ).into(),
                                shadcn::Text::small(
                                    "Our hassle-free return process includes free return shipping and full refunds processed within 48 hours of receiving the returned item.",
                                ).into(),
                            ],
                            16.0,
                        )],
                    ),
                ],
            )
            .value(ctx.accordion_single_open.clone())
            .collapsible(true)
            .on_value_change(Message::SetAccordionSingleOpen)
            .into(),
            512.0,
        ),
        [0.0, 24.0, 0.0, 24.0],
        false,
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
