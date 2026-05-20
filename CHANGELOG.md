# Changelog

## [0.3.0] - 2026-05-20

### Added

- Added direct `BADownloadManagerDelegate` coverage with the `DownloadManagerDelegate` Rust trait, `DownloadManagerEvent` stream, and `install_global_download_manager_delegate` helper.
- Added direct `BAManagedAssetPackDownloadDelegate` coverage with the `ManagedAssetPackDownloadDelegate` Rust trait, matching async event stream, and installer helper.
- Added self-hosted managed-extension support through `DownloaderExtensionHandler::should_download_asset_pack`, `ManagedDownloaderExtensionConfiguration`, `install_global_managed_downloader_extension`, and the Swift principal class `BackgroundAssetsRustManagedDownloaderExtension`.
- Added typed `ManagedBackgroundAssetsError` and `ManagedBackgroundAssetsErrorCode` wrappers on top of `BackgroundAssetsError`, preserving managed asset-pack identifiers and file paths.
- Added `examples/05_managed_extension_handler.rs` plus non-runtime tests covering the new public async/event surfaces.

### Notes

- Phase 32 completeness + async sweep.

## [0.2.0] - 2026-05-20

### Added

- `async_api` module behind the `async` feature, providing executor-agnostic async/await wrappers for callback-based Background Assets APIs. Uses `doom-fish-utils::completion` so callers can use any executor (tokio, async-std, pollster, etc.).
- New example `examples/04_async_download.rs` showing the async download flow.

## [0.1.1] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [0.1.0] - 2026-05-19

### Added

- Added `AssetPack`, `AssetPackStatus`, `Manifest`, and `AssetPackManager` wrappers for the managed Background Assets surface on macOS 26+.
- Added `Download`, `UrlDownload`, `DownloadManager`, `DownloadPriority`, and `DownloadStatus` wrappers for the shared download model.
- Added `BackgroundAssetsError` with preserved framework domain, code, localized message, asset-pack identifier, and file-path metadata.
- Added Swift `@_cdecl` bridge thunks for asset-pack metadata, manifest parsing, URL download creation, download-manager operations, and managed asset-pack queries.
- Added async wrappers using `doom-fish-utils::completion::AsyncCompletion` plus `BoundedAsyncStream` streams for asset-pack status updates and downloader-extension events.
- Added a Rust-hosted downloader-extension bridge with the Swift principal class `BackgroundAssetsRustDownloaderExtension`.
- Added examples for listing pack status, requesting a pack download, and registering an extension handler.
- Added an ignored smoke test for environments that provide a signed app or extension context.
