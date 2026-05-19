use backgroundassets::{
    install_global_downloader_extension, AppExtensionInfo, AuthenticationChallenge,
    ChallengeDisposition, ContentRequest, Download, DownloaderExtensionHandler, UrlDownload,
};

struct ExampleExtension;

impl DownloaderExtensionHandler for ExampleExtension {
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

fn main() {
    let events = install_global_downloader_extension(ExampleExtension, 16);
    println!("Registered Rust downloader extension handler.");
    println!(
        "Set EXPrincipalClass to BackgroundAssetsRustDownloaderExtension in your extension Info.plist."
    );
    println!("Event stream closed? {}", events.is_closed());
}
