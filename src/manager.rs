use core::ffi::{c_char, c_void};
use std::fmt;
use std::ptr;

#[cfg(feature = "async")]
use std::ffi::CStr;
#[cfg(feature = "async")]
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

#[cfg(feature = "async")]
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion};
#[cfg(feature = "async")]
use doom_fish_utils::stream::{AsyncStreamSender, BoundedAsyncStream};

use crate::asset_pack::AssetPackSnapshot;
#[cfg(feature = "async")]
use crate::asset_pack::{collect_asset_packs, AssetPack, AssetPackStatus};
use crate::error::{BackgroundAssetsError, BridgeErrorPayload};
use crate::ffi;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DownloadProgress {
    pub completed_unit_count: i64,
    pub total_unit_count: i64,
    pub fraction_completed: f64,
    pub localized_description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UpdateCheck {
    pub updating_ids: Vec<String>,
    pub removed_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadStatusUpdate {
    Began {
        asset_pack: AssetPackSnapshot,
    },
    Paused {
        asset_pack: AssetPackSnapshot,
    },
    Downloading {
        asset_pack: AssetPackSnapshot,
        progress: DownloadProgress,
    },
    Finished {
        asset_pack: AssetPackSnapshot,
    },
    Failed {
        asset_pack: AssetPackSnapshot,
        error: BackgroundAssetsError,
    },
}

pub trait ManagedAssetPackDownloadDelegate: Send + 'static {
    fn download_of_asset_pack_began(&mut self, _asset_pack: &AssetPackSnapshot) {}

    fn download_of_asset_pack_paused(&mut self, _asset_pack: &AssetPackSnapshot) {}

    fn download_of_asset_pack_has_progress(
        &mut self,
        _asset_pack: &AssetPackSnapshot,
        _progress: &DownloadProgress,
    ) {
    }

    fn download_of_asset_pack_finished(&mut self, _asset_pack: &AssetPackSnapshot) {}

    fn download_of_asset_pack_failed(
        &mut self,
        _asset_pack: &AssetPackSnapshot,
        _error: &BackgroundAssetsError,
    ) {
    }
}

pub type ManagedAssetPackDownloadEvent = DownloadStatusUpdate;

pub struct AssetPackManager {
    ptr: *mut c_void,
}

impl AssetPackManager {
    pub fn shared() -> Option<Self> {
        Self::from_raw(unsafe { ffi::ba_asset_pack_manager_shared() })
    }

    fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    #[cfg(feature = "async")]
    pub(crate) const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn asset_pack_is_available_locally(&self, asset_pack_id: &str) -> bool {
        ffi::required_cstring(asset_pack_id, "asset_pack_id").is_ok_and(|asset_pack_id| unsafe {
            ffi::ba_asset_pack_manager_asset_pack_is_available_locally(
                self.ptr,
                asset_pack_id.as_ptr(),
            )
        })
    }

    pub fn contents(
        &self,
        path: &str,
        asset_pack_id: Option<&str>,
    ) -> Result<Vec<u8>, BackgroundAssetsError> {
        let path = ffi::required_cstring(path, "path")?;
        let asset_pack_id_cstr = asset_pack_id
            .map(|value| ffi::required_cstring(value, "asset_pack_id"))
            .transpose()?;
        let mut length = 0isize;
        let mut error: *mut c_char = ptr::null_mut();
        let bytes = unsafe {
            ffi::ba_asset_pack_manager_contents(
                self.ptr,
                path.as_ptr(),
                asset_pack_id_cstr
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                &mut length,
                &mut error,
            )
        };
        if !error.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(unsafe { ffi::owned_bytes(bytes, length) })
    }

    pub fn descriptor(
        &self,
        path: &str,
        asset_pack_id: Option<&str>,
    ) -> Result<i32, BackgroundAssetsError> {
        let path = ffi::required_cstring(path, "path")?;
        let asset_pack_id_cstr = asset_pack_id
            .map(|value| ffi::required_cstring(value, "asset_pack_id"))
            .transpose()?;
        let mut error: *mut c_char = ptr::null_mut();
        let descriptor = unsafe {
            ffi::ba_asset_pack_manager_descriptor(
                self.ptr,
                path.as_ptr(),
                asset_pack_id_cstr
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                &mut error,
            )
        };
        if !error.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(descriptor)
    }

    pub fn url(&self, path: &str) -> Result<String, BackgroundAssetsError> {
        let path = ffi::required_cstring(path, "path")?;
        let mut error: *mut c_char = ptr::null_mut();
        let url = unsafe { ffi::ba_asset_pack_manager_url(self.ptr, path.as_ptr(), &mut error) };
        if !error.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(unsafe { ffi::owned_string(url) })
    }

    #[cfg(feature = "async")]
    pub async fn all_asset_packs(&self) -> Result<Vec<AssetPack>, BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<OpaquePtr>::create();
        unsafe { ffi::ba_asset_pack_manager_all_asset_packs_async(self.ptr, ctx, object_async_cb) };
        let OpaquePtr(ptr) = future.await.map_err(BackgroundAssetsError::message)?;
        Ok(collect_asset_packs(ptr))
    }

    #[cfg(feature = "async")]
    pub async fn asset_pack(
        &self,
        asset_pack_id: &str,
    ) -> Result<AssetPack, BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::<OpaquePtr>::create();
        unsafe {
            ffi::ba_asset_pack_manager_asset_pack_async(
                self.ptr,
                asset_pack_id.as_ptr(),
                ctx,
                object_async_cb,
            );
        };
        future
            .await
            .map_err(BackgroundAssetsError::message)
            .and_then(|OpaquePtr(ptr)| {
                AssetPack::from_raw(ptr).ok_or_else(|| {
                    BackgroundAssetsError::message("asset-pack pointer must not be null")
                })
            })
    }

    #[cfg(feature = "async")]
    pub async fn status_relative_to(
        &self,
        asset_pack: &AssetPack,
    ) -> Result<AssetPackStatus, BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<String>::create();
        unsafe {
            ffi::ba_asset_pack_manager_status_relative_async(
                self.ptr,
                asset_pack.raw_ptr(),
                ctx,
                string_async_cb,
            );
        };
        let value = future.await.map_err(BackgroundAssetsError::message)?;
        let bits = value.parse::<u64>().map_err(|error| {
            BackgroundAssetsError::message(format!("invalid asset-pack status payload: {error}"))
        })?;
        Ok(AssetPackStatus::new(bits))
    }

    #[cfg(feature = "async")]
    pub async fn local_status(
        &self,
        asset_pack_id: &str,
    ) -> Result<AssetPackStatus, BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::<String>::create();
        unsafe {
            ffi::ba_asset_pack_manager_local_status_async(
                self.ptr,
                asset_pack_id.as_ptr(),
                ctx,
                string_async_cb,
            );
        };
        let value = future.await.map_err(BackgroundAssetsError::message)?;
        let bits = value.parse::<u64>().map_err(|error| {
            BackgroundAssetsError::message(format!("invalid local-status payload: {error}"))
        })?;
        Ok(AssetPackStatus::new(bits))
    }

    #[cfg(feature = "async")]
    pub async fn ensure_local_availability(
        &self,
        asset_pack: &AssetPack,
        require_latest_version: bool,
    ) -> Result<(), BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<String>::create();
        unsafe {
            ffi::ba_asset_pack_manager_ensure_local_availability_async(
                self.ptr,
                asset_pack.raw_ptr(),
                require_latest_version,
                ctx,
                string_async_cb,
            );
        };
        future.await.map_err(BackgroundAssetsError::message)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn check_for_updates(&self) -> Result<UpdateCheck, BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<String>::create();
        unsafe {
            ffi::ba_asset_pack_manager_check_for_updates_async(self.ptr, ctx, string_async_cb);
        };
        let payload = future.await.map_err(BackgroundAssetsError::message)?;
        let bridge: UpdateCheckPayload = serde_json::from_str(&payload).map_err(|error| {
            BackgroundAssetsError::message(format!("invalid update-check payload: {error}"))
        })?;
        Ok(UpdateCheck {
            updating_ids: bridge.updating_ids,
            removed_ids: bridge.removed_ids,
        })
    }

    #[cfg(feature = "async")]
    pub async fn remove(&self, asset_pack_id: &str) -> Result<(), BackgroundAssetsError> {
        let asset_pack_id = ffi::required_cstring(asset_pack_id, "asset_pack_id")?;
        let (future, ctx) = AsyncCompletion::<String>::create();
        unsafe {
            ffi::ba_asset_pack_manager_remove_async(
                self.ptr,
                asset_pack_id.as_ptr(),
                ctx,
                string_async_cb,
            );
        };
        future.await.map_err(BackgroundAssetsError::message)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub fn status_updates(
        &self,
        capacity: usize,
    ) -> Result<DownloadStatusStream, BackgroundAssetsError> {
        self.make_status_stream(None, capacity)
    }

    #[cfg(feature = "async")]
    pub fn status_updates_for_asset_pack(
        &self,
        asset_pack_id: &str,
        capacity: usize,
    ) -> Result<DownloadStatusStream, BackgroundAssetsError> {
        self.make_status_stream(Some(asset_pack_id), capacity)
    }

    #[cfg(feature = "async")]
    fn make_status_stream(
        &self,
        asset_pack_id: Option<&str>,
        capacity: usize,
    ) -> Result<DownloadStatusStream, BackgroundAssetsError> {
        let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
        let ctx = Box::into_raw(Box::new(StreamContext { sender })).cast::<c_void>();
        let asset_pack_id_cstr = asset_pack_id
            .map(|value| ffi::required_cstring(value, "asset_pack_id"))
            .transpose()?;
        let bridge_ptr = unsafe {
            ffi::ba_asset_pack_manager_status_updates_stream_create(
                self.ptr,
                asset_pack_id_cstr
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                ctx,
                stream_callback,
            )
        };

        if bridge_ptr.is_null() {
            unsafe { drop(Box::from_raw(ctx.cast::<StreamContext>())) };
            return Err(BackgroundAssetsError::message(
                "failed to create asset-pack status stream",
            ));
        }

        Ok(DownloadStatusStream { stream, bridge_ptr })
    }
}

