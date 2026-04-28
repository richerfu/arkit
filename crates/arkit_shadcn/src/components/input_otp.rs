use super::*;

fn input_otp<Message>(
    value: impl Into<String>,
    digits: usize,
    on_input: impl Fn(String) -> Message + Clone + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    let value = value.into();
    arkit::row_component()
        .percent_width(1.0)
        .children(
            (0..digits)
                .map(|idx| {
                    let otp = value.clone();
                    let on_input = on_input.clone();
                    let ch = value
                        .chars()
                        .nth(idx)
                        .map(|c| c.to_string())
                        .unwrap_or_default();
                    input_surface(
                        arkit::text_input_component::<Message, arkit::Theme>()
                            .value(ch)
                            .width(36.0)
                            .height(36.0)
                            .font_size(typography::SM)
                            .on_input(move |next| {
                                let mut current = otp.chars().collect::<Vec<_>>();
                                if current.len() < digits {
                                    current.resize(digits, '\0');
                                }
                                current[idx] = next.chars().next().unwrap_or('\0');
                                let next_value = current
                                    .into_iter()
                                    .filter(|ch| *ch != '\0')
                                    .collect::<String>();
                                on_input(next_value)
                            }),
                    )
                    .into()
                })
                .collect::<Vec<Element<Message>>>(),
        )
        .into()
}
