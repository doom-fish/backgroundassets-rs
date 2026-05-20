# backgroundassets-rs

Safe Rust bindings for Apple's `BackgroundAssets` framework — on-demand asset packs, manifest parsing, managed asset-pack availability, downloader-extension bridging, and URL download descriptors on macOS.

> **Status:** v0.3.0 adds direct `BADownloadManagerDelegate` and `BAManagedAssetPackDownloadDelegate` wrappers, the self-hosted managed-extension principal class, and typed `ManagedBackgroundAssetsError` support alongside the executor-agnostic `async_api` module.

## Quick start

```toml
backgroundassets = { version = "0.3", features = ["async"] }
```

```rust,no_run
use backgroundassets::async_api::AsyncAssetPackManager;
use backgroundassets::AssetPackStatus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let Some(manager) = AsyncAssetPackManager::shared() else {
            return Ok(());
        };
        let packs = manager.all_asset_packs().await?;

        for pack in packs {
            let status = manager.status_relative_to(&pack).await?;
            println!("{} => {:?}", pack.id(), status);
            if status.contains(AssetPackStatus::DOWNLOAD_AVAILABLE) {
                manager.ensure_local_availability(&pack, false).await?;
            }
        }

        Ok::<(), backgroundassets::BackgroundAssetsError>(())
    })?;
    Ok(())
}
```

## Feature table

| Area | Surface |
| --- | --- |
| Asset packs | `AssetPack`, `AssetPackStatus`, `Manifest`, `AssetPackManager` |
| Downloads | `Download`, `UrlDownload`, `DownloadManager`, `DownloadManagerDelegate`, `DownloadManagerEvent`, `DownloadPriority`, `DownloadStatus` |
| Managed delegates | `ManagedAssetPackDownloadDelegate`, `ManagedAssetPackDownloadEvent`, `ManagedBackgroundAssetsError`, `ManagedBackgroundAssetsErrorCode` |
| Extension metadata | `AppExtensionInfo`, `AuthenticationChallenge`, `ChallengeDisposition` |
| Async workflows | `async_api::{AsyncAssetPackManager, AsyncDownloadManager}`, status-update streams, direct delegate event streams, and managed-extension registration *(requires `async`)* |
| Rust-hosted extension | `install_global_downloader_extension`, `install_global_managed_downloader_extension`, and Swift principal classes `BackgroundAssetsRustDownloaderExtension` / `BackgroundAssetsRustManagedDownloaderExtension` |

## Highlights

- Safe wrappers for `AssetPack`, `Manifest`, `Download`, `UrlDownload`, `DownloadManager`, `AssetPackManager`, and `BAAppExtensionInfo`.
- Direct `BADownloadManagerDelegate` and `BAManagedAssetPackDownloadDelegate` coverage via Rust traits, installer helpers, and bounded async event streams.
- Executor-agnostic async wrappers in `async_api`, backed by Swift `Task` thunks and `doom-fish-utils::completion::AsyncCompletion`.
- Rust-side principal classes for both `BADownloaderExtension` and self-hosted `ManagedDownloaderExtension` flows.
- Typed managed-error support layered on `BackgroundAssetsError`, preserving the framework domain, numeric code, message, asset-pack identifier, and file path metadata.

## Examples

All examples require the `async` feature:

- `01_list_pack_status`
- `02_download_pack`
- `03_extension_handler`
- `04_async_download`
- `05_managed_extension_handler`

Run them with:

```bash
cargo run --features async --example 01_list_pack_status
cargo run --features async --example 02_download_pack -- com.example.asset-pack
cargo run --features async --example 03_extension_handler
cargo run --features async --example 04_async_download -- com.example.asset-pack
cargo run --features async --example 05_managed_extension_handler
```

## Availability

- The crate targets the Background Assets surface shipped in the macOS 26 SDK.
- `BADownload`, `BAURLDownload`, and `BADownloadManager` are available on earlier macOS releases in the Apple SDK, but the managed asset-pack surface (`AssetPack`, `Manifest`, `AssetPackManager`, managed delegates, and the managed principal class) requires macOS 26.
- `AssetPackManager::status_relative_to`, `local_status`, `asset_pack_is_available_locally`, and `ensure_local_availability(..., require_latest = true)` require macOS 26.4 at runtime because that is where Apple introduced those APIs.

## Extension integration

The Swift bridge exports two ready-to-use principal classes:

- `BackgroundAssetsRustDownloaderExtension` for the standard `BADownloaderExtension` flow.
- `BackgroundAssetsRustManagedDownloaderExtension` for the macOS 26 self-hosted `ManagedDownloaderExtension` flow.

Use the standard class with `install_global_downloader_extension(...)`.
Use the managed class with `install_global_managed_downloader_extension(...)`, which also registers the managed download-manager delegate configuration expected by the self-hosted path.

For direct delegate observation outside the extension principal class, use:

- `install_global_download_manager_delegate(...)`
- `install_global_managed_asset_pack_download_delegate(...)`

See `examples/03_extension_handler.rs` and `examples/05_managed_extension_handler.rs` for the Rust side of the pattern.

## Coverage

See [COVERAGE.md](COVERAGE.md) and [COVERAGE_AUDIT.md](COVERAGE_AUDIT.md) for the audited SDK surface and full macOS 26.5 coverage summary.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
