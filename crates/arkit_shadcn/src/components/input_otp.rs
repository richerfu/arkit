use super::*;

pub fn input_otp(value: Signal<String>, digits: usize) -> Element {
    let chars = value.get().chars().collect::<Vec<_>>();
    arkit::row_component()
        .percent_width(1.0)
        .children(
            (0..digits)
                .map(|idx| {
                    let otp = value.clone();
                    input_surface(
                        arkit::text_input_component()
                            .value(chars.get(idx).map(char::to_string).unwrap_or_default())
                            .width(36.0)
                            .height(36.0)
                            .font_size(typography::SM)
                            .on_change(move |next| {
                                let mut current = otp.get().chars().collect::<Vec<_>>();
                                if current.len() < digits {
                                    current.resize(digits, '\0');
                                }
                                current[idx] = next.chars().next().unwrap_or('\0');
                                let next_value = current
                                    .into_iter()
                                    .filter(|ch| *ch != '\0')
                                    .collect::<String>();
                                if otp.get() != next_value {
                                    otp.set(next_value);
                                }
                            }),
                    )
                    .into()
                })
                .collect(),
        )
        .into()
}
