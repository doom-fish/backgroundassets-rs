import BackgroundAssets
import ExtensionFoundation
import Foundation

@_silgen_name("ba_rust_download_manager_delegate_did_begin")
func ba_rust_download_manager_delegate_did_begin(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_did_pause")
func ba_rust_download_manager_delegate_did_pause(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_did_write_bytes")
func ba_rust_download_manager_delegate_did_write_bytes(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?,
    _ bytesWritten: Int64,
    _ totalBytesWritten: Int64,
    _ totalBytesExpectedToWrite: Int64
)

@_silgen_name("ba_rust_download_manager_delegate_challenge_disposition")
func ba_rust_download_manager_delegate_challenge_disposition(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?,
    _ challengeJSON: UnsafePointer<CChar>?
) -> Int32

@_silgen_name("ba_rust_download_manager_delegate_failed")
func ba_rust_download_manager_delegate_failed(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?,
    _ errorJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_finished")
func ba_rust_download_manager_delegate_finished(
    _ context: UnsafeMutableRawPointer?,
    _ downloadJSON: UnsafePointer<CChar>?,
    _ fileURL: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_began")
func ba_rust_managed_asset_pack_download_delegate_began(
    _ context: UnsafeMutableRawPointer?,
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_paused")
func ba_rust_managed_asset_pack_download_delegate_paused(
    _ context: UnsafeMutableRawPointer?,
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_progress")
func ba_rust_managed_asset_pack_download_delegate_progress(
    _ context: UnsafeMutableRawPointer?,
    _ assetPackJSON: UnsafePointer<CChar>?,
    _ progressJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_finished")
func ba_rust_managed_asset_pack_download_delegate_finished(
    _ context: UnsafeMutableRawPointer?,
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_failed")
func ba_rust_managed_asset_pack_download_delegate_failed(
    _ context: UnsafeMutableRawPointer?,
    _ assetPackJSON: UnsafePointer<CChar>?,
    _ errorJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_extension_should_download_asset_pack")
func ba_rust_managed_extension_should_download_asset_pack(
    _ assetPackJSON: UnsafePointer<CChar>?
) -> Bool

@_silgen_name("ba_rust_managed_extension_challenge_disposition")
func ba_rust_managed_extension_challenge_disposition(
    _ download: UnsafeMutableRawPointer?,
    _ challengeJSON: UnsafePointer<CChar>?
) -> Int32

private func rustDisposition(from rawValue: Int32) -> URLSession.AuthChallengeDisposition {
    switch rawValue {
    case 0:
        return .useCredential
    case 2:
        return .cancelAuthenticationChallenge
    case 3:
        return .rejectProtectionSpace
    default:
        return .performDefaultHandling
    }
}

private func withDownloadJSON(
    _ download: BADownload,
    _ body: (UnsafePointer<CChar>?) -> Void
) {
    ((try? bridgeJSON(downloadSnapshot(download))) ?? "{}").withCString(body)
}

@available(macOS 26.0, *)
private func managedAssetPackSnapshot(_ assetPack: __BAAssetPack) -> AssetPackSnapshot {
    AssetPackSnapshot(
        id: assetPack.id,
        downloadSize: assetPack.downloadSize,
        version: assetPack.version,
        description: assetPack.description
    )
}

@available(macOS 26.0, *)
private func withManagedAssetPackJSON(
    _ assetPack: __BAAssetPack,
    _ body: (UnsafePointer<CChar>?) -> Void
) {
    ((try? bridgeJSON(managedAssetPackSnapshot(assetPack))) ?? "{}").withCString(body)
}

final class BackgroundAssetsRustDownloadManagerDelegateProxy: NSObject, BADownloadManagerDelegate, @unchecked Sendable {
    private let context: RustContext

    init(context: RustContext) {
        self.context = context
        super.init()
    }

    func downloadDidBegin(_ download: BADownload) {
        withDownloadJSON(download) {
            ba_rust_download_manager_delegate_did_begin(context.pointer, $0)
        }
    }

    func downloadDidPause(_ download: BADownload) {
        withDownloadJSON(download) {
            ba_rust_download_manager_delegate_did_pause(context.pointer, $0)
        }
    }

    func download(
        _ download: BADownload,
        didWriteBytes bytesWritten: Int64,
        totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        withDownloadJSON(download) { downloadJSON in
            ba_rust_download_manager_delegate_did_write_bytes(
                context.pointer,
                downloadJSON,
                bytesWritten,
                totalBytesWritten,
                totalBytesExpectedToWrite
            )
        }
    }

    func download(
        _ download: BADownload,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        let challengeJSON = (try? bridgeJSON(challengeSnapshot(challenge))) ?? "{}"
        let rawDisposition = downloadJSON.withCString { downloadPtr in
            challengeJSON.withCString { challengePtr in
                ba_rust_download_manager_delegate_challenge_disposition(
                    context.pointer,
                    downloadPtr,
                    challengePtr
                )
            }
        }
        completionHandler(rustDisposition(from: rawDisposition), nil)
    }

    func download(_ download: BADownload, failedWithError error: Error) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        downloadJSON.withCString { downloadPtr in
            errorString(error).withCString { errorPtr in
                ba_rust_download_manager_delegate_failed(context.pointer, downloadPtr, errorPtr)
            }
        }
    }

    func download(_ download: BADownload, finishedWithFileURL fileURL: URL) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        downloadJSON.withCString { downloadPtr in
            fileURL.absoluteString.withCString { fileURLPtr in
                ba_rust_download_manager_delegate_finished(context.pointer, downloadPtr, fileURLPtr)
            }
        }
    }
}

@_cdecl("ba_download_manager_delegate_install")
public func ba_download_manager_delegate_install(
    _ context: UnsafeMutableRawPointer?,
    _ retain: ContextReferenceCallback,
    _ release: @escaping ContextReferenceCallback,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let context else {
        writeErrorOut(errorOut, "delegate context must not be null")
        return nil
    }
    if let reason = downloadManagerUnavailableReason {
        writeErrorOut(errorOut, reason)
        return nil
    }
    let delegate = BackgroundAssetsRustDownloadManagerDelegateProxy(
        context: RustContext(context, retain: retain, release: release)
    )
    BADownloadManager.shared.delegate = delegate
    return retained(delegate)
}

@_cdecl("ba_download_manager_delegate_clear_if_matches")
public func ba_download_manager_delegate_clear_if_matches(
    _ delegatePtr: UnsafeMutableRawPointer?
) {
    guard let delegatePtr, downloadManagerUnavailableReason == nil else { return }
    let delegate = borrowed(delegatePtr, as: BackgroundAssetsRustDownloadManagerDelegateProxy.self)
    if let current = BADownloadManager.shared.delegate as AnyObject?, current === delegate {
        BADownloadManager.shared.delegate = nil
    }
}

@available(macOS 26.0, *)
final class BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy: NSObject, __BAManagedAssetPackDownloadDelegate, @unchecked Sendable {
    private let context: RustContext

    init(context: RustContext) {
        self.context = context
        super.init()
    }

    func download(ofAssetPackBegan assetPack: __BAAssetPack) {
        withManagedAssetPackJSON(assetPack) {
            ba_rust_managed_asset_pack_download_delegate_began(context.pointer, $0)
        }
    }

    func download(ofAssetPackPaused assetPack: __BAAssetPack) {
        withManagedAssetPackJSON(assetPack) {
            ba_rust_managed_asset_pack_download_delegate_paused(context.pointer, $0)
        }
    }

    func download(of assetPack: __BAAssetPack, hasProgress progress: Progress) {
        let assetPackJSON = (try? bridgeJSON(managedAssetPackSnapshot(assetPack))) ?? "{}"
        let progressJSON = (try? bridgeJSON(progressSnapshot(progress))) ?? "{}"
        assetPackJSON.withCString { assetPackPtr in
            progressJSON.withCString { progressPtr in
                ba_rust_managed_asset_pack_download_delegate_progress(
                    context.pointer,
                    assetPackPtr,
                    progressPtr
                )
            }
        }
    }

    func download(ofAssetPackFinished assetPack: __BAAssetPack) {
        withManagedAssetPackJSON(assetPack) {
            ba_rust_managed_asset_pack_download_delegate_finished(context.pointer, $0)
        }
    }

    func download(of assetPack: __BAAssetPack, failedWithError error: any Error) {
        let assetPackJSON = (try? bridgeJSON(managedAssetPackSnapshot(assetPack))) ?? "{}"
        assetPackJSON.withCString { assetPackPtr in
            errorString(error).withCString { errorPtr in
                ba_rust_managed_asset_pack_download_delegate_failed(
                    context.pointer,
                    assetPackPtr,
                    errorPtr
                )
            }
        }
    }
}

@_cdecl("ba_asset_pack_manager_delegate_install")
public func ba_asset_pack_manager_delegate_install(
    _ context: UnsafeMutableRawPointer?,
    _ retain: ContextReferenceCallback,
    _ release: @escaping ContextReferenceCallback,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let context else {
        writeErrorOut(errorOut, "delegate context must not be null")
        return nil
    }
    if let reason = assetPackManagerUnavailableReason {
        writeErrorOut(errorOut, reason)
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    let delegate = BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy(
        context: RustContext(context, retain: retain, release: release)
    )
    __BAAssetPackManager.shared.delegate = delegate
    return retained(delegate)
}

@_cdecl("ba_asset_pack_manager_delegate_clear_if_matches")
public func ba_asset_pack_manager_delegate_clear_if_matches(
    _ delegatePtr: UnsafeMutableRawPointer?
) {
    guard let delegatePtr, assetPackManagerUnavailableReason == nil, #available(macOS 26.0, *) else { return }
    let delegate = borrowed(delegatePtr, as: BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy.self)
    if let current = __BAAssetPackManager.shared.delegate, current === delegate {
        __BAAssetPackManager.shared.delegate = nil
    }
}

@available(macOS 26.0, *)
@objc(BackgroundAssetsRustManagedDownloaderExtension)
public final class BackgroundAssetsRustManagedDownloaderExtension: NSObject, ManagedDownloaderExtension {
    @MainActor
    public override init() {
        super.init()
    }

    public func shouldDownload(_ assetPack: AssetPack) -> Bool {
        DownloadsRequestScope.shared.enter()
        defer { DownloadsRequestScope.shared.leave() }
        let json = (try? bridgeJSON(assetPackSnapshot(assetPack))) ?? "{}"
        return json.withCString(ba_rust_managed_extension_should_download_asset_pack)
    }

    public func backgroundDownload(
        _ download: BADownload,
        didReceive challenge: URLAuthenticationChallenge
    ) async -> (URLSession.AuthChallengeDisposition, URLCredential?) {
        let rawDisposition = (try? bridgeJSON(challengeSnapshot(challenge)))?.withCString {
            ba_rust_managed_extension_challenge_disposition(
                Unmanaged.passUnretained(download).toOpaque(),
                $0
            )
        } ?? 1
        return (rustDisposition(from: rawDisposition), nil)
    }
}
