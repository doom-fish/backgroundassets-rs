# backgroundassets coverage audit (vs MacOSX26.5.sdk)

SDK_PUBLIC_FAMILIES: 15
VERIFIED_FAMILIES: 12
GAPS: 3
COVERAGE_PCT: 80.00%

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
| `BAAppExtensionInfo` | `AppExtensionInfo` + `AppExtensionInfoSnapshot` |
| `BADownloaderExtension` | `DownloaderExtensionHandler`, `install_global_downloader_extension`, `BackgroundAssetsRustDownloaderExtension` |
| `BAErrorDomain` + `BAErrorCode` | `BackgroundAssetsError` |
| `AssetPack` / `BAAssetPack` | `AssetPack` + `AssetPackSnapshot` |
| `AssetPack.Status` / `BAAssetPackStatus` | `AssetPackStatus` |
| `AssetPackManager` / `BAAssetPackManager` | `AssetPackManager`, `DownloadStatusUpdate`, `DownloadStatusStream`, `UpdateCheck` |
| `AssetPackManifest` / `BAAssetPackManifest` | `Manifest` |
| `BAManagedErrorDomain` + `BAManagedErrorCode` | `BackgroundAssetsError` |
| Managed asset-pack availability stream | `AssetPackManager::status_updates`, `status_updates_for_asset_pack` |

## 🔴 GAPS

| Apple symbol family | Reason | Planned follow-up |
| --- | --- | --- |
| `BADownloadManagerDelegate` | The crate does not expose the Objective-C delegate protocol directly in 0.1.0. | Add a Rust event bridge for manual URL-download progress callbacks. |
| `BAManagedAssetPackDownloadDelegate` | Managed progress is exposed through `AssetPackManager` status streams instead of a delegate wrapper. | Add a direct delegate wrapper if consumers need the exact delegate object model. |
| `BAManagedDownloaderExtension`, `ManagedDownloaderExtensionConfiguration`, `ManagedBackgroundAssetsError` | The initial release ships a generic `BADownloaderExtension` bridge but not the managed-extension refinements. | Add a managed-extension registration layer and typed managed error helpers. |
