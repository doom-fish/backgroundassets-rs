//! Executor-agnostic async API for `backgroundassets`.
//!
//! Enabled with the `async` Cargo feature. These wrappers expose `Future`-based
//! entrypoints for Background Assets discovery and download-management APIs while
//! remaining runtime-neutral.
//!
//! ## Wrapped APIs
//!
//! | Rust wrapper | Underlying API |
//! | --- | --- |
//! | [`AsyncAssetPackManager::all_asset_packs`] | `BAAssetPackManager.allAssetPacks` |
//! | [`AsyncAssetPackManager::asset_pack`] | `BAAssetPackManager.assetPack(withID:)` |
//! | [`AsyncAssetPackManager::status_relative_to`] | `BAAssetPackManager.status(relativeTo:)` |
//! | [`AsyncAssetPackManager::local_status`] | `BAAssetPackManager.localStatus(ofAssetPackWithID:)` |
//! | [`AsyncAssetPackManager::ensure_local_availability`] | `BAAssetPackManager.ensureLocalAvailability(...)` |
//! | [`AsyncAssetPackManager::check_for_updates`] | `BAAssetPackManager.checkForUpdates()` |
//! | [`AsyncAssetPackManager::remove`] | `BAAssetPackManager.remove(assetPackWithID:)` |
//! | [`AsyncDownloadManager::current_downloads`] | `BADownloadManager.fetchCurrentDownloads(...)` |
//! | [`AsyncDownloadManager::with_exclusive_control`] | `BADownloadManager.withExclusiveControl(...)` |
//! | [`install_global_download_manager_delegate`] | `BADownloadManager.delegate` |
//! | [`install_global_managed_asset_pack_download_delegate`] | `BAAssetPackManager.delegate` |
//! | [`install_global_managed_downloader_extension`] | `ManagedDownloaderExtension` |
//!
//! Multi-fire status updates continue to use [`DownloadStatusStream`],
//! [`DownloadManagerEventStream`], and [`ManagedAssetPackDownloadEventStream`].
//!
//! ## Example
//! ```no_run
//! use backgroundassets::async_api::AsyncAssetPackManager;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! pollster::block_on(async {
//!     let Some(manager) = AsyncAssetPackManager::shared() else {
//!         return Ok::<(), backgroundassets::BackgroundAssetsError>(());
//!     };
//!     let packs = manager.all_asset_packs().await?;
//!     println!("found {} packs", packs.len());
//!     Ok(())
//! })?;
//! # Ok(())
//! # }
//! ```

use core::ffi::{c_char, c_void};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use serde::Deserialize;

use crate::asset_pack::{collect_asset_packs, AssetPack, AssetPackStatus};
use crate::download::{collect_downloads, Download, DownloadManager};
use crate::error::BackgroundAssetsError;
use crate::ffi;
use crate::manager::AssetPackManager;
pub use crate::download::{
    install_global_download_manager_delegate, DownloadManagerDelegate, DownloadManagerEvent,
    DownloadManagerEventStream, DownloadWriteProgress,
};
pub use crate::extension::{
    install_global_managed_downloader_extension, ManagedDownloaderExtensionConfiguration,
    ManagedDownloaderExtensionRegistration,
};
pub use crate::manager::{
    install_global_managed_asset_pack_download_delegate, DownloadProgress,
    DownloadStatusStream, DownloadStatusUpdate, ManagedAssetPackDownloadDelegate,
    ManagedAssetPackDownloadEvent, ManagedAssetPackDownloadEventStream, UpdateCheck,
};

struct OpaquePtr(*mut c_void);

// SAFETY: The pointer is an opaque retained Objective-C/Swift object or boxed
// array handle that is never dereferenced directly in Rust.
unsafe impl Send for OpaquePtr {}

#[derive(Debug, Deserialize)]
struct UpdateCheckPayload {
    #[serde(rename = "updatingIDs")]
    updating_ids: Vec<String>,
    #[serde(rename = "removedIDs")]
    removed_ids: Vec<String>,
}