impl Clone for AssetPackManager {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for AssetPackManager {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for AssetPackManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetPackManager").finish_non_exhaustive()
    }
}

// SAFETY: `AssetPackManager` wraps a retained Swift box around the shared actor.
// Calls back into Swift; Rust never dereferences the raw pointer directly.
unsafe impl Send for AssetPackManager {}
// SAFETY: See `Send` justification above.
unsafe impl Sync for AssetPackManager {}

#[cfg(feature = "async")]
pub struct ManagedAssetPackDownloadEventStream {
    inner: BoundedAsyncStream<ManagedAssetPackDownloadEvent>,
    bridge_ptr: *mut c_void,
}

#[cfg(feature = "async")]
impl ManagedAssetPackDownloadEventStream {
    pub fn next(
        &self,
    ) -> impl std::future::Future<Output = Option<ManagedAssetPackDownloadEvent>> + '_ {
        self.inner.next()
    }

    pub fn try_next(&self) -> Option<ManagedAssetPackDownloadEvent> {
        self.inner.try_next()
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    pub fn buffered_count(&self) -> usize {
        self.inner.buffered_count()
    }
}

#[cfg(feature = "async")]
impl fmt::Debug for ManagedAssetPackDownloadEventStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagedAssetPackDownloadEventStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

