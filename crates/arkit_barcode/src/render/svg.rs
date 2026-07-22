//! SVG export for barcode bitmaps.

use crate::bitmap::BarcodeBitmap;

pub(crate) fn render(bitmap: &BarcodeBitmap) -> String {
    let width = bitmap.width();
    let height = bitmap.height();
    let light = css_color(bitmap.light());
    let dark = css_color(bitmap.dark());

    // Estimate capacity: header + background + dark runs.
    let mut out = String::with_capacity((width * height / 2) as usize + 256);
    out.push_str(&format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" "#,
            r#"width="{w}" height="{h}" viewBox="0 0 {w} {h}" "#,
            r#"shape-rendering="crispEdges">"#
        ),
        w = width,
        h = height,
    ));
    out.push_str(&format!(
        r##"<rect width="100%" height="100%" fill="{light}"/>"##
    ));

    // Merge horizontal dark runs to keep SVG smaller.
    for y in 0..height {
        let mut x = 0_u32;
        while x < width {
            if !bitmap.is_dark(x, y) {
                x += 1;
                continue;
            }
            let start = x;
            x += 1;
            while x < width && bitmap.is_dark(x, y) {
                x += 1;
            }
            let run = x - start;
            out.push_str(&format!(
                r##"<rect x="{start}" y="{y}" width="{run}" height="1" fill="{dark}"/>"##
            ));
        }
    }
    out.push_str("</svg>");
    out
}

fn css_color(argb: u32) -> String {
    let a = ((argb >> 24) & 0xFF) as f32 / 255.0;
    let r = (argb >> 16) & 0xFF;
    let g = (argb >> 8) & 0xFF;
    let b = argb & 0xFF;
    if (a - 1.0).abs() < f32::EPSILON {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("rgba({r},{g},{b},{a:.3})")
    }
}
