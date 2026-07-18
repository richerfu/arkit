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
        }
    }
}

impl std::error::Error for CanvasError {}
