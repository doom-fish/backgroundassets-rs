#[cfg(feature = "async")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "async")]
use std::sync::Arc;

use backgroundassets::{
    AssetPackManager, DownloadManager, DownloadPriority, Manifest, UrlDownload, UrlDownloadOptions,
};

const URL: &str = "https://example.com/assets/pack.bin";
const GROUP: &str = "group.fish.doom.backgroundassets.tests";

fn mentions_any(message: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| message.contains(needle))
}

#[test]
fn download_manager_reports_a_missing_bundle_identifier() {
    let error = DownloadManager::shared().unwrap_err();
    assert!(
        error.message_text().contains("bundle identifier"),
        "{error}"
    );
}

#[test]
fn asset_pack_manager_reports_its_unmet_requirements() {
    let error = AssetPackManager::shared().unwrap_err();
    assert!(
        mentions_any(error.message_text(), &["bundle identifier", "macOS 26"]),
        "{error}"
    );
}

#[test]
fn download_priority_constants_come_from_the_framework() {
    let min = DownloadPriority::min().raw_value();
    let default = DownloadPriority::default_value().raw_value();
    let max = DownloadPriority::max().raw_value();
    assert!(min < default && default < max, "{min} {default} {max}");
    assert_eq!(
        DownloadPriority::default(),
        DownloadPriority::default_value()
    );
}

#[test]
fn url_download_rejects_invalid_arguments_before_calling_the_framework() {
    let zero = UrlDownload::new("pack", URL, 0, GROUP).unwrap_err();
    assert!(zero.message_text().contains("file_size"), "{zero}");

    let huge = UrlDownload::new("pack", URL, u64::MAX, GROUP).unwrap_err();
    assert!(huge.message_text().contains("file_size"), "{huge}");

    for raw in [
        DownloadPriority::max().raw_value() + 1,
        DownloadPriority::min().raw_value() - 1,
        i64::MAX,
    ] {
        let options = UrlDownloadOptions {
            priority: DownloadPriority::new(raw),
            ..UrlDownloadOptions::default()
        };
        let error = UrlDownload::with_options("pack", URL, 1024, GROUP, options).unwrap_err();
        assert!(error.message_text().contains("priority"), "{error}");
    }

    let nul = UrlDownload::new("pa\0ck", URL, 1024, GROUP).unwrap_err();
    assert!(nul.message_text().contains("NUL"), "{nul}");

    let insecure =
        UrlDownload::new("pack", "http://example.com/pack.bin", 1024, GROUP).unwrap_err();
    assert!(
        mentions_any(insecure.message_text(), &["https", "13.3"]),
        "{insecure}"
    );
}

#[test]
fn url_download_turns_framework_exceptions_into_errors() {
    let error = UrlDownload::new("pack", URL, 1024, GROUP).unwrap_err();
    assert!(
        error.domain() == "BackgroundAssetsObjCBridge" || error.message_text().contains("13.3"),
        "{error}"
    );
    assert!(!error.message_text().is_empty());
}

#[test]
fn manifest_from_bytes_returns_the_framework_error() {
    let error = Manifest::from_bytes(b"{}", GROUP).unwrap_err();
    assert!(!error.message_text().is_empty());
    assert!(!error.domain().is_empty());
}

#[test]
fn manifest_from_url_reports_a_missing_file() {
    let error =
        Manifest::from_url("/nonexistent/backgroundassets/manifest.json", GROUP).unwrap_err();
    assert!(!error.message_text().is_empty());
}

#[test]
fn manifest_from_url_rejects_an_unparsable_file_url() {
    let error = Manifest::from_url("file://[", GROUP).unwrap_err();
    assert!(
        mentions_any(error.message_text(), &["valid URL", "macOS 26"]),
        "{error}"
    );
}

#[cfg(feature = "async")]
struct DropCounter(Arc<AtomicUsize>);

#[cfg(feature = "async")]
impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[cfg(feature = "async")]
impl backgroundassets::DownloadManagerDelegate for DropCounter {}

#[cfg(feature = "async")]
impl backgroundassets::ManagedAssetPackDownloadDelegate for DropCounter {}

#[cfg(feature = "async")]
#[test]
fn failed_delegate_installation_releases_the_handler() {
    let drops = Arc::new(AtomicUsize::new(0));

    let error = backgroundassets::install_global_download_manager_delegate(
        DropCounter(Arc::clone(&drops)),
        4,
    )
    .unwrap_err();
    assert!(
        error.message_text().contains("bundle identifier"),
        "{error}"
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);

    let error = backgroundassets::install_global_managed_asset_pack_download_delegate(
        DropCounter(Arc::clone(&drops)),
        4,
    )
    .unwrap_err();
    assert!(
        mentions_any(error.message_text(), &["bundle identifier", "macOS 26"]),
        "{error}"
    );
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[cfg(feature = "async")]
#[test]
fn async_managers_report_unavailability() {
    use backgroundassets::async_api::{AsyncAssetPackManager, AsyncDownloadManager};

    assert!(AsyncDownloadManager::shared().is_err());
    assert!(AsyncAssetPackManager::shared().is_err());
}
