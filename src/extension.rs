use core::ffi::{c_char, c_void};
#[cfg(feature = "async")]
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::ptr;

#[cfg(feature = "async")]
use doom_fish_utils::stream::BoundedAsyncStream;
use serde::{Deserialize, Serialize};

use crate::asset_pack::AssetPackSnapshot;
use crate::download::{ContentRequest, Download, DownloadSnapshot};
use crate::error::BackgroundAssetsError;
use crate::ffi;
#[cfg(feature = "async")]
use crate::handler::{HandlerState, Registry};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppExtensionInfoSnapshot {
    #[serde(alias = "restrictedDownloadSizeRemaining")]
    pub restricted_download_size_remaining: Option<i64>,
    #[serde(alias = "restrictedEssentialDownloadSizeRemaining")]
    pub restricted_essential_download_size_remaining: Option<i64>,
}

pub struct AppExtensionInfo {
    ptr: *mut c_void,
}

impl AppExtensionInfo {
    #[cfg(feature = "async")]
    pub(crate) fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    #[cfg(feature = "async")]
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
    #[serde(alias = "authenticationMethod")]
    pub authentication_method: String,
    #[serde(alias = "previousFailureCount")]
    pub previous_failure_count: i64,
    #[serde(alias = "proposedCredentialUser")]
    pub proposed_credential_user: Option<String>,
    #[serde(alias = "proposedCredentialHasPassword")]
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

pub trait ManagedDownloaderExtensionHandler: Send + 'static {
    fn should_download_asset_pack(&mut self, _asset_pack: &AssetPackSnapshot) -> bool {
        true
    }

