# BackgroundAssets.framework coverage

Audited against:

- SDK: `MacOSX.sdk/System/Library/Frameworks/BackgroundAssets.framework`
- Crate: `backgroundassets` v0.1.0

## Managed asset packs

| API family | Status | Notes |
| --- | --- | --- |
| `AssetPack` | ✅ implemented | Opaque wrapper with metadata accessors, `download`, `download_for_request`, and snapshots. |
| `AssetPack.Status` / `BAAssetPackStatus` | ✅ implemented | Exposed as `AssetPackStatus`. |
| `AssetPackManager` / `BAAssetPackManager` | ✅ implemented | Shared manager, async discovery, status queries, update checks, file reads, removals, and status-update streams. |
| `AssetPackManifest` / `BAAssetPackManifest` | ✅ implemented | Exposed as `Manifest`. |
| `BAManagedErrorDomain` + `BAManagedErrorCode` | ✅ implemented | Preserved via `BackgroundAssetsError`. |
| `BAManagedAssetPackDownloadDelegate` | ⏭️ deferred | Managed-download delegate callbacks are modeled through `AssetPackManager::status_updates*` instead of a direct delegate wrapper in 0.1.0. |
| `BAManagedDownloaderExtension` / `ManagedDownloaderExtension` | ⏭️ deferred | The crate ships a generic `BADownloaderExtension` bridge today; the managed-extension refinements are documented in the audit as a future gap. |

## Download model

| API family | Status | Notes |
| --- | --- | --- |
| `BAContentRequest` | ✅ implemented | Exposed as `ContentRequest`. |
| `BADownload.State` | ✅ implemented | Exposed as `DownloadStatus`. |
| `BADownload.Priority` + min/default/max | ✅ implemented | Exposed as `DownloadPriority`. |
| `BADownload` | ✅ implemented | Metadata, essentiality, and `removing_essential`. |
| `BAURLDownload` | ✅ implemented | Safe constructor plus option bundle for headers, method, essential flag, and priority. |
| `BADownloadManager` | ✅ implemented | Shared manager, scheduling, cancellation, foreground start, exclusive control, and async current-download discovery. |
| `BADownloadManagerDelegate` | ⏭️ deferred | Direct delegate callbacks are deferred in favor of the asset-pack status stream and extension event stream. |
| `BAErrorDomain` + `BAErrorCode` | ✅ implemented | Preserved via `BackgroundAssetsError`. |

## Extension integration

| API family | Status | Notes |
| --- | --- | --- |
| `BAAppExtensionInfo` | ✅ implemented | Opaque wrapper plus snapshot API. |
| `BADownloaderExtension` | ✅ implemented | Rust trait + global registration + Swift principal class `BackgroundAssetsRustDownloaderExtension`. |
| Challenge handling | ✅ implemented | Modeled through `AuthenticationChallenge` + `ChallengeDisposition`. |

## Summary

The initial crate release focuses on the surfaces needed for managed asset packs, manifest parsing, URL download descriptors, and Rust-driven downloader extensions. Direct delegate protocols for `BADownloadManager` and the managed extension refinements remain documented gaps for a later release.
