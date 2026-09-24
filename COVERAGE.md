# BackgroundAssets.framework coverage

Audited against:

- SDK: `MacOSX26.5.sdk/System/Library/Frameworks/BackgroundAssets.framework`
- Crate: `backgroundassets` v0.4.0

## Managed asset packs

| API family | Status | Notes |
| --- | --- | --- |
| `AssetPack` | ✅ implemented | Opaque wrapper with metadata accessors, `download`, `download_for_request`, and snapshots. |
| `AssetPack.Status` / `BAAssetPackStatus` | ✅ implemented | Exposed as `AssetPackStatus`. |
| `AssetPackManager` / `BAAssetPackManager` | ✅ implemented | Shared manager, async discovery, status queries, update checks, file reads, removals, and status-update streams. |
| `AssetPackManifest` / `BAAssetPackManifest` | ✅ implemented | Exposed as `Manifest`. |
| `BAManagedErrorDomain` + `BAManagedErrorCode` | ✅ implemented | Preserved via `BackgroundAssetsError`, `ManagedBackgroundAssetsError`, and `ManagedBackgroundAssetsErrorCode`. |
| `ManagedBackgroundAssetsError` | ✅ implemented | Typed Rust wrapper layered on the base error payload, preserving managed asset-pack identifiers and file paths. |
| `BAManagedAssetPackDownloadDelegate` | ✅ implemented | Rust trait + installer helper `install_global_managed_asset_pack_download_delegate` + bounded async event stream. |
| `BAManagedDownloaderExtension` | ✅ implemented | Swift principal class `BackgroundAssetsRustManagedDownloaderExtension` forwards `shouldDownload(_:)` and challenges to `ManagedDownloaderExtensionHandler`. |
| `ManagedDownloaderExtensionConfiguration` | ➖ not wrapped | Internal-visibility protocol. The principal class keeps the system's configuration, which owns the extension's connection and download-manager delegate. |

## Download model

| API family | Status | Notes |
| --- | --- | --- |
| `BAContentRequest` | ✅ implemented | Exposed as `ContentRequest`; values the crate doesn't know arrive as `ContentRequest::Unknown`. |
| `BADownload.State` | ✅ implemented | Exposed as `DownloadStatus`. |
| `BADownload.Priority` + min/default/max | ✅ implemented | Exposed as `DownloadPriority`. |
| `BADownload` | ✅ implemented | Metadata, essentiality, and `removing_essential`. |
| `BAURLDownload` | ✅ implemented | Safe constructor plus option bundle for headers, method, essential flag, and priority. |
| `BADownloadManager` | ✅ implemented | Shared manager, scheduling, cancellation, foreground start, exclusive control that runs a closure while the lock is held, and async current-download discovery. The synchronous `fetchCurrentDownloads:` (13.3) isn't wrapped. |
| `BADownloadManagerDelegate` | ✅ implemented | Rust trait + installer helper `install_global_download_manager_delegate` + bounded async event stream mirroring the delegate callbacks. |
| `BAErrorDomain` + `BAErrorCode` | ✅ implemented | Domain and numeric code preserved via `BackgroundAssetsError`; there is no typed `BAErrorCode` enum. |

## Extension integration

| API family | Status | Notes |
| --- | --- | --- |
| `BAAppExtensionInfo` | ✅ implemented | Opaque wrapper plus snapshot API. |
| `BADownloaderExtension` | ✅ implemented | Rust trait + global registration + Swift principal class `BackgroundAssetsRustDownloaderExtension`. |
| `ManagedDownloaderExtension` | ✅ implemented | `install_global_managed_downloader_extension` + Swift principal class `BackgroundAssetsRustManagedDownloaderExtension`, which keeps the system's default implementations. |
| Challenge handling | ✅ implemented | Modeled through `AuthenticationChallenge` + `ChallengeDisposition`. |

## Not covered

- macOS 27 SDK additions: `AssetPackManager.manifest`, `resolvedLanguage`, `locallyAvailableLanguages`, `reconcilePreferredLanguages`, set-based `ensureLocalAvailability(of:requireLatestVersions:)`, `contents` / `descriptor` / `url(asLocalizedFor:)`, `AssetPack.language`, and the async `withExclusiveControl(_ body:)` overlay (the crate's closure-based `with_exclusive_control` provides the same locking on macOS 13+).
- Deprecated initializers and `status(ofAssetPackWithID:)`, which have wrapped replacements.

## Summary

The release wraps the macOS 26.5 Background Assets API families listed above, including the download-manager and managed asset-pack delegates, both extension principal classes, and typed managed errors. The macOS 27 additions are not wrapped.