#[cfg(feature = "async")]
impl Drop for ManagedAssetPackDownloadEventStream {
    fn drop(&mut self) {
        if !self.bridge_ptr.is_null() {
            unsafe {
                ffi::ba_asset_pack_manager_delegate_clear_if_matches(self.bridge_ptr);
                ffi::ba_object_release(self.bridge_ptr);
            }
        }
    }
}

#[cfg(feature = "async")]
struct ManagedAssetPackDownloadDelegateState {
    handler: Box<dyn ManagedAssetPackDownloadDelegate>,
    sender: AsyncStreamSender<ManagedAssetPackDownloadEvent>,
}

#[cfg(feature = "async")]
fn managed_asset_pack_delegate_state(
) -> &'static Mutex<Option<ManagedAssetPackDownloadDelegateState>> {
    static STATE: OnceLock<Mutex<Option<ManagedAssetPackDownloadDelegateState>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

#[cfg(feature = "async")]
pub fn install_global_managed_asset_pack_download_delegate<H>(
    handler: H,
    capacity: usize,
) -> Result<ManagedAssetPackDownloadEventStream, BackgroundAssetsError>
where
    H: ManagedAssetPackDownloadDelegate,
{
    let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
    if let Ok(mut state) = managed_asset_pack_delegate_state().lock() {
        *state = Some(ManagedAssetPackDownloadDelegateState {
            handler: Box::new(handler),
            sender,
        });
    }

    let bridge_ptr = unsafe { ffi::ba_asset_pack_manager_delegate_install() };
    if bridge_ptr.is_null() {
        return Err(BackgroundAssetsError::message(
            "failed to install managed asset-pack delegate",
        ));
    }

    Ok(ManagedAssetPackDownloadEventStream {
        inner: stream,
        bridge_ptr,
    })
}

#[cfg(feature = "async")]
fn string_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg(feature = "async")]
pub struct DownloadStatusStream {
    stream: BoundedAsyncStream<DownloadStatusUpdate>,
    bridge_ptr: *mut c_void,
}

