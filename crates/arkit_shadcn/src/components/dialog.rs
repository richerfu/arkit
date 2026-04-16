use super::*;
use std::rc::Rc;

pub(crate) const DIALOG_MAX_WIDTH: f32 = 512.0;
const DIALOG_VIEWPORT_INSET: f32 = spacing::LG;

pub(crate) fn modal_overlay<Message: 'static>(
    open: bool,
    panel: Element<Message>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    arkit::modal_overlay(
        if open { Some(panel) } else { None },
        arkit::ModalOverlaySpec {
            open,
            presentation: arkit::ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: on_dismiss.is_some(),
            backdrop_color: 0x80000000,
            viewport_inset: DIALOG_VIEWPORT_INSET,
        },
        on_dismiss,
    )
}

pub(crate) fn dialog_panel<Message: Send + 'static>(
    content: Vec<Element<Message>>,
    on_close: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    let close_button = icon_button::<Message>("x")
        .theme(ButtonVariant::Ghost)
        .width(28.0)
        .height(28.0)
        .padding(arkit::Padding::ZERO)
        .opacity(0.7_f32)
        .on_click(move || {
            if let Some(close) = on_close.as_ref() {
                close();
            }
        });

    shadow_sm(
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .max_width_constraint(DIALOG_MAX_WIDTH)
            .padding([spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL])
            .border_radius([radii().lg, radii().lg, radii().lg, radii().lg])
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(colors().border)
            .background_color(colors().background)
            .children(vec![
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .justify_content_end()
                    .children(vec![close_button.into()])
                    .into(),
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .children(vec![stack(content, spacing::LG)])
                    .into(),
            ]),
    )
    .into()
}

fn dialog_impl<Message: Send + 'static>(
    _title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message> {
    let dismiss = Rc::new(move || on_open_change(false));
    modal_overlay(
        open,
        dialog_panel(content, Some(dismiss.clone())),
        Some(dismiss),
    )
}

pub fn dialog_message<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    dialog_impl(
        title,
        open,
        move |value| dispatch_message(on_open_change(value)),
        content,
    )
}

pub fn dialog_footer<Message: 'static>(actions: Vec<Element<Message>>) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(
            actions
                .into_iter()
                .rev()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component::<Message, arkit::Theme>()
                            .percent_width(1.0)
                            .margin([spacing::SM, 0.0, 0.0, 0.0])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub fn dialog_header<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<Message> {
    let title = title.into();
    let description = description.into();
    let mut children = vec![arkit::text::<Message, arkit::Theme>(title)
        .font_size(typography::LG)
        .font_weight(FontWeight::W600)
        .font_color(colors().foreground)
        .line_height(18.0)
        .into()];
    if !description.is_empty() {
        children.push(
            margin_top(
                arkit::text::<Message, arkit::Theme>(description)
                    .font_size(typography::SM)
                    .font_color(colors().muted_foreground)
                    .line_height(20.0),
                spacing::SM,
            )
            .into(),
        );
    }
    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
