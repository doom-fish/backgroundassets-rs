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
        let Ok(manager) = AsyncAssetPackManager::shared() else {
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
| Rust-hosted extension | `DownloaderExtensionHandler`, `ManagedDownloaderExtensionHandler`, `install_global_downloader_extension`, `install_global_managed_downloader_extension`, and Swift principal classes `BackgroundAssetsRustDownloaderExtension` / `BackgroundAssetsRustManagedDownloaderExtension` |

## Highlights

- Safe wrappers for `AssetPack`, `Manifest`, `Download`, `UrlDownload`, `DownloadManager`, `AssetPackManager`, and `BAAppExtensionInfo`.
- Direct `BADownloadManagerDelegate` and `BAManagedAssetPackDownloadDelegate` coverage via Rust traits, installer helpers, and bounded async event streams.
- Executor-agnostic async wrappers in `async_api`, backed by Swift `Task` thunks and `doom-fish-utils::completion::AsyncCompletion`.
- Rust-side principal classes for both `BADownloaderExtension` and self-hosted `ManagedDownloaderExtension` flows.
- `DownloadManager::with_exclusive_control(before, body)` runs `body` while the cross-process lock is held, like Apple's `withExclusiveControl(_:)` overlay.
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

Building needs the macOS 26 SDK (Xcode 26, Swift 6.2). The Swift bridge deploys to macOS 13.0, and every API that needs a newer system is checked at runtime:

- macOS 13.0: `DownloadManager`, `Download`, `AppExtensionInfo`, the download-manager delegate, and the `BackgroundAssetsRustDownloaderExtension` principal class.
- macOS 13.3: creating a `UrlDownload`, `Download::is_essential` / `removing_essential`, and `AppExtensionInfo::restricted_essential_download_size_remaining`. Earlier systems get an error, `false` or `None`.
- macOS 26.0: the managed asset-pack surface (`AssetPack`, `Manifest`, `AssetPackManager`, the managed delegate, and the `BackgroundAssetsRustManagedDownloaderExtension` principal class). Earlier systems get an error.
- macOS 26.4: `AssetPackManager::status_relative_to`, `local_status`, `asset_pack_is_available_locally`, and `ensure_local_availability(..., true)`.

The macOS 27 additions (`AssetPackManager.manifest`, language resolution and localized lookups, `AssetPack.language`, and set-based `ensureLocalAvailability`) aren't wrapped yet.

## Process requirements

Background Assets terminates a process that breaks some of its rules instead of reporting an error. The crate checks these rules first and returns a `BackgroundAssetsError`:

- `DownloadManager::shared()` and `install_global_download_manager_delegate` need a bundle identifier. In the app (not in its extension) the `Info.plist` also needs a `BAInitialDownloadRestrictions` dictionary with a non-empty `BADownloadDomainAllowList` and `BADownloadAllowance` / `BAEssentialDownloadAllowance` numbers of 0 or more, plus an `https` `BAManifestURL`.
- `AssetPackManager::shared()` and `install_global_managed_asset_pack_download_delegate` need a bundle identifier, a code signature with a team identifier, a `BAAppGroupID` string in the `Info.plist`, and that app group's user defaults. Plain `cargo run` and `cargo test` binaries meet none of these, so they get an error.
- `UrlDownload` needs a file size between 1 and `isize::MAX`, a priority within `DownloadPriority::min()..=DownloadPriority::max()`, and an `https` URL. Exceptions the framework raises, for example for an app group the process doesn't belong to, become errors.
- `schedule_download` and `start_foreground_download` reject essential downloads (call `removing_essential()` first) and refuse to run while the extension is answering `downloads(for:)`; return the downloads from the handler instead.
- A download plan for a periodic content request can't contain essential downloads.

## Extension integration

The Swift bridge exports two ready-to-use principal classes:

- `BackgroundAssetsRustDownloaderExtension` for the standard `BADownloaderExtension` flow. Register a `DownloaderExtensionHandler` with `install_global_downloader_extension(...)`.
- `BackgroundAssetsRustManagedDownloaderExtension` for the macOS 26 self-hosted `ManagedDownloaderExtension` flow. It keeps the system's configuration and default implementations, which service the system connection, schedule the asset packs, and store finished downloads. Register a `ManagedDownloaderExtensionHandler` with `install_global_managed_downloader_extension(...)` to filter asset packs (`should_download_asset_pack`) and answer authentication challenges.

Both install functions return an `ExtensionEventStream`. Dropping the stream unregisters the handler, so keep it for as long as the extension runs.

For direct delegate observation, use `install_global_download_manager_delegate(...)` and `install_global_managed_asset_pack_download_delegate(...)`. Each returns a stream; dropping it removes the delegate and stops the handler. Inside a managed extension the system owns the download-manager delegate, so observe progress from the app with `AssetPackManager::status_updates` or the managed asset-pack delegate instead.

Handlers run on the framework's callback threads, one call at a time per handler. A download's file URL is only valid during `download_finished`: move the file there, because the event stream delivers it after the framework has deleted it. `with_exclusive_control` also runs its closure on the framework's callback queue while the lock is held; don't block it on work that needs that queue.

See `examples/03_extension_handler.rs` and `examples/05_managed_extension_handler.rs` for the Rust side of the pattern.

## Coverage

See [COVERAGE.md](COVERAGE.md) and [COVERAGE_AUDIT.md](COVERAGE_AUDIT.md) for the wrapped API families of the macOS 26.5 SDK and what isn't covered.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