#[cfg(feature = "async")]
impl DownloadStatusStream {
    pub fn next(&self) -> impl std::future::Future<Output = Option<DownloadStatusUpdate>> + '_ {
        self.stream.next()
    }

    pub fn try_next(&self) -> Option<DownloadStatusUpdate> {
        self.stream.try_next()
    }

    pub fn is_closed(&self) -> bool {
        self.stream.is_closed()
    }

    pub fn buffered_count(&self) -> usize {
        self.stream.buffered_count()
    }
}

#[cfg(feature = "async")]
impl fmt::Debug for DownloadStatusStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadStatusStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

#[cfg(feature = "async")]
impl Drop for DownloadStatusStream {
    fn drop(&mut self) {
        if !self.bridge_ptr.is_null() {
            unsafe { ffi::ba_object_release(self.bridge_ptr) };
        }
    }
}

#[derive(Deserialize)]
struct AssetPackSnapshotPayload {
    id: String,
    #[serde(rename = "downloadSize")]
    download_size: i64,
    version: i64,
    description: String,
}

#[derive(Deserialize)]
struct ProgressPayload {
    #[serde(rename = "completedUnitCount")]
    completed_unit_count: i64,
    #[serde(rename = "totalUnitCount")]
    total_unit_count: i64,
    #[serde(rename = "fractionCompleted")]
    fraction_completed: f64,
    #[serde(rename = "localizedDescription")]
    localized_description: String,
}

#[cfg(feature = "async")]
#[derive(Deserialize)]
struct UpdateCheckPayload {
    #[serde(rename = "updatingIDs")]
    updating_ids: Vec<String>,
    #[serde(rename = "removedIDs")]
    removed_ids: Vec<String>,
}

#[derive(Deserialize)]
struct DownloadStatusUpdatePayload {
    kind: String,
    #[serde(rename = "assetPack")]
    asset_pack: AssetPackSnapshotPayload,
    progress: Option<ProgressPayload>,
    error: Option<BridgeErrorPayload>,
}

impl From<AssetPackSnapshotPayload> for AssetPackSnapshot {
    fn from(value: AssetPackSnapshotPayload) -> Self {
        Self {
            id: value.id,
            download_size: value.download_size,
            version: value.version,
            description: value.description,
        }
    }
}

impl From<ProgressPayload> for DownloadProgress {
    fn from(value: ProgressPayload) -> Self {
        Self {
            completed_unit_count: value.completed_unit_count,
            total_unit_count: value.total_unit_count,
            fraction_completed: value.fraction_completed,
            localized_description: value.localized_description,
        }
    }
}

fn asset_pack_snapshot_from_json(json: &str) -> Option<AssetPackSnapshot> {
    serde_json::from_str::<AssetPackSnapshotPayload>(json)
        .ok()
        .map(Into::into)
}

fn progress_from_json(json: &str) -> Option<DownloadProgress> {
    serde_json::from_str::<ProgressPayload>(json)
        .ok()
        .map(Into::into)
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_asset_pack_download_delegate_began(
    asset_pack_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json)) else {
            return;
        };
        if let Ok(mut state_guard) = managed_asset_pack_delegate_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.download_of_asset_pack_began(&asset_pack);
                state
                    .sender
                    .push(ManagedAssetPackDownloadEvent::Began { asset_pack });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_asset_pack_download_delegate_paused(
    asset_pack_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json)) else {
            return;
        };
        if let Ok(mut state_guard) = managed_asset_pack_delegate_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.download_of_asset_pack_paused(&asset_pack);
                state
                    .sender
                    .push(ManagedAssetPackDownloadEvent::Paused { asset_pack });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_asset_pack_download_delegate_progress(
    asset_pack_json: *const c_char,
    progress_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
        let _ = progress_json;
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json)) else {
            return;
        };
        let Some(progress) = progress_from_json(&string_from_ptr(progress_json)) else {
            return;
        };
        if let Ok(mut state_guard) = managed_asset_pack_delegate_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state
                    .handler
                    .download_of_asset_pack_has_progress(&asset_pack, &progress);
                state.sender.push(ManagedAssetPackDownloadEvent::Downloading {
                    asset_pack,
                    progress,
                });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_asset_pack_download_delegate_finished(
    asset_pack_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json)) else {
            return;
        };
        if let Ok(mut state_guard) = managed_asset_pack_delegate_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.download_of_asset_pack_finished(&asset_pack);
                state
                    .sender
                    .push(ManagedAssetPackDownloadEvent::Finished { asset_pack });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_asset_pack_download_delegate_failed(
    asset_pack_json: *const c_char,
    error_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
        let _ = error_json;
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json)) else {
            return;
        };
        let error = BackgroundAssetsError::from_json_str(&string_from_ptr(error_json));
        if let Ok(mut state_guard) = managed_asset_pack_delegate_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state
                    .handler
                    .download_of_asset_pack_failed(&asset_pack, &error);
                state.sender.push(ManagedAssetPackDownloadEvent::Failed {
                    asset_pack,
                    error,
                });
            }
        }
    }
}

