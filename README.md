# backgroundassets-rs

Safe Rust bindings for Apple's `BackgroundAssets` framework — on-demand asset packs, manifest parsing, managed asset-pack availability, downloader-extension bridging, and URL download descriptors on macOS.

> **Status:** v0.1.0 wraps the asset-pack surface needed by `foundation-models-rs`, plus `BADownload`, `BAURLDownload`, `BADownloadManager`, `BAAppExtensionInfo`, and a Rust-hosted `BADownloaderExtension` adapter.

## Quick start

```toml
backgroundassets = { version = "0.1", features = ["async"] }
```

```rust,no_run
use backgroundassets::{AssetPackManager, AssetPackStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let Some(manager) = AssetPackManager::shared() else {
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
| Downloads | `Download`, `UrlDownload`, `DownloadManager`, `DownloadPriority`, `DownloadStatus` |
| Extension metadata | `AppExtensionInfo`, `AuthenticationChallenge`, `ChallengeDisposition` |
| Async workflows | `AssetPackManager` async methods, status-update streams, `DownloadManager::current_downloads`, downloader-extension event stream *(requires `async`)* |
| Rust-hosted extension | `install_global_downloader_extension` + Swift principal class `BackgroundAssetsRustDownloaderExtension` *(requires `async` for events)* |

## Highlights

- Safe opaque wrappers for `AssetPack`, `Manifest`, `Download`, `UrlDownload`, `DownloadManager`, `AssetPackManager`, and `BAAppExtensionInfo`.
- Executor-agnostic async wrappers backed by Swift `Task` thunks and `doom-fish-utils::completion::AsyncCompletion`.
- Managed asset-pack status streaming backed by `doom-fish-utils::stream::BoundedAsyncStream`.
- Rust-side `BADownloaderExtension` handler registration for extension code that wants to keep its scheduling logic in Rust.
- Typed error payloads that preserve the framework error domain, numeric code, message, asset-pack identifier, and file path metadata.

## Examples

All examples require the `async` feature:

- `01_list_pack_status`
- `02_download_pack`
- `03_extension_handler`

Run them with:

```bash
cargo run --features async --example 01_list_pack_status
cargo run --features async --example 02_download_pack -- com.example.asset-pack
cargo run --features async --example 03_extension_handler
```

## Availability

- The crate targets the Background Assets surface shipped in the macOS 26 SDK.
- `BADownload`, `BAURLDownload`, and `BADownloadManager` are available on earlier macOS releases in the Apple SDK, but the asset-pack surface (`AssetPack`, `Manifest`, `AssetPackManager`) requires macOS 26.
- `AssetPackManager::status_relative_to`, `local_status`, `asset_pack_is_available_locally`, and `ensure_local_availability(..., require_latest = true)` require macOS 26.4 at runtime because that is where Apple introduced those APIs.

## Extension integration

The Swift bridge exports a ready-to-use principal class named `BackgroundAssetsRustDownloaderExtension`. To use it in an app extension:

1. Link the Rust crate into the extension target.
2. Set `EXPrincipalClass` to `BackgroundAssetsRustDownloaderExtension` in the extension's `Info.plist`.
3. Register a Rust handler at startup with `install_global_downloader_extension(...)`.

See `examples/03_extension_handler.rs` for the Rust side of the pattern.

## Coverage

See [COVERAGE.md](COVERAGE.md) and [COVERAGE_AUDIT.md](COVERAGE_AUDIT.md) for the audited SDK surface and documented gaps.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
