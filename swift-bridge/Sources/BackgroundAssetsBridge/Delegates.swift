import BackgroundAssets
import ExtensionFoundation
import Foundation

@_silgen_name("ba_rust_download_manager_delegate_did_begin")
func ba_rust_download_manager_delegate_did_begin(
    _ downloadJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_did_pause")
func ba_rust_download_manager_delegate_did_pause(
    _ downloadJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_did_write_bytes")
func ba_rust_download_manager_delegate_did_write_bytes(
    _ downloadJSON: UnsafePointer<CChar>?,
    _ bytesWritten: Int64,
    _ totalBytesWritten: Int64,
    _ totalBytesExpectedToWrite: Int64
)

@_silgen_name("ba_rust_download_manager_delegate_challenge_disposition")
func ba_rust_download_manager_delegate_challenge_disposition(
    _ downloadJSON: UnsafePointer<CChar>?,
    _ challengeJSON: UnsafePointer<CChar>?
) -> Int32

@_silgen_name("ba_rust_download_manager_delegate_failed")
func ba_rust_download_manager_delegate_failed(
    _ downloadJSON: UnsafePointer<CChar>?,
    _ errorJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_download_manager_delegate_finished")
func ba_rust_download_manager_delegate_finished(
    _ downloadJSON: UnsafePointer<CChar>?,
    _ fileURL: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_began")
func ba_rust_managed_asset_pack_download_delegate_began(
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_paused")
func ba_rust_managed_asset_pack_download_delegate_paused(
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_progress")
func ba_rust_managed_asset_pack_download_delegate_progress(
    _ assetPackJSON: UnsafePointer<CChar>?,
    _ progressJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_finished")
func ba_rust_managed_asset_pack_download_delegate_finished(
    _ assetPackJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_managed_asset_pack_download_delegate_failed")
func ba_rust_managed_asset_pack_download_delegate_failed(
    _ assetPackJSON: UnsafePointer<CChar>?,
    _ errorJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_extension_should_download_asset_pack")
func ba_rust_extension_should_download_asset_pack(
    _ assetPackJSON: UnsafePointer<CChar>?
) -> Bool

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

private func decodeDownloadPointers(_ json: String) -> Set<BADownload> {
    guard let data = json.data(using: .utf8),
          let values = try? JSONDecoder().decode([UInt64].self, from: data)
    else {
        return []
    }

    let downloads = values.compactMap { value -> BADownload? in
        guard let ptr = UnsafeMutableRawPointer(bitPattern: UInt(value)) else {
            return nil
        }
        return Unmanaged<AnyObject>.fromOpaque(ptr).takeRetainedValue() as? BADownload
    }
    return Set(downloads)
}

private func withDownloadJSON(
    _ download: BADownload,
    _ body: (UnsafePointer<CChar>?) -> Void
) {
    ((try? bridgeJSON(downloadSnapshot(download))) ?? "{}").withCString(body)
}

private func withAssetPackJSON(
    _ assetPack: AssetPack,
    _ body: (UnsafePointer<CChar>?) -> Void
) {
    ((try? bridgeJSON(assetPackSnapshot(assetPack))) ?? "{}").withCString(body)
}

@available(macOS 26.0, *)
private func objectiveCManagedAssetPackManager() -> NSObject? {
    guard let managerClass = NSClassFromString("BAAssetPackManager") as? NSObject.Type,
          managerClass.responds(to: NSSelectorFromString("sharedManager")),
          let unmanaged = managerClass.perform(NSSelectorFromString("sharedManager")),
          let manager = unmanaged.takeUnretainedValue() as? NSObject
    else {
        return nil
    }
    return manager
}

@available(macOS 26.0, *)
private func objectiveCManagedAssetPackSnapshot(_ assetPack: NSObject) -> AssetPackSnapshot {
    let identifier =
        (assetPack.value(forKey: "identifier") as? String)
        ?? (assetPack.value(forKey: "id") as? String)
        ?? ""
    let downloadSize =
        (assetPack.value(forKey: "downloadSize") as? NSNumber)?.intValue
        ?? (assetPack.value(forKey: "downloadSize") as? Int)
        ?? 0
    let version =
        (assetPack.value(forKey: "version") as? NSNumber)?.intValue
        ?? (assetPack.value(forKey: "version") as? Int)
        ?? 0

    return AssetPackSnapshot(
        id: identifier,
        downloadSize: downloadSize,
        version: version,
        description: String(describing: assetPack)
    )
}

@available(macOS 26.0, *)
private func withManagedAssetPackObjectJSON(
    _ assetPack: NSObject,
    _ body: (UnsafePointer<CChar>?) -> Void
) {
    ((try? bridgeJSON(objectiveCManagedAssetPackSnapshot(assetPack))) ?? "{}").withCString(body)
}

public final class BackgroundAssetsRustDownloadManagerDelegateProxy: NSObject, BADownloadManagerDelegate, @unchecked Sendable {
    public override init() {
        super.init()
    }

    public func downloadDidBegin(_ download: BADownload) {
        withDownloadJSON(download, ba_rust_download_manager_delegate_did_begin)
    }

    public func downloadDidPause(_ download: BADownload) {
        withDownloadJSON(download, ba_rust_download_manager_delegate_did_pause)
    }

    public func download(
        _ download: BADownload,
        didWriteBytes bytesWritten: Int64,
        totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        withDownloadJSON(download) { downloadJSON in
            ba_rust_download_manager_delegate_did_write_bytes(
                downloadJSON,
                bytesWritten,
                totalBytesWritten,
                totalBytesExpectedToWrite
            )
        }
    }

    public func download(
        _ download: BADownload,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        let challengeJSON = (try? bridgeJSON(challengeSnapshot(challenge))) ?? "{}"
        let rawDisposition = downloadJSON.withCString { downloadPtr in
            challengeJSON.withCString { challengePtr in
                ba_rust_download_manager_delegate_challenge_disposition(downloadPtr, challengePtr)
            }
        }
        completionHandler(rustDisposition(from: rawDisposition), nil)
    }

    public func download(_ download: BADownload, failedWithError error: Error) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        downloadJSON.withCString { downloadPtr in
            errorString(error).withCString { errorPtr in
                ba_rust_download_manager_delegate_failed(downloadPtr, errorPtr)
            }
        }
    }

    public func download(_ download: BADownload, finishedWithFileURL fileURL: URL) {
        let downloadJSON = (try? bridgeJSON(downloadSnapshot(download))) ?? "{}"
        downloadJSON.withCString { downloadPtr in
            fileURL.absoluteString.withCString { fileURLPtr in
                ba_rust_download_manager_delegate_finished(downloadPtr, fileURLPtr)
            }
        }
    }
}

@_cdecl("ba_download_manager_delegate_install")
public func ba_download_manager_delegate_install() -> UnsafeMutableRawPointer? {
    let delegate = BackgroundAssetsRustDownloadManagerDelegateProxy()
    BADownloadManager.shared.delegate = delegate
    return retained(delegate)
}

@_cdecl("ba_download_manager_delegate_clear_if_matches")
public func ba_download_manager_delegate_clear_if_matches(
    _ delegatePtr: UnsafeMutableRawPointer?
) {
    guard let delegatePtr else { return }
    let delegate = borrowed(delegatePtr, as: BackgroundAssetsRustDownloadManagerDelegateProxy.self)
    if let current = BADownloadManager.shared.delegate as AnyObject?, current === delegate {
        BADownloadManager.shared.delegate = nil
    }
}

@available(macOS 26.0, *)
final class BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy: NSObject {
    @objc(downloadOfAssetPackBegan:)
    func downloadOfAssetPackBegan(_ assetPack: NSObject) {
        withManagedAssetPackObjectJSON(assetPack, ba_rust_managed_asset_pack_download_delegate_began)
    }

    @objc(downloadOfAssetPackPaused:)
    func downloadOfAssetPackPaused(_ assetPack: NSObject) {
        withManagedAssetPackObjectJSON(assetPack, ba_rust_managed_asset_pack_download_delegate_paused)
    }

    @objc(downloadOfAssetPack:hasProgress:)
    func downloadOfAssetPack(_ assetPack: NSObject, hasProgress progress: Progress) {
        let assetPackJSON = (try? bridgeJSON(objectiveCManagedAssetPackSnapshot(assetPack))) ?? "{}"
        let progressJSON = (try? bridgeJSON(progressSnapshot(progress))) ?? "{}"
        assetPackJSON.withCString { assetPackPtr in
            progressJSON.withCString { progressPtr in
                ba_rust_managed_asset_pack_download_delegate_progress(assetPackPtr, progressPtr)
            }
        }
    }

    @objc(downloadOfAssetPackFinished:)
    func downloadOfAssetPackFinished(_ assetPack: NSObject) {
        withManagedAssetPackObjectJSON(assetPack, ba_rust_managed_asset_pack_download_delegate_finished)
    }

    @objc(downloadOfAssetPack:failedWithError:)
    func downloadOfAssetPack(_ assetPack: NSObject, failedWithError error: Error) {
        let assetPackJSON = (try? bridgeJSON(objectiveCManagedAssetPackSnapshot(assetPack))) ?? "{}"
        assetPackJSON.withCString { assetPackPtr in
            errorString(error).withCString { errorPtr in
                ba_rust_managed_asset_pack_download_delegate_failed(assetPackPtr, errorPtr)
            }
        }
    }
}

@_cdecl("ba_asset_pack_manager_delegate_install")
public func ba_asset_pack_manager_delegate_install() -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *),
          let manager = objectiveCManagedAssetPackManager()
    else {
        return nil
    }

    let delegate = BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy()
    manager.setValue(delegate, forKey: "delegate")
    return retained(delegate)
}

@_cdecl("ba_asset_pack_manager_delegate_clear_if_matches")
public func ba_asset_pack_manager_delegate_clear_if_matches(
    _ delegatePtr: UnsafeMutableRawPointer?
) {
    guard #available(macOS 26.0, *),
          let delegatePtr,
          let manager = objectiveCManagedAssetPackManager()
    else {
        return
    }

    let delegate = borrowed(delegatePtr, as: BackgroundAssetsRustManagedAssetPackDownloadDelegateProxy.self)
    if let current = manager.value(forKey: "delegate") as? NSObject, current === delegate {
        manager.setValue(nil, forKey: "delegate")
    }
}

private let managedExtensionDownloadManagerDelegateProxy = BackgroundAssetsRustDownloadManagerDelegateProxy()

@available(macOS 26.0, *)
public struct BackgroundAssetsRustManagedDownloaderExtensionConfiguration: ManagedDownloaderExtensionConfiguration {
    public static var downloadManagerDelegate: BackgroundAssetsRustDownloadManagerDelegateProxy {
        managedExtensionDownloadManagerDelegateProxy
    }

    public init() {}

    public func accept(connection: NSXPCConnection) -> Bool {
        true
    }
}

@available(macOS 26.0, *)
@objc(BackgroundAssetsRustManagedDownloaderExtension)
public final class BackgroundAssetsRustManagedDownloaderExtension: NSObject, ManagedDownloaderExtension {
    public typealias Configuration = BackgroundAssetsRustManagedDownloaderExtensionConfiguration

    @MainActor
    public override init() {
        super.init()
    }

    @MainActor
    public var configuration: BackgroundAssetsRustManagedDownloaderExtensionConfiguration {
        .init()
    }

    public func shouldDownload(_ assetPack: AssetPack) -> Bool {
        let json = (try? bridgeJSON(assetPackSnapshot(assetPack))) ?? "{}"
        return json.withCString(ba_rust_extension_should_download_asset_pack)
    }

    public func downloads(
        for request: BAContentRequest,
        manifestURL: URL,
        extensionInfo: BAAppExtensionInfo
    ) -> Set<BADownload> {
        let manifest = manifestURL.absoluteString
        let payload = manifest.withCString {
            ba_rust_extension_downloads_for_request(
                Int32(request.rawValue),
                $0,
                Unmanaged.passUnretained(extensionInfo).toOpaque()
            )
        }
        guard let payload else { return [] }
        let json = String(cString: payload)
        ba_string_free(payload)
        return decodeDownloadPointers(json)
    }

    public func backgroundDownload(
        _ download: BADownload,
        didReceive challenge: URLAuthenticationChallenge
    ) async -> (URLSession.AuthChallengeDisposition, URLCredential?) {
        let rawDisposition = (try? bridgeJSON(challengeSnapshot(challenge)))?.withCString {
            ba_rust_extension_challenge_disposition(
                Unmanaged.passUnretained(download).toOpaque(),
                $0
            )
        } ?? 1
        return (rustDisposition(from: rawDisposition), nil)
    }

    public func backgroundDownload(_ failedDownload: BADownload, failedWithError error: Error) {
        errorString(error).withCString {
            ba_rust_extension_download_failed(
                Unmanaged.passUnretained(failedDownload).toOpaque(),
                $0
            )
        }
    }

    public func backgroundDownload(_ finishedDownload: BADownload, finishedWithFileURL fileURL: URL) {
        fileURL.absoluteString.withCString {
            ba_rust_extension_download_finished(
                Unmanaged.passUnretained(finishedDownload).toOpaque(),
                $0
            )
        }
    }

    public func extensionWillTerminate() {
        ba_rust_extension_will_terminate()
    }
}