impl TryFrom<DownloadStatusUpdatePayload> for DownloadStatusUpdate {
    type Error = BackgroundAssetsError;

    fn try_from(value: DownloadStatusUpdatePayload) -> Result<Self, Self::Error> {
        let asset_pack = value.asset_pack.into();
        Ok(match value.kind.as_str() {
            "began" => Self::Began { asset_pack },
            "paused" => Self::Paused { asset_pack },
            "downloading" => Self::Downloading {
                asset_pack,
                progress: value
                    .progress
                    .ok_or_else(|| BackgroundAssetsError::message("missing progress payload"))?
                    .into(),
            },
            "finished" => Self::Finished { asset_pack },
            "failed" => Self::Failed {
                asset_pack,
                error: value
                    .error
                    .map(Into::into)
                    .ok_or_else(|| BackgroundAssetsError::message("missing error payload"))?,
            },
            other => {
                return Err(BackgroundAssetsError::message(format!(
                    "unknown download status update kind: {other}"
                )))
            }
        })
    }
}

#[cfg(feature = "async")]
struct OpaquePtr(*mut c_void);
#[cfg(feature = "async")]
unsafe impl Send for OpaquePtr {}

#[cfg(feature = "async")]
unsafe extern "C" fn object_async_cb(result: *mut c_void, error: *const c_char, ctx: *mut c_void) {
    if !error.is_null() {
        let message = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<OpaquePtr>::complete_err(ctx, message) };
    } else if !result.is_null() {
        unsafe { AsyncCompletion::complete_ok(ctx, OpaquePtr(result)) };
    } else {
        unsafe { AsyncCompletion::<OpaquePtr>::complete_err(ctx, "missing object result".into()) };
    }
}

#[cfg(feature = "async")]
unsafe extern "C" fn string_async_cb(result: *mut c_void, error: *const c_char, ctx: *mut c_void) {
    if !error.is_null() {
        let message = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<String>::complete_err(ctx, message) };
    } else if !result.is_null() {
        let value = unsafe { ffi::owned_string(result.cast::<c_char>()) };
        unsafe { AsyncCompletion::complete_ok(ctx, value) };
    } else {
        unsafe { AsyncCompletion::<String>::complete_err(ctx, "missing string result".into()) };
    }
}

#[cfg(feature = "async")]
struct StreamContext {
    sender: AsyncStreamSender<DownloadStatusUpdate>,
}

#[cfg(feature = "async")]
unsafe extern "C" fn stream_callback(ctx: *mut c_void, event_json: *mut c_char, done: bool) {
    if done {
        if !ctx.is_null() {
            unsafe { drop(Box::from_raw(ctx.cast::<StreamContext>())) };
        }
        return;
    }
    if ctx.is_null() || event_json.is_null() {
        return;
    }

    let json = unsafe { ffi::owned_string(event_json) };
    let Ok(payload) = serde_json::from_str::<DownloadStatusUpdatePayload>(&json) else {
        return;
    };
    let Ok(update) = DownloadStatusUpdate::try_from(payload) else {
        return;
    };
    let context = unsafe { &mut *ctx.cast::<StreamContext>() };
    context.sender.push(update);
}

#[cfg(test)]
mod tests {
    use super::{asset_pack_snapshot_from_json, progress_from_json};

    #[test]
    fn asset_pack_snapshot_payload_decodes_json() {
        let snapshot = asset_pack_snapshot_from_json(
            r#"{"id":"pack.one","downloadSize":1024,"version":3,"description":"Pack One"}"#,
        )
        .unwrap();

        assert_eq!(snapshot.id, "pack.one");
        assert_eq!(snapshot.download_size, 1024);
        assert_eq!(snapshot.version, 3);
        assert_eq!(snapshot.description, "Pack One");
    }

    #[test]
    fn progress_payload_decodes_json() {
        let progress = progress_from_json(
            r#"{"completedUnitCount":10,"totalUnitCount":20,"fractionCompleted":0.5,"localizedDescription":"halfway"}"#,
        )
        .unwrap();

        assert_eq!(progress.completed_unit_count, 10);
        assert_eq!(progress.total_unit_count, 20);
        assert!((progress.fraction_completed - 0.5).abs() < f64::EPSILON);
        assert_eq!(progress.localized_description, "halfway");
    }
}
