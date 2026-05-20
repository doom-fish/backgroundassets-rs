use core::ffi::c_char;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ffi;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BackgroundAssetsError {
    domain: String,
    code: i64,
    message: String,
    asset_pack_id: Option<String>,
    file_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BridgeErrorPayload {
    #[serde(default = "default_bridge_domain")]
    pub domain: String,
    #[serde(default = "default_bridge_code")]
    pub code: i64,
    #[serde(default)]
    pub message: String,
    #[serde(default, rename = "assetPackID")]
    pub asset_pack_id: Option<String>,
    #[serde(default, rename = "filePath")]
    pub file_path: Option<String>,
}

const fn default_bridge_code() -> i64 {
    -1
}

fn default_bridge_domain() -> String {
    "BackgroundAssetsBridge".into()
}

fn is_managed_error_domain(domain: &str) -> bool {
    matches!(domain, "BAManagedErrorDomain" | "BackgroundAssets.ManagedBackgroundAssetsError")
        || domain.ends_with(".ManagedBackgroundAssetsError")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedBackgroundAssetsErrorCode {
    AssetPackNotFound,
    FileNotFound,
    Unknown(i64),
}

impl ManagedBackgroundAssetsErrorCode {
    pub const fn from_raw(value: i64) -> Self {
        match value {
            0 => Self::AssetPackNotFound,
            1 => Self::FileNotFound,
            other => Self::Unknown(other),
        }
    }

    pub const fn raw_value(self) -> i64 {
        match self {
            Self::AssetPackNotFound => 0,
            Self::FileNotFound => 1,
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManagedBackgroundAssetsError {
    #[serde(flatten)]
    inner: BackgroundAssetsError,
    managed_code: ManagedBackgroundAssetsErrorCode,
}

impl ManagedBackgroundAssetsError {
    pub fn from_background_assets_error(error: BackgroundAssetsError) -> Option<Self> {
        is_managed_error_domain(error.domain()).then(|| Self {
            managed_code: ManagedBackgroundAssetsErrorCode::from_raw(error.code()),
            inner: error,
        })
    }

    pub const fn managed_code(&self) -> ManagedBackgroundAssetsErrorCode {
        self.managed_code
    }

    pub fn as_background_assets_error(&self) -> &BackgroundAssetsError {
        &self.inner
    }

    pub fn into_background_assets_error(self) -> BackgroundAssetsError {
        self.inner
    }

    pub fn domain(&self) -> &str {
        self.inner.domain()
    }

    pub const fn code(&self) -> i64 {
        self.inner.code()
    }

    pub fn message_text(&self) -> &str {
        self.inner.message_text()
    }

    pub fn asset_pack_id(&self) -> Option<&str> {
        self.inner.asset_pack_id()
    }

    pub fn file_path(&self) -> Option<&str> {
        self.inner.file_path()
    }
}

impl TryFrom<BackgroundAssetsError> for ManagedBackgroundAssetsError {
    type Error = BackgroundAssetsError;

    fn try_from(value: BackgroundAssetsError) -> Result<Self, Self::Error> {
        Self::from_background_assets_error(value.clone()).ok_or(value)
    }
}

impl From<ManagedBackgroundAssetsError> for BackgroundAssetsError {
    fn from(value: ManagedBackgroundAssetsError) -> Self {
        value.inner
    }
}

impl fmt::Display for ManagedBackgroundAssetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for ManagedBackgroundAssetsError {}

impl BackgroundAssetsError {
    pub(crate) fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            domain: default_bridge_domain(),
            code: default_bridge_code(),
            message: message.into(),
            asset_pack_id: None,
            file_path: None,
        }
    }

    pub(crate) fn message(message: impl Into<String>) -> Self {
        Self::invalid_argument(message)
    }

    pub(crate) fn from_json_str(json: &str) -> Self {
        serde_json::from_str::<BridgeErrorPayload>(json).map_or_else(
            |_| Self {
                domain: default_bridge_domain(),
                code: default_bridge_code(),
                message: json.to_string(),
                asset_pack_id: None,
                file_path: None,
            },
            Into::into,
        )
    }

    pub(crate) fn from_owned_json_ptr(ptr: *mut c_char) -> Self {
        Self::from_json_str(&unsafe { ffi::owned_string(ptr) })
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub const fn code(&self) -> i64 {
        self.code
    }

    pub fn message_text(&self) -> &str {
        &self.message
    }

    pub fn asset_pack_id(&self) -> Option<&str> {
        self.asset_pack_id.as_deref()
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn managed_error_code(&self) -> Option<ManagedBackgroundAssetsErrorCode> {
        is_managed_error_domain(self.domain())
            .then(|| ManagedBackgroundAssetsErrorCode::from_raw(self.code()))
    }

    pub fn into_managed_background_assets_error(
        self,
    ) -> Result<ManagedBackgroundAssetsError, Self> {
        ManagedBackgroundAssetsError::try_from(self)
    }
}

impl From<BridgeErrorPayload> for BackgroundAssetsError {
    fn from(value: BridgeErrorPayload) -> Self {
        Self {
            domain: value.domain,
            code: value.code,
            message: if value.message.is_empty() {
                "BackgroundAssets operation failed".into()
            } else {
                value.message
            },
            asset_pack_id: value.asset_pack_id,
            file_path: value.file_path,
        }
    }
}

impl fmt::Display for BackgroundAssetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}:{})", self.message, self.domain, self.code)
    }
}

impl std::error::Error for BackgroundAssetsError {}

#[cfg(test)]
mod tests {
    use super::{BackgroundAssetsError, ManagedBackgroundAssetsErrorCode};

    #[test]
    fn managed_error_conversion_preserves_asset_pack_id() {
        let error = BackgroundAssetsError::from_json_str(
            r#"{"domain":"BackgroundAssets.ManagedBackgroundAssetsError","code":0,"message":"missing","assetPackID":"pack.one"}"#,
        );

        assert_eq!(
            error.managed_error_code(),
            Some(ManagedBackgroundAssetsErrorCode::AssetPackNotFound)
        );
        let managed = error.into_managed_background_assets_error().unwrap();
        assert_eq!(managed.asset_pack_id(), Some("pack.one"));
        assert_eq!(managed.file_path(), None);
    }

    #[test]
    fn managed_error_conversion_preserves_file_path() {
        let error = BackgroundAssetsError::from_json_str(
            r#"{"domain":"BAManagedErrorDomain","code":1,"message":"missing","filePath":"Assets/file.bin"}"#,
        );

        assert_eq!(
            error.managed_error_code(),
            Some(ManagedBackgroundAssetsErrorCode::FileNotFound)
        );
        let managed = error.into_managed_background_assets_error().unwrap();
        assert_eq!(managed.asset_pack_id(), None);
        assert_eq!(managed.file_path(), Some("Assets/file.bin"));
    }

    #[test]
    fn non_managed_error_rejects_conversion() {
        let error = BackgroundAssetsError::from_json_str(
            r#"{"domain":"BackgroundAssetsBridge","code":-1,"message":"nope"}"#,
        );

        assert_eq!(error.managed_error_code(), None);
        assert!(error.into_managed_background_assets_error().is_err());
    }
}
