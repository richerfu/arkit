//! Reactive barcode encoding hook (encode off the UI thread).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use arkit_arkui::ArkImageSource;
use arkit_prelude::*;

use crate::async_job::{
    self, base64_png_async, encode_artifact_async, png_bytes_async, save_png_async, JobEpoch,
};
use crate::bitmap::BarcodeBitmap;
use crate::error::{BarcodeError, BarcodeResult};
use crate::request::BarcodeOptions;

/// Encoding lifecycle observed by [`use_barcode`].
#[derive(Debug, Clone, PartialEq)]
pub enum BarcodePhase {
    /// Contents were empty / whitespace after trim.
    Empty,
    /// Encode (or re-encode) is running on a background worker.
    Encoding,
    Ready(BarcodeArtifact),
    Error(BarcodeError),
}

/// Memoized encode result ready for display and export.
#[derive(Debug, Clone, PartialEq)]
pub struct BarcodeArtifact {
    pub bitmap: Arc<BarcodeBitmap>,
    pub image: ArkImageSource,
}

/// Handle returned by [`use_barcode`].
///
/// Export helpers are **async** and never run PNG / I/O on the UI thread.
#[derive(Clone, Copy)]
pub struct BarcodeHandle {
    phase: Signal<BarcodePhase>,
}

impl BarcodeHandle {
    pub fn phase(&self) -> BarcodePhase {
        self.phase.cloned()
    }

    pub fn image(&self) -> Option<ArkImageSource> {
        match self.phase.cloned() {
            BarcodePhase::Ready(artifact) => Some(artifact.image),
            _ => None,
        }
    }

    pub fn bitmap(&self) -> Option<Arc<BarcodeBitmap>> {
        match self.phase.cloned() {
            BarcodePhase::Ready(artifact) => Some(artifact.bitmap),
            _ => None,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.phase.cloned(), BarcodePhase::Ready(_))
    }

    pub fn is_encoding(&self) -> bool {
        matches!(self.phase.cloned(), BarcodePhase::Encoding)
    }

    /// PNG encode on a blocking worker; `on_done` runs on the UI executor.
    pub fn png_bytes_async(&self, on_done: impl FnOnce(BarcodeResult<Vec<u8>>) + 'static) {
        export_bitmap(self.bitmap(), on_done, png_bytes_async);
    }

    /// Base64 PNG on a blocking worker; `on_done` runs on the UI executor.
    pub fn base64_png_async(&self, on_done: impl FnOnce(BarcodeResult<String>) + 'static) {
        export_bitmap(self.bitmap(), on_done, base64_png_async);
    }

    /// Write PNG on a blocking worker; `on_done` runs on the UI executor.
    pub fn save_png_async(
        &self,
        path: impl AsRef<Path>,
        on_done: impl FnOnce(BarcodeResult<PathBuf>) + 'static,
    ) {
        let path = async_job::path_buf(path);
        match self.bitmap() {
            Some(bitmap) => {
                async_job::spawn_ui_result(save_png_async(bitmap, path), on_done);
            }
            None => on_done(Err(not_ready())),
        }
    }
}

fn export_bitmap<T, Fut>(
    bitmap: Option<Arc<BarcodeBitmap>>,
    on_done: impl FnOnce(BarcodeResult<T>) + 'static,
    work: impl FnOnce(Arc<BarcodeBitmap>) -> Fut + 'static,
) where
    T: 'static,
    Fut: std::future::Future<Output = BarcodeResult<T>> + 'static,
{
    match bitmap {
        Some(bitmap) => async_job::spawn_ui_result(work(bitmap), on_done),
        None => on_done(Err(not_ready())),
    }
}

fn not_ready() -> BarcodeError {
    BarcodeError::encode_failed("barcode is not ready for export")
}

/// Encode `contents` whenever they or `options` change.
///
/// Encoding runs on Tokio's blocking pool. The UI thread only schedules work
/// and applies the latest result (`BarcodePhase::Encoding` while in flight).
///
/// Accepts [`Signal`] or [`ReadSignal`] for either argument.
pub fn use_barcode(
    contents: impl Into<ReadSignal<String>>,
    options: impl Into<ReadSignal<BarcodeOptions>>,
) -> BarcodeHandle {
    let contents = contents.into();
    let options = options.into();
    let phase = use_signal(|| BarcodePhase::Empty);
    let epoch = use_hook(JobEpoch::default);

    use_effect(move || {
        let text = contents();
        let opts = options();
        schedule_encode(text, opts, phase, epoch.clone());
    });

    BarcodeHandle { phase }
}

/// Shared scheduler for the hook and the declarative component.
pub(crate) fn schedule_encode(
    contents: String,
    options: BarcodeOptions,
    mut phase: Signal<BarcodePhase>,
    epoch: JobEpoch,
) {
    if contents.trim().is_empty() {
        let _ = epoch.next();
        phase.set(BarcodePhase::Empty);
        return;
    }

    let job = epoch.next();
    phase.set(BarcodePhase::Encoding);
    let request = options.to_request(contents);

    arkit_prelude::dioxus_core::spawn(async move {
        let result = encode_artifact_async(request).await;
        if !epoch.is_current(job) {
            return;
        }
        match result {
            Ok((bitmap, image)) => {
                phase.set(BarcodePhase::Ready(BarcodeArtifact { bitmap, image }))
            }
            Err(error) => phase.set(BarcodePhase::Error(error)),
        }
    });
}
