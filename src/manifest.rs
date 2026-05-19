use core::ffi::{c_char, c_void};
use std::fmt;
use std::ptr;

use crate::asset_pack::{collect_asset_packs, AssetPack};
use crate::download::{collect_downloads, ContentRequest, Download};
use crate::error::BackgroundAssetsError;
use crate::ffi;

pub struct Manifest {
    ptr: *mut c_void,
}

impl Manifest {
    pub fn from_url(url: &str, app_group_id: &str) -> Result<Self, BackgroundAssetsError> {
        let url = ffi::required_cstring(url, "url")?;
        let app_group_id = ffi::required_cstring(app_group_id, "app_group_id")?;
        let mut error: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::ba_manifest_create_from_url(url.as_ptr(), app_group_id.as_ptr(), &mut error)
        };
        if ptr.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(Self { ptr })
    }

    pub fn from_bytes(bytes: &[u8], app_group_id: &str) -> Result<Self, BackgroundAssetsError> {
        let app_group_id = ffi::required_cstring(app_group_id, "app_group_id")?;
        let length = isize::try_from(bytes.len()).map_err(|_| {
            BackgroundAssetsError::invalid_argument("manifest data is too large to bridge")
        })?;
        let mut error: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::ba_manifest_create_from_data(
                bytes.as_ptr(),
                length,
                app_group_id.as_ptr(),
                &mut error,
            )
        };
        if ptr.is_null() {
            return Err(BackgroundAssetsError::from_owned_json_ptr(error));
        }
        Ok(Self { ptr })
    }

    pub fn description(&self) -> String {
        unsafe { ffi::owned_string(ffi::ba_manifest_description(self.ptr)) }
    }

    pub fn asset_packs(&self) -> Vec<AssetPack> {
        collect_asset_packs(unsafe { ffi::ba_manifest_asset_packs(self.ptr) })
    }

    pub fn all_downloads(&self) -> Vec<Download> {
        collect_downloads(unsafe { ffi::ba_manifest_all_downloads(self.ptr) })
    }

    pub fn all_downloads_for_request(&self, request: ContentRequest) -> Vec<Download> {
        collect_downloads(unsafe {
            ffi::ba_manifest_all_downloads_for_request(self.ptr, request.as_raw())
        })
    }
}

impl Clone for Manifest {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for Manifest {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manifest")
            .field("description", &self.description())
            .field("asset_packs", &self.asset_packs())
            .finish()
    }
}

// SAFETY: `Manifest` wraps a retained Swift `ManifestBox` around the sendable
// `AssetPackManifest` value; Rust never dereferences the pointer directly.
unsafe impl Send for Manifest {}
// SAFETY: See `Send` justification above.
unsafe impl Sync for Manifest {}
