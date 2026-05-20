#![cfg(feature = "async")]

use backgroundassets::{
    install_global_managed_downloader_extension, AppExtensionInfo, AssetPackSnapshot,
    AuthenticationChallenge, ChallengeDisposition, ContentRequest, Download,
    DownloadManagerDelegate, DownloadManagerEvent, DownloadPriority, DownloadSnapshot,
    DownloadStatus, DownloadWriteProgress, DownloaderExtensionHandler, ExtensionEvent,
    ManagedDownloaderExtensionConfiguration,
};

struct ExampleExtension;

impl DownloaderExtensionHandler for ExampleExtension {
    fn should_download_asset_pack(&mut self, _asset_pack: &AssetPackSnapshot) -> bool {
        true
    }

    fn downloads(
        &mut self,
        _request: ContentRequest,
        _manifest_url: &str,
        _extension_info: &AppExtensionInfo,
    ) -> Result<Vec<Download>, backgroundassets::BackgroundAssetsError> {
        Ok(Vec::new())
    }

    fn did_receive_challenge(
        &mut self,
        _download: &Download,
        _challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        ChallengeDisposition::PerformDefaultHandling
    }
}

struct ExampleDownloadManagerDelegate;

impl DownloadManagerDelegate for ExampleDownloadManagerDelegate {}

#[test]
fn managed_extension_registration_exposes_streams() {
    let registration = install_global_managed_downloader_extension(
        ExampleExtension,
        ManagedDownloaderExtensionConfiguration::new(ExampleDownloadManagerDelegate)
            .extension_event_capacity(0)
            .download_manager_event_capacity(0),
    );

    assert_eq!(registration.extension_events().buffered_count(), 0);
    assert_eq!(registration.download_manager_events().buffered_count(), 0);
    assert!(!registration.extension_events().is_closed());
    assert!(!registration.download_manager_events().is_closed());
}

#[test]
fn new_event_variants_serialize_as_expected() {
    let download = DownloadSnapshot {
        identifier: "download.one".into(),
        unique_identifier: "unique.one".into(),
        status: DownloadStatus::Downloading,
        priority: DownloadPriority::new(7),
        is_essential: true,
        is_url_download: true,
    };
    let progress = DownloadWriteProgress {
        bytes_written: 128,
        total_bytes_written: 512,
        total_bytes_expected_to_write: 1024,
    };
    let manager_event = DownloadManagerEvent::Progress {
        download,
        progress,
    };
    let manager_json = serde_json::to_value(&manager_event).unwrap();
    assert_eq!(manager_json["kind"], "progress");
    assert_eq!(manager_json["download"]["identifier"], "download.one");

    let extension_event = ExtensionEvent::ShouldDownloadAssetPack {
        asset_pack: AssetPackSnapshot {
            id: "pack.one".into(),
            download_size: 1024,
            version: 2,
            description: "Pack One".into(),
        },
        should_download: false,
    };
    let extension_json = serde_json::to_value(&extension_event).unwrap();
    assert_eq!(extension_json["kind"], "should_download_asset_pack");
    assert_eq!(extension_json["should_download"], false);
}
