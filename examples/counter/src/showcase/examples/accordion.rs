use super::super::layout::{max_width, v_stack};
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(_ctx: DemoContext) -> Element {
    let item1 = use_signal(|| true);
    let item2 = use_signal(|| false);
    let item3 = use_signal(|| false);

    top_center_canvas(
        max_width(
            shadcn::accordion(vec![
                shadcn::accordion_item(
                    "Product Information",
                    item1,
                    vec![v_stack(
                        vec![
                            shadcn::text_sm(
                                "Our flagship product combines cutting-edge technology with sleek design. Built with premium materials, it offers unparalleled performance and reliability.",
                            ),
                            shadcn::text_sm(
                                "Key features include advanced processing capabilities, and an intuitive user interface designed for both beginners and experts.",
                            ),
                        ],
                        16.0,
                    )],
                ),
                shadcn::accordion_item(
                    "Shipping Details",
                    item2,
                    vec![v_stack(
                        vec![
                            shadcn::text_sm(
                                "We offer worldwide shipping through trusted courier partners. Standard delivery takes 3-5 business days, while express shipping ensures delivery within 1-2 business days.",
                            ),
                            shadcn::text_sm(
                                "All orders are carefully packaged and fully insured. Track your shipment in real-time through our dedicated tracking portal.",
                            ),
                        ],
                        16.0,
                    )],
                ),
                shadcn::accordion_item(
                    "Return Policy",
                    item3,
                    vec![v_stack(
                        vec![
                            shadcn::text_sm(
                                "We stand behind our products with a comprehensive 30-day return policy. If you're not completely satisfied, simply return the item in its original condition.",
                            ),
                            shadcn::text_sm(
                                "Our hassle-free return process includes free return shipping and full refunds processed within 48 hours of receiving the returned item.",
                            ),
                        ],
                        16.0,
                    )],
                ),
            ]),
            512.0,
        ),
        [0.0, 24.0, 0.0, 24.0],
        false,
    )
}
