# BackgroundAssets.framework coverage

Audited against:

- SDK: `MacOSX.sdk/System/Library/Frameworks/BackgroundAssets.framework`
- Crate: `backgroundassets` v0.3.0

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
| `BAManagedDownloaderExtension` / `ManagedDownloaderExtensionConfiguration` | ✅ implemented | Rust-managed installation surface plus Swift principal class `BackgroundAssetsRustManagedDownloaderExtension`. |

## Download model

| API family | Status | Notes |
| --- | --- | --- |
| `BAContentRequest` | ✅ implemented | Exposed as `ContentRequest`. |
| `BADownload.State` | ✅ implemented | Exposed as `DownloadStatus`. |
| `BADownload.Priority` + min/default/max | ✅ implemented | Exposed as `DownloadPriority`. |
| `BADownload` | ✅ implemented | Metadata, essentiality, and `removing_essential`. |
| `BAURLDownload` | ✅ implemented | Safe constructor plus option bundle for headers, method, essential flag, and priority. |
| `BADownloadManager` | ✅ implemented | Shared manager, scheduling, cancellation, foreground start, exclusive control, and async current-download discovery. |
| `BADownloadManagerDelegate` | ✅ implemented | Rust trait + installer helper `install_global_download_manager_delegate` + bounded async event stream mirroring the delegate callbacks. |
| `BAErrorDomain` + `BAErrorCode` | ✅ implemented | Preserved via `BackgroundAssetsError`. |

## Extension integration

| API family | Status | Notes |
| --- | --- | --- |
| `BAAppExtensionInfo` | ✅ implemented | Opaque wrapper plus snapshot API. |
| `BADownloaderExtension` | ✅ implemented | Rust trait + global registration + Swift principal class `BackgroundAssetsRustDownloaderExtension`. |
| `ManagedDownloaderExtension` | ✅ implemented | Managed install helper + Swift principal class `BackgroundAssetsRustManagedDownloaderExtension`. |
| Challenge handling | ✅ implemented | Modeled through `AuthenticationChallenge` + `ChallengeDisposition`. |

## Summary

The current release covers the full audited Background Assets surface for macOS 26.5, including direct delegate-property wrappers for download-manager and managed asset-pack callbacks, the self-hosted managed extension flow, and typed managed errors.
