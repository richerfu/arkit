use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct TableExample {
    ctx: DemoContext,
}

impl TableExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for TableExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::Table::new(
                        vec![
                            String::from("Invoice"),
                            String::from("Status"),
                            String::from("Method"),
                            String::from("Amount"),
                        ],
                        vec![
                            vec![
                                String::from("INV001"),
                                String::from("Paid"),
                                String::from("Credit Card"),
                                String::from("$250.00"),
                            ],
                            vec![
                                String::from("INV002"),
                                String::from("Pending"),
                                String::from("PayPal"),
                                String::from("$150.00"),
                            ],
                            vec![
                                String::from("INV003"),
                                String::from("Unpaid"),
                                String::from("Bank Transfer"),
                                String::from("$350.00"),
                            ],
                            vec![
                                String::from("INV004"),
                                String::from("Paid"),
                                String::from("Credit Card"),
                                String::from("$450.00"),
                            ],
                        ],
                    )
                    .into(),
                    560.0,
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
