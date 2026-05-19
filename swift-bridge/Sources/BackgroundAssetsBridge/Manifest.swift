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

    do {
        let manifest = try AssetPackManifest(
            contentsOf: urlFromRawPath(String(cString: rawURL)),
            appGroupID: String(cString: appGroupID)
        )
        return retained(ManifestBox(manifest))
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

    do {
        let data = Data(bytes: bytes, count: length)
        let manifest = try AssetPackManifest(
            from: data,
            appGroupID: String(cString: appGroupID)
        )
        return retained(ManifestBox(manifest))
    } catch {
        writeErrorOut(errorOut, error)
        return nil
    }
}

@_cdecl("ba_manifest_description")
public func ba_manifest_description(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr else { return nil }
    return ffiString(manifest(from: ptr).description)
}

@_cdecl("ba_manifest_asset_packs")
public func ba_manifest_asset_packs(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    let assetPacks = sortedAssetPacks(manifest(from: ptr).assetPacks)
    return retained(AssetPackArrayBox(assetPacks))
}

@_cdecl("ba_manifest_all_downloads")
public func ba_manifest_all_downloads(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    let downloads = sortedDownloads(manifest(from: ptr).allDownloads(for: nil))
    return retained(DownloadArrayBox(downloads))
}

@_cdecl("ba_manifest_all_downloads_for_request")
public func ba_manifest_all_downloads_for_request(
    _ ptr: UnsafeMutableRawPointer?,
    _ request: Int
) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    let downloads = sortedDownloads(
        manifest(from: ptr).allDownloads(for: BAContentRequest(rawValue: request))
    )
    return retained(DownloadArrayBox(downloads))
}
