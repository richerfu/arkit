//! High-performance native Lottie rendering for arkit.
//!
//! Rendering is isolated behind the facade's `lottie` feature. A dedicated
//! worker drives ThorVG directly into the XComponent native window buffer, so
//! animation frames do not allocate an intermediate bitmap or rerender the
//! Dioxus tree. The optional `network` feature resolves HTTP/HTTPS JSON on the
//! framework's Tokio runtime before handing bytes to the render worker.

mod component;
mod controller;
mod error;
mod model;
#[cfg(feature = "network")]
mod network;
#[cfg(not(feature = "network"))]
#[path = "network_stub.rs"]
mod network;
mod renderer;
mod surface;
mod worker;

pub use component::{LottiePlayer, LottiePlayerProps};
pub use controller::LottieController;
pub use error::{LottieError, LottieErrorKind, LottieResult};
pub use model::{
    LottieAlignment, LottieComposition, LottieFit, LottieFrame, LottieNetworkSource,
    LottieRepeatMode, LottieSource, LottieStatus,
};
