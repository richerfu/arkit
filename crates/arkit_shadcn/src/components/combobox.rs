use super::*;

pub fn combobox<Message>(
    options: Vec<String>,
    selected: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    super::select::select_message(options, selected, open, on_open_change, on_select)
}
