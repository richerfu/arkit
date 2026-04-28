use super::floating_layer::{floating_panel_with_builder, FloatingAlign, FloatingSide};
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

const COMBOBOX_PANEL_FALLBACK_WIDTH: f32 = 240.0;

fn combobox<Message>(
    options: Vec<String>,
    selected: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    let selected = selected.into();
    let on_open_change = Rc::new(move |value| dispatch_message(on_open_change(value)));
    let on_select = Rc::new(move |value| dispatch_message(on_select(value)));

    let trigger = touch_activate(
        shadow_sm(crate::styles::rounded(
            crate::styles::border(
                arkit::row_component::<Message, arkit::Theme>()
                    .height(40.0)
                    .percent_width(1.0)
                    .background_color(colors().background)
                    .padding([8.0, spacing::MD, 8.0, spacing::MD])
                    .align_items_center()
                    .justify_content(JustifyContent::SpaceBetween)
                    .children(vec![
                        arkit::row_component::<Message, arkit::Theme>()
                            .align_items_center()
                            .children(vec![
                                lucide::icon("search")
                                    .size(16.0)
                                    .color(colors().muted_foreground)
                                    .render::<Message, arkit::Theme>(),
                                arkit::row_component::<Message, arkit::Theme>()
                                    .margin([0.0, 0.0, 0.0, spacing::SM])
                                    .children(vec![arkit::text::<Message, arkit::Theme>(
                                        if selected.is_empty() {
                                            String::from("Search an option")
                                        } else {
                                            selected.clone()
                                        },
                                    )
                                    .font_size(typography::SM)
                                    .font_color(if selected.is_empty() {
                                        colors().muted_foreground
                                    } else {
                                        colors().foreground
                                    })
                                    .line_height(20.0)
                                    .into()])
                                    .into(),
                            ])
                            .into(),
                        lucide::icon("chevrons-up-down")
                            .size(16.0)
                            .color(colors().muted_foreground)
                            .render::<Message, arkit::Theme>(),
                    ]),
            ),
            radii().md,
        )),
        {
            let on_open_change = on_open_change.clone();
            move || on_open_change(!open)
        },
    )
    .into();

    let panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>> = Rc::new({
        let options = options.clone();
        let selected = selected.clone();
        let on_select = on_select.clone();
        let on_open_change = on_open_change.clone();
        move |trigger_width| {
            let items = options
                .iter()
                .cloned()
                .map(|option| {
                    let active = selected == option;
                    let option_value = option.clone();
                    let on_select = on_select.clone();
                    let on_open_change = on_open_change.clone();
                    touch_activate(
                        arkit::row_component::<Message, arkit::Theme>()
                            .percent_width(1.0)
                            .height(36.0)
                            .align_items_center()
                            .justify_content(JustifyContent::SpaceBetween)
                            .padding([8.0, spacing::SM, 8.0, spacing::SM])
                            .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
                            .background_color(if active { colors().accent } else { 0x00000000 })
                            .children(vec![
                                arkit::text::<Message, arkit::Theme>(option.clone())
                                    .font_size(typography::SM)
                                    .font_color(if active {
                                        colors().accent_foreground
                                    } else {
                                        colors().foreground
                                    })
                                    .line_height(20.0)
                                    .into(),
                                if active {
                                    lucide::icon("check")
                                        .size(16.0)
                                        .color(colors().foreground)
                                        .render::<Message, arkit::Theme>()
                                } else {
                                    arkit::row_component::<Message, arkit::Theme>()
                                        .width(16.0)
                                        .height(16.0)
                                        .into()
                                },
                            ]),
                        move || {
                            on_select(option_value.clone());
                            on_open_change(false);
                        },
                    )
                    .into()
                })
                .collect::<Vec<_>>();

            let mut panel = arkit::column_component::<Message, arkit::Theme>().children(vec![
                arkit::row_component::<Message, arkit::Theme>()
                    .padding([8.0, spacing::SM, 8.0, spacing::SM])
                    .children(vec![arkit::text::<Message, arkit::Theme>("Suggestions")
                        .font_size(typography::XS)
                        .font_color(colors().muted_foreground)
                        .line_height(16.0)
                        .into()])
                    .into(),
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .padding([spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS])
                    .children(items)
                    .into(),
            ]);

            panel = panel.width(trigger_width.unwrap_or(COMBOBOX_PANEL_FALLBACK_WIDTH));

            panel_surface(panel).into()
        }
    });

    floating_panel_with_builder(
        trigger,
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        panel_builder,
        Some(Rc::new(move || on_open_change(false))),
    )
}
