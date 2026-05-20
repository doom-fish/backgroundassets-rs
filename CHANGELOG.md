# Changelog

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
