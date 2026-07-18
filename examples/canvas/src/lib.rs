//! W3C-style Canvas 2D API showcase.

use std::f32::consts::{PI, TAU};

use arkit::entry;
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut rotation = use_signal(|| 0.0_f32);
    let mut dashed = use_signal(|| true);
    let angle = rotation();
    let show_dash = dashed();
    let renderer = CanvasRenderer::new(move |context| {
        draw_showcase(context, angle, show_dash);
    });

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: 0xFFF1F5F9u32,
            column {
                padding: 16.0,
                background_color: 0xFFFFFFFFu32,
                text {
                    font_size: 24.0,
                    line_height: 30.0,
                    font_weight: 700,
                    "Canvas 2D"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF475569u32,
                    "W3C-style context · ArkUI native custom draw · high-DPI logical pixels"
                }
                row {
                    margin_top: 12.0,
                    button {
                        width: 150.0,
                        onclick: move |_| rotation += PI / 12.0,
                        "Rotate"
                    }
                    button {
                        width: 150.0,
                        margin_left: 8.0,
                        onclick: move |_| {
                            let value = dashed();
                            dashed.set(!value);
                        },
                        if show_dash { "Solid line" } else { "Dashed line" }
                    }
                }
            }
            column {
                percent_width: 1.0,
                layout_weight: 1.0,
                padding: 16.0,
                Canvas {
                    draw: renderer,
                    percent_width: 1.0,
                    percent_height: 1.0,
                }
            }
        }
    }
}

fn draw_showcase(context: &mut CanvasRenderingContext2D<'_>, rotation: f32, dashed: bool) {
    let width = context.width();
    if let Ok(background) = context.create_linear_gradient(0.0, 0.0, width, context.height()) {
        let _ = background.add_color_stop(0.0, "#f8fafc");
        let _ = background.add_color_stop(0.55, "#eef2ff");
        let _ = background.add_color_stop(1.0, "#ecfeff");
        context.set_fill_style(background);
    }
    context.fill_rect(0.0, 0.0, width, context.height());

    draw_grid(context);

    let card = context
        .create_linear_gradient(width * 0.16, 70.0, width * 0.84, 230.0)
        .ok();
    context.save();
    context.translate(width * 0.5, 150.0);
    context.rotate(rotation);
    context.set_shadow_color("rgba(30, 41, 59, 0.32)");
    context.set_shadow_blur(22.0);
    context.set_shadow_offset_y(10.0);
    context.begin_path();
    let _ = context.round_rect(-112.0, -62.0, 224.0, 124.0, [26.0, 10.0, 26.0, 10.0]);
    if let Some(card) = card {
        let _ = card.add_color_stop(0.0, "#2563eb");
        let _ = card.add_color_stop(0.52, "#7c3aed");
        let _ = card.add_color_stop(1.0, "#db2777");
        context.set_fill_style(card);
    }
    context.fill();
    context.set_shadow_color("transparent");
    context.set_stroke_style("rgba(255, 255, 255, 0.78)");
    context.set_line_width(2.0);
    context.stroke();
    context.set_fill_style("white");
    context.set_font("700 20px sans-serif");
    context.set_text_align(CanvasTextAlign::Center);
    context.fill_text_max_width("gradient · shadow · roundRect", 0.0, 6.0, 196.0);
    context.restore();

    // arcTo, cubic curves and dashes share one W3C current path.
    context.begin_path();
    context.move_to(width * 0.06, 292.0);
    let _ = context.arc_to(width * 0.16, 238.0, width * 0.27, 292.0, 28.0);
    context.bezier_curve_to(
        width * 0.36,
        220.0,
        width * 0.51,
        380.0,
        width * 0.68,
        300.0,
    );
    context.set_stroke_style("#e11d48");
    context.set_line_width(6.0);
    context.set_line_cap(CanvasLineCap::Round);
    if dashed {
        context.set_line_dash(&[14.0, 8.0]);
    }
    context.stroke();

    // Ellipse uses arbitrary radii and rotation; a radial gradient is fixed
    // in the coordinate space in which it was created.
    context.begin_path();
    let _ = context.ellipse(width * 0.84, 298.0, 42.0, 28.0, 0.45, 0.0, TAU, false);
    let radial = context
        .create_radial_gradient(width * 0.82, 286.0, 2.0, width * 0.84, 298.0, 44.0)
        .ok();
    if let Some(radial) = radial {
        let _ = radial.add_color_stop(0.0, "#fef08a");
        let _ = radial.add_color_stop(0.45, "#14b8a6");
        let _ = radial.add_color_stop(1.0, "#0f766e");
        context.set_fill_style(radial);
    }
    context.fill();
    context.set_line_dash(&[]);
    context.set_stroke_style("#0f766e");
    context.set_line_width(3.0);
    context.stroke();

    // Path2D supports SVG construction, cloning and transformed addPath.
    if let Ok(star) = Path2D::from_svg(
        "M 0 -24 L 7 -8 L 24 -8 L 11 3 L 16 21 L 0 11 L -16 21 L -11 3 L -24 -8 L -7 -8 Z",
    ) {
        let mut stars = Path2D::new();
        stars.add_path(
            &star,
            Some(DomMatrix2D::new(1.0, 0.0, 0.0, 1.0, width * 0.12, 382.0)),
        );
        stars.add_path(
            &star,
            Some(DomMatrix2D::new(0.7, 0.0, 0.0, 0.7, width * 0.24, 382.0)),
        );
        if let Ok(conic) = context.create_conic_gradient(-PI * 0.5, width * 0.17, 382.0) {
            let _ = conic.add_color_stop(0.0, "#f59e0b");
            let _ = conic.add_color_stop(0.5, "#ef4444");
            let _ = conic.add_color_stop(1.0, "#f59e0b");
            context.set_fill_style(conic);
        }
        context.fill_path(&stars);
    }

    draw_image_data_sample(context, width * 0.52, 356.0);

    context.set_fill_style("#0f172a");
    context.set_font("700 22px sans-serif");
    context.set_text_align(CanvasTextAlign::Center);
    context.set_text_baseline(CanvasTextBaseline::Alphabetic);
    context.fill_text_max_width(
        "Path2D · ImageData · Pattern · TextMetrics",
        width * 0.5,
        452.0,
        width - 24.0,
    );

    context.set_font("400 14px sans-serif");
    context.set_fill_style("#475569");
    let metrics = context.measure_text("CanvasRenderingContext2D");
    context.fill_text(
        &format!(
            "measured {:.1}px · ascent {:.1}px · DPR {:.2}",
            metrics.width,
            metrics.actual_bounding_box_ascent,
            context.device_pixel_ratio()
        ),
        width * 0.5,
        480.0,
    );

    context.set_font("700 28px sans-serif");
    context.set_line_width(1.5);
    context.set_stroke_style("#2563eb");
    context.stroke_text_max_width("native Canvas 2D", width * 0.5, 520.0, width - 48.0);

    // Apply text states to the visible native glyphs and exercise gradient
    // text under a transform applied after gradient creation.
    if let Ok(text_gradient) = context.create_linear_gradient(16.0, 0.0, width - 16.0, 0.0) {
        let _ = text_gradient.add_color_stop(0.0, "#0891b2");
        let _ = text_gradient.add_color_stop(0.5, "#7c3aed");
        let _ = text_gradient.add_color_stop(1.0, "#e11d48");
        context.set_fill_style(text_gradient);
    }
    context.save();
    context.translate(width * 0.5, 550.0);
    context.rotate(-0.025);
    context.set_font("small-caps condensed 700 12pt sans-serif");
    context.set_font_variant_caps(CanvasFontVariantCaps::AllSmallCaps);
    context.set_font_kerning(CanvasFontKerning::None);
    context.set_text_rendering(CanvasTextRendering::GeometricPrecision);
    context.set_letter_spacing("0.06em");
    context.fill_text_max_width(
        "font caps · stretch · spacing · geometric text",
        0.0,
        0.0,
        width - 32.0,
    );
    context.restore();
}

