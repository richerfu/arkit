//! Barcode and QR code generation for Arkit.
//!
//! Pure encoding, multi-format export (SVG / PNG / base64 / file), a reactive
//! hook, and a declarative [`Barcode`] component. Independent of the camera
//! scan pipeline (`camera-scan`); enable via the facade `barcode` feature.
//!
//! # Threading
//!
//! [`encode_barcode`] and bitmap export methods are **synchronous** CPU work —
//! call them from a worker if you use them directly. The [`use_barcode`] hook
//! and [`Barcode`] component always schedule encode on Tokio's blocking pool
//! and never run rxing / PNG compression on the UI thread.
//!
//! ```ignore
//! use arkit::barcode::{encode_barcode, Barcode, BarcodeRequest};
//!
//! // Pure / tests / your own worker:
//! let bitmap = encode_barcode(&BarcodeRequest::qr("https://example.com", 256))?;
//!
//! // UI (async internally):
//! rsx! { Barcode { contents: pay_url, format: BarcodeFormat::QrCode, size: 220.0 } }
//! ```

mod async_job;
mod bitmap;
mod component;
mod encode;
mod error;
mod format;
mod hook;
mod render;
mod request;

pub use bitmap::BarcodeBitmap;
pub use component::{Barcode, BarcodeProps};
pub use encode::encode_barcode;
pub use error::{BarcodeError, BarcodeErrorKind, BarcodeResult};
pub use format::{BarcodeFormat, QrEcLevel};
pub use hook::{use_barcode, BarcodeArtifact, BarcodeHandle, BarcodePhase};
pub use request::{BarcodeOptions, BarcodeRequest, DEFAULT_MARGIN, MAX_BARCODE_EDGE};
