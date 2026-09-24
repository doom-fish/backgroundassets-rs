import BackgroundAssets
import Foundation

@_cdecl("ba_manifest_create_from_url")
public func ba_manifest_create_from_url(
    _ rawURL: UnsafePointer<CChar>?,
    _ appGroupID: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let rawURL, let appGroupID else {
        writeErrorOut(errorOut, "manifest URL and app-group identifier are required")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    guard let url = urlFromRawPath(String(cString: rawURL)) else {
        writeErrorOut(errorOut, "manifest URL is not a valid URL")
        return nil
    }

    let group = String(cString: appGroupID)
    do {
        let manifest = try AssetPackManifest(
            contentsOf: url,
            appGroupID: group
        )
        return retained(ManifestBox(manifest, appGroupID: group))
    } catch {
        writeErrorOut(errorOut, error)
        return nil
    }
}

@_cdecl("ba_manifest_create_from_data")
public func ba_manifest_create_from_data(
    _ bytes: UnsafeRawPointer?,
    _ length: Int,
    _ appGroupID: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let bytes, let appGroupID else {
        writeErrorOut(errorOut, "manifest data and app-group identifier are required")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }

    let group = String(cString: appGroupID)
    do {
        let data = Data(bytes: bytes, count: length)
        let manifest = try AssetPackManifest(
            from: data,
            appGroupID: group
        )
        return retained(ManifestBox(manifest, appGroupID: group))
    } catch {
        writeErrorOut(errorOut, error)
        return nil
    }
}

@_cdecl("ba_manifest_description")
public func ba_manifest_description(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr, #available(macOS 26.0, *) else { return nil }
    return ffiString(manifest(from: ptr).description)
}

@_cdecl("ba_manifest_asset_packs")
public func ba_manifest_asset_packs(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr, #available(macOS 26.0, *) else { return nil }
    let box = borrowed(ptr, as: ManifestBox.self)
    return retained(AssetPackArrayBox(sortedAssetPacks(box.value.assetPacks), appGroupID: box.appGroupID))
}

@available(macOS 26.0, *)
private func manifestDownloads(
    _ ptr: UnsafeMutableRawPointer?,
    for request: BAContentRequest?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let ptr else {
        writeErrorOut(errorOut, "manifest pointer must not be null")
        return nil
    }
    let box = borrowed(ptr, as: ManifestBox.self)
    if let rejection = appGroupMembershipRejection(box.appGroupID) {
        writeErrorOut(errorOut, rejection)
        return nil
    }
    return retained(DownloadArrayBox(sortedDownloads(box.value.allDownloads(for: request))))
}

@_cdecl("ba_manifest_all_downloads")
public func ba_manifest_all_downloads(
    _ ptr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    return manifestDownloads(ptr, for: nil, errorOut)
}

@_cdecl("ba_manifest_all_downloads_for_request")
public func ba_manifest_all_downloads_for_request(
    _ ptr: UnsafeMutableRawPointer?,
    _ request: Int,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    return manifestDownloads(ptr, for: BAContentRequest(rawValue: request), errorOut)
}
