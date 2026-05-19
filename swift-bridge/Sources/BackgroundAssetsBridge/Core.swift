import BackgroundAssets
import ExtensionFoundation
import Foundation
import System

let unavailableMessage = "BackgroundAssets requires macOS 26.0 or newer"

struct BridgeErrorPayload: Encodable {
    let domain: String
    let code: Int
    let message: String
    let assetPackID: String?
    let filePath: String?
}

struct AssetPackSnapshot: Encodable {
    let id: String
    let downloadSize: Int
    let version: Int
    let description: String
}

struct DownloadSnapshot: Encodable {
    let identifier: String
    let uniqueIdentifier: String
    let status: Int
    let priority: Int
    let isEssential: Bool
    let isURLDownload: Bool
}

struct ProgressSnapshot: Encodable {
    let completedUnitCount: Int64
    let totalUnitCount: Int64
    let fractionCompleted: Double
    let localizedDescription: String
}

struct ExtensionInfoSnapshot: Encodable {
    let restrictedDownloadSizeRemaining: Int?
    let restrictedEssentialDownloadSizeRemaining: Int?
}

struct AuthenticationChallengeSnapshot: Encodable {
    let host: String
    let authenticationMethod: String
    let previousFailureCount: Int
    let proposedCredentialUser: String?
    let proposedCredentialHasPassword: Bool
}

struct UpdateCheckPayload: Encodable {
    let updatingIDs: [String]
    let removedIDs: [String]
}

struct DownloadStatusUpdatePayload: Encodable {
    let kind: String
    let assetPack: AssetPackSnapshot
    let progress: ProgressSnapshot?
    let error: BridgeErrorPayload?
}

final class AssetPackBox {
    let value: AssetPack

    init(_ value: AssetPack) {
        self.value = value
    }
}

final class ManifestBox {
    let value: AssetPackManifest

    init(_ value: AssetPackManifest) {
        self.value = value
    }
}

final class ManagerBox {
    let value: AssetPackManager

    init(_ value: AssetPackManager) {
        self.value = value
    }
}

final class AssetPackArrayBox {
    let value: [AssetPack]

    init(_ value: [AssetPack]) {
        self.value = value
    }
}

final class DownloadArrayBox {
    let value: [BADownload]

    init(_ value: [BADownload]) {
        self.value = value
    }
}

final class StatusUpdatesBridge {
    let task: Task<Void, Never>

    init(_ task: Task<Void, Never>) {
        self.task = task
    }

    deinit {
        task.cancel()
    }
}

final class AsyncCallbackBox: @unchecked Sendable {
    private let callback: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
    private let context: UnsafeMutableRawPointer?

    init(
        callback: @escaping @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
        context: UnsafeMutableRawPointer?
    ) {
        self.callback = callback
        self.context = context
    }

    func succeed(_ result: UnsafeMutableRawPointer?) {
        callback(result, nil, context)
    }

    func succeed(string: String) {
        if let result = ffiString(string) {
            callback(UnsafeMutableRawPointer(result), nil, context)
        } else {
            fail(message: "failed to allocate bridge string")
        }
    }

    func fail(message: String) {
        message.withCString { callback(nil, $0, context) }
    }

    func fail(error: any Error) {
        errorString(error).withCString { callback(nil, $0, context) }
    }
}

final class StreamCallbackBox: @unchecked Sendable {
    private let callback: @convention(c) (UnsafeMutableRawPointer?, UnsafeMutablePointer<CChar>?, Bool) -> Void
    private let context: UnsafeMutableRawPointer?

    init(
        callback: @escaping @convention(c) (UnsafeMutableRawPointer?, UnsafeMutablePointer<CChar>?, Bool) -> Void,
        context: UnsafeMutableRawPointer?
    ) {
        self.callback = callback
        self.context = context
    }

    func push(json: String) {
        if let string = ffiString(json) {
            callback(context, string, false)
        }
    }

    func finish() {
        callback(context, nil, true)
    }
}

@inline(__always)
func ffiString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

@inline(__always)
func retained(_ object: some AnyObject) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

@inline(__always)
func borrowed<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as type: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

@inline(__always)
func assetPack(from ptr: UnsafeMutableRawPointer) -> AssetPack {
    borrowed(ptr, as: AssetPackBox.self).value
}

