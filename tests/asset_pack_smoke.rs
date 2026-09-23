#![cfg(feature = "async")]

use backgroundassets::AssetPackManager;

#[test]
#[ignore = "requires a signed app or extension context with declared Background Assets"]
fn asset_pack_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let manager = AssetPackManager::shared()?;
    pollster::block_on(async {
        let packs = manager.all_asset_packs().await?;
        for pack in packs {
            let snapshot = pack.snapshot();
            assert!(!snapshot.id.is_empty());
            assert_eq!(snapshot.id, pack.id());
            manager.status_relative_to(&pack).await?;
        }
        Ok::<(), backgroundassets::BackgroundAssetsError>(())
    })?;
    Ok(())
}
