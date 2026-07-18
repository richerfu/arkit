//! A native Canvas 2D component shaped after the web platform API.
//!
//! [`CanvasRenderingContext2D`] follows the naming, defaults, coordinate
//! system, drawing-state stack, and path model of the W3C/WHATWG Canvas 2D
//! context. The implementation paints directly into ArkUI's custom-draw
//! canvas, so a separate XComponent surface is not required.
//!
//! It includes the Canvas 2D state, path, paint, image, text, and pixel APIs
//! over a persistent native backing store.

mod color;
mod color_space;
mod component;
mod context;
mod error;
mod filter;
mod image;
mod native;
mod path;
mod state;
mod text;

pub use color::{
    CanvasColor, CanvasGradient, CanvasPattern, CanvasPatternRepetition, CanvasStyle,
    IntoCanvasStyle,
};
pub use component::{Canvas, CanvasController, CanvasProps, CanvasRenderer};
pub use context::CanvasRenderingContext2D;
pub use error::{CanvasError, CanvasResult};
pub use image::{
    CanvasImage, Float16, ImageData, ImageDataArray, ImageDataPixelFormat, ImageDataSettings,
};
pub use path::{CanvasRadius, IntoCanvasRadii, Path2D};
pub use state::{
    CanvasColorSpace, CanvasColorType, CanvasFont, CanvasFontKerning, CanvasFontStretch,
    CanvasFontStyle, CanvasFontVariantCaps, CanvasImageSmoothingQuality, CanvasLineCap,
    CanvasLineJoin, CanvasRenderingContext2DSettings, CanvasTextAlign, CanvasTextBaseline,
    CanvasTextDirection, CanvasTextMetrics, CanvasTextRendering, CanvasTextSpacing, DomMatrix2D,
    FillRule, GlobalCompositeOperation, IntoCanvasFont, IntoCanvasTextSpacing,
};