@inline(__always)
func manifest(from ptr: UnsafeMutableRawPointer) -> AssetPackManifest {
    borrowed(ptr, as: ManifestBox.self).value
}

@inline(__always)
func manager(from ptr: UnsafeMutableRawPointer) -> AssetPackManager {
    borrowed(ptr, as: ManagerBox.self).value
}

@inline(__always)
func download(from ptr: UnsafeMutableRawPointer) -> BADownload {
    borrowed(ptr, as: BADownload.self)
}

@inline(__always)
func downloadManager(from ptr: UnsafeMutableRawPointer) -> BADownloadManager {
    borrowed(ptr, as: BADownloadManager.self)
}

@inline(__always)
func extensionInfo(from ptr: UnsafeMutableRawPointer) -> BAAppExtensionInfo {
    borrowed(ptr, as: BAAppExtensionInfo.self)
}

func bridgeJSON<T: Encodable>(_ value: T) throws -> String {
    let encoder = JSONEncoder()
    if #available(macOS 10.13, *) {
        encoder.outputFormatting = [.sortedKeys]
    }
    let data = try encoder.encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw NSError(
            domain: "BackgroundAssetsBridge",
            code: -1,
            userInfo: [NSLocalizedDescriptionKey: "Failed to encode UTF-8 JSON"]
        )
    }
    return string
}

func errorPayload(_ error: any Error) -> BridgeErrorPayload {
    let nsError = error as NSError
    return BridgeErrorPayload(
        domain: nsError.domain,
        code: nsError.code,
        message: nsError.localizedDescription,
        assetPackID: nil,
        filePath: nsError.userInfo[NSFilePathErrorKey] as? String
    )
}

func errorString(_ error: any Error) -> String {
    (try? bridgeJSON(errorPayload(error)))
        ?? "{\"code\":-1,\"domain\":\"BackgroundAssetsBridge\",\"message\":\"Unexpected bridge error\"}"
}

func messageErrorString(_ message: String) -> String {
    (try? bridgeJSON(BridgeErrorPayload(
        domain: "BackgroundAssetsBridge",
        code: -1,
        message: message,
        assetPackID: nil,
        filePath: nil
    ))) ?? "{\"code\":-1,\"domain\":\"BackgroundAssetsBridge\",\"message\":\"Unexpected bridge error\"}"
}

func writeErrorOut(
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ error: any Error
) {
    errorOut?.pointee = ffiString(errorString(error))
}

func writeErrorOut(
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ message: String
) {
    errorOut?.pointee = ffiString(messageErrorString(message))
}

func copyDataToHeap(
    _ data: Data,
    _ lengthOut: UnsafeMutablePointer<Int>?
) -> UnsafeMutableRawPointer? {
    lengthOut?.pointee = data.count
    let allocationSize = max(data.count, 1)
    let pointer = UnsafeMutableRawPointer.allocate(
        byteCount: allocationSize,
        alignment: MemoryLayout<UInt8>.alignment
    )
    data.copyBytes(to: pointer.assumingMemoryBound(to: UInt8.self), count: data.count)
    return pointer
}

func urlFromRawPath(_ rawPath: String) -> URL {
    rawPath.hasPrefix("file:") ? URL(string: rawPath)! : URL(fileURLWithPath: rawPath)
}

func assetPackSnapshot(_ assetPack: AssetPack) -> AssetPackSnapshot {
    AssetPackSnapshot(
        id: assetPack.id,
        downloadSize: assetPack.downloadSize,
        version: assetPack.version,
        description: assetPack.description
    )
}

func downloadSnapshot(_ download: BADownload) -> DownloadSnapshot {
    DownloadSnapshot(
        identifier: download.identifier,
        uniqueIdentifier: download.uniqueIdentifier,
        status: download.state.rawValue,
        priority: download.priority.rawValue,
        isEssential: download.isEssential,
        isURLDownload: download is BAURLDownload
    )
}

func progressSnapshot(_ progress: Progress) -> ProgressSnapshot {
    ProgressSnapshot(
        completedUnitCount: progress.completedUnitCount,
        totalUnitCount: progress.totalUnitCount,
        fractionCompleted: progress.fractionCompleted,
        localizedDescription: progress.localizedDescription
    )
}

