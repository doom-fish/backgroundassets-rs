use core::ffi::c_void;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use serde::Serialize;

use crate::download::{ContentRequest, Download};
use crate::ffi;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AssetPackSnapshot {
    pub id: String,
    pub download_size: i64,
    pub version: i64,
    pub description: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct AssetPackStatus(u64);

impl AssetPackStatus {
    pub const DOWNLOAD_AVAILABLE: Self = Self(1 << 0);
    pub const UPDATE_AVAILABLE: Self = Self(1 << 1);
    pub const UP_TO_DATE: Self = Self(1 << 2);
    pub const OUT_OF_DATE: Self = Self(1 << 3);
    pub const OBSOLETE: Self = Self(1 << 4);
    pub const DOWNLOADING: Self = Self(1 << 5);
    pub const DOWNLOADED: Self = Self(1 << 6);

    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for AssetPackStatus {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for AssetPackStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for AssetPackStatus {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for AssetPackStatus {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

pub struct AssetPack {
    pub(crate) ptr: *mut c_void,
}

impl AssetPack {
    pub(crate) fn from_raw(ptr: *mut c_void) -> Option<Self> {
        (!ptr.is_null()).then_some(Self { ptr })
    }

    pub(crate) const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn id(&self) -> String {
        unsafe { ffi::owned_string(ffi::ba_asset_pack_identifier(self.ptr)) }
    }

    pub fn download_size(&self) -> i64 {
        unsafe { ffi::ba_asset_pack_download_size(self.ptr) as i64 }
    }

    pub fn version(&self) -> i64 {
        unsafe { ffi::ba_asset_pack_version(self.ptr) as i64 }
    }

    pub fn description(&self) -> String {
        unsafe { ffi::owned_string(ffi::ba_asset_pack_description(self.ptr)) }
    }

    pub fn user_info(&self) -> Option<Vec<u8>> {
        let mut length = 0isize;
        let ptr = unsafe { ffi::ba_asset_pack_user_info_copy(self.ptr, &mut length) };
        (!ptr.is_null()).then(|| unsafe { ffi::owned_bytes(ptr, length) })
    }

    pub fn snapshot(&self) -> AssetPackSnapshot {
        AssetPackSnapshot {
            id: self.id(),
            download_size: self.download_size(),
            version: self.version(),
            description: self.description(),
        }
    }

    pub fn download(&self) -> Option<Download> {
        Download::from_raw(unsafe { ffi::ba_asset_pack_download(self.ptr) })
    }

    pub fn download_for_request(&self, request: ContentRequest) -> Option<Download> {
        Download::from_raw(unsafe {
            ffi::ba_asset_pack_download_for_request(self.ptr, request.as_raw())
        })
    }
}

impl Clone for AssetPack {
    fn clone(&self) -> Self {
        Self {
            ptr: ffi::retained(self.ptr),
        }
    }
}

impl Drop for AssetPack {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::ba_object_release(self.ptr) };
        }
    }
}

impl fmt::Debug for AssetPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetPack")
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

// SAFETY: `AssetPack` wraps a retained Swift `AssetPackBox` pointer. The underlying
// Swift type is `Sendable`; Rust never dereferences the raw pointer directly.
unsafe impl Send for AssetPack {}
// SAFETY: See `Send` justification above.
unsafe impl Sync for AssetPack {}

pub(crate) fn collect_asset_packs(array_ptr: *mut c_void) -> Vec<AssetPack> {
    if array_ptr.is_null() {
        return Vec::new();
    }

    let length =
        usize::try_from(unsafe { ffi::ba_asset_pack_array_len(array_ptr) }).unwrap_or_default();
    let mut asset_packs = Vec::with_capacity(length);
    for index in 0..length {
        let index = isize::try_from(index).expect("asset-pack array length originates from isize");
        let item_ptr = unsafe { ffi::ba_asset_pack_array_get(array_ptr, index) };
        if let Some(asset_pack) = AssetPack::from_raw(item_ptr) {
            asset_packs.push(asset_pack);
        }
    }
    unsafe { ffi::ba_object_release(array_ptr) };
    asset_packs
}
