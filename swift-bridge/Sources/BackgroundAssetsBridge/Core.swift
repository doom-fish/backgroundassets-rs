import BackgroundAssets
import ExtensionFoundation
import Foundation
import Security
import System

let unavailableMessage = "The managed asset-pack API requires macOS 26.0 or newer"

private func infoNumberIsNonNegative(_ value: Any?) -> Bool {
    guard let number = value as? NSNumber else { return false }
    return number.int64Value >= 0
}

let downloadManagerUnavailableReason: String? = {
    guard let bundleIdentifier = Bundle.main.bundleIdentifier, !bundleIdentifier.isEmpty else {
        return "BADownloadManager requires the process to have a bundle identifier"
    }
    guard Bundle.main.bundleURL.pathExtension != "appex" else { return nil }
    guard let info = Bundle.main.infoDictionary else {
        return "BADownloadManager requires the app to have an Info.plist"
    }
    guard let restrictions = info["BAInitialDownloadRestrictions"] as? [String: Any] else {
        return "the app's Info.plist must contain a BAInitialDownloadRestrictions dictionary"
    }
    guard let domains = restrictions["BADownloadDomainAllowList"] as? [Any],
          !domains.isEmpty,
          domains.allSatisfy({ ($0 as? String)?.isEmpty == false })
    else {
        return "BAInitialDownloadRestrictions must contain a BADownloadDomainAllowList array of at least one domain"
    }
    guard infoNumberIsNonNegative(restrictions["BADownloadAllowance"]) else {
        return "BAInitialDownloadRestrictions must contain a BADownloadAllowance number that is 0 or greater"
    }
    if #available(macOS 13.3, *), !infoNumberIsNonNegative(restrictions["BAEssentialDownloadAllowance"]) {
        return "BAInitialDownloadRestrictions must contain a BAEssentialDownloadAllowance number that is 0 or greater"
    }
    guard let manifestURL = info["BAManifestURL"] as? String,
          URL(string: manifestURL)?.scheme?.lowercased() == "https"
    else {
        return "the app's Info.plist must contain a BAManifestURL string with an https URL"
    }
    return nil
}()

private func processTeamIdentifier() -> String? {
    var code: SecCode?
    guard SecCodeCopySelf([], &code) == errSecSuccess, let code else { return nil }
    var staticCode: SecStaticCode?
    guard SecCodeCopyStaticCode(code, [], &staticCode) == errSecSuccess, let staticCode else { return nil }
    var information: CFDictionary?
    guard SecCodeCopySigningInformation(
        staticCode,
        SecCSFlags(rawValue: kSecCSSigningInformation),
        &information
    ) == errSecSuccess,
        let information = information as? [String: Any]
    else {
        return nil
    }
    return information[kSecCodeInfoTeamIdentifier as String] as? String
}

let assetPackManagerUnavailableReason: String? = {
    guard #available(macOS 26.0, *) else { return unavailableMessage }
    guard let bundleIdentifier = Bundle.main.bundleIdentifier, !bundleIdentifier.isEmpty else {
        return "AssetPackManager requires the main bundle to have a bundle identifier"
    }
    guard processTeamIdentifier() != nil else {
        return "AssetPackManager requires the process to be signed with a team identifier"
    }
    guard let appGroupID = Bundle.main.object(forInfoDictionaryKey: "BAAppGroupID") as? String,
          !appGroupID.isEmpty
    else {
        return "AssetPackManager requires a BAAppGroupID string in the main bundle's Info.plist"
    }
    guard UserDefaults(suiteName: appGroupID) != nil else {
        return "AssetPackManager requires the user defaults of the app group \(appGroupID)"
    }
    return nil
}()

final class DownloadsRequestScope: @unchecked Sendable {
    static let shared = DownloadsRequestScope()

    private let lock = NSLock()
    private var depth = 0

    func enter() {
        lock.lock()
        depth += 1
        lock.unlock()
    }

    func leave() {
        lock.lock()
        depth -= 1
        lock.unlock()
    }

    var isActive: Bool {
        lock.lock()
        defer { lock.unlock() }
        return depth > 0
    }
}

func downloadSchedulingRejection(_ download: BADownload, operation: String) -> String? {
    if DownloadsRequestScope.shared.isActive {
        return "\(operation) can't be called while the downloader extension is answering downloads(for:); return the downloads instead"
    }
    if #available(macOS 13.3, *), download.isEssential {
        return "\(operation) doesn't accept essential downloads; call removing_essential() first"
    }
    return nil
}

public typealias ContextReferenceCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

final class RustContext: @unchecked Sendable {
    let pointer: UnsafeMutableRawPointer
    private let release: ContextReferenceCallback

    init(
        _ pointer: UnsafeMutableRawPointer,
        retain: ContextReferenceCallback,
        release: @escaping ContextReferenceCallback
    ) {
        retain(pointer)
        self.pointer = pointer
        self.release = release
    }

    deinit {
        release(pointer)
    }
}

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

@available(macOS 26.0, *)
final class AssetPackBox {
    let value: AssetPack

    init(_ value: AssetPack) {
        self.value = value
    }
}

@available(macOS 26.0, *)
final class ManifestBox {
    let value: AssetPackManifest

    init(_ value: AssetPackManifest) {
        self.value = value
    }
}

@available(macOS 26.0, *)
final class ManagerBox {
    let value: AssetPackManager

    init(_ value: AssetPackManager) {
        self.value = value
    }
}

