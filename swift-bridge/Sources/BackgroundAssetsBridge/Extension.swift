import BackgroundAssets
import Foundation

@_silgen_name("ba_rust_extension_downloads_for_request")
func ba_rust_extension_downloads_for_request(
    _ request: Int32,
    _ manifestURL: UnsafePointer<CChar>?,
    _ extensionInfo: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("ba_rust_extension_challenge_disposition")
func ba_rust_extension_challenge_disposition(
    _ download: UnsafeMutableRawPointer?,
    _ challengeJSON: UnsafePointer<CChar>?
) -> Int32

@_silgen_name("ba_rust_extension_download_failed")
func ba_rust_extension_download_failed(
    _ download: UnsafeMutableRawPointer?,
    _ errorJSON: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_extension_download_finished")
func ba_rust_extension_download_finished(
    _ download: UnsafeMutableRawPointer?,
    _ fileURL: UnsafePointer<CChar>?
)

@_silgen_name("ba_rust_extension_will_terminate")
func ba_rust_extension_will_terminate()

private func disposition(from rawValue: Int32) -> URLSession.AuthChallengeDisposition {
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

@objc(BackgroundAssetsRustDownloaderExtension)
public final class BackgroundAssetsRustDownloaderExtension: NSObject, BADownloaderExtension {
    @MainActor
    public override init() {
        super.init()
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
        return (disposition(from: rawDisposition), nil)
    }

    public func backgroundDownload(_ failedDownload: BADownload, failedWithError error: any Error) {
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
