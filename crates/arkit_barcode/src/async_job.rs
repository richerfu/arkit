//! Off-main-thread encode / export helpers.
//!
//! Heavy work (rxing encode, PNG compress, filesystem write) runs on Tokio's
//! blocking pool. Callers schedule from the UI thread and apply results on the
//! Dioxus UI executor after `await`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use arkit_arkui::ArkImageSource;
use arkit_prelude::dioxus_core;

use crate::bitmap::BarcodeBitmap;
use crate::encode::encode_barcode;
use crate::error::{BarcodeError, BarcodeResult};
use crate::request::BarcodeRequest;

/// Monotonic job id used to drop stale encode results.
#[derive(Clone, Default)]
pub(crate) struct JobEpoch {
    inner: Arc<AtomicU64>,
}

impl JobEpoch {
    pub fn next(&self) -> u64 {
        self.inner.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
    }

    pub fn current(&self) -> u64 {
        self.inner.load(Ordering::SeqCst)
    }

    pub fn is_current(&self, job: u64) -> bool {
        self.current() == job
    }
}

/// Encode + build display source on a blocking worker.
pub(crate) async fn encode_artifact_async(
    handle: tokio::runtime::Handle,
    request: BarcodeRequest,
) -> BarcodeResult<(Arc<BarcodeBitmap>, ArkImageSource)> {
    run_blocking(handle, move || {
        let bitmap = encode_barcode(&request)?;
        let image = bitmap.to_ark_image_source();
        Ok((Arc::new(bitmap), image))
    })
    .await
}

pub(crate) async fn png_bytes_async(
    handle: tokio::runtime::Handle,
    bitmap: Arc<BarcodeBitmap>,
) -> BarcodeResult<Vec<u8>> {
    run_blocking(handle, move || bitmap.to_png_bytes()).await
}

pub(crate) async fn base64_png_async(
    handle: tokio::runtime::Handle,
    bitmap: Arc<BarcodeBitmap>,
) -> BarcodeResult<String> {
    run_blocking(handle, move || bitmap.to_base64_png()).await
}

pub(crate) async fn save_png_async(
    handle: tokio::runtime::Handle,
    bitmap: Arc<BarcodeBitmap>,
    path: PathBuf,
) -> BarcodeResult<PathBuf> {
    run_blocking(handle, move || bitmap.write_png(path)).await
}

/// Run `work` on Tokio's blocking pool; await the join from the UI async task.
async fn run_blocking<T, F>(handle: tokio::runtime::Handle, work: F) -> BarcodeResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> BarcodeResult<T> + Send + 'static,
{
    handle
        .spawn(async move {
            match tokio::task::spawn_blocking(work).await {
                Ok(result) => result,
                Err(error) if error.is_cancelled() => {
                    Err(BarcodeError::encode_failed("barcode worker cancelled"))
                }
                Err(_) => Err(BarcodeError::encode_failed(
                    "barcode worker stopped unexpectedly",
                )),
            }
        })
        .await
        .unwrap_or_else(|error| {
            if error.is_cancelled() {
                Err(BarcodeError::encode_failed("barcode task cancelled"))
            } else {
                Err(BarcodeError::encode_failed(
                    "barcode task stopped unexpectedly",
                ))
            }
        })
}

/// Spawn a UI-thread async task that runs `future` then invokes `on_done` with
/// the result (still on the UI executor).
pub(crate) fn spawn_ui_result<T, F, Fut>(future: Fut, on_done: F)
where
    T: 'static,
    F: FnOnce(T) + 'static,
    Fut: std::future::Future<Output = T> + 'static,
{
    dioxus_core::spawn(async move {
        let value = future.await;
        on_done(value);
    });
}

/// Convenience for path-typed save jobs.
pub(crate) fn path_buf(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().to_path_buf()
}
