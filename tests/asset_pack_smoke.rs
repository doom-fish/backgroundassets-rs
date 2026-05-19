#![cfg(feature = "async")]

use backgroundassets::AssetPackManager;

#[test]
#[ignore = "requires a signed app or extension context with declared Background Assets"]
fn asset_pack_smoke() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let Some(manager) = AssetPackManager::shared() else {
            return Ok::<(), backgroundassets::BackgroundAssetsError>(());
        };

        let packs = manager.all_asset_packs().await.unwrap_or_default();
        for pack in packs {
            let _ = pack.snapshot();
            let _ = manager.status_relative_to(&pack).await.ok();
        }
        Ok(())
    })?;
    Ok(())
}
