import BackgroundAssets
import Foundation

@_cdecl("ba_asset_pack_identifier")
public func ba_asset_pack_identifier(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr, #available(macOS 26.0, *) else { return nil }
    return ffiString(assetPack(from: ptr).id)
}

@_cdecl("ba_asset_pack_download_size")
public func ba_asset_pack_download_size(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr, #available(macOS 26.0, *) else { return 0 }
    return assetPack(from: ptr).downloadSize
}

@_cdecl("ba_asset_pack_version")
public func ba_asset_pack_version(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr, #available(macOS 26.0, *) else { return 0 }
    return assetPack(from: ptr).version
}

@_cdecl("ba_asset_pack_description")
public func ba_asset_pack_description(_ ptr: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let ptr, #available(macOS 26.0, *) else { return nil }
    return ffiString(assetPack(from: ptr).description)
}

@_cdecl("ba_asset_pack_user_info_copy")
public func ba_asset_pack_user_info_copy(
    _ ptr: UnsafeMutableRawPointer?,
    _ lengthOut: UnsafeMutablePointer<Int>?
) -> UnsafeMutableRawPointer? {
    guard let ptr, #available(macOS 26.0, *) else {
        lengthOut?.pointee = 0
        return nil
    }
    guard let data = assetPack(from: ptr).userInfo else {
        lengthOut?.pointee = 0
        return nil
    }
    return copyDataToHeap(data, lengthOut)
}

@_cdecl("ba_asset_pack_download")
public func ba_asset_pack_download(
    _ ptr: UnsafeMutableRawPointer?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let ptr else {
        writeErrorOut(errorOut, "asset-pack pointer must not be null")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    let box = borrowed(ptr, as: AssetPackBox.self)
    if let rejection = appGroupMembershipRejection(box.appGroupID) {
        writeErrorOut(errorOut, rejection)
        return nil
    }
    return retained(box.value.download(for: nil))
}

@_cdecl("ba_asset_pack_download_for_request")
public func ba_asset_pack_download_for_request(
    _ ptr: UnsafeMutableRawPointer?,
    _ request: Int,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let ptr else {
        writeErrorOut(errorOut, "asset-pack pointer must not be null")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }
    let box = borrowed(ptr, as: AssetPackBox.self)
    if let rejection = appGroupMembershipRejection(box.appGroupID) {
        writeErrorOut(errorOut, rejection)
        return nil
    }
    return retained(box.value.download(for: BAContentRequest(rawValue: request)))
}

@_cdecl("ba_asset_pack_array_len")
public func ba_asset_pack_array_len(_ ptr: UnsafeMutableRawPointer?) -> Int {
    guard let ptr, #available(macOS 26.0, *) else { return 0 }
    return borrowed(ptr, as: AssetPackArrayBox.self).value.count
}

@_cdecl("ba_asset_pack_array_get")
public func ba_asset_pack_array_get(
    _ ptr: UnsafeMutableRawPointer?,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    guard let ptr, #available(macOS 26.0, *) else { return nil }
    let box = borrowed(ptr, as: AssetPackArrayBox.self)
    guard box.value.indices.contains(index) else { return nil }
    return retained(AssetPackBox(box.value[index], appGroupID: box.appGroupID))
}
