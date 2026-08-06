//! W3C-style Canvas 2D API showcase.

mod tiger;

use std::f32::consts::{PI, TAU};
use std::rc::Rc;

use arkit::prelude::*;

use tiger::TigerScene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demo {
    Tiger,
    Api,
    Pipeline,
}

#[component]
pub fn CanvasPage() -> Element {
    let mut active = use_signal(|| Demo::Tiger);
    let mut rotation = use_signal(|| 0.0_f32);
    let mut dashed = use_signal(|| true);
    let mut tiger_zoom = use_signal(|| 1.0_f32);
    let tiger = use_hook(|| Rc::new(TigerScene::load()));
    let mut pipeline = use_signal(|| None::<Rc<PipelineResult>>);
    let demo = active();
    let angle = rotation();
    let show_dash = dashed();
    let zoom = tiger_zoom();
    let pipeline_scene = pipeline();
    let renderer = match demo {
        Demo::Tiger => {
            let tiger = tiger.clone();
            CanvasRenderer::new(move |context| tiger.draw(context, zoom))
        }
        Demo::Api => CanvasRenderer::new(move |context| draw_showcase(context, angle, show_dash)),
        Demo::Pipeline => CanvasRenderer::new(move |context| match &pipeline_scene {
            Some(pipeline) => draw_pipeline_scene(context, pipeline),
            None => draw_pipeline_loading(context),
        }),
    };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: 0xFFF1F5F9u32,
            column {
                padding: 16.0,
                background_color: 0xFFFFFFFFu32,
                text {
                    font_size: 24.0,
                    line_height: 30.0,
                    font_weight: 700,
                    match demo {
                        Demo::Tiger => "Canvas Tiger",
                        Demo::Api => "Canvas 2D",
                        Demo::Pipeline => "Canvas pipeline",
                    }
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF475569u32,
                    match demo {
                        Demo::Tiger => "240 vector Path2D layers · native fill/stroke · scale-to-fit",
                        Demo::Api => "W3C-style context · ArkUI native custom draw · high-DPI logical pixels",
                        Demo::Pipeline => "SVG decode · Lottie frame · OffscreenCanvas · PNG round-trip",
                    }
                }
                row {
                    margin_top: 12.0,
                    button {
                        width: 104.0,
                        onclick: move |_| active.set(Demo::Tiger),
                        "Tiger"
                    }
                    button {
                        width: 104.0,
                        margin_left: 8.0,
                        onclick: move |_| active.set(Demo::Api),
                        "Canvas 2D"
                    }
                    button {
                        width: 104.0,
                        margin_left: 8.0,
                        onclick: move |_| {
                            if pipeline.peek().is_none() {
                                pipeline.set(Some(Rc::new(build_pipeline_scene())));
                            }
                            active.set(Demo::Pipeline);
                        },
                        "Pipeline"
                    }
                }
                if demo == Demo::Tiger {
                    row {
                        margin_top: 8.0,
                        button {
                            width: 96.0,
                            onclick: move |_| tiger_zoom.set((tiger_zoom() - 0.1).max(0.6)),
                            "Zoom -"
                        }
                        button {
                            width: 96.0,
                            margin_left: 8.0,
                            onclick: move |_| tiger_zoom.set(1.0),
                            "Reset"
                        }
                        button {
                            width: 96.0,
                            margin_left: 8.0,
                            onclick: move |_| tiger_zoom.set((tiger_zoom() + 0.1).min(1.5)),
                            "Zoom +"
                        }
                    }
                } else if demo == Demo::Api {
                    row {
                        margin_top: 8.0,
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
            }
            column {
                width: "100%",
                layout_weight: 1.0,
                padding: 16.0,
                Canvas {
                    draw: renderer,
                    width: "100%",
                    height: "100%",
                }
            }
        }
    }
}

type PipelineResult = Result<CanvasImage, Box<dyn std::error::Error>>;

