use core::ffi::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr;
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "async")]
use doom_fish_utils::stream::{AsyncStreamSender, BoundedAsyncStream};
use serde::{Deserialize, Serialize};

use crate::download::{ContentRequest, Download, DownloadSnapshot};
use crate::error::BackgroundAssetsError;
use crate::ffi;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppExtensionInfoSnapshot {
    pub restricted_download_size_remaining: Option<i64>,
    pub restricted_essential_download_size_remaining: Option<i64>,
}

pub struct AppExtensionInfo {
    ptr: *mut c_void,
}

impl AppExtensionInfo {
    pub(crate) fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    pub(crate) unsafe fn retained_from_borrowed(ptr: *mut c_void) -> Option<Self> {
        Self::from_raw(ffi::retained(ptr))
    }

    pub fn snapshot(&self) -> AppExtensionInfoSnapshot {
        let json = unsafe { ffi::owned_string(ffi::ba_app_extension_info_snapshot_json(self.ptr)) };
        serde_json::from_str(&json).unwrap_or_default()
    }

    pub fn restricted_download_size_remaining(&self) -> Option<i64> {
        self.snapshot().restricted_download_size_remaining
    }

    pub fn restricted_essential_download_size_remaining(&self) -> Option<i64> {
        self.snapshot().restricted_essential_download_size_remaining
    }
}

impl Clone for AppExtensionInfo {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for AppExtensionInfo {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for AppExtensionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppExtensionInfo")
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

// SAFETY: `BAAppExtensionInfo` is documented by the SDK as Swift-sendable, and Rust
// only keeps a retained opaque object pointer.
unsafe impl Send for AppExtensionInfo {}
// SAFETY: See `Send` justification above.
unsafe impl Sync for AppExtensionInfo {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[repr(i32)]
pub enum ChallengeDisposition {
    UseCredential = 0,
    #[default]
    PerformDefaultHandling = 1,
    CancelAuthenticationChallenge = 2,
    RejectProtectionSpace = 3,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub host: String,
    pub authentication_method: String,
    pub previous_failure_count: i64,
    pub proposed_credential_user: Option<String>,
    pub proposed_credential_has_password: bool,
}

pub trait DownloaderExtensionHandler: Send + 'static {
    fn downloads(
        &mut self,
        request: ContentRequest,
        manifest_url: &str,
        extension_info: &AppExtensionInfo,
    ) -> Result<Vec<Download>, BackgroundAssetsError>;

    fn did_receive_challenge(
        &mut self,
        _download: &Download,
        _challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        ChallengeDisposition::PerformDefaultHandling
    }

    fn download_failed(&mut self, _download: &Download, _error: &BackgroundAssetsError) {}

    fn download_finished(&mut self, _download: &Download, _file_url: &str) {}

    fn extension_will_terminate(&mut self) {}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtensionEvent {
    DownloadsRequested {
        request: ContentRequest,
        manifest_url: String,
        extension_info: AppExtensionInfoSnapshot,
        planned_downloads: Vec<DownloadSnapshot>,
    },
    DownloadPlanFailed {
        request: ContentRequest,
        manifest_url: String,
        error: BackgroundAssetsError,
    },
    ChallengeRequested {
        download: DownloadSnapshot,
        challenge: AuthenticationChallenge,
        disposition: ChallengeDisposition,
    },
    DownloadFailed {
        download: DownloadSnapshot,
        error: BackgroundAssetsError,
    },
    DownloadFinished {
        download: DownloadSnapshot,
        file_url: String,
    },
    Terminating,
}

#[cfg(feature = "async")]
pub struct ExtensionEventStream {
    inner: BoundedAsyncStream<ExtensionEvent>,
}

#[cfg(feature = "async")]
impl ExtensionEventStream {
    pub fn next(&self) -> impl std::future::Future<Output = Option<ExtensionEvent>> + '_ {
        self.inner.next()
    }

