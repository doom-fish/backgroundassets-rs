use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::ptr;

#[cfg(feature = "async")]
use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion};
#[cfg(feature = "async")]
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Serialize;

use crate::error::BackgroundAssetsError;
use crate::ffi;

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
        let identifier = ffi::required_cstring(identifier, "identifier")?;
        let url = ffi::required_cstring(url, "url")?;
        let method = ffi::required_cstring(&options.method, "method")?;
        let headers_json = serde_json::to_string(&options.headers)
            .map_err(|error| BackgroundAssetsError::invalid_argument(error.to_string()))?;
        let headers_json = ffi::required_cstring(&headers_json, "headers")?;
        let app_group_id = ffi::required_cstring(app_group_id, "app_group_id")?;
        let mut error: *mut c_char = ptr::null_mut();

        let priority = isize::try_from(options.priority.raw_value()).map_err(|_| {
            BackgroundAssetsError::invalid_argument("download priority is out of range")
        })?;
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
                &mut error,
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

pub struct DownloadManager {
    ptr: *mut c_void,
}

impl DownloadManager {
    pub fn shared() -> Option<Self> {
        Self::from_raw(unsafe { ffi::ba_download_manager_shared() })
    }

    fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    pub fn schedule_download(&self, download: &Download) -> Result<(), BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let scheduled = unsafe {
            ffi::ba_download_manager_schedule_download(self.ptr, download.ptr, &mut error)
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
            ffi::ba_download_manager_start_foreground_download(self.ptr, download.ptr, &mut error)
        };
        if started {
            Ok(())
        } else {
            Err(BackgroundAssetsError::from_owned_json_ptr(error))
        }
    }

    pub fn cancel_download(&self, download: &Download) -> Result<(), BackgroundAssetsError> {
        let mut error: *mut c_char = ptr::null_mut();
        let cancelled =
            unsafe { ffi::ba_download_manager_cancel_download(self.ptr, download.ptr, &mut error) };
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
        let OpaquePtr(ptr) = future.await.map_err(BackgroundAssetsError::message)?;
        Ok(collect_downloads(ptr))
    }

    #[cfg(feature = "async")]
    pub async fn with_exclusive_control(
        &self,
        before: Option<SystemTime>,
    ) -> Result<bool, BackgroundAssetsError> {
        let (future, ctx) = AsyncCompletion::<String>::create();
        let (seconds, has_before) = before.map_or((0.0, false), |timestamp| {
            let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
            (duration.as_secs_f64(), true)
        });
        unsafe {
            ffi::ba_download_manager_with_exclusive_control_async(
                self.ptr,
                seconds,
                has_before,
                ctx,
                string_async_cb,
            );
        }
        let value = future.await.map_err(BackgroundAssetsError::message)?;
        Ok(matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "ok"
        ))
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