fn parse_status_response(
    value: String,
    context: &'static str,
) -> Result<AssetPackStatus, BackgroundAssetsError> {
    value
        .parse::<u64>()
        .map(AssetPackStatus::new)
        .map_err(|error| {
            BackgroundAssetsError::message(format!("invalid {context} payload: {error}"))
        })
}

fn parse_update_check_response(value: String) -> Result<UpdateCheck, BackgroundAssetsError> {
    let payload = serde_json::from_str::<UpdateCheckPayload>(&value).map_err(|error| {
        BackgroundAssetsError::message(format!("invalid update-check payload: {error}"))
    })?;
    Ok(UpdateCheck {
        updating_ids: payload.updating_ids,
        removed_ids: payload.removed_ids,
    })
}

fn parse_bool_response(value: String) -> Result<bool, BackgroundAssetsError> {
    match value.as_str() {
        "1" | "true" | "TRUE" | "yes" | "ok" => Ok(true),
        "0" | "false" | "FALSE" | "no" => Ok(false),
        other => Err(BackgroundAssetsError::message(format!(
            "invalid exclusive-control payload: {other}"
        ))),
    }
}

fn asset_pack_from_raw(OpaquePtr(ptr): OpaquePtr) -> Result<AssetPack, BackgroundAssetsError> {
    AssetPack::from_raw(ptr)
        .ok_or_else(|| BackgroundAssetsError::message("asset-pack pointer must not be null"))
}

fn poll_object_future<T>(
    inner: &mut AsyncCompletionFuture<OpaquePtr>,
    cx: &mut Context<'_>,
    map_ok: fn(OpaquePtr) -> Result<T, BackgroundAssetsError>,
) -> Poll<Result<T, BackgroundAssetsError>> {
    Pin::new(inner).poll(cx).map(|result| {
        result
            .map_err(BackgroundAssetsError::message)
            .and_then(map_ok)
    })
}

fn poll_string_future<T, F>(
    inner: &mut AsyncCompletionFuture<String>,
    cx: &mut Context<'_>,
    map_ok: F,
) -> Poll<Result<T, BackgroundAssetsError>>
where
    F: FnOnce(String) -> Result<T, BackgroundAssetsError>,
{
    Pin::new(inner).poll(cx).map(|result| {
        result
            .map_err(BackgroundAssetsError::message)
            .and_then(map_ok)
    })
}

unsafe extern "C" fn object_async_cb(result: *mut c_void, error: *const c_char, ctx: *mut c_void) {
    if !error.is_null() {
        let message = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<OpaquePtr>::complete_err(ctx, message) };
    } else if !result.is_null() {
        unsafe { AsyncCompletion::<OpaquePtr>::complete_ok(ctx, OpaquePtr(result)) };
    } else {
        unsafe { AsyncCompletion::<OpaquePtr>::complete_err(ctx, "missing object result".into()) };
    }
}

unsafe extern "C" fn string_async_cb(result: *mut c_void, error: *const c_char, ctx: *mut c_void) {
    if !error.is_null() {
        let message = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, message) };
    } else if !result.is_null() {
        let value = unsafe { ffi::owned_string(result.cast::<c_char>()) };
        unsafe { AsyncCompletion::<String>::complete_ok(ctx, value) };
    } else {
        unsafe { AsyncCompletion::<String>::complete_err(ctx, "missing string result".into()) };
    }
}

pub struct AllAssetPacksFuture {
    inner: AsyncCompletionFuture<OpaquePtr>,
}

impl fmt::Debug for AllAssetPacksFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AllAssetPacksFuture")
            .finish_non_exhaustive()
    }
}

impl Future for AllAssetPacksFuture {
    type Output = Result<Vec<AssetPack>, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_object_future(&mut self.inner, cx, |OpaquePtr(ptr)| {
            Ok(collect_asset_packs(ptr))
        })
    }
}

pub struct AssetPackFuture {
    inner: AsyncCompletionFuture<OpaquePtr>,
}

impl fmt::Debug for AssetPackFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetPackFuture").finish_non_exhaustive()
    }
}

impl Future for AssetPackFuture {
    type Output = Result<AssetPack, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_object_future(&mut self.inner, cx, asset_pack_from_raw)
    }
}