fn draw_image_data_sample(context: &mut CanvasRenderingContext2D<'_>, x: f32, y: f32) {
    let Ok(mut pixels) = ImageData::new_with_settings(
        12,
        12,
        ImageDataSettings {
            color_space: Some(CanvasColorSpace::DisplayP3),
            pixel_format: ImageDataPixelFormat::RgbaFloat16,
        },
    ) else {
        return;
    };
    let Some(data) = pixels.rgba_float16_mut() else {
        return;
    };
    for row in 0..12_usize {
        for column in 0..12_usize {
            let offset = (row * 12 + column) * 4;
            let accent = (row / 3 + column / 3) % 2 == 0;
            let color = if accent {
                [0.1, 0.35, 1.0, 1.0]
            } else {
                [0.82, 0.88, 1.0, 1.0]
            };
            for (channel, value) in data[offset..offset + 4].iter_mut().zip(color) {
                *channel = Float16::from_f32(value);
            }
        }
    }
    let image = CanvasImage::from_image_data(&pixels);
    context.save();
    context.translate(x, y);
    context.rotate(-0.12);
    context.draw_image_scaled(&image, 0.0, 0.0, 74.0, 58.0);
    context.restore();

    let pattern = context.create_pattern(&image, CanvasPatternRepetition::Repeat);
    pattern.set_transform(DomMatrix2D::scaling(2.0, 2.0));
    context.set_fill_style(pattern);
    context.save();
    context.translate(x + 121.0, y + 29.0);
    context.rotate(0.1);
    context.begin_path();
    let _ = context.round_rect(-37.0, -29.0, 74.0, 58.0, 12.0);
    context.fill();
    context.restore();
}

fn draw_grid(context: &mut CanvasRenderingContext2D<'_>) {
    context.save();
    context.set_stroke_style("#e2e8f0");
    context.set_line_width(1.0);
    context.begin_path();
    let mut x = 0.0;
    while x <= context.width() {
        context.move_to(x, 0.0);
        context.line_to(x, context.height());
        x += 24.0;
    }
    let mut y = 0.0;
    while y <= context.height() {
        context.move_to(0.0, y);
        context.line_to(context.width(), y);
        y += 24.0;
    }
    context.stroke();
    context.restore();
}
