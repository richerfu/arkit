use std::fmt;

pub type CanvasResult<T> = Result<T, CanvasError>;

/// Typed failures for Canvas operations that throw on the web platform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasError {
    /// Equivalent to the web platform's `IndexSizeError` for negative radii.
    NegativeRadius,
    /// Equivalent to `RangeError` for a round-rect radii sequence outside 1..=4.
    InvalidRadiiCount,
    /// The supplied SVG path data could not be parsed.
    InvalidSvgPath,
    /// Pixel data length does not match width × height × 4.
    InvalidImageData,
    /// The requested `ImageData` storage could not be allocated.
    ImageDataAllocation,
    /// A gradient color stop offset was outside the inclusive 0..=1 range.
    InvalidColorStop,
    /// Equivalent to `NotSupportedError` for a non-finite numeric argument.
    NonFinite,
    /// The requested canvas dimensions cannot be represented safely.
    InvalidDimensions,
    /// The platform image decoder rejected the source data.
    ImageDecode(u32),
    /// The platform image encoder rejected the requested output.
    ImageEncode(u32),
    /// The requested image output format is not supported by the platform.
    UnsupportedImageFormat,
    /// Font bytes or a font file could not be loaded as a native typeface.
    InvalidFont,
    /// A font file could not be read.
    FontIo,
    /// `Path2D::round()` only accepts paths made from straight polygon edges.
    UnsupportedRoundedPath,
}

impl fmt::Display for CanvasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeRadius => formatter.write_str("canvas radius must not be negative"),
            Self::InvalidRadiiCount => {
                formatter.write_str("canvas round-rect requires one to four radii")
            }
            Self::InvalidSvgPath => formatter.write_str("invalid SVG path data"),
            Self::InvalidImageData => formatter.write_str("invalid canvas image data dimensions"),
            Self::ImageDataAllocation => formatter.write_str("canvas image data allocation failed"),
            Self::InvalidColorStop => {
                formatter.write_str("canvas gradient stop must be between zero and one")
            }
            Self::NonFinite => formatter.write_str("canvas argument must be finite"),
            Self::InvalidDimensions => formatter.write_str("invalid canvas dimensions"),
            Self::ImageDecode(code) => {
                write!(
                    formatter,
                    "canvas image decode failed with native code {code}"
                )
            }
            Self::ImageEncode(code) => {
                write!(
                    formatter,
                    "canvas image encode failed with native code {code}"
                )
            }
            Self::UnsupportedImageFormat => {
                formatter.write_str("canvas image format is not supported")
            }
            Self::InvalidFont => formatter.write_str("invalid canvas font data"),
            Self::FontIo => formatter.write_str("failed to read canvas font file"),
            Self::UnsupportedRoundedPath => {
                formatter.write_str("rounded Path2D requires a straight-edged polygon")
            }
        }
    }
}

impl std::error::Error for CanvasError {}
