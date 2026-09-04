//! Native video playback for arkit.
//!
//! [`VideoPlayer`] owns an OpenHarmony AVPlayer on a dedicated worker thread
//! and presents it through an ArkUI XComponent. The public controller/view
//! split follows the mature mobile-player model: source and presentation live
//! on the component, while playback, seeking, tracks, volume, looping, and
//! rate are available imperatively through [`VideoController`].

mod component;
mod controller;
mod controls;
mod error;
mod model;
mod surface;
mod worker;

pub use component::{VideoPlayer, VideoPlayerProps};
pub use controller::VideoController;
pub use controls::{VideoControlIcons, VideoControlLabels, VideoControls, VideoControlsStyle};
pub use error::{VideoError, VideoErrorKind, VideoResult};
pub use model::{
    VideoBuffering, VideoFileSource, VideoMetadata, VideoNetworkSource, VideoProgress,
    VideoResizeMode, VideoSize, VideoSnapshot, VideoSource, VideoStatus, VideoSubtitleCue,
    VideoSubtitleSource, VideoTrack, VideoTrackType,
};
pub use ohos_avplayer_binding::AvPlayerSeekMode as VideoSeekMode;
