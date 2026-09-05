use std::fmt::{Debug, Formatter};
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::sync::Arc;
use std::time::Duration;

use crate::VideoError;

pub use ohos_avplayer_binding::{
    AvPlayerBuffering as VideoBuffering, AvPlayerTrack as VideoTrack,
    AvPlayerTrackType as VideoTrackType, VideoSize,
};

/// A URL media source with optional HTTP request headers.
///
/// Header values are deliberately omitted from `Debug`. The stable `key`
/// controls prop equality and source replacement; change it when content or
/// authorization behind an otherwise stable URL changes.
#[derive(Clone, PartialEq, Eq)]
pub struct VideoNetworkSource {
    key: Arc<str>,
    url: Arc<str>,
    headers: Arc<[(Arc<str>, Arc<str>)]>,
}

impl VideoNetworkSource {
    pub fn new(url: impl Into<Arc<str>>) -> Self {
        let url = url.into();
        Self {
            key: url.clone(),
            url,
            headers: Arc::from([]),
        }
    }

    pub fn with_key(mut self, key: impl Into<Arc<str>>) -> Self {
        self.key = key.into();
        self
    }

    pub fn with_header(mut self, name: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        let mut headers = self.headers.to_vec();
        headers.push((name.into(), value.into()));
        self.headers = headers.into();
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn headers(&self) -> &[(Arc<str>, Arc<str>)] {
        &self.headers
    }
}

impl Debug for VideoNetworkSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VideoNetworkSource")
            .field("key", &self.key)
            .field("url", &self.url)
            .field("header_count", &self.headers.len())
            .finish()
    }
}

/// A retained file-descriptor media source.
#[derive(Clone)]
pub struct VideoFileSource {
    key: Arc<str>,
    descriptor: Arc<OwnedFd>,
    offset: u64,
    size: u64,
}

impl VideoFileSource {
    pub fn new(key: impl Into<Arc<str>>, descriptor: OwnedFd, offset: u64, size: u64) -> Self {
        Self::shared(key, Arc::new(descriptor), offset, size)
    }

    pub fn shared(
        key: impl Into<Arc<str>>,
        descriptor: Arc<OwnedFd>,
        offset: u64,
        size: u64,
    ) -> Self {
        Self {
            key: key.into(),
            descriptor,
            offset,
            size,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn raw_fd(&self) -> RawFd {
        self.descriptor.as_raw_fd()
    }
}

impl Debug for VideoFileSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VideoFileSource")
            .field("key", &self.key)
            .field("offset", &self.offset)
            .field("size", &self.size)
            .finish()
    }
}

impl PartialEq for VideoFileSource {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for VideoFileSource {}

/// Network or file-descriptor source accepted by [`crate::VideoPlayer`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum VideoSource {
    Network(VideoNetworkSource),
    File(VideoFileSource),
}

impl VideoSource {
    pub fn url(url: impl Into<Arc<str>>) -> Self {
        Self::Network(VideoNetworkSource::new(url))
    }

    pub fn network(source: VideoNetworkSource) -> Self {
        Self::Network(source)
    }

    pub fn file(source: VideoFileSource) -> Self {
        Self::File(source)
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Network(source) => source.key(),
            Self::File(source) => source.key(),
        }
    }

    pub(crate) fn validate(&self) -> crate::VideoResult<()> {
        match self {
            Self::Network(source) if source.url().trim().is_empty() => Err(
                VideoError::invalid_source("VideoSource::url", "URL must not be empty"),
            ),
            Self::File(source) if source.size() == 0 => Err(VideoError::invalid_source(
                "VideoSource::file",
                "file source size must be greater than zero",
            )),
            _ => Ok(()),
        }
    }
}

impl PartialEq for VideoSource {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for VideoSource {}

/// External WebVTT/SRT subtitle source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoSubtitleSource {
    key: Arc<str>,
    url: Arc<str>,
}

impl VideoSubtitleSource {
    pub fn url(url: impl Into<Arc<str>>) -> Self {
        let url = url.into();
        Self {
            key: url.clone(),
            url,
        }
    }

    pub fn with_key(mut self, key: impl Into<Arc<str>>) -> Self {
        self.key = key.into();
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url_value(&self) -> &str {
        &self.url
    }
}

/// How decoded frames fill the XComponent surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VideoResizeMode {
    #[default]
    Contain,
    Cover,
    Stretch,
    None,
}

impl VideoResizeMode {
    pub(crate) const fn native(self) -> ohos_native_window_binding::NativeWindowScalingMode {
        use ohos_native_window_binding::NativeWindowScalingMode;
        match self {
            Self::Contain => NativeWindowScalingMode::Fit,
            Self::Cover => NativeWindowScalingMode::Crop,
            Self::Stretch => NativeWindowScalingMode::Stretch,
            Self::None => NativeWindowScalingMode::NoScaleCrop,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum VideoStatus {
    #[default]
    Idle,
    WaitingForSurface,
    Loading,
    Ready,
    Playing,
    Paused,
    Buffering,
    Completed,
    Stopped,
    Error(VideoError),
}

impl VideoStatus {
    pub fn is_ready(&self) -> bool {
        matches!(
            self,
            Self::Ready
                | Self::Playing
                | Self::Paused
                | Self::Buffering
                | Self::Completed
                | Self::Stopped
        )
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing | Self::Buffering)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VideoProgress {
    pub position: Duration,
    pub duration: Duration,
    pub buffered: Duration,
}

impl VideoProgress {
    pub fn fraction(self) -> f64 {
        if self.duration.is_zero() {
            0.0
        } else {
            (self.position.as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoMetadata {
    pub duration: Duration,
    pub size: VideoSize,
    pub is_live: bool,
    pub tracks: Vec<VideoTrack>,
    pub available_bitrates: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoSubtitleCue {
    pub text: String,
    pub start: Duration,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoSnapshot {
    pub status: VideoStatus,
    pub progress: VideoProgress,
    pub size: VideoSize,
    pub is_live: bool,
    pub volume: f32,
    pub muted: bool,
    pub playback_rate: f32,
    pub looping: bool,
    pub fullscreen: bool,
    pub tracks: Vec<VideoTrack>,
    pub available_bitrates: Vec<u32>,
}

impl Default for VideoSnapshot {
    fn default() -> Self {
        Self {
            status: VideoStatus::Idle,
            progress: VideoProgress::default(),
            size: VideoSize::default(),
            is_live: false,
            volume: 1.0,
            muted: false,
            playback_rate: 1.0,
            looping: false,
            fullscreen: false,
            tracks: Vec::new(),
            available_bitrates: Vec::new(),
        }
    }
}
