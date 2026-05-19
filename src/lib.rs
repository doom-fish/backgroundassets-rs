#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::doc_markdown,
    clippy::future_not_send,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_safety_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::return_self_not_must_use,
    clippy::use_self
)]

pub mod asset_pack;
pub mod download;
pub mod error;
pub mod extension;
mod ffi;
pub mod manager;
pub mod manifest;

pub use asset_pack::{AssetPack, AssetPackSnapshot, AssetPackStatus};
pub use download::{
    ContentRequest, Download, DownloadManager, DownloadPriority, DownloadSnapshot, DownloadStatus,
    UrlDownload, UrlDownloadOptions,
};
pub use error::BackgroundAssetsError;
#[cfg(feature = "async")]
pub use extension::{install_global_downloader_extension, ExtensionEventStream};
pub use extension::{
    AppExtensionInfo, AppExtensionInfoSnapshot, AuthenticationChallenge, ChallengeDisposition,
    DownloaderExtensionHandler, ExtensionEvent,
};
pub use manager::AssetPackManager;
#[cfg(feature = "async")]
pub use manager::{DownloadProgress, DownloadStatusStream, DownloadStatusUpdate, UpdateCheck};
pub use manifest::Manifest;
