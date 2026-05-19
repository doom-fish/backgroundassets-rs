use backgroundassets::AssetPackManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let Some(manager) = AssetPackManager::shared() else {
            println!("BackgroundAssets unavailable on this system.");
            return Ok::<(), backgroundassets::BackgroundAssetsError>(());
        };

        let packs = manager.all_asset_packs().await?;
        if packs.is_empty() {
            println!("No managed asset packs were reported.");
            return Ok(());
        }

        for pack in packs {
            let status = manager.status_relative_to(&pack).await?;
            println!(
                "{} v{} size={} status={:?}",
                pack.id(),
                pack.version(),
                pack.download_size(),
                status
            );
        }

        Ok(())
    })?;
    Ok(())
}
