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
