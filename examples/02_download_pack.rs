use std::env;
use std::thread;

use backgroundassets::{AssetPackManager, DownloadStatusUpdate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        let Some(manager) = AssetPackManager::shared() else {
            println!("BackgroundAssets unavailable on this system.");
            return Ok::<(), backgroundassets::BackgroundAssetsError>(());
        };

        let pack = if let Some(asset_pack_id) = env::args().nth(1) {
            manager.asset_pack(&asset_pack_id).await?
        } else {
            let mut packs = manager.all_asset_packs().await?;
            if packs.is_empty() {
                println!("No managed asset packs were reported.");
                return Ok(());
            }
            packs.remove(0)
        };

        let target_id = pack.id();
        let updates = manager.status_updates_for_asset_pack(&target_id, 32)?;
        let worker_manager = manager.clone();
        let worker_pack = pack.clone();
        let worker = thread::spawn(move || {
            pollster::block_on(async move {
                worker_manager
                    .ensure_local_availability(&worker_pack, true)
                    .await
            })
        });

        println!("Waiting for download events for {target_id}...");
        while let Some(update) = updates.next().await {
            println!("{update:?}");
            match update {
                DownloadStatusUpdate::Finished { asset_pack } if asset_pack.id == target_id => {
                    break;
                }
                DownloadStatusUpdate::Failed { asset_pack, .. } if asset_pack.id == target_id => {
                    break;
                }
                _ => {}
            }
        }

        worker
            .join()
            .expect("ensure_local_availability thread panicked")?;
        Ok(())
    })?;
    Ok(())
}
