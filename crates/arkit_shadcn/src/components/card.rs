use super::*;

pub(super) fn card<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    card_surface(
        arkit::column_component::<Message, arkit::Theme>()
            .width(arkit::Length::Fill)
            .children(vec![stack(children, spacing::XXL)]),
    )
    .into()
}

fn card_header<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .children(vec![arkit::column_component::<Message, arkit::Theme>()
            .width(arkit::Length::Fill)
            .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
            .children(vec![
                card_title(title),
                arkit::row_component::<Message, arkit::Theme>()
                    .margin_top(spacing::XS)
                    .children(vec![card_description(description)])
                    .into(),
            ])
            .into()])
        .into()
}

fn card_title<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::MD)
        .font_weight(FontWeight::W600)
        .font_color(colors().foreground)
        .line_height(16.0)
        .text_letter_spacing(-0.2_f32)
        .into()
}

fn card_description<Message: 'static>(content: impl Into<String>) -> Element<Message> {
    muted_text(content).into()
}

fn card_content<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
        .children(children)
        .into()
}

fn card_footer<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .padding([0.0, spacing::XXL, 0.0, spacing::XXL])
        .align_items_center()
        .children(children)
        .into()
}

// Struct component API
pub struct Card<Message = ()> {
    children: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> Card<Message> {
    pub fn new(children: Vec<Element<Message>>) -> Self {
        Self {
            children: std::cell::RefCell::new(Some(children)),
        }
    }
}

impl_component_widget!(Card<Message>, Message, |value: &Card<Message>| {
    card(super::take_component_slot(&value.children, "card children"))
});

pub struct CardHeader<Message = ()> {
    title: String,
    description: Option<String>,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> CardHeader<Message> {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: Some(description.into()),
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(CardHeader<Message>, Message, |value: &CardHeader<
    Message,
>| {
    card_header(
        value.title.clone(),
        value.description.clone().unwrap_or_default(),
    )
});

pub struct CardTitle<Message = ()> {
    content: String,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> CardTitle<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(CardTitle<Message>, Message, |value: &CardTitle<Message>| {
    card_title(value.content.clone())
});

pub struct CardDescription<Message = ()> {
    content: String,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> CardDescription<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(
    CardDescription<Message>,
    Message,
    |value: &CardDescription<Message>| { card_description(value.content.clone()) }
);

pub struct CardContent<Message = ()> {
    children: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> CardContent<Message> {
    pub fn new(children: Vec<Element<Message>>) -> Self {
        Self {
            children: std::cell::RefCell::new(Some(children)),
        }
    }
}

impl_component_widget!(CardContent<Message>, Message, |value: &CardContent<
    Message,
>| {
    card_content(super::take_component_slot(
        &value.children,
        "card content children",
    ))
});

pub struct CardFooter<Message = ()> {
    children: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> CardFooter<Message> {
    pub fn new(children: Vec<Element<Message>>) -> Self {
        Self {
            children: std::cell::RefCell::new(Some(children)),
        }
    }
}

impl_component_widget!(CardFooter<Message>, Message, |value: &CardFooter<
    Message,
>| {
    card_footer(super::take_component_slot(
        &value.children,
        "card footer children",
    ))
});