@available(macOS 26.0, *)
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
        messageErrorString(message).withCString { callback(nil, $0, context) }
    }

    func fail(error: any Error) {
        errorString(error).withCString { callback(nil, $0, context) }
    }
}

final class StreamCallbackBox: @unchecked Sendable {
    private let callback: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, Bool) -> Void
    private let context: RustContext

    init(
        callback: @escaping @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, Bool) -> Void,
        context: RustContext
    ) {
        self.callback = callback
        self.context = context
    }

    func push(json: String) {
        json.withCString { callback(context.pointer, $0, false) }
    }

    func finish() {
        callback(context.pointer, nil, true)
    }
}

final class ExclusiveControlJob: @unchecked Sendable {
    private let callback: @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void
    private let lock = NSLock()
    private var job: UnsafeMutableRawPointer?

    init(
        callback: @escaping @convention(c) (UnsafeMutableRawPointer?, Bool, UnsafePointer<CChar>?) -> Void,
        job: UnsafeMutableRawPointer
    ) {
        self.callback = callback
        self.job = job
    }

    deinit {
        if let job {
            messageErrorString("the exclusive-control handler was released without being called")
                .withCString { callback(job, false, $0) }
        }
    }

    func run(acquiredLock: Bool, error: (any Error)?) {
        lock.lock()
        let job = self.job
        self.job = nil
        lock.unlock()
        guard let job else { return }
        if let error {
            errorString(error).withCString { callback(job, acquiredLock, $0) }
        } else {
            callback(job, acquiredLock, nil)
        }
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
@available(macOS 26.0, *)
func assetPack(from ptr: UnsafeMutableRawPointer) -> AssetPack {
    borrowed(ptr, as: AssetPackBox.self).value
}

@inline(__always)
@available(macOS 26.0, *)
func manifest(from ptr: UnsafeMutableRawPointer) -> AssetPackManifest {
    borrowed(ptr, as: ManifestBox.self).value
}

@inline(__always)
@available(macOS 26.0, *)
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
    encoder.outputFormatting = [.sortedKeys]
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
    if #available(macOS 26.0, *), let managedError = error as? ManagedBackgroundAssetsError {
        let nsError = managedError as NSError
        switch managedError {
        case let .assetPackNotFound(withID: assetPackID):
            return BridgeErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                message: nsError.localizedDescription,
                assetPackID: assetPackID,
                filePath: nil
            )
        case let .fileNotFound(at: filePath):
            return BridgeErrorPayload(
                domain: nsError.domain,
                code: nsError.code,
                message: nsError.localizedDescription,
                assetPackID: nil,
                filePath: String(describing: filePath)
            )
        @unknown default:
            break
        }
    }

    let nsError = error as NSError
    let assetPackID =
        (nsError.userInfo["BAAssetPackIdentifierErrorKey"] as? String)
        ?? (nsError.userInfo["assetPackID"] as? String)
        ?? (nsError.userInfo["assetPackIdentifier"] as? String)
    return BridgeErrorPayload(
        domain: nsError.domain,
        code: nsError.code,
        message: nsError.localizedDescription,
        assetPackID: assetPackID,
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

func urlFromRawPath(_ rawPath: String) -> URL? {
    rawPath.hasPrefix("file:") ? URL(string: rawPath) : URL(fileURLWithPath: rawPath)
}

@available(macOS 26.0, *)
func assetPackSnapshot(_ assetPack: AssetPack) -> AssetPackSnapshot {
    AssetPackSnapshot(
        id: assetPack.id,
        downloadSize: assetPack.downloadSize,
        version: assetPack.version,
        description: assetPack.description
    )
}

func downloadSnapshot(_ download: BADownload) -> DownloadSnapshot {
    let isEssential: Bool
    if #available(macOS 13.3, *) {
        isEssential = download.isEssential
    } else {
        isEssential = false
    }
    return DownloadSnapshot(
        identifier: download.identifier,
        uniqueIdentifier: download.uniqueIdentifier,
        status: download.state.rawValue,
        priority: download.priority.rawValue,
        isEssential: isEssential,
        isURLDownload: download is BAURLDownload
    )
}

func progressSnapshot(_ progress: Progress) -> ProgressSnapshot {
    let fractionCompleted = progress.fractionCompleted
    return ProgressSnapshot(
        completedUnitCount: progress.completedUnitCount,
        totalUnitCount: progress.totalUnitCount,
        fractionCompleted: fractionCompleted.isFinite ? fractionCompleted : 0,
        localizedDescription: progress.localizedDescription
    )
}

func extensionInfoSnapshot(_ info: BAAppExtensionInfo) -> ExtensionInfoSnapshot {
    let restrictedEssentialDownloadSizeRemaining: Int?
    if #available(macOS 13.3, *) {
        restrictedEssentialDownloadSizeRemaining = info.restrictedEssentialDownloadSizeRemaining
    } else {
        restrictedEssentialDownloadSizeRemaining = nil
    }
    return ExtensionInfoSnapshot(
        restrictedDownloadSizeRemaining: info.restrictedDownloadSizeRemaining,
        restrictedEssentialDownloadSizeRemaining: restrictedEssentialDownloadSizeRemaining
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

@available(macOS 26.0, *)
func statusUpdatePayload(_ update: AssetPackManager.DownloadStatusUpdate) -> DownloadStatusUpdatePayload? {
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
        return nil
    }
}

@available(macOS 26.0, *)
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
