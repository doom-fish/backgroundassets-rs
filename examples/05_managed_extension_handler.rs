use backgroundassets::{
    install_global_managed_downloader_extension, AssetPackSnapshot, AuthenticationChallenge,
    ChallengeDisposition, Download, ManagedDownloaderExtensionHandler,
};

struct ExampleExtension;

impl ManagedDownloaderExtensionHandler for ExampleExtension {
    fn should_download_asset_pack(&mut self, asset_pack: &AssetPackSnapshot) -> bool {
        println!(
            "considering managed asset pack {} v{}",
            asset_pack.id, asset_pack.version
        );
        true
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
    let events = install_global_managed_downloader_extension(ExampleExtension, 16);

    println!("Registered Rust managed downloader extension handler.");
    println!(
        "Set EXPrincipalClass to BackgroundAssetsRustManagedDownloaderExtension in your extension Info.plist."
    );
    println!("Extension stream closed? {}", events.is_closed());
}
