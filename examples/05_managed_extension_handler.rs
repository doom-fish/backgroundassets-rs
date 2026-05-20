use backgroundassets::{
    install_global_managed_downloader_extension, AppExtensionInfo, AssetPackSnapshot,
    AuthenticationChallenge, ChallengeDisposition, ContentRequest, Download,
    DownloadManagerDelegate, DownloadSnapshot, DownloadWriteProgress, DownloaderExtensionHandler,
    ManagedDownloaderExtensionConfiguration, UrlDownload,
};

struct ExampleExtension;

impl DownloaderExtensionHandler for ExampleExtension {
    fn should_download_asset_pack(&mut self, asset_pack: &AssetPackSnapshot) -> bool {
        println!("considering managed asset pack {} v{}", asset_pack.id, asset_pack.version);
        true
    }

    fn downloads(
        &mut self,
        request: ContentRequest,
        manifest_url: &str,
        extension_info: &AppExtensionInfo,
    ) -> Result<Vec<Download>, backgroundassets::BackgroundAssetsError> {
        println!(
            "request={request:?} manifest={manifest_url} remaining={:?}",
            extension_info.restricted_download_size_remaining()
        );

        let download = UrlDownload::new(
            "com.example.backgroundassets.sample",
            "https://example.com/assets/sample.bin",
            1024,
            "group.com.example.backgroundassets",
        )?;
        Ok(vec![download.into_download()])
    }

    fn did_receive_challenge(
        &mut self,
        _download: &Download,
        challenge: &AuthenticationChallenge,
    ) -> ChallengeDisposition {
        println!(
            "challenge host={} method={}",
            challenge.host, challenge.authentication_method
        );
        ChallengeDisposition::PerformDefaultHandling
    }
}

struct ExampleDownloadManagerDelegate;

impl DownloadManagerDelegate for ExampleDownloadManagerDelegate {
    fn download_did_begin(&mut self, download: &DownloadSnapshot) {
        println!("managed download began: {}", download.identifier);
    }

    fn download_did_write_bytes(
        &mut self,
        download: &DownloadSnapshot,
        progress: &DownloadWriteProgress,
    ) {
        println!(
            "managed download progress: {} {}/{}",
            download.identifier,
            progress.total_bytes_written,
            progress.total_bytes_expected_to_write
        );
    }
}

fn main() {
    let configuration = ManagedDownloaderExtensionConfiguration::new(ExampleDownloadManagerDelegate)
        .extension_event_capacity(16)
        .download_manager_event_capacity(16);
    let registration = install_global_managed_downloader_extension(ExampleExtension, configuration);

    println!("Registered Rust managed downloader extension handler.");
    println!(
        "Set EXPrincipalClass to BackgroundAssetsRustManagedDownloaderExtension in your extension Info.plist."
    );
    println!(
        "Extension stream closed? {} | download-manager stream closed? {}",
        registration.extension_events().is_closed(),
        registration.download_manager_events().is_closed()
    );
}