    pub fn try_next(&self) -> Option<ExtensionEvent> {
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
impl fmt::Debug for ExtensionEventStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionEventStream")
            .field("buffered_count", &self.buffered_count())
            .field("is_closed", &self.is_closed())
            .finish()
    }
}

#[cfg(feature = "async")]
struct ExtensionState {
    handler: Box<dyn DownloaderExtensionHandler>,
    sender: AsyncStreamSender<ExtensionEvent>,
}

#[cfg(feature = "async")]
fn extension_state() -> &'static Mutex<Option<ExtensionState>> {
    static STATE: OnceLock<Mutex<Option<ExtensionState>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

#[cfg(feature = "async")]
pub fn install_global_downloader_extension<H>(handler: H, capacity: usize) -> ExtensionEventStream
where
    H: DownloaderExtensionHandler,
{
    let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
    if let Ok(mut state) = extension_state().lock() {
        *state = Some(ExtensionState {
            handler: Box::new(handler),
            sender,
        });
    }
    ExtensionEventStream { inner: stream }
}

fn string_from_ptr(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

fn json_cstring<T: Serialize>(value: &T) -> *mut c_char {
    serde_json::to_string(value)
        .ok()
        .and_then(|json| CString::new(json).ok())
        .map_or(ptr::null_mut(), CString::into_raw)
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_downloads_for_request(
    request: i32,
    manifest_url: *const c_char,
    extension_info: *mut c_void,
) -> *mut c_char {
    #[cfg(not(feature = "async"))]
    {
        let _ = request;
        let _ = manifest_url;
        let _ = extension_info;
        return json_cstring(&Vec::<u64>::new());
    }

    #[cfg(feature = "async")]
    {
        let request = ContentRequest::from_raw(request as isize).unwrap_or(ContentRequest::Install);
        let manifest_url = string_from_ptr(manifest_url);
        let Some(extension_info) =
            (unsafe { AppExtensionInfo::retained_from_borrowed(extension_info) })
        else {
            return json_cstring(&Vec::<u64>::new());
        };

        let Ok(mut state_guard) = extension_state().lock() else {
            return json_cstring(&Vec::<u64>::new());
        };
        let Some(state) = state_guard.as_mut() else {
            return json_cstring(&Vec::<u64>::new());
        };

        match state
            .handler
            .downloads(request, &manifest_url, &extension_info)
        {
            Ok(downloads) => {
                state.sender.push(ExtensionEvent::DownloadsRequested {
                    request,
                    manifest_url: manifest_url.clone(),
                    extension_info: extension_info.snapshot(),
                    planned_downloads: downloads.iter().map(Download::snapshot).collect(),
                });
                let retained_ptrs: Vec<u64> = downloads
                    .iter()
                    .filter_map(|download| {
                        let ptr = ffi::retained(download.raw_ptr());
                        (!ptr.is_null()).then_some(ptr as usize as u64)
                    })
                    .collect();
                json_cstring(&retained_ptrs)
            }
            Err(error) => {
                state.sender.push(ExtensionEvent::DownloadPlanFailed {
                    request,
                    manifest_url,
                    error,
                });
                json_cstring(&Vec::<u64>::new())
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_challenge_disposition(
    download: *mut c_void,
    challenge_json: *const c_char,
) -> i32 {
    #[cfg(not(feature = "async"))]
    {
        let _ = download;
        let _ = challenge_json;
        return ChallengeDisposition::PerformDefaultHandling as i32;
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let challenge =
            serde_json::from_str::<AuthenticationChallenge>(&string_from_ptr(challenge_json))
                .unwrap_or_default();
        let Ok(mut state_guard) = extension_state().lock() else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let Some(state) = state_guard.as_mut() else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let disposition = state.handler.did_receive_challenge(&download, &challenge);
        state.sender.push(ExtensionEvent::ChallengeRequested {
            download: download.snapshot(),
            challenge,
            disposition,
        });
        disposition as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_download_failed(
    download: *mut c_void,
    error_json: *const c_char,
) {
    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return;
        };
        let error = BackgroundAssetsError::from_json_str(&string_from_ptr(error_json));
        if let Ok(mut state_guard) = extension_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.download_failed(&download, &error);
                state.sender.push(ExtensionEvent::DownloadFailed {
                    download: download.snapshot(),
                    error,
                });
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_download_finished(
    download: *mut c_void,
    file_url: *const c_char,
) {
    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return;
        };
        let file_url = string_from_ptr(file_url);
        if let Ok(mut state_guard) = extension_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.download_finished(&download, &file_url);
                state.sender.push(ExtensionEvent::DownloadFinished {
                    download: download.snapshot(),
                    file_url,
                });
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn ba_rust_extension_will_terminate() {
    #[cfg(feature = "async")]
    {
        if let Ok(mut state_guard) = extension_state().lock() {
            if let Some(state) = state_guard.as_mut() {
                state.handler.extension_will_terminate();
                state.sender.push(ExtensionEvent::Terminating);
            }
        }
    }
}
