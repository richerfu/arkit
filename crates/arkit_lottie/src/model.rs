use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

use crate::LottieError;

const DEFAULT_NETWORK_TIMEOUT: Duration = Duration::from_secs(20);
const DEFAULT_MAX_DOWNLOAD_BYTES: usize = 16 * 1024 * 1024;

/// A remote Lottie JSON request.
///
/// Loading requires the crate's `network` feature (the arkit facade exposes it
/// as `lottie-network`). Without that feature the player reports a typed
/// network error without starting a request.
///
/// The URL is also the default composition identity. Use [`Self::with_key`]
/// when the content at one URL is versioned out-of-band or when request
/// headers change. Header values are intentionally omitted from `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub struct LottieNetworkSource {
    key: Arc<str>,
    url: Arc<str>,
    headers: Arc<[(Arc<str>, Arc<str>)]>,
    timeout: Duration,
    max_download_bytes: usize,
}

impl LottieNetworkSource {
    pub fn new(url: impl Into<Arc<str>>) -> Self {
        let url = url.into();
        Self {
            key: url.clone(),
            url,
            headers: Arc::from([]),
            timeout: DEFAULT_NETWORK_TIMEOUT,
            max_download_bytes: DEFAULT_MAX_DOWNLOAD_BYTES,
        }
    }

    /// Override the stable composition identity used for props comparison and
    /// stale-response rejection.
    pub fn with_key(mut self, key: impl Into<Arc<str>>) -> Self {
        self.key = key.into();
        self
    }

    /// Add an HTTP request header. Names and values are validated before the
    /// request starts; values are never included in diagnostics.
    pub fn with_header(mut self, name: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        let mut headers = self.headers.to_vec();
        headers.push((name.into(), value.into()));
        self.headers = headers.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Limit the decompressed response body retained in memory. The default is
    /// 16 MiB.
    pub fn with_max_download_bytes(mut self, max_download_bytes: usize) -> Self {
        self.max_download_bytes = max_download_bytes;
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn max_download_bytes(&self) -> usize {
        self.max_download_bytes
    }

    #[cfg(feature = "network")]
    pub(crate) fn headers(&self) -> &[(Arc<str>, Arc<str>)] {
        &self.headers
    }
}

impl Debug for LottieNetworkSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LottieNetworkSource")
            .field("key", &self.key)
            .field("url", &self.url)
            .field("header_count", &self.headers.len())
            .field("timeout", &self.timeout)
            .field("max_download_bytes", &self.max_download_bytes)
            .finish()
    }
}

/// In-memory Lottie JSON plus a stable cache identity.
///
/// Equality intentionally compares only `key`: keep a key stable to reuse a
/// composition, and change it whenever the bytes change. This avoids scanning
/// a potentially large JSON document during every Dioxus props comparison.
#[derive(Clone)]
pub struct LottieSource {
    key: SourceKey,
    data: SourceData,
}

#[derive(Clone)]
enum SourceKey {
    Static(&'static str),
    Shared(Arc<str>),
}

impl SourceKey {
    fn as_str(&self) -> &str {
        match self {
            Self::Static(value) => value,
            Self::Shared(value) => value,
        }
    }
}

#[derive(Clone)]
enum SourceData {
    Static(&'static [u8]),
    Shared(Arc<[u8]>),
    Network(LottieNetworkSource),
}

impl SourceData {
    fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Static(value) => Some(value),
            Self::Shared(value) => Some(value),
            Self::Network(_) => None,
        }
    }

    fn as_network(&self) -> Option<&LottieNetworkSource> {
        match self {
            Self::Network(source) => Some(source),
            Self::Static(_) | Self::Shared(_) => None,
        }
    }
}

impl LottieSource {
    pub fn new(key: impl Into<Arc<str>>, data: impl Into<Arc<[u8]>>) -> Self {
        Self {
            key: SourceKey::Shared(key.into()),
            data: SourceData::Shared(data.into()),
        }
    }

