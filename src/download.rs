use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::ptr;

#[cfg(feature = "async")]
use std::ffi::CStr;
#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;
#[cfg(feature = "async")]
use std::task::{Context, Poll};
#[cfg(feature = "async")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "async")]
use doom_fish_utils::callback_context::CallbackContext;
#[cfg(feature = "async")]
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
#[cfg(feature = "async")]
use doom_fish_utils::panic_safe::{catch_user_panic, catch_user_panic_result};
#[cfg(feature = "async")]
use doom_fish_utils::stream::BoundedAsyncStream;
use serde::{Deserialize, Serialize};

use crate::error::BackgroundAssetsError;
use crate::extension::{AuthenticationChallenge, ChallengeDisposition};
use crate::ffi;
#[cfg(feature = "async")]
use crate::handler::HandlerState;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ContentRequest {
    Install,
    Update,
    Periodic,
}

impl ContentRequest {
    pub(crate) const fn as_raw(self) -> isize {
        match self {
            Self::Install => 1,
            Self::Update => 2,
            Self::Periodic => 3,
        }
    }

    #[cfg(feature = "async")]
    pub(crate) const fn from_raw(value: isize) -> Option<Self> {
        match value {
            1 => Some(Self::Install),
            2 => Some(Self::Update),
            3 => Some(Self::Periodic),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub enum DownloadStatus {
    Failed,
    Created,
    Waiting,
    Downloading,
    Finished,
    Unknown(i64),
}

impl DownloadStatus {
    const fn from_raw(value: i64) -> Self {
        match value {
            -1 => Self::Failed,
            0 => Self::Created,
            1 => Self::Waiting,
            2 => Self::Downloading,
            3 => Self::Finished,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DownloadPriority(i64);

impl DownloadPriority {
    pub const fn new(raw_value: i64) -> Self {
        Self(raw_value)
    }

    pub const fn raw_value(self) -> i64 {
        self.0
    }

    pub fn min() -> Self {
        Self(unsafe { ffi::ba_downloader_priority_min() as i64 })
    }

    pub fn default_value() -> Self {
        Self(unsafe { ffi::ba_downloader_priority_default() as i64 })
    }

    pub fn max() -> Self {
        Self(unsafe { ffi::ba_downloader_priority_max() as i64 })
    }
}

impl Default for DownloadPriority {
    fn default() -> Self {
        Self::default_value()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DownloadSnapshot {
    pub identifier: String,
    pub unique_identifier: String,
    pub status: DownloadStatus,
    pub priority: DownloadPriority,
    pub is_essential: bool,
    pub is_url_download: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DownloadWriteProgress {
    pub bytes_written: i64,
    pub total_bytes_written: i64,
    pub total_bytes_expected_to_write: i64,
}

impl DownloadWriteProgress {
    #[allow(clippy::cast_precision_loss)]
    pub fn fraction_completed(&self) -> Option<f64> {
        (self.total_bytes_expected_to_write > 0)
            .then(|| self.total_bytes_written as f64 / self.total_bytes_expected_to_write as f64)
    }
}

pub trait DownloadManagerDelegate: Send + 'static {
    fn download_did_begin(&mut self, _download: &DownloadSnapshot) {}

    fn download_did_pause(&mut self, _download: &DownloadSnapshot) {}

    fn download_did_write_bytes(
        &mut self,
        _download: &DownloadSnapshot,
        _progress: &DownloadWriteProgress,
    ) {
    }

    fn did_receive_challenge(
        &mut self,
        _download: &DownloadSnapshot,
        _challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        ChallengeDisposition::PerformDefaultHandling
    }

    fn download_failed(&mut self, _download: &DownloadSnapshot, _error: &BackgroundAssetsError) {}

    fn download_finished(&mut self, _download: &DownloadSnapshot, _file_url: &str) {}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadManagerEvent {
    Began {
        download: DownloadSnapshot,
    },
    Paused {
        download: DownloadSnapshot,
    },
    Progress {
        download: DownloadSnapshot,
        progress: DownloadWriteProgress,
    },
    ChallengeRequested {
        download: DownloadSnapshot,
        challenge: AuthenticationChallenge,
        disposition: ChallengeDisposition,
    },
    Failed {
        download: DownloadSnapshot,
        error: BackgroundAssetsError,
    },
    Finished {
        download: DownloadSnapshot,
        file_url: String,
    },
}

pub struct Download {
    pub(crate) ptr: *mut c_void,
}

impl Download {
    pub(crate) fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    #[cfg(feature = "async")]
    pub(crate) unsafe fn retained_from_borrowed(ptr: *mut c_void) -> Option<Self> {
        Self::from_raw(ffi::retained(ptr))
    }

    #[cfg(feature = "async")]
    pub(crate) const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn identifier(&self) -> String {
        unsafe { ffi::owned_string(ffi::ba_download_identifier(self.ptr)) }
    }

    pub fn unique_identifier(&self) -> String {
        unsafe { ffi::owned_string(ffi::ba_download_unique_identifier(self.ptr)) }
    }

    pub fn status(&self) -> DownloadStatus {
        DownloadStatus::from_raw(unsafe { ffi::ba_download_status(self.ptr) as i64 })
    }

    pub fn priority(&self) -> DownloadPriority {
        DownloadPriority::new(unsafe { ffi::ba_download_priority(self.ptr) as i64 })
    }

    pub fn is_essential(&self) -> bool {
        unsafe { ffi::ba_download_is_essential(self.ptr) }
    }

    pub fn is_url_download(&self) -> bool {
        unsafe { ffi::ba_download_is_url_download(self.ptr) }
    }

    pub fn removing_essential(&self) -> Option<Self> {
        Self::from_raw(unsafe { ffi::ba_download_removing_essential(self.ptr) })
    }

    pub fn snapshot(&self) -> DownloadSnapshot {
        DownloadSnapshot {
            identifier: self.identifier(),
            unique_identifier: self.unique_identifier(),
            status: self.status(),
            priority: self.priority(),
            is_essential: self.is_essential(),
            is_url_download: self.is_url_download(),
        }
    }
}

impl Clone for Download {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for Download {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Download")
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

// SAFETY: `Download` wraps a retained Objective-C `BADownload` pointer. The SDK
// documents it as Swift-sendable, and Rust never dereferences the pointer directly.
unsafe impl Send for Download {}
// SAFETY: See `Send` justification above.
unsafe impl Sync for Download {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UrlDownloadOptions {
    pub method: String,
    pub headers: BTreeMap<String, String>,
    pub essential: bool,
    pub priority: DownloadPriority,
}

impl Default for UrlDownloadOptions {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            headers: BTreeMap::new(),
            essential: false,
            priority: DownloadPriority::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UrlDownload {
    inner: Download,
}

impl UrlDownload {
    pub fn new(
        identifier: &str,
        url: &str,
        file_size: u64,
        app_group_id: &str,
    ) -> Result<Self, BackgroundAssetsError> {
        Self::with_options(
            identifier,
            url,
            file_size,
            app_group_id,
            UrlDownloadOptions::default(),
        )
    }

    pub fn with_options(
        identifier: &str,
        url: &str,
        file_size: u64,
        app_group_id: &str,
        options: UrlDownloadOptions,
    ) -> Result<Self, BackgroundAssetsError> {
        validate_file_size(file_size)?;
        let priority = validate_priority(
            options.priority,
            DownloadPriority::min(),
            DownloadPriority::max(),
        )?;
        let identifier = ffi::required_cstring(identifier, "identifier")?;
        let url = ffi::required_cstring(url, "url")?;
        let method = ffi::required_cstring(&options.method, "method")?;
        let headers_json = serde_json::to_string(&options.headers)
            .map_err(|error| BackgroundAssetsError::invalid_argument(error.to_string()))?;
        let headers_json = ffi::required_cstring(&headers_json, "headers")?;
        let app_group_id = ffi::required_cstring(app_group_id, "app_group_id")?;
        let mut error: *mut c_char = ptr::null_mut();

        let ptr = unsafe {
            ffi::ba_url_download_create(
                identifier.as_ptr(),
                url.as_ptr(),
                method.as_ptr(),
                headers_json.as_ptr(),
                file_size,
                app_group_id.as_ptr(),
                options.essential,
                priority,
                &raw mut error,
            )
        };

        if ptr.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }

        Ok(Self {
            inner: Download { ptr },
        })
    }

    pub fn into_download(self) -> Download {
        self.inner
    }
}

impl Deref for UrlDownload {
    type Target = Download;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn validate_file_size(file_size: u64) -> Result<(), BackgroundAssetsError> {
    if file_size == 0 || isize::try_from(file_size).is_err() {
        return Err(BackgroundAssetsError::invalid_argument(format!(
            "file_size must be between 1 and {} bytes, got {file_size}",
            isize::MAX
        )));
    }
    Ok(())
}

fn validate_priority(
    priority: DownloadPriority,
    min: DownloadPriority,
    max: DownloadPriority,
) -> Result<isize, BackgroundAssetsError> {
    if priority.raw_value() < min.raw_value() || priority.raw_value() > max.raw_value() {
        return Err(BackgroundAssetsError::invalid_argument(format!(
            "download priority {} is outside {}..={}",
            priority.raw_value(),
            min.raw_value(),
            max.raw_value()
        )));
    }
    isize::try_from(priority.raw_value())
        .map_err(|_| BackgroundAssetsError::invalid_argument("download priority is out of range"))
}

pub struct DownloadManager {
    ptr: *mut c_void,
}

impl DownloadManager {
    pub fn shared() -> Result<Self, BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::ba_download_manager_shared(&raw mut error) };
        if ptr.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(Self { ptr })
    }

    #[cfg(feature = "async")]
    pub(crate) const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn schedule_download(&self, download: &Download) -> Result<(), BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let scheduled = unsafe {
            ffi::ba_download_manager_schedule_download(self.ptr, download.ptr, &raw mut error)
        };
        if scheduled {
            Ok(())
        } else {
            Err(BackgroundAssetsError::from_owned_json_ptr(error))
        }
    }

    pub fn start_foreground_download(
        &self,
        download: &Download,
    ) -> Result<(), BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let started = unsafe {
            ffi::ba_download_manager_start_foreground_download(
                self.ptr,
                download.ptr,
                &raw mut error,
            )
        };
        if started {
            Ok(())
        } else {
            Err(BackgroundAssetsError::from_owned_json_ptr(error))
        }
    }

    pub fn cancel_download(&self, download: &Download) -> Result<(), BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let cancelled = unsafe {
            ffi::ba_download_manager_cancel_download(self.ptr, download.ptr, &raw mut error)
        };
        if cancelled {
            Ok(())
        } else {
            Err(BackgroundAssetsError::from_owned_json_ptr(error))
        }
    }

    #[cfg(feature = "async")]
    pub async fn current_downloads(&self) -> Result<Vec<Download>, BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<OpaquePtr>::create();
        unsafe {
            ffi::ba_download_manager_fetch_current_downloads_async(
                self.ptr,
                ctx,
                downloads_async_cb,
            );
        }
        let OpaquePtr(ptr) = future.await.map_err(BackgroundAssetsError::from_json_str)?;
        Ok(collect_downloads(ptr))
    }

    #[cfg(feature = "async")]
    pub fn with_exclusive_control<F, R>(
        &self,
        before: Option<SystemTime>,
        body: F,
    ) -> ExclusiveControlFuture<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (future, job) = exclusive_control_job(body);
        let (seconds, has_before) = before.map_or((0.0, false), |timestamp| {
            let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
            (duration.as_secs_f64(), true)
        });
        unsafe {
            ffi::ba_download_manager_with_exclusive_control(
                self.ptr,
                seconds,
                has_before,
                job,
                exclusive_control_callback,
            );
        }
        future
    }
}

#[cfg(feature = "async")]
type ExclusiveControlJob = Box<dyn FnOnce(Result<(), String>) + Send>;

#[cfg(feature = "async")]
fn exclusive_control_job<F, R>(body: F) -> (ExclusiveControlFuture<R>, *mut c_void)
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (future, ctx) = AsyncCompletion::<R>::create();
    let completion = CompletionPtr(ctx);
    let job: ExclusiveControlJob = Box::new(move |acquired| {
        let ctx = completion.into_raw();
        let result = acquired.and_then(|()| {
            catch_user_panic_result("DownloadManager::with_exclusive_control body", body)
                .ok_or_else(|| "the exclusive-control body panicked".to_owned())
        });
        unsafe { AsyncCompletion::complete_with_result(ctx, result) };
    });
    (
        ExclusiveControlFuture { inner: future },
        Box::into_raw(Box::new(job)).cast::<c_void>(),
    )
}

#[cfg(feature = "async")]
struct CompletionPtr(*mut c_void);

#[cfg(feature = "async")]
unsafe impl Send for CompletionPtr {}

#[cfg(feature = "async")]
impl CompletionPtr {
    fn into_raw(self) -> *mut c_void {
        self.0
    }
}

#[cfg(feature = "async")]
unsafe extern "C" fn exclusive_control_callback(
    job: *mut c_void,
    acquired_lock: bool,
    error_json: *const c_char,
) {
    if job.is_null() {
        return;
    }
    let job = unsafe { Box::from_raw(job.cast::<ExclusiveControlJob>()) };
    let acquired = if acquired_lock {
        Ok(())
    } else if error_json.is_null() {
        Err("exclusive control over the download manager was not acquired".to_owned())
    } else {
        Err(string_from_ptr(error_json))
    };
    catch_user_panic("DownloadManager::with_exclusive_control", move || {
        job(acquired);
    });
}

#[cfg(feature = "async")]
pub struct ExclusiveControlFuture<R> {
    inner: AsyncCompletionFuture<R>,
}

#[cfg(feature = "async")]
impl<R> fmt::Debug for ExclusiveControlFuture<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExclusiveControlFuture")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "async")]
impl<R> Future for ExclusiveControlFuture<R> {
    type Output = Result<R, BackgroundAssetsError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(BackgroundAssetsError::from_json_str))
    }
}

impl Clone for DownloadManager {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for DownloadManager {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for DownloadManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadManager").finish_non_exhaustive()
    }
}

#[cfg(feature = "async")]
type DownloadManagerDelegateState = HandlerState<dyn DownloadManagerDelegate, DownloadManagerEvent>;

#[cfg(feature = "async")]
type DownloadManagerDelegateContext = CallbackContext<DownloadManagerDelegateState>;

#[cfg(feature = "async")]
pub struct DownloadManagerEventStream {
    inner: BoundedAsyncStream<DownloadManagerEvent>,
    context: DownloadManagerDelegateContext,
    bridge_ptr: *mut c_void,
}

#[cfg(feature = "async")]
impl DownloadManagerEventStream {
    pub fn next(&self) -> impl std::future::Future<Output = Option<DownloadManagerEvent>> + '_ {
        self.inner.next()
    }

    pub fn try_next(&self) -> Option<DownloadManagerEvent> {
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
impl fmt::Debug for DownloadManagerEventStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadManagerEventStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

#[cfg(feature = "async")]
impl Drop for DownloadManagerEventStream {
    fn drop(&mut self) {
        self.context.deactivate();
        unsafe {
            ffi::ba_download_manager_delegate_clear_if_matches(self.bridge_ptr);
            ffi::ba_object_release(self.bridge_ptr);
        }
    }
}

#[cfg(feature = "async")]
pub fn install_global_download_manager_delegate<H>(
    handler: H,
    capacity: usize,
) -> Result<DownloadManagerEventStream, BackgroundAssetsError>
where
    H: DownloadManagerDelegate,
{
    let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
    let handler: Box<dyn DownloadManagerDelegate> = Box::new(handler);
    let context = DownloadManagerDelegateContext::new(HandlerState::new(handler, sender));
    let mut error: *mut c_char = ptr::null_mut();
    let bridge_ptr = unsafe {
        ffi::ba_download_manager_delegate_install(
            context.as_ptr(),
            DownloadManagerDelegateContext::RETAIN,
            DownloadManagerDelegateContext::RELEASE,
            &raw mut error,
        )
    };
    if bridge_ptr.is_null() {
        return Err(BackgroundAssetsError::from_owned_json_ptr(error));
    }
    Ok(DownloadManagerEventStream {
        inner: stream,
        context,
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

#[derive(Deserialize)]
struct DownloadSnapshotPayload {
    identifier: String,
    #[serde(rename = "uniqueIdentifier")]
    unique_identifier: String,
    status: i64,
    priority: i64,
    #[serde(rename = "isEssential")]
    is_essential: bool,
    #[serde(rename = "isURLDownload")]
    is_url_download: bool,
}

impl From<DownloadSnapshotPayload> for DownloadSnapshot {
    fn from(value: DownloadSnapshotPayload) -> Self {
        Self {
            identifier: value.identifier,
            unique_identifier: value.unique_identifier,
            status: DownloadStatus::from_raw(value.status),
            priority: DownloadPriority::new(value.priority),
            is_essential: value.is_essential,
            is_url_download: value.is_url_download,
        }
    }
}

#[cfg_attr(not(feature = "async"), allow(dead_code))]
fn download_snapshot_from_json(json: &str) -> Option<DownloadSnapshot> {
    serde_json::from_str::<DownloadSnapshotPayload>(json)
        .ok()
        .map(Into::into)
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_did_begin(
    ctx: *mut c_void,
    download_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (ctx, download_json);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return;
        };
        let site = "DownloadManagerDelegate::download_did_begin";
        unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                state.call(site, |handler| handler.download_did_begin(&download));
                state.sender.push(DownloadManagerEvent::Began { download });
            })
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_did_pause(
    ctx: *mut c_void,
    download_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (ctx, download_json);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return;
        };
        let site = "DownloadManagerDelegate::download_did_pause";
        unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                state.call(site, |handler| handler.download_did_pause(&download));
                state.sender.push(DownloadManagerEvent::Paused { download });
            })
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_did_write_bytes(
    ctx: *mut c_void,
    download_json: *const c_char,
    bytes_written: i64,
    total_bytes_written: i64,
    total_bytes_expected_to_write: i64,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (
            ctx,
            download_json,
            bytes_written,
            total_bytes_written,
            total_bytes_expected_to_write,
        );
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return;
        };
        let progress = DownloadWriteProgress {
            bytes_written,
            total_bytes_written,
            total_bytes_expected_to_write,
        };
        let site = "DownloadManagerDelegate::download_did_write_bytes";
        unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                state.call(site, |handler| {
                    handler.download_did_write_bytes(&download, &progress);
                });
                state
                    .sender
                    .push(DownloadManagerEvent::Progress { download, progress });
            })
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_challenge_disposition(
    ctx: *mut c_void,
    download_json: *const c_char,
    challenge_json: *const c_char,
) -> i32 {
    #[cfg(not(feature = "async"))]
    {
        let _ = (ctx, download_json, challenge_json);
        ChallengeDisposition::PerformDefaultHandling as i32
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let challenge =
            serde_json::from_str::<AuthenticationChallenge>(&string_from_ptr(challenge_json))
                .unwrap_or_default();
        let site = "DownloadManagerDelegate::did_receive_challenge";
        let disposition = unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                let disposition = state
                    .call(site, |handler| {
                        handler.did_receive_challenge(&download, &challenge)
                    })
                    .unwrap_or_default();
                state.sender.push(DownloadManagerEvent::ChallengeRequested {
                    download,
                    challenge,
                    disposition,
                });
                disposition
            })
        };
        disposition.unwrap_or_default() as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_failed(
    ctx: *mut c_void,
    download_json: *const c_char,
    error_json: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (ctx, download_json, error_json);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return;
        };
        let error = BackgroundAssetsError::from_json_str(string_from_ptr(error_json));
        let site = "DownloadManagerDelegate::download_failed";
        unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                state.call(site, |handler| handler.download_failed(&download, &error));
                state
                    .sender
                    .push(DownloadManagerEvent::Failed { download, error });
            })
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_download_manager_delegate_finished(
    ctx: *mut c_void,
    download_json: *const c_char,
    file_url: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (ctx, download_json, file_url);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = download_snapshot_from_json(&string_from_ptr(download_json)) else {
            return;
        };
        let file_url = string_from_ptr(file_url);
        let site = "DownloadManagerDelegate::download_finished";
        unsafe {
            DownloadManagerDelegateContext::with(ctx, site, |state| {
                state.call(site, |handler| {
                    handler.download_finished(&download, &file_url);
                });
                state
                    .sender
                    .push(DownloadManagerEvent::Finished { download, file_url });
            })
        };
    }
}

