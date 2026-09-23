# backgroundassets coverage audit (vs MacOSX26.5.sdk)

SDK_PUBLIC_FAMILIES: 19
VERIFIED_FAMILIES: 18
GAPS: 1
COVERAGE_PCT: 94.74%

The counts are over 19 API families that this crate chose to track in the macOS 26.5 SDK, not over individual symbols. Deprecated APIs with wrapped replacements are left out, and the macOS 27 SDK additions listed under GAPS are outside the count.

Audit sources:

- `MacOSX.sdk/System/Library/Frameworks/BackgroundAssets.framework/Versions/A/Headers/*.h`
- `MacOSX.sdk/System/Library/Frameworks/BackgroundAssets.framework/Versions/A/Modules/BackgroundAssets.swiftmodule/arm64e-apple-macos.swiftinterface`

## 🟢 VERIFIED

| Apple symbol family | Wrapped by |
| --- | --- |
| `BAContentRequest` | `ContentRequest` |
| `BADownload.State` | `DownloadStatus` |
| `BADownload.Priority` + min/default/max | `DownloadPriority` |
| `BADownload` | `Download` |
| `BAURLDownload` | `UrlDownload` |
| `BADownloadManager` core operations | `DownloadManager` (the synchronous `fetchCurrentDownloads:` isn't wrapped) |
| `BADownloadManagerDelegate` | `DownloadManagerDelegate`, `DownloadManagerEvent`, `DownloadManagerEventStream`, `install_global_download_manager_delegate` |
| `BAAppExtensionInfo` | `AppExtensionInfo` + `AppExtensionInfoSnapshot` |
| `BADownloaderExtension` | `DownloaderExtensionHandler`, `install_global_downloader_extension`, `BackgroundAssetsRustDownloaderExtension` |
| `BAErrorDomain` + `BAErrorCode` | `BackgroundAssetsError` |
| `AssetPack` / `BAAssetPack` | `AssetPack` + `AssetPackSnapshot` |
| `AssetPack.Status` / `BAAssetPackStatus` | `AssetPackStatus` |
| `AssetPackManager` / `BAAssetPackManager` | `AssetPackManager`, `DownloadStatusUpdate`, `DownloadStatusStream`, `UpdateCheck` |
| `AssetPackManifest` / `BAAssetPackManifest` | `Manifest` |
| `BAManagedAssetPackDownloadDelegate` | `ManagedAssetPackDownloadDelegate`, `ManagedAssetPackDownloadEventStream`, `install_global_managed_asset_pack_download_delegate` |
| `BAManagedErrorDomain` + `BAManagedErrorCode` | `BackgroundAssetsError`, `ManagedBackgroundAssetsError`, `ManagedBackgroundAssetsErrorCode` |
| `ManagedDownloaderExtension`, `ManagedBackgroundAssetsError` | `install_global_managed_downloader_extension`, `ManagedDownloaderExtensionHandler`, `BackgroundAssetsRustManagedDownloaderExtension`, `ManagedBackgroundAssetsError` |
| Managed asset-pack availability stream | `AssetPackManager::status_updates`, `status_updates_for_asset_pack` |

## GAPS

| Apple symbol family | Status |
| --- | --- |
| `ManagedDownloaderExtensionConfiguration` | Not wrapped. It's an internal-visibility protocol; implementing it replaced the system configuration that services the extension's connection. The principal class now keeps the system configuration. |

Outside the count (macOS 27 SDK): `AssetPackManager.manifest`, `resolvedLanguage`, `locallyAvailableLanguages`, `reconcilePreferredLanguages`, set-based `ensureLocalAvailability(of:requireLatestVersions:)`, `contents` / `descriptor` / `url(asLocalizedFor:)`, `AssetPack.language`, and the async `withExclusiveControl(_ body:)` overlay.