pub struct AssetPackStatusFuture {
    inner: AsyncCompletionFuture<String>,
    context: &'static str,
}

impl fmt::Debug for AssetPackStatusFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetPackStatusFuture")
            .field("context", &self.context)
            .finish_non_exhaustive()
    }
}

impl Future for AssetPackStatusFuture {
    type Output = Result<AssetPackStatus, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let context = self.context;
        poll_string_future(&mut self.inner, cx, |value| {
            parse_status_response(value, context)
        })
    }
}

pub struct EnsureLocalAvailabilityFuture {
    inner: AsyncCompletionFuture<String>,
}

impl fmt::Debug for EnsureLocalAvailabilityFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnsureLocalAvailabilityFuture")
            .finish_non_exhaustive()
    }
}

impl Future for EnsureLocalAvailabilityFuture {
    type Output = Result<(), BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_string_future(&mut self.inner, cx, |_| Ok(()))
    }
}

pub struct CheckForUpdatesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl fmt::Debug for CheckForUpdatesFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CheckForUpdatesFuture")
            .finish_non_exhaustive()
    }
}

impl Future for CheckForUpdatesFuture {
    type Output = Result<UpdateCheck, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_string_future(&mut self.inner, cx, parse_update_check_response)
    }
}

pub struct RemoveAssetPackFuture {
    inner: AsyncCompletionFuture<String>,
}

impl fmt::Debug for RemoveAssetPackFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoveAssetPackFuture")
            .finish_non_exhaustive()
    }
}

impl Future for RemoveAssetPackFuture {
    type Output = Result<(), BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_string_future(&mut self.inner, cx, |_| Ok(()))
    }
}

pub struct CurrentDownloadsFuture {
    inner: AsyncCompletionFuture<OpaquePtr>,
}

impl fmt::Debug for CurrentDownloadsFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CurrentDownloadsFuture")
            .finish_non_exhaustive()
    }
}

impl Future for CurrentDownloadsFuture {
    type Output = Result<Vec<Download>, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_object_future(&mut self.inner, cx, |OpaquePtr(ptr)| {
            Ok(collect_downloads(ptr))
        })
    }
}

pub struct ExclusiveControlFuture {
    inner: AsyncCompletionFuture<String>,
}

impl fmt::Debug for ExclusiveControlFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExclusiveControlFuture")
            .finish_non_exhaustive()
    }
}

impl Future for ExclusiveControlFuture {
    type Output = Result<bool, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_string_future(&mut self.inner, cx, parse_bool_response)
    }
}

#[derive(Clone, Debug)]
pub struct AsyncAssetPackManager {
    inner: AssetPackManager,
}

impl AsyncAssetPackManager {
    pub fn shared() -> Option<Self> {
        AssetPackManager::shared().map(Self::new)
    }

    pub fn new(inner: AssetPackManager) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &AssetPackManager {
        &self.inner
    }

    pub fn into_inner(self) -> AssetPackManager {
        self.inner
    }

