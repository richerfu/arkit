use super::card::card;
use super::*;

fn chart(values: Vec<f32>) -> Element {
    let palette = [
        colors().chart_1,
        colors().chart_2,
        colors().chart_3,
        colors().chart_4,
        colors().chart_5,
    ];

    card(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let percent = value.clamp(0.0, 100.0);
                let tone = palette[index % palette.len()];

                arkit::column_component()
                    .percent_width(1.0)
                    .children(vec![
                        arkit::row_component()
                            .percent_width(1.0)
                            .align_items_center()
                            .justify_content(JustifyContent::SpaceBetween)
                            .children(vec![
                                muted_text(format!("Series {}", index + 1)).into(),
                                body_text_regular(format!("{percent:.0}%")).into(),
                            ])
                            .into(),
                        arkit::row_component()
                            .margin([spacing::XXS, 0.0, 0.0, 0.0])
                            .children(vec![rounded_progress(
                                arkit::progress(percent, 100.0)
                                    .progress_color(tone)
                                    .height(8.0),
                            )
                            .into()])
                            .into(),
                    ])
                    .into()
            })
            .collect(),
    )
}

fn chart_card(title: impl Into<String>, values: Vec<f32>) -> Element {
    card(vec![title_text(title).into(), chart(values)])
}
