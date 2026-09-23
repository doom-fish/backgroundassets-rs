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
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_api;
pub mod download;
pub mod error;
pub mod extension;
mod ffi;
#[cfg(feature = "async")]
mod handler;
pub mod manager;
pub mod manifest;

pub use asset_pack::{AssetPack, AssetPackSnapshot, AssetPackStatus};
pub use download::{
    ContentRequest, Download, DownloadManager, DownloadManagerDelegate, DownloadManagerEvent,
    DownloadPriority, DownloadSnapshot, DownloadStatus, DownloadWriteProgress, UrlDownload,
    UrlDownloadOptions,
};
pub use error::{
    BackgroundAssetsError, ManagedBackgroundAssetsError, ManagedBackgroundAssetsErrorCode,
};
#[cfg(feature = "async")]
pub use download::{
    install_global_download_manager_delegate, DownloadManagerEventStream, ExclusiveControlFuture,
};
#[cfg(feature = "async")]
pub use extension::{
    install_global_downloader_extension, install_global_managed_downloader_extension,
    ExtensionEventStream,
};
pub use extension::{
    AppExtensionInfo, AppExtensionInfoSnapshot, AuthenticationChallenge, ChallengeDisposition,
    DownloaderExtensionHandler, ExtensionEvent, ManagedDownloaderExtensionHandler,
};
pub use manager::{
    AssetPackManager, ManagedAssetPackDownloadDelegate, ManagedAssetPackDownloadEvent,
};
#[cfg(feature = "async")]
pub use manager::{
    install_global_managed_asset_pack_download_delegate, DownloadProgress,
    DownloadStatusStream, DownloadStatusUpdate, ManagedAssetPackDownloadEventStream, UpdateCheck,
};
pub use manifest::Manifest;
