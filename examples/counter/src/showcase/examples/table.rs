use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            shadcn::table(
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
            ),
            560.0,
        ),
        true,
        24.0,
    )
}