    pub fn all_asset_packs(&self) -> AllAssetPacksFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_all_asset_packs_async(
                self.inner.raw_ptr(),
                ctx,
                object_async_cb,
            );
        }
        AllAssetPacksFuture { inner: future }
    }

    pub fn asset_pack(
        &self,
        asset_pack_id: &str,
    ) -> Result<AssetPackFuture, BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_asset_pack_async(
                self.inner.raw_ptr(),
                asset_pack_id.as_ptr(),
                ctx,
                object_async_cb,
            );
        }
        Ok(AssetPackFuture { inner: future })
    }

    pub fn status_relative_to(&self, asset_pack: &AssetPack) -> AssetPackStatusFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_status_relative_async(
                self.inner.raw_ptr(),
                asset_pack.raw_ptr(),
                ctx,
                string_async_cb,
            );
        }
        AssetPackStatusFuture {
            inner: future,
            context: "asset-pack status",
        }
    }

    pub fn local_status(
        &self,
        asset_pack_id: &str,
    ) -> Result<AssetPackStatusFuture, BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_local_status_async(
                self.inner.raw_ptr(),
                asset_pack_id.as_ptr(),
                ctx,
                string_async_cb,
            );
        }
        Ok(AssetPackStatusFuture {
            inner: future,
            context: "local-status",
        })
    }

    pub fn ensure_local_availability(
        &self,
        asset_pack: &AssetPack,
        require_latest_version: bool,
    ) -> EnsureLocalAvailabilityFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_ensure_local_availability_async(
                self.inner.raw_ptr(),
                asset_pack.raw_ptr(),
                require_latest_version,
                ctx,
                string_async_cb,
            );
        }
        EnsureLocalAvailabilityFuture { inner: future }
    }

    pub fn check_for_updates(&self) -> CheckForUpdatesFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_check_for_updates_async(
                self.inner.raw_ptr(),
                ctx,
                string_async_cb,
            );
        }
        CheckForUpdatesFuture { inner: future }
    }

    pub fn remove(
        &self,
        asset_pack_id: &str,
    ) -> Result<RemoveAssetPackFuture, BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_asset_pack_manager_remove_async(
                self.inner.raw_ptr(),
                asset_pack_id.as_ptr(),
                ctx,
                string_async_cb,
            );
        }
        Ok(RemoveAssetPackFuture { inner: future })
    }

    pub fn status_updates(
        &self,
        capacity: usize,
    ) -> Result<DownloadStatusStream, BackgroundAssetsError> {
        self.inner.status_updates(capacity)
    }

    pub fn status_updates_for_asset_pack(
        &self,
        asset_pack_id: &str,
        capacity: usize,
    ) -> Result<DownloadStatusStream, BackgroundAssetsError> {
        self.inner
            .status_updates_for_asset_pack(asset_pack_id, capacity)
    }
}

#[derive(Clone, Debug)]
pub struct AsyncDownloadManager {
    inner: DownloadManager,
}

impl AsyncDownloadManager {
    pub fn shared() -> Option<Self> {
        DownloadManager::shared().map(Self::new)
    }

    pub fn new(inner: DownloadManager) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &DownloadManager {
        &self.inner
    }

    pub fn into_inner(self) -> DownloadManager {
        self.inner
    }

    pub fn current_downloads(&self) -> CurrentDownloadsFuture {
        let (future, ctx) = AsyncCompletion::create();
        unsafe {
            ffi::ba_download_manager_fetch_current_downloads_async(
                self.inner.raw_ptr(),
                ctx,
                object_async_cb,
            );
        }
        CurrentDownloadsFuture { inner: future }
    }

    pub fn with_exclusive_control(&self, before: Option<SystemTime>) -> ExclusiveControlFuture {
        let (future, ctx) = AsyncCompletion::create();
        let (seconds, has_before) = before.map_or((0.0, false), |timestamp| {
            let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
            (duration.as_secs_f64(), true)
        });
        unsafe {
            ffi::ba_download_manager_with_exclusive_control_async(
                self.inner.raw_ptr(),
                seconds,
                has_before,
                ctx,
                string_async_cb,
            );
        }
        ExclusiveControlFuture { inner: future }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_bool_response, parse_status_response, parse_update_check_response};

    #[test]
    fn parse_bool_response_accepts_expected_values() {
        assert!(parse_bool_response("true".into()).unwrap());
        assert!(parse_bool_response("1".into()).unwrap());
        assert!(!parse_bool_response("false".into()).unwrap());
        assert!(!parse_bool_response("0".into()).unwrap());
    }

    #[test]
    fn parse_status_response_rejects_invalid_payloads() {
        let error = parse_status_response("not-a-number".into(), "asset-pack status").unwrap_err();
        assert!(error
            .message_text()
            .contains("invalid asset-pack status payload"));
    }

    #[test]
    fn parse_update_check_response_decodes_json() {
        let update = parse_update_check_response(
            r#"{"updatingIDs":["pack.one"],"removedIDs":["pack.two"]}"#.into(),
        )
        .unwrap();
        assert_eq!(update.updating_ids, vec!["pack.one"]);
        assert_eq!(update.removed_ids, vec!["pack.two"]);
    }
}
