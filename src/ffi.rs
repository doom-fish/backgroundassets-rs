use core::ffi::{c_char, c_void};
use std::ffi::{CStr, CString};

use crate::error::BackgroundAssetsError;

#[cfg(feature = "async")]
pub type AsyncCallback =
    unsafe extern "C" fn(result: *mut c_void, error: *const c_char, ctx: *mut c_void);

#[cfg(feature = "async")]
pub type StreamCallback =
    unsafe extern "C" fn(ctx: *mut c_void, event_json: *mut c_char, done: bool);

extern "C" {
    pub fn ba_string_free(string: *mut c_char);
    pub fn ba_bytes_free(bytes: *mut c_void);
    pub fn ba_object_release(ptr: *mut c_void);
    pub fn ba_object_retain(ptr: *mut c_void) -> *mut c_void;

    pub fn ba_downloader_priority_min() -> isize;
    pub fn ba_downloader_priority_default() -> isize;
    pub fn ba_downloader_priority_max() -> isize;

    pub fn ba_asset_pack_identifier(ptr: *mut c_void) -> *mut c_char;
    pub fn ba_asset_pack_download_size(ptr: *mut c_void) -> isize;
    pub fn ba_asset_pack_version(ptr: *mut c_void) -> isize;
    pub fn ba_asset_pack_description(ptr: *mut c_void) -> *mut c_char;
    pub fn ba_asset_pack_user_info_copy(ptr: *mut c_void, length_out: *mut isize) -> *mut c_void;
    pub fn ba_asset_pack_download(ptr: *mut c_void) -> *mut c_void;
    pub fn ba_asset_pack_download_for_request(ptr: *mut c_void, request: isize) -> *mut c_void;
    pub fn ba_asset_pack_array_len(ptr: *mut c_void) -> isize;
    pub fn ba_asset_pack_array_get(ptr: *mut c_void, index: isize) -> *mut c_void;

    pub fn ba_manifest_create_from_url(
        raw_url: *const c_char,
        app_group_id: *const c_char,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ba_manifest_create_from_data(
        bytes: *const u8,
        length: isize,
        app_group_id: *const c_char,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ba_manifest_description(ptr: *mut c_void) -> *mut c_char;
    pub fn ba_manifest_asset_packs(ptr: *mut c_void) -> *mut c_void;
    pub fn ba_manifest_all_downloads(ptr: *mut c_void) -> *mut c_void;
    pub fn ba_manifest_all_downloads_for_request(ptr: *mut c_void, request: isize) -> *mut c_void;

    pub fn ba_download_identifier(ptr: *mut c_void) -> *mut c_char;
    pub fn ba_download_unique_identifier(ptr: *mut c_void) -> *mut c_char;
    pub fn ba_download_status(ptr: *mut c_void) -> isize;
    pub fn ba_download_priority(ptr: *mut c_void) -> isize;
    pub fn ba_download_is_essential(ptr: *mut c_void) -> bool;
    pub fn ba_download_removing_essential(ptr: *mut c_void) -> *mut c_void;
    pub fn ba_download_is_url_download(ptr: *mut c_void) -> bool;
    pub fn ba_download_array_len(ptr: *mut c_void) -> isize;
    pub fn ba_download_array_get(ptr: *mut c_void, index: isize) -> *mut c_void;
    pub fn ba_url_download_create(
        identifier: *const c_char,
        raw_url: *const c_char,
        method: *const c_char,
        headers_json: *const c_char,
        file_size: u64,
        app_group_id: *const c_char,
        essential: bool,
        priority: isize,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;

    pub fn ba_download_manager_shared() -> *mut c_void;
    pub fn ba_download_manager_schedule_download(
        manager_ptr: *mut c_void,
        download_ptr: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> bool;
    pub fn ba_download_manager_start_foreground_download(
        manager_ptr: *mut c_void,
        download_ptr: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> bool;
    pub fn ba_download_manager_cancel_download(
        manager_ptr: *mut c_void,
        download_ptr: *mut c_void,
        error_out: *mut *mut c_char,
    ) -> bool;
    #[cfg(feature = "async")]
    pub fn ba_download_manager_fetch_current_downloads_async(
        manager_ptr: *mut c_void,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_download_manager_with_exclusive_control_async(
        manager_ptr: *mut c_void,
        before_epoch_seconds: f64,
        has_before_date: bool,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );

    pub fn ba_app_extension_info_snapshot_json(ptr: *mut c_void) -> *mut c_char;

    pub fn ba_asset_pack_manager_shared() -> *mut c_void;
    pub fn ba_asset_pack_manager_asset_pack_is_available_locally(
        ptr: *mut c_void,
        asset_pack_id: *const c_char,
    ) -> bool;
    pub fn ba_asset_pack_manager_contents(
        ptr: *mut c_void,
        raw_path: *const c_char,
        asset_pack_id: *const c_char,
        length_out: *mut isize,
        error_out: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn ba_asset_pack_manager_descriptor(
        ptr: *mut c_void,
        raw_path: *const c_char,
        asset_pack_id: *const c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    pub fn ba_asset_pack_manager_url(
        ptr: *mut c_void,
        raw_path: *const c_char,
        error_out: *mut *mut c_char,
    ) -> *mut c_char;
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_all_asset_packs_async(
        ptr: *mut c_void,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_asset_pack_async(
        ptr: *mut c_void,
        asset_pack_id: *const c_char,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_status_relative_async(
        ptr: *mut c_void,
        asset_pack_ptr: *mut c_void,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_local_status_async(
        ptr: *mut c_void,
        asset_pack_id: *const c_char,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_ensure_local_availability_async(
        ptr: *mut c_void,
        asset_pack_ptr: *mut c_void,
        require_latest_version: bool,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_check_for_updates_async(
        ptr: *mut c_void,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_remove_async(
        ptr: *mut c_void,
        asset_pack_id: *const c_char,
        ctx: *mut c_void,
        cb: AsyncCallback,
    );
    #[cfg(feature = "async")]
    pub fn ba_asset_pack_manager_status_updates_stream_create(
        ptr: *mut c_void,
        asset_pack_id: *const c_char,
        ctx: *mut c_void,
        cb: StreamCallback,
    ) -> *mut c_void;
}

pub unsafe fn owned_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let value = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ba_string_free(ptr);
    value
}

pub unsafe fn owned_bytes(ptr: *mut c_void, length: isize) -> Vec<u8> {
    if ptr.is_null() {
        return Vec::new();
    }
    let length = usize::try_from(length).unwrap_or_default();
    let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), length).to_vec();
    ba_bytes_free(ptr);
    bytes
}

pub fn retained(ptr: *mut c_void) -> *mut c_void {
    unsafe { ba_object_retain(ptr) }
}

pub fn required_cstring(
    value: &str,
    field: &'static str,
) -> Result<CString, BackgroundAssetsError> {
    CString::new(value).map_err(|error| {
        BackgroundAssetsError::invalid_argument(format!(
            "{field} contains an interior NUL byte: {error}"
        ))
    })
}
