import BackgroundAssets
import Foundation

// swiftlint:disable function_parameter_count identifier_name

@_cdecl("ba_download_identifier")
public func ba_download_identifier(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr else { return nil }
    return ffiString(download(from: ptr).identifier)
}

@_cdecl("ba_download_unique_identifier")
public func ba_download_unique_identifier(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr else { return nil }
    return ffiString(download(from: ptr).uniqueIdentifier)
}

@_cdecl("ba_download_status")
public func ba_download_status(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr else { return 0 }
    return download(from: ptr).state.rawValue
}

@_cdecl("ba_download_priority")
public func ba_download_priority(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr else { return BADownload.Priority.default.rawValue }
    return download(from: ptr).priority.rawValue
}

@_cdecl("ba_download_is_essential")
public func ba_download_is_essential(_ ptr: UnsafeMutableRawPointer?) -> Bool {
    guard let ptr else { return false }
    return download(from: ptr).isEssential
}

@_cdecl("ba_download_removing_essential")
public func ba_download_removing_essential(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    return retained(download(from: ptr).removingEssential())
}

@_cdecl("ba_download_is_url_download")
public func ba_download_is_url_download(_ ptr: UnsafeMutableRawPointer?) -> Bool {
    guard let ptr else { return false }
    return download(from: ptr) is BAURLDownload
}

@_cdecl("ba_download_array_len")
public func ba_download_array_len(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr else { return 0 }
    return borrowed(ptr, as: DownloadArrayBox.self).value.count
}

@_cdecl("ba_download_array_get")
public func ba_download_array_get(
    _ ptr: UnsafeMutableRawPointer?,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    let downloads = borrowed(ptr, as: DownloadArrayBox.self).value
    guard downloads.indices.contains(index) else { return nil }
    return retained(downloads[index])
}

@_cdecl("ba_url_download_create")
public func ba_url_download_create(
    _ identifier: UnsafePointer<CChar>?,
    _ rawURL: UnsafePointer<CChar>?,
    _ method: UnsafePointer<CChar>?,
    _ headersJSON: UnsafePointer<CChar>?,
    _ fileSize: UInt64,
    _ appGroupID: UnsafePointer<CChar>?,
    _ essential: Bool,
    _ priority: Int,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let identifier, let rawURL, let appGroupID else {
        writeErrorOut(errorOut, "identifier, URL, and app-group identifier are required")
        return nil
    }

    guard let url = URL(string: String(cString: rawURL)) else {
        writeErrorOut(errorOut, "invalid URL for BAURLDownload")
        return nil
    }
    if url.scheme?.lowercased() != "https" {
        writeErrorOut(errorOut, "BAURLDownload requires an https URL")
        return nil
    }

    var request = URLRequest(url: url)
    request.httpMethod = method.map { String(cString: $0) }.flatMap { $0.isEmpty ? nil : $0 } ?? "GET"

    if let headersJSON,
       let data = String(cString: headersJSON).data(using: .utf8),
       let headers = try? JSONSerialization.jsonObject(with: data) as? [String: String] {
        for (key, value) in headers {
            request.setValue(value, forHTTPHeaderField: key)
        }
    }

    let download = BAURLDownload(
        identifier: String(cString: identifier),
        request: request,
        essential: essential,
        fileSize: Int(fileSize),
        applicationGroupIdentifier: String(cString: appGroupID),
        priority: BADownload.Priority(rawValue: priority)
    )
    return retained(download)
}

@_cdecl("ba_download_manager_shared")
public func ba_download_manager_shared() -> UnsafeMutableRawPointer? {
    retained(BADownloadManager.shared)
}

@_cdecl("ba_download_manager_schedule_download")
public func ba_download_manager_schedule_download(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ downloadPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Bool {
    guard let managerPtr, let downloadPtr else {
        writeErrorOut(errorOut, "download manager and download pointers are required")
        return false
    }

    do {
        try downloadManager(from: managerPtr).scheduleDownload(download(from: downloadPtr))
        return true
    } catch {
        writeErrorOut(errorOut, error)
        return false
    }
}

@_cdecl("ba_download_manager_start_foreground_download")
public func ba_download_manager_start_foreground_download(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ downloadPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Bool {
    guard let managerPtr, let downloadPtr else {
        writeErrorOut(errorOut, "download manager and download pointers are required")
        return false
    }

    do {
        try downloadManager(from: managerPtr).startForegroundDownload(download(from: downloadPtr))
        return true
    } catch {
        writeErrorOut(errorOut, error)
        return false
    }
}

@_cdecl("ba_download_manager_cancel_download")
public func ba_download_manager_cancel_download(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ downloadPtr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Bool {
    guard let managerPtr, let downloadPtr else {
        writeErrorOut(errorOut, "download manager and download pointers are required")
        return false
    }

    do {
        try downloadManager(from: managerPtr).cancel(download(from: downloadPtr))
        return true
    } catch {
        writeErrorOut(errorOut, error)
        return false
    }
}

@_cdecl("ba_download_manager_fetch_current_downloads_async")
public func ba_download_manager_fetch_current_downloads_async(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let managerPtr else {
        messageErrorString("download manager pointer must not be null").withCString { cb(nil, $0, ctx) }
        return
    }

    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    let manager = downloadManager(from: managerPtr)
    manager.fetchCurrentDownloads { downloads, error in
        if let error {
            callbackBox.fail(error: error)
        } else {
            callbackBox.succeed(retained(DownloadArrayBox(sortedDownloads(downloads))))
        }
    }
}

@_cdecl("ba_download_manager_with_exclusive_control_async")
public func ba_download_manager_with_exclusive_control_async(
    _ managerPtr: UnsafeMutableRawPointer?,
    _ beforeEpochSeconds: Double,
    _ hasBeforeDate: Bool,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let managerPtr else {
        messageErrorString("download manager pointer must not be null").withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = downloadManager(from: managerPtr)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    let handler: @Sendable (Bool, Error?) -> Void = { acquiredLock, error in
        if let error {
            callbackBox.fail(error: error)
        } else {
            callbackBox.succeed(string: acquiredLock ? "true" : "false")
        }
    }

    if hasBeforeDate {
        manager.withExclusiveControl(beforeDate: Date(timeIntervalSince1970: beforeEpochSeconds), perform: handler)
    } else {
        manager.withExclusiveControl(handler)
    }
}
