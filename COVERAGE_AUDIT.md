# backgroundassets coverage audit (vs MacOSX26.5.sdk)

SDK_PUBLIC_FAMILIES: 18
VERIFIED_FAMILIES: 18
GAPS: 0
COVERAGE_PCT: 100.00%

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
| `BADownloadManager` core operations | `DownloadManager` |
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
| `ManagedDownloaderExtension`, `ManagedDownloaderExtensionConfiguration`, `ManagedBackgroundAssetsError` | `install_global_managed_downloader_extension`, `ManagedDownloaderExtensionConfiguration`, `ManagedDownloaderExtensionRegistration`, `BackgroundAssetsRustManagedDownloaderExtension`, `ManagedBackgroundAssetsError` |
| Managed asset-pack availability stream | `AssetPackManager::status_updates`, `status_updates_for_asset_pack` |

## ✅ GAPS

No remaining audited gaps for the macOS 26.5 Background Assets surface.