#[cfg(feature = "async")]
struct OpaquePtr(*mut c_void);
#[cfg(feature = "async")]
unsafe impl Send for OpaquePtr {}

#[cfg(feature = "async")]
unsafe extern "C" fn downloads_async_cb(
    result: *mut c_void,
    error: *const c_char,
    ctx: *mut c_void,
) {
    if !error.is_null() {
        let message = unsafe { error_from_cstr(error) };
        unsafe { AsyncCompletion::<OpaquePtr>::complete_err(ctx, message) };
    } else if !result.is_null() {
        unsafe { AsyncCompletion::complete_ok(ctx, OpaquePtr(result)) };
    } else {
        unsafe {
            AsyncCompletion::<OpaquePtr>::complete_err(
                ctx,
                "download-array pointer must not be null".into(),
            );
        };
    }
}

pub(crate) fn collect_downloads(array_ptr: *mut c_void) -> Vec<Download> {
    if array_ptr.is_null() {
        return Vec::new();
    }

    let length =
        usize::try_from(unsafe { ffi::ba_download_array_len(array_ptr) }).unwrap_or_default();
    let mut downloads = Vec::with_capacity(length);
    for index in 0..length {
        let index = isize::try_from(index).expect("download array length originates from isize");
        let item_ptr = unsafe { ffi::ba_download_array_get(array_ptr, index) };
        if let Some(download) = Download::from_raw(item_ptr) {
            downloads.push(download);
        }
    }
    unsafe { ffi::ba_object_release(array_ptr) };
    downloads
}

