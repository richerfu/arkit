use super::*;

pub fn input_otp(value: Signal<String>, digits: usize) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(
            (0..digits)
                .map(|idx| {
                    let otp = value.clone();
                    let watch = value.clone();
                    input_surface(
                        arkit::text_input_component()
                            // Use watch_signal to reactively update each input's
                            // displayed character when the `value` signal changes.
                            // The previous code read `value.get()` once at construction
                            // time, so external value changes never updated the inputs.
                            .watch_signal(watch, move |node, val| {
                                let ch = val
                                    .chars()
                                    .nth(idx)
                                    .map(|c| c.to_string())
                                    .unwrap_or_default();
                                node.set_text_input_text(ch)
                            })
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