func extensionInfoSnapshot(_ info: BAAppExtensionInfo) -> ExtensionInfoSnapshot {
    ExtensionInfoSnapshot(
        restrictedDownloadSizeRemaining: info.restrictedDownloadSizeRemaining,
        restrictedEssentialDownloadSizeRemaining: info.restrictedEssentialDownloadSizeRemaining
    )
}

func challengeSnapshot(_ challenge: URLAuthenticationChallenge) -> AuthenticationChallengeSnapshot {
    AuthenticationChallengeSnapshot(
        host: challenge.protectionSpace.host,
        authenticationMethod: challenge.protectionSpace.authenticationMethod,
        previousFailureCount: challenge.previousFailureCount,
        proposedCredentialUser: challenge.proposedCredential?.user,
        proposedCredentialHasPassword: challenge.proposedCredential?.hasPassword ?? false
    )
}

func statusUpdatePayload(_ update: AssetPackManager.DownloadStatusUpdate) -> DownloadStatusUpdatePayload {
    switch update {
    case let .began(assetPack):
        return DownloadStatusUpdatePayload(
            kind: "began",
            assetPack: assetPackSnapshot(assetPack),
            progress: nil,
            error: nil
        )
    case let .paused(assetPack):
        return DownloadStatusUpdatePayload(
            kind: "paused",
            assetPack: assetPackSnapshot(assetPack),
            progress: nil,
            error: nil
        )
    case let .downloading(assetPack, progress):
        return DownloadStatusUpdatePayload(
            kind: "downloading",
            assetPack: assetPackSnapshot(assetPack),
            progress: progressSnapshot(progress),
            error: nil
        )
    case let .finished(assetPack):
        return DownloadStatusUpdatePayload(
            kind: "finished",
            assetPack: assetPackSnapshot(assetPack),
            progress: nil,
            error: nil
        )
    case let .failed(assetPack, error):
        return DownloadStatusUpdatePayload(
            kind: "failed",
            assetPack: assetPackSnapshot(assetPack),
            progress: nil,
            error: errorPayload(error)
        )
    @unknown default:
        return DownloadStatusUpdatePayload(
            kind: "unknown",
            assetPack: AssetPackSnapshot(id: "", downloadSize: 0, version: 0, description: ""),
            progress: nil,
            error: nil
        )
    }
}

func sortedAssetPacks(_ assetPacks: some Sequence<AssetPack>) -> [AssetPack] {
    assetPacks.sorted { lhs, rhs in
        if lhs.id == rhs.id {
            return lhs.version < rhs.version
        }
        return lhs.id < rhs.id
    }
}

func sortedDownloads(_ downloads: some Sequence<BADownload>) -> [BADownload] {
    downloads.sorted { lhs, rhs in
        if lhs.identifier == rhs.identifier {
            return lhs.uniqueIdentifier < rhs.uniqueIdentifier
        }
        return lhs.identifier < rhs.identifier
    }
}

@_cdecl("ba_string_free")
public func ba_string_free(_ string: UnsafeMutablePointer<CChar>?) {
    guard let string else { return }
    free(string)
}

@_cdecl("ba_bytes_free")
public func ba_bytes_free(_ bytes: UnsafeMutableRawPointer?) {
    bytes?.deallocate()
}

@_cdecl("ba_object_release")
public func ba_object_release(_ ptr: UnsafeMutableRawPointer?) {
    guard let ptr else { return }
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

@_cdecl("ba_object_retain")
public func ba_object_retain(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    let object = Unmanaged<AnyObject>.fromOpaque(ptr).takeUnretainedValue()
    return Unmanaged.passRetained(object).toOpaque()
}

@_cdecl("ba_downloader_priority_min")
public func ba_downloader_priority_min() -> Int {
    BADownload.Priority.min.rawValue
}

@_cdecl("ba_downloader_priority_default")
public func ba_downloader_priority_default() -> Int {
    BADownload.Priority.default.rawValue
}

@_cdecl("ba_downloader_priority_max")
public func ba_downloader_priority_max() -> Int {
    BADownload.Priority.max.rawValue
}

@_cdecl("ba_app_extension_info_snapshot_json")
public func ba_app_extension_info_snapshot_json(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr else {
        return ffiString("{}")
    }
    let snapshot = extensionInfoSnapshot(extensionInfo(from: ptr))
    return ffiString((try? bridgeJSON(snapshot)) ?? "{}")
}
