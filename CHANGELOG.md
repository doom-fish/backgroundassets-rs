# Changelog

All notable changes to `backgroundassets` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - Unreleased

### Security

- Strings that Rust returns to the Swift bridge (the download plan of `DownloaderExtensionHandler::downloads`) were allocated with `CString::into_raw` and freed with libc `free`, which corrupts the heap under a custom `#[global_allocator]`. Swift now frees them through the new `ba_rust_string_free` export.
- The asset-pack status stream freed its context on the first "done" callback, so a second "done" read freed memory and later updates wrote through a dangling pointer. The context is now a `doom_fish_utils::callback_context::CallbackContext` that the Swift stream box retains until it's deallocated; late and duplicate callbacks are ignored.
- The download-manager and managed asset-pack delegates keep a retained per-installation context in their Swift proxy, so a callback that races with dropping the stream never sees freed state.

### Fixed

- `DownloadManager::with_exclusive_control` finished before the caller's work ran, so the lock was already released and the result claimed exclusivity it didn't have. It now takes a closure and runs it inside the framework handler while the lock is held, like Apple's `withExclusiveControl(_:)` overlay; a closure panic becomes an error.
- A panic in a delegate, stream or extension handler is contained instead of unwinding across the FFI boundary, and the event still reaches the stream.
- User handlers no longer run while a global mutex is held: each installation has its own handler lock, so a handler can install, replace or drop registrations without deadlocking.
- Dropping a delegate or extension event stream now unregisters its handler. Replacing a registration closes the previous stream.
- The managed principal class `BackgroundAssetsRustManagedDownloaderExtension` replaced the system configuration with one whose `accept(connection:)` accepted every connection without exporting anything, and it overrode the download methods the managed flow must not implement. It now keeps the system's configuration and default implementations, forwarding only `shouldDownload(_:)` and authentication challenges to Rust.
- Swift traps: `Int(fileSize)` for sizes above `Int.max` and `URL(string:)!` for an unparsable `file:` manifest URL now return errors.
- Calls that Background Assets answers by terminating the process now return errors: `DownloadManager::shared()` without a bundle identifier or the required `Info.plist` keys, `AssetPackManager::shared()` without a bundle identifier, team identifier or app group, `UrlDownload` with a zero size or an out-of-range priority, `schedule_download` / `start_foreground_download` with an essential download or from inside `downloads(for:)`, and essential downloads in a periodic download plan.
- `BAURLDownload` exceptions (for example an app group the process doesn't belong to) are caught by a new Objective-C shim and returned as errors instead of aborting.
- Async operations kept the bridge's JSON error as the message of a generic error, so domain, code, asset-pack identifier and file path were lost and `managed_error_code()` never matched. They now decode the payload.
- Authentication challenges and `AppExtensionInfo` snapshots never decoded the camelCase keys the bridge sends, so handlers saw empty challenges and the restricted download sizes were always `None`.
- Progress with a non-finite fraction no longer drops the delegate event.
- Package.swift now deploys to macOS 13.0, matching the README, and every newer API is guarded at runtime (13.3, 26.0, 26.4). It previously required macOS 26, which made the availability checks dead.
- Async futures that returned retained bridge objects (asset packs, asset-pack arrays, current downloads) leaked those objects when the future was dropped before the framework answered, or completed and was never polled. The completion now owns them and releases them when the future is gone.
- `build.rs` no longer adds the toolchain's `usr/lib/swift-5.5/macosx` directory to the rpath. It shadowed the SDK's concurrency library for the whole binary and broke linking alongside bridges that use newer concurrency APIs.

### Changed

- **Breaking:** `DownloadManager::with_exclusive_control` and `AsyncDownloadManager::with_exclusive_control` take `(before, body)` and return `ExclusiveControlFuture<R>` resolving to the closure's result, instead of `Result<bool, _>`.
- **Breaking:** `DownloadManager::shared()`, `AssetPackManager::shared()`, `AsyncDownloadManager::shared()` and `AsyncAssetPackManager::shared()` return `Result<Self, BackgroundAssetsError>` instead of `Option<Self>`.
- **Breaking:** `install_global_managed_downloader_extension(handler, capacity)` takes a `ManagedDownloaderExtensionHandler` and returns an `ExtensionEventStream`.
- **Breaking:** dropping the stream returned by `install_global_downloader_extension` or `install_global_managed_downloader_extension` unregisters the handler; both functions are `#[must_use]`.
- **Breaking:** `DownloaderExtensionHandler::should_download_asset_pack` moved to `ManagedDownloaderExtensionHandler`; the standard extension never called it.
- The doom-fish-utils requirement is now `>=0.4.1, <0.5`, and `rust-version` is 1.82.

### Added

- `ManagedDownloaderExtensionHandler` trait with `should_download_asset_pack` and `did_receive_challenge`.
- `ExclusiveControlFuture<R>`, re-exported at the crate root and from `async_api`.
- Tests that call the Swift bridge (manager preconditions, `UrlDownload` validation and exception handling, manifest errors, delegate installation failure) and tests that drive the delegate, stream, extension and exclusive-control callbacks through their real contexts.

### Removed

- **Breaking:** `ManagedDownloaderExtensionConfiguration` and `ManagedDownloaderExtensionRegistration`. The system owns the managed extension's configuration and download-manager delegate.

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
