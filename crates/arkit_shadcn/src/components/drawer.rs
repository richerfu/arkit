use super::*;

pub fn drawer<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    super::dialog::dialog_message(title, open, on_open_change, content)
}
