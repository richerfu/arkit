use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::str;
use std::sync::OnceLock;

use arkit::ohos_arkui_binding::api::attribute_option::DrawableDescriptor;
use arkit::ohos_arkui_binding::arkui_input_binding::ArkUIErrorCode;
use arkit::ohos_arkui_binding::common::error::ArkUIError;
use arkit::ohos_arkui_binding::image_native_binding::types::ImageSize;
use arkit::ohos_arkui_binding::image_native_binding::{DecodingOptions, ImageSource, PixelMap};
use arkit::Element;
use ohos_display_binding::default_display_virtual_pixel_ratio;

use crate::embed::{embedded_icon, normalize_icon_name};

pub const DEFAULT_ICON_SIZE: f32 = 24.0;
pub const DEFAULT_STROKE_WIDTH: f32 = 2.0;
pub const DEFAULT_ICON_COLOR: u32 = 0xFF171717;

static DISPLAY_SCALE: OnceLock<f32> = OnceLock::new();

#[derive(Debug)]
pub enum IconError {
    MissingIcon(String),
    InvalidSvg(String),
    InvalidUtf8(str::Utf8Error),
    ImageDecode(String),
    ArkUI(String),
}

impl Display for IconError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingIcon(name) => write!(f, "lucide icon not found: {name}"),
            Self::InvalidSvg(name) => write!(f, "invalid lucide svg payload: {name}"),
            Self::InvalidUtf8(error) => write!(f, "invalid utf8 svg payload: {error}"),
            Self::ImageDecode(error) => write!(f, "{error}"),
            Self::ArkUI(error) => write!(f, "{error}"),
        }
    }
}

impl StdError for IconError {}

impl From<str::Utf8Error> for IconError {
    fn from(value: str::Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

#[derive(Debug, Clone)]
pub struct IconElement {
    spec: IconSpec,
}

#[derive(Debug, Clone)]
pub(crate) struct IconSpec {
    pub(crate) name: String,
    pub(crate) size: f32,
    pub(crate) color: u32,
    pub(crate) stroke_width: f32,
}

pub fn icon(name: impl Into<String>) -> IconElement {
    IconElement {
        spec: IconSpec {
            name: normalize_icon_name(&name.into()),
            size: DEFAULT_ICON_SIZE,
            color: DEFAULT_ICON_COLOR,
            stroke_width: DEFAULT_STROKE_WIDTH,
        },
    }
}

pub fn try_icon(name: impl Into<String>) -> Result<Element, IconError> {
    icon(name).try_render()
}

impl IconElement {
    pub fn name(&self) -> &str {
        &self.spec.name
    }

    pub fn size(mut self, value: f32) -> Self {
        self.spec.size = value.max(1.0);
        self
    }

    pub fn color(mut self, value: u32) -> Self {
        self.spec.color = value;
        self
    }

    pub fn stroke_width(mut self, value: f32) -> Self {
        self.spec.stroke_width = value.max(0.1);
        self
    }

    pub fn try_render(self) -> Result<Element, IconError> {
        build_icon_element(&self.spec, false)
    }

    pub fn render(self) -> Element {
        build_icon_element(&self.spec, true).unwrap_or_else(|_| {
            arkit::row_component()
                .width(self.spec.size)
                .height(self.spec.size)
                .into()
        })
    }
}

fn build_icon_element(spec: &IconSpec, allow_fallback: bool) -> Result<Element, IconError> {
    let svg = match rendered_icon_svg(spec) {
        Ok(payload) => payload,
        Err(_error) if allow_fallback => missing_icon_svg(spec),
        Err(error) => return Err(error),
    };
    let size = spec.size;
    let alt = spec.name.clone();

    Ok(arkit::image_component()
        .native_with_cleanup(move |image| {
            let native = NativeIconImage::decode(svg, size).map_err(icon_to_arkui_error)?;
            image.set_image_src(native.drawable())?;
            Ok(move || drop(native))
        })
        .native(move |image| image.set_image_alt(alt))
        .width(size)
        .height(size)
        .into())
}

fn icon_to_arkui_error(error: IconError) -> ArkUIError {
    ArkUIError::new(ArkUIErrorCode::ParamInvalid, error.to_string())
}

fn rendered_icon_svg(spec: &IconSpec) -> Result<String, IconError> {
    let embedded =
        embedded_icon(&spec.name).ok_or_else(|| IconError::MissingIcon(spec.name.clone()))?;
    let raw_svg = str::from_utf8(embedded.data.as_ref())?;
    let body = extract_svg_body(raw_svg, &spec.name)?;
    Ok(compose_svg(spec, body))
}

fn extract_svg_body<'a>(raw_svg: &'a str, name: &str) -> Result<&'a str, IconError> {
    let (_, content) = raw_svg
        .split_once('>')
        .ok_or_else(|| IconError::InvalidSvg(name.to_string()))?;
    let (body, _) = content
        .rsplit_once("</svg>")
        .ok_or_else(|| IconError::InvalidSvg(name.to_string()))?;
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

fn missing_icon_svg(spec: &IconSpec) -> String {
    compose_svg(
        spec,
        r#"<path d="M4 4l16 16" /><path d="M20 4 4 20" /><rect x="3" y="3" width="18" height="18" rx="2" />"#,
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

struct NativeIconImage {
    pixel_map: PixelMap,
    drawable: Option<DrawableDescriptor>,
}

impl NativeIconImage {
    fn decode(svg: String, size: f32) -> Result<Self, IconError> {
        let mut svg_bytes = svg.into_bytes();
        let source = ImageSource::create_from_data(svg_bytes.as_mut_slice())
            .map_err(|error| IconError::ImageDecode(error.to_string()))?;

        let mut options =
            DecodingOptions::new().map_err(|error| IconError::ImageDecode(error.to_string()))?;
        let edge = icon_raster_edge(size);
        options
            .set_desired_size(ImageSize {
                width: edge,
                height: edge,
            })
            .map_err(|error| IconError::ImageDecode(error.to_string()))?;

        let pixel_map = source
            .create_pixelmap(&mut options)
            .map_err(|error| IconError::ImageDecode(error.to_string()))?;
        let drawable = DrawableDescriptor::from_pixel_map(pixel_map.handle())
            .map_err(|error| IconError::ArkUI(error.to_string()))?;

        Ok(Self {
            pixel_map,
            drawable: Some(drawable),
        })
    }

    fn drawable(&self) -> &DrawableDescriptor {
        self.drawable
            .as_ref()
            .expect("native icon drawable should exist while mounted")
    }
}

fn icon_raster_edge(size_vp: f32) -> u32 {
    let scale = *DISPLAY_SCALE.get_or_init(|| {
        let ratio = default_display_virtual_pixel_ratio();
        if ratio.is_finite() && ratio >= 1.0 {
            ratio
        } else {
            1.0
        }
    });

    (size_vp.max(1.0) * scale).ceil().max(1.0) as u32
}

impl Drop for NativeIconImage {
    fn drop(&mut self) {
        if let Some(drawable) = self.drawable.take() {
            drawable.dispose();
        }
        let _ = &self.pixel_map;
    }
}