fn build_pipeline_scene() -> PipelineResult {
    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="220" height="160" viewBox="0 0 220 160">
      <defs><linearGradient id="g"><stop stop-color="#22d3ee"/><stop offset="1" stop-color="#6366f1"/></linearGradient></defs>
      <rect width="220" height="160" rx="28" fill="url(#g)"/>
      <circle cx="70" cy="80" r="38" fill="#fef08a"/>
      <path d="M125 45h58v18h-58zm0 32h42v14h-42zm0 27h52v14h-52z" fill="white"/>
    </svg>"##;

    let svg = CanvasImage::decode(
        SVG,
        CanvasImageDecodeOptions {
            frame_index: 0,
            desired_size: Some([330, 240]),
        },
    )?;
    CanvasFontRegistry::register(CanvasFontFace::from_file(
        "Pipeline Sans",
        "/system/fonts/HarmonyOS_Sans.ttf",
    )?);

    let mut lottie_frame = None;
    LottieFrameRenderer::new(LottieSource::embedded(
        "canvas-pipeline-orbit",
        include_bytes!("../../lottie/assets/orbit.json"),
    ))
    .with_options(LottieFrameRenderOptions {
        size: Some([220, 220]),
        ..LottieFrameRenderOptions::default()
    })
    .render_range(18..19, |frame| {
        lottie_frame = CanvasImage::from_rgba(frame.rgba.to_vec(), frame.width, frame.height).ok();
    })?;

    let mut offscreen = OffscreenCanvas::new(720, 480)?;
    {
        let mut context = offscreen.get_context_2d();
        let background = context.create_linear_gradient(0.0, 0.0, 720.0, 480.0)?;
        background.add_color_stop(0.0, "#081225")?;
        background.add_color_stop(1.0, "#172554")?;
        context.set_fill_style(background);
        context.fill_rect(0.0, 0.0, 720.0, 480.0);
        context.draw_image(&svg, 36.0, 42.0);
        if let Some(frame) = &lottie_frame {
            context.draw_image_scaled(frame, 454.0, 30.0, 230.0, 230.0);
        }

        let points = [
            (48.0, 340.0),
            (146.0, 286.0),
            (264.0, 342.0),
            (372.0, 292.0),
            (452.0, 392.0),
            (286.0, 438.0),
            (128.0, 418.0),
        ];
        let mut polygon = Path2D::new();
        polygon.move_to(points[0].0, points[0].1);
        for point in &points[1..] {
            polygon.line_to(point.0, point.1);
        }
        polygon.close_path();
        let rounded = polygon.round(20.0)?;
        context.set_fill_style("rgba(34, 211, 238, 0.22)");
        context.fill_path(&rounded);
        context.set_stroke_style("#67e8f9");
        context.set_line_width(5.0);
        context.stroke_path(&rounded);
        context.set_fill_style("white");
        context.set_font("700 24px sans-serif");
        context.set_text_align(CanvasTextAlign::Center);
        context.fill_text("Path2D.round()", 250.0, 375.0);
        context.set_font("600 16px 'Pipeline Sans'");
        context.fill_text("decoded SVG", 202.0, 272.0);
        context.fill_text("Lottie → RGBA", 568.0, 278.0);
    }

    let png =
        offscreen.convert_to_blob(CanvasImageFormat::Png, CanvasImageEncodeOptions::default())?;
    Ok(CanvasImage::decode(
        &png,
        CanvasImageDecodeOptions::default(),
    )?)
}

fn draw_pipeline_scene(context: &mut CanvasRenderingContext2D<'_>, pipeline: &PipelineResult) {
    context.set_fill_style("#020617");
    context.fill_rect(0.0, 0.0, context.width(), context.height());
    match pipeline {
        Ok(image) => {
            let scale = (context.width() / image.width() as f32)
                .min(context.height() / image.height() as f32)
                .min(1.0);
            let width = image.width() as f32 * scale;
            let height = image.height() as f32 * scale;
            context.draw_image_scaled(
                image,
                (context.width() - width) * 0.5,
                (context.height() - height) * 0.5,
                width,
                height,
            );
        }
        Err(error) => {
            context.set_fill_style("#fecaca");
            context.set_font("600 16px sans-serif");
            context.set_text_align(CanvasTextAlign::Center);
            context.fill_text_max_width(
                &format!("Pipeline failed: {error}"),
                context.width() * 0.5,
                context.height() * 0.5,
                context.width() - 32.0,
            );
        }
    }
}

fn draw_pipeline_loading(context: &mut CanvasRenderingContext2D<'_>) {
    context.set_fill_style("#020617");
    context.fill_rect(0.0, 0.0, context.width(), context.height());
    context.set_fill_style("#cbd5e1");
    context.set_font("600 16px sans-serif");
    context.set_text_align(CanvasTextAlign::Center);
    context.fill_text(
        "Preparing off-screen pipeline…",
        context.width() * 0.5,
        context.height() * 0.5,
    );
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