#[cfg(test)]
mod tests {
    use super::{download_snapshot_from_json, DownloadStatus, DownloadWriteProgress};

    #[test]
    fn download_snapshot_payload_decodes_json() {
        let snapshot = download_snapshot_from_json(
            r#"{"identifier":"download.one","uniqueIdentifier":"abc","status":2,"priority":7,"isEssential":true,"isURLDownload":true}"#,
        )
        .unwrap();

        assert_eq!(snapshot.identifier, "download.one");
        assert_eq!(snapshot.unique_identifier, "abc");
        assert_eq!(snapshot.status, DownloadStatus::Downloading);
        assert!(snapshot.is_essential);
        assert!(snapshot.is_url_download);
    }

    #[test]
    fn download_write_progress_reports_fraction() {
        let progress = DownloadWriteProgress {
            bytes_written: 128,
            total_bytes_written: 512,
            total_bytes_expected_to_write: 1024,
        };

        assert_eq!(progress.fraction_completed(), Some(0.5));
    }
}

#[cfg(all(test, feature = "async"))]
mod async_tests {
    use std::ffi::CString;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    use doom_fish_utils::stream::BoundedAsyncStream;

    use super::{
        ba_rust_download_manager_delegate_challenge_disposition,
        ba_rust_download_manager_delegate_did_begin, exclusive_control_callback,
        exclusive_control_job, DownloadManagerDelegate, DownloadManagerDelegateContext,
        DownloadManagerEvent, DownloadSnapshot,
    };
    use crate::extension::{AuthenticationChallenge, ChallengeDisposition};
    use crate::handler::HandlerState;