    pub fn json(key: impl Into<Arc<str>>, json: impl Into<Arc<str>>) -> Self {
        let json = json.into();
        Self {
            key: SourceKey::Shared(key.into()),
            data: SourceData::Shared(Arc::from(json.as_bytes())),
        }
    }

    /// Construct a network-backed source using the URL as its stable key.
    /// Loading requires the crate's `network` feature.
    pub fn url(url: impl Into<Arc<str>>) -> Self {
        Self::network(LottieNetworkSource::new(url))
    }

    pub fn network(source: LottieNetworkSource) -> Self {
        Self {
            key: SourceKey::Shared(source.key.clone()),
            data: SourceData::Network(source),
        }
    }

    /// Construct a fully static source without allocating or copying while
    /// Dioxus builds props.
    pub const fn embedded(key: &'static str, data: &'static [u8]) -> Self {
        Self {
            key: SourceKey::Static(key),
            data: SourceData::Static(data),
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Return the in-memory JSON bytes. Network sources return an empty slice
    /// until they are resolved internally by [`crate::LottiePlayer`]. Use
    /// [`Self::inline_bytes`] when the distinction matters.
    pub fn bytes(&self) -> &[u8] {
        self.inline_bytes().unwrap_or_default()
    }

    pub fn inline_bytes(&self) -> Option<&[u8]> {
        self.data.as_bytes()
    }

    pub fn network_source(&self) -> Option<&LottieNetworkSource> {
        self.data.as_network()
    }

    pub fn is_network(&self) -> bool {
        self.network_source().is_some()
    }
}

impl Debug for LottieSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LottieSource")
            .field("key", &self.key())
            .field("byte_len", &self.inline_bytes().map(<[u8]>::len))
            .field("network", &self.network_source())
            .finish()
    }
}

impl PartialEq for LottieSource {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for LottieSource {}

/// How the composition is scaled into the native surface.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LottieFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
}

/// Placement used when the scaled composition does not exactly fill a surface.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LottieAlignment {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl LottieAlignment {
    #[cfg_attr(not(target_env = "ohos"), allow(dead_code))]
    pub(crate) const fn factors(self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::Top => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::Left => (0.0, 0.5),
            Self::Center => (0.5, 0.5),
            Self::Right => (1.0, 0.5),
            Self::BottomLeft => (0.0, 1.0),
            Self::Bottom => (0.5, 1.0),
            Self::BottomRight => (1.0, 1.0),
        }
    }
}

/// Behavior at the end of the composition timeline.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LottieRepeatMode {
    None,
    #[default]
    Loop,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LottieComposition {
    pub width: f32,
    pub height: f32,
    pub frames: f32,
    pub duration_seconds: f32,
    pub frames_per_second: f32,
}

impl LottieComposition {
    pub fn is_valid(self) -> bool {
        self.width.is_finite()
            && self.width > 0.0
            && self.height.is_finite()
            && self.height > 0.0
            && self.frames.is_finite()
            && self.frames > 0.0
            && self.duration_seconds.is_finite()
            && self.duration_seconds > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LottieFrame {
    pub frame: f32,
    pub progress: f32,
    pub elapsed_seconds: f32,
}

impl Default for LottieFrame {
    fn default() -> Self {
        Self {
            frame: 0.0,
            progress: 0.0,
            elapsed_seconds: 0.0,
        }
    }
}

/// Coarse playback state. Per-frame values are reported separately.
#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
pub enum LottieStatus {
    #[default]
    Idle,
    WaitingForSurface,
    Loading,
    Ready,
    Playing,
    Paused,
    Completed,
    Error(LottieError),
}

impl LottieStatus {
    pub fn is_ready(&self) -> bool {
        matches!(
            self,
            Self::Ready | Self::Playing | Self::Paused | Self::Completed
        )
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing)
    }
}