    fn did_receive_challenge(
        &mut self,
        _download: &Download,
        _challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        ChallengeDisposition::PerformDefaultHandling
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtensionEvent {
    ShouldDownloadAssetPack {
        asset_pack: AssetPackSnapshot,
        should_download: bool,
    },
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
    registration_id: u64,
    unregister: fn(u64),
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
impl Drop for ExtensionEventStream {
    fn drop(&mut self) {
        (self.unregister)(self.registration_id);
    }
}

#[cfg(feature = "async")]
static EXTENSION: Registry<dyn DownloaderExtensionHandler, ExtensionEvent> = Registry::new();

#[cfg(feature = "async")]
static MANAGED_EXTENSION: Registry<dyn ManagedDownloaderExtensionHandler, ExtensionEvent> =
    Registry::new();

#[cfg(feature = "async")]
#[must_use = "dropping the stream unregisters the handler"]
pub fn install_global_downloader_extension<H>(handler: H, capacity: usize) -> ExtensionEventStream
where
    H: DownloaderExtensionHandler,
{
    let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
    let handler: Box<dyn DownloaderExtensionHandler> = Box::new(handler);
    let registration_id = EXTENSION.install(HandlerState::new(handler, sender));
    ExtensionEventStream {
        inner: stream,
        registration_id,
        unregister: |id| EXTENSION.remove(id),
    }
}

#[cfg(feature = "async")]
#[must_use = "dropping the stream unregisters the handler"]
pub fn install_global_managed_downloader_extension<H>(
    handler: H,
    capacity: usize,
) -> ExtensionEventStream
where
    H: ManagedDownloaderExtensionHandler,
{
    let (stream, sender) = BoundedAsyncStream::new(capacity.max(1));
    let handler: Box<dyn ManagedDownloaderExtensionHandler> = Box::new(handler);
    let registration_id = MANAGED_EXTENSION.install(HandlerState::new(handler, sender));
    ExtensionEventStream {
        inner: stream,
        registration_id,
        unregister: |id| MANAGED_EXTENSION.remove(id),
    }
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

#[cfg_attr(not(feature = "async"), allow(dead_code))]
fn json_cstring<T: Serialize>(value: &T) -> *mut c_char {
    serde_json::to_string(value)
        .ok()
        .and_then(|json| CString::new(json).ok())
        .map_or(ptr::null_mut(), CString::into_raw)
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_string_free(string: *mut c_char) {
    if !string.is_null() {
        drop(unsafe { CString::from_raw(string) });
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

#[cfg_attr(not(feature = "async"), allow(dead_code))]
fn asset_pack_snapshot_from_json(json: &str) -> Option<AssetPackSnapshot> {
    serde_json::from_str::<AssetPackSnapshotPayload>(json)
        .ok()
        .map(Into::into)
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_extension_should_download_asset_pack(
    asset_pack_json: *const c_char,
) -> bool {
    #[cfg(not(feature = "async"))]
    {
        let _ = asset_pack_json;
        true
    }

    #[cfg(feature = "async")]
    {
        let Some(asset_pack) = asset_pack_snapshot_from_json(&string_from_ptr(asset_pack_json))
        else {
            return true;
        };
        let Some(state) = MANAGED_EXTENSION.current() else {
            return true;
        };
        let should_download = state
            .call(
                "ManagedDownloaderExtensionHandler::should_download_asset_pack",
                |handler| handler.should_download_asset_pack(&asset_pack),
            )
            .unwrap_or(true);
        state.sender.push(ExtensionEvent::ShouldDownloadAssetPack {
            asset_pack,
            should_download,
        });
        should_download
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_managed_extension_challenge_disposition(
    download: *mut c_void,
    challenge_json: *const c_char,
) -> i32 {
    #[cfg(not(feature = "async"))]
    {
        let _ = (download, challenge_json);
        ChallengeDisposition::PerformDefaultHandling as i32
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let challenge =
            serde_json::from_str::<AuthenticationChallenge>(&string_from_ptr(challenge_json))
                .unwrap_or_default();
        let Some(state) = MANAGED_EXTENSION.current() else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let disposition = state
            .call(
                "ManagedDownloaderExtensionHandler::did_receive_challenge",
                |handler| handler.did_receive_challenge(&download, &challenge),
            )
            .unwrap_or_default();
        state.sender.push(ExtensionEvent::ChallengeRequested {
            download: download.snapshot(),
            challenge,
            disposition,
        });
        disposition as i32
    }
}

#[cfg(feature = "async")]
fn check_download_plan(
    request: ContentRequest,
    downloads: &[Download],
) -> Result<(), BackgroundAssetsError> {
    if !request.allows_essential_downloads() && downloads.iter().any(Download::is_essential) {
        return Err(BackgroundAssetsError::invalid_argument(
            "essential downloads are only allowed for install and update content requests",
        ));
    }
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_downloads_for_request(
    request: isize,
    manifest_url: *const c_char,
    extension_info: *mut c_void,
) -> *mut c_char {
    #[cfg(not(feature = "async"))]
    {
        let _ = (request, manifest_url, extension_info);
        ptr::null_mut()
    }

    #[cfg(feature = "async")]
    {
        let request = ContentRequest::from_raw(request);
        let manifest_url = string_from_ptr(manifest_url);
        let Some(extension_info) =
            (unsafe { AppExtensionInfo::retained_from_borrowed(extension_info) })
        else {
            return ptr::null_mut();
        };
        let Some(state) = EXTENSION.current() else {
            return ptr::null_mut();
        };

        let planned = state
            .call("DownloaderExtensionHandler::downloads", |handler| {
                handler.downloads(request, &manifest_url, &extension_info)
            })
            .unwrap_or_else(|| {
                Err(BackgroundAssetsError::message(
                    "download plan handler panicked across the FFI boundary",
                ))
            })
            .and_then(|downloads| check_download_plan(request, &downloads).map(|()| downloads));

        match planned {
            Ok(downloads) => {
                state.sender.push(ExtensionEvent::DownloadsRequested {
                    request,
                    manifest_url,
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
                ptr::null_mut()
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
        let _ = (download, challenge_json);
        ChallengeDisposition::PerformDefaultHandling as i32
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let challenge =
            serde_json::from_str::<AuthenticationChallenge>(&string_from_ptr(challenge_json))
                .unwrap_or_default();
        let Some(state) = EXTENSION.current() else {
            return ChallengeDisposition::PerformDefaultHandling as i32;
        };
        let disposition = state
            .call(
                "DownloaderExtensionHandler::did_receive_challenge",
                |handler| handler.did_receive_challenge(&download, &challenge),
            )
            .unwrap_or_default();
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
    #[cfg(not(feature = "async"))]
    {
        let _ = (download, error_json);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return;
        };
        let error = BackgroundAssetsError::from_json_str(string_from_ptr(error_json));
        let Some(state) = EXTENSION.current() else {
            return;
        };
        state.call("DownloaderExtensionHandler::download_failed", |handler| {
            handler.download_failed(&download, &error);
        });
        state.sender.push(ExtensionEvent::DownloadFailed {
            download: download.snapshot(),
            error,
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn ba_rust_extension_download_finished(
    download: *mut c_void,
    file_url: *const c_char,
) {
    #[cfg(not(feature = "async"))]
    {
        let _ = (download, file_url);
    }

    #[cfg(feature = "async")]
    {
        let Some(download) = (unsafe { Download::retained_from_borrowed(download) }) else {
            return;
        };
        let file_url = string_from_ptr(file_url);
        let Some(state) = EXTENSION.current() else {
            return;
        };
        state.call("DownloaderExtensionHandler::download_finished", |handler| {
            handler.download_finished(&download, &file_url);
        });
        state.sender.push(ExtensionEvent::DownloadFinished {
            download: download.snapshot(),
            file_url,
        });
    }
}

#[no_mangle]
pub extern "C" fn ba_rust_extension_will_terminate() {
    #[cfg(feature = "async")]
    {
        if let Some(state) = EXTENSION.current() {
            state.call(
                "DownloaderExtensionHandler::extension_will_terminate",
                |handler| {
                    handler.extension_will_terminate();
                },
            );
            state.sender.push(ExtensionEvent::Terminating);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::{
        ba_rust_string_free, json_cstring, AppExtensionInfoSnapshot, AuthenticationChallenge,
    };

    #[test]
    fn rust_allocated_json_is_freed_by_the_rust_export() {
        let json = json_cstring(&vec![1_u64, 2, u64::MAX]);
        assert!(!json.is_null());
        let text = unsafe { CStr::from_ptr(json) }.to_str().unwrap().to_owned();
        assert_eq!(text, "[1,2,18446744073709551615]");
        unsafe { ba_rust_string_free(json) };
        unsafe { ba_rust_string_free(std::ptr::null_mut()) };
    }

    #[test]
    fn bridge_payloads_decode_the_keys_swift_encodes() {
        let challenge: AuthenticationChallenge = serde_json::from_str(
            r#"{"authenticationMethod":"NSURLAuthenticationMethodHTTPBasic","host":"cdn.example.com","previousFailureCount":1,"proposedCredentialHasPassword":false}"#,
        )
        .unwrap();
        assert_eq!(challenge.host, "cdn.example.com");
        assert_eq!(
            challenge.authentication_method,
            "NSURLAuthenticationMethodHTTPBasic"
        );
        assert_eq!(challenge.previous_failure_count, 1);
        assert_eq!(challenge.proposed_credential_user, None);

        let info: AppExtensionInfoSnapshot = serde_json::from_str(
            r#"{"restrictedDownloadSizeRemaining":4096,"restrictedEssentialDownloadSizeRemaining":1024}"#,
        )
        .unwrap();
        assert_eq!(info.restricted_download_size_remaining, Some(4096));
        assert_eq!(
            info.restricted_essential_download_size_remaining,
            Some(1024)
        );

        let serialized = serde_json::to_value(&challenge).unwrap();
        assert_eq!(
            serialized["authentication_method"],
            "NSURLAuthenticationMethodHTTPBasic"
        );
    }
}

#[cfg(all(test, feature = "async"))]
mod async_tests {
    use std::ffi::CString;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::{
        ba_rust_extension_will_terminate, ba_rust_managed_extension_should_download_asset_pack,
        install_global_downloader_extension, install_global_managed_downloader_extension,
        AppExtensionInfo, ExtensionEvent, ManagedDownloaderExtensionHandler,
    };
    use crate::asset_pack::AssetPackSnapshot;
    use crate::download::{ContentRequest, Download};
    use crate::error::BackgroundAssetsError;
    use crate::DownloaderExtensionHandler;

    #[test]
    fn plans_without_essential_downloads_pass_for_every_request() {
        for request in [
            ContentRequest::Install,
            ContentRequest::Update,
            ContentRequest::Periodic,
            ContentRequest::Unknown(9),
        ] {
            assert!(super::check_download_plan(request, &[]).is_ok());
        }
    }

    struct Filter {
        calls: Arc<AtomicUsize>,
    }

    impl ManagedDownloaderExtensionHandler for Filter {
        fn should_download_asset_pack(&mut self, asset_pack: &AssetPackSnapshot) -> bool {
            self.calls.fetch_add(1, Ordering::SeqCst);
            asset_pack.id != "pack.skip"
        }
    }

    #[test]
    fn managed_extension_handler_is_unregistered_when_its_stream_drops() {
        let calls = Arc::new(AtomicUsize::new(0));
        let events = install_global_managed_downloader_extension(
            Filter {
                calls: Arc::clone(&calls),
            },
            4,
        );
        let skip =
            CString::new(r#"{"description":"Skip","downloadSize":1,"id":"pack.skip","version":1}"#)
                .unwrap();

        assert!(!unsafe { ba_rust_managed_extension_should_download_asset_pack(skip.as_ptr()) });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            events.try_next(),
            Some(ExtensionEvent::ShouldDownloadAssetPack { asset_pack, should_download: false })
                if asset_pack.id == "pack.skip"
        ));

        drop(events);
        assert!(unsafe { ba_rust_managed_extension_should_download_asset_pack(skip.as_ptr()) });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    struct Terminator {
        terminations: Arc<AtomicUsize>,
    }

    impl DownloaderExtensionHandler for Terminator {
        fn downloads(
            &mut self,
            _request: ContentRequest,
            _manifest_url: &str,
            _extension_info: &AppExtensionInfo,
        ) -> Result<Vec<Download>, BackgroundAssetsError> {
            Ok(Vec::new())
        }

        fn extension_will_terminate(&mut self) {
            self.terminations.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn replacing_the_extension_handler_closes_the_previous_stream() {
        let first_count = Arc::new(AtomicUsize::new(0));
        let second_count = Arc::new(AtomicUsize::new(0));
        let first = install_global_downloader_extension(
            Terminator {
                terminations: Arc::clone(&first_count),
            },
            4,
        );
        let second = install_global_downloader_extension(
            Terminator {
                terminations: Arc::clone(&second_count),
            },
            4,
        );
        assert!(first.is_closed());

        ba_rust_extension_will_terminate();
        assert_eq!(first_count.load(Ordering::SeqCst), 0);
        assert_eq!(second_count.load(Ordering::SeqCst), 1);
        assert!(matches!(
            second.try_next(),
            Some(ExtensionEvent::Terminating)
        ));

        drop(first);
        ba_rust_extension_will_terminate();
        assert_eq!(second_count.load(Ordering::SeqCst), 2);

        drop(second);
        ba_rust_extension_will_terminate();
        assert_eq!(second_count.load(Ordering::SeqCst), 2);
    }
}