    const DOWNLOAD_JSON: &str = r#"{"identifier":"download.one","uniqueIdentifier":"abc","status":2,"priority":0,"isEssential":false,"isURLDownload":true}"#;
    const CHALLENGE_JSON: &str = r#"{"authenticationMethod":"NSURLAuthenticationMethodServerTrust","host":"example.com","previousFailureCount":2,"proposedCredentialHasPassword":true,"proposedCredentialUser":"user"}"#;

    #[test]
    fn exclusive_control_body_runs_before_the_handler_returns() {
        let ran = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&ran);
        let (future, job) = exclusive_control_job(move || {
            flag.store(true, Ordering::SeqCst);
            42
        });
        assert!(!ran.load(Ordering::SeqCst));

        unsafe { exclusive_control_callback(job, true, ptr::null()) };

        assert!(ran.load(Ordering::SeqCst));
        assert_eq!(pollster::block_on(future).unwrap(), 42);
    }

    #[test]
    fn exclusive_control_failure_skips_the_body_and_keeps_the_error() {
        let ran = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&ran);
        let (future, job) = exclusive_control_job(move || flag.store(true, Ordering::SeqCst));
        let error =
            CString::new(r#"{"domain":"BAErrorDomain","code":51,"message":"inactive"}"#).unwrap();

        unsafe { exclusive_control_callback(job, false, error.as_ptr()) };

        assert!(!ran.load(Ordering::SeqCst));
        let error = pollster::block_on(future).unwrap_err();
        assert_eq!(error.domain(), "BAErrorDomain");
        assert_eq!(error.code(), 51);
        assert_eq!(error.message_text(), "inactive");
    }

    #[test]
    fn exclusive_control_not_acquired_without_error_is_an_error() {
        let (future, job) = exclusive_control_job(|| 1);
        unsafe { exclusive_control_callback(job, false, ptr::null()) };
        let error = pollster::block_on(future).unwrap_err();
        assert!(error.message_text().contains("not acquired"), "{error}");
    }

    #[test]
    fn exclusive_control_body_panic_becomes_an_error() {
        let (future, job) = exclusive_control_job(|| -> u8 { panic!("body panic") });
        unsafe { exclusive_control_callback(job, true, ptr::null()) };
        let error = pollster::block_on(future).unwrap_err();
        assert!(error.message_text().contains("panicked"), "{error}");
    }

    struct Value(Arc<AtomicUsize>);

    impl Drop for Value {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn exclusive_control_completes_after_the_future_is_dropped() {
        let dropped = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&dropped);
        let (future, job) = exclusive_control_job(move || Value(counter));
        drop(future);
        unsafe { exclusive_control_callback(job, true, ptr::null()) };
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }

    struct Probe {
        began: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
        disposition: ChallengeDisposition,
        panic_on_begin: bool,
    }

    impl DownloadManagerDelegate for Probe {
        fn download_did_begin(&mut self, download: &DownloadSnapshot) {
            assert_eq!(download.identifier, "download.one");
            self.began.fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic_on_begin, "handler panic");
        }

        fn did_receive_challenge(
            &mut self,
            _download: &DownloadSnapshot,
            challenge: &AuthenticationChallenge,
        ) -> ChallengeDisposition {
            assert_eq!(challenge.host, "example.com");
            assert_eq!(
                challenge.authentication_method,
                "NSURLAuthenticationMethodServerTrust"
            );
            assert_eq!(challenge.previous_failure_count, 2);
            assert_eq!(challenge.proposed_credential_user.as_deref(), Some("user"));
            assert!(challenge.proposed_credential_has_password);
            self.disposition
        }
    }

    impl Drop for Probe {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn probe_context(
        panic_on_begin: bool,
    ) -> (
        BoundedAsyncStream<DownloadManagerEvent>,
        DownloadManagerDelegateContext,
        Arc<AtomicUsize>,
        Arc<AtomicUsize>,
    ) {
        let began = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let (stream, sender) = BoundedAsyncStream::new(8);
        let handler: Box<dyn DownloadManagerDelegate> = Box::new(Probe {
            began: Arc::clone(&began),
            drops: Arc::clone(&drops),
            disposition: ChallengeDisposition::CancelAuthenticationChallenge,
            panic_on_begin,
        });
        let context = DownloadManagerDelegateContext::new(HandlerState::new(handler, sender));
        (stream, context, began, drops)
    }

    #[test]
    fn delegate_callbacks_stop_when_the_stream_side_is_dropped() {
        let (stream, context, began, drops) = probe_context(false);
        let foreign = context.retained_ptr();
        let download = CString::new(DOWNLOAD_JSON).unwrap();

        unsafe { ba_rust_download_manager_delegate_did_begin(foreign, download.as_ptr()) };
        assert_eq!(began.load(Ordering::SeqCst), 1);
        assert!(matches!(
            stream.try_next(),
            Some(DownloadManagerEvent::Began { download }) if download.identifier == "download.one"
        ));

        drop(context);
        unsafe { ba_rust_download_manager_delegate_did_begin(foreign, download.as_ptr()) };
        assert_eq!(began.load(Ordering::SeqCst), 1);
        assert!(stream.try_next().is_none());
        assert_eq!(drops.load(Ordering::SeqCst), 0);

        unsafe { (DownloadManagerDelegateContext::RELEASE)(foreign) };
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(stream.is_closed());
    }

    #[test]
    fn delegate_handler_panic_is_contained_and_the_event_is_still_delivered() {
        let (stream, context, began, _drops) = probe_context(true);
        let download = CString::new(DOWNLOAD_JSON).unwrap();

        unsafe { ba_rust_download_manager_delegate_did_begin(context.as_ptr(), download.as_ptr()) };
        unsafe { ba_rust_download_manager_delegate_did_begin(context.as_ptr(), download.as_ptr()) };

        assert_eq!(began.load(Ordering::SeqCst), 2);
        assert!(stream.try_next().is_some());
        assert!(stream.try_next().is_some());
    }

    #[test]
    fn challenge_disposition_comes_from_the_handler_while_active() {
        let (stream, context, _began, _drops) = probe_context(false);
        let download = CString::new(DOWNLOAD_JSON).unwrap();
        let challenge = CString::new(CHALLENGE_JSON).unwrap();

        let raw = unsafe {
            ba_rust_download_manager_delegate_challenge_disposition(
                context.as_ptr(),
                download.as_ptr(),
                challenge.as_ptr(),
            )
        };
        assert_eq!(
            raw,
            ChallengeDisposition::CancelAuthenticationChallenge as i32
        );
        assert!(matches!(
            stream.try_next(),
            Some(DownloadManagerEvent::ChallengeRequested { disposition, .. })
                if disposition == ChallengeDisposition::CancelAuthenticationChallenge
        ));

        context.deactivate();
        let raw = unsafe {
            ba_rust_download_manager_delegate_challenge_disposition(
                context.as_ptr(),
                download.as_ptr(),
                challenge.as_ptr(),
            )
        };
        assert_eq!(raw, ChallengeDisposition::PerformDefaultHandling as i32);
    }
}
