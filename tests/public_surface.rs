#![cfg(feature = "async")]

use backgroundassets::{
    install_global_managed_downloader_extension, AssetPackSnapshot, AuthenticationChallenge,
    ChallengeDisposition, Download, DownloadManagerEvent, DownloadPriority, DownloadSnapshot,
    DownloadStatus, DownloadWriteProgress, ExtensionEvent, ManagedDownloaderExtensionHandler,
};

struct ExampleExtension;

impl ManagedDownloaderExtensionHandler for ExampleExtension {
    fn should_download_asset_pack(&mut self, _asset_pack: &AssetPackSnapshot) -> bool {
        true
    }

    fn did_receive_challenge(
        &mut self,
        _download: &Download,
        _challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        ChallengeDisposition::PerformDefaultHandling
    }
}

#[test]
fn managed_extension_registration_exposes_an_open_stream() {
    let events = install_global_managed_downloader_extension(ExampleExtension, 0);

    assert_eq!(events.buffered_count(), 0);
    assert!(!events.is_closed());
    assert!(events.try_next().is_none());
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
