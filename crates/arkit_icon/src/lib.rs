//! Lucide icon embedding for arkit — declarative dioxus implementation.
//!
//! [`icon`] renders an ArkUI `Image` element whose `src` attribute carries an
//! [`ArkImageSource`] (SVG → PixelMap → DrawableDescriptor). The renderer
//! resolves and holds the native resource for the node's lifetime — no
//! direct native-node escape hatch.

mod embed;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::str;
use std::sync::Arc;

use arkit_arkui::ArkImageSource;
use arkit_prelude::*;
use rustc_hash::FxHashMap;

use crate::embed::embedded_icon;
pub use embed::{has_icon, icon_names};

pub const DEFAULT_ICON_SIZE: f32 = 24.0;
pub const DEFAULT_ICON_COLOR: u32 = 0xFF171717;
pub const DEFAULT_STROKE_WIDTH: f32 = 2.0;

fn pixel_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

#[derive(Debug, Clone)]
struct IconSpec {
    name: String,
    size: f32,
    color: u32,
    stroke_width: f32,
    pixel_ratio: f32,
}

impl IconSpec {
    fn render_key(&self) -> String {
        format!(
            "arkit-icon:{}:{:08x}:{:08x}:{:08x}:{:08x}",
            self.name,
            self.size.to_bits(),
            self.color,
            self.stroke_width.to_bits(),
            self.pixel_ratio.to_bits(),
        )
    }

    fn raster_edge(&self) -> u32 {
        ((self.size * self.pixel_ratio).max(1.0).round()) as u32
    }
}

const SVG_CACHE_CAPACITY: usize = 128;

#[derive(Default)]
struct SvgCache {
    entries: FxHashMap<String, Arc<str>>,
    order: VecDeque<String>,
}

impl SvgCache {
    fn get(&self, key: &str) -> Option<Arc<str>> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: String, value: Arc<str>) {
        if let Some(entry) = self.entries.get_mut(&key) {
            *entry = value;
            return;
        }
        self.entries.insert(key.clone(), value);
        self.order.push_back(key);
        while self.entries.len() > SVG_CACHE_CAPACITY {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
            }
        }
    }
}

thread_local! {
    // Arkit's Dioxus runtime is UI-thread confined. A thread-local cache keeps
    // icon rendering off a process-wide mutex while retaining a hard bound per
    // runtime thread.
    static SVG_CACHE: RefCell<SvgCache> = RefCell::new(SvgCache::default());
}

/// Render an icon by name as a dioxus `Element`.
///
/// The SVG is rasterized to a `DrawableDescriptor` by the renderer when it
/// commits the `src` attribute. `size`/`color` control the rasterized
/// dimensions and stroke color.
pub fn icon(name: impl Into<String>, size: f32, color: u32) -> Element {
    icon_with_stroke(name, size, color, DEFAULT_STROKE_WIDTH)
}

pub fn icon_with_stroke(
    name: impl Into<String>,
    size: f32,
    color: u32,
    stroke_width: f32,
) -> Element {
    let spec = IconSpec {
        name: embed::normalize_icon_name(&name.into()),
        size: size.max(1.0),
        color,
        stroke_width: stroke_width.max(0.1),
        pixel_ratio: pixel_ratio(),
    };
    let edge = spec.size;
    let render_key = spec.render_key();
    let element_key = render_key.clone();

    // Render the SVG (from embedded asset or fallback), then wrap as an
    // ArkImageSource that the renderer resolves to a native DrawableDescriptor.
    let svg = rendered_icon_svg(&spec, &render_key)
        .unwrap_or_else(|_| Arc::<str>::from(missing_icon_svg(&spec)));
    let px_edge = spec.raster_edge();
    let source = ArkImageSource::svg_shared(render_key, svg, px_edge, px_edge);

    rsx! {
        image {
            key: "{element_key}",
            src: dioxus_core::AttributeValue::any_value(source),
            object_fit: "cover",
            width: edge,
            height: edge,
        }
    }
}

fn rendered_icon_svg(spec: &IconSpec, cache_key: &str) -> Result<Arc<str>, String> {
    if let Some(svg) = SVG_CACHE.with_borrow(|cache| cache.get(cache_key)) {
        return Ok(svg);
    }

    let embedded =
        embedded_icon(&spec.name).ok_or_else(|| format!("icon not found: {}", spec.name))?;
    let raw_svg = str::from_utf8(embedded.data.as_ref()).map_err(|e| e.to_string())?;
    let body = extract_svg_body(raw_svg, &spec.name)?;
    let svg = Arc::<str>::from(compose_svg(spec, body));

    SVG_CACHE.with_borrow_mut(|cache| {
        cache.insert(cache_key.to_owned(), svg.clone());
    });

    Ok(svg)
}

fn missing_icon_svg(spec: &IconSpec) -> String {
    compose_svg(
        spec,
        r#"<path d="M4 4l16 16" /><path d="M20 4 4 20" /><rect x="3" y="3" width="18" height="18" rx="2" />"#,
    )
}

fn extract_svg_body<'a>(raw_svg: &'a str, name: &str) -> Result<&'a str, String> {
    let (_, content) = raw_svg
        .split_once('>')
        .ok_or_else(|| format!("invalid svg: {name}"))?;
    let (body, _) = content
        .rsplit_once("</svg>")
        .ok_or_else(|| format!("invalid svg: {name}"))?;
    Ok(body.trim())
}

fn compose_svg(spec: &IconSpec, body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="{color}" stroke-width="{stroke_width}" stroke-linecap="round" stroke-linejoin="round">{body}</svg>"#,
        size = format_dimension(spec.size),
        color = svg_color(spec.color),
        stroke_width = format_dimension(spec.stroke_width),
    )
}

fn svg_color(value: u32) -> String {
    let [alpha, red, green, blue] = value.to_be_bytes();
    if alpha == 0xFF {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("rgba({red}, {green}, {blue}, {:.3})", alpha as f32 / 255.0)
    }
}

fn format_dimension(value: f32) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < f32::EPSILON {
        format!("{rounded:.0}")
    } else {
        format!("{value:.3}")
    }
}
