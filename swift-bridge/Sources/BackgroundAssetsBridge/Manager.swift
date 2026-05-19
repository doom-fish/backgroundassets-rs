import BackgroundAssets
import Foundation
import System

// swiftlint:disable identifier_name

@_cdecl("ba_asset_pack_manager_shared")
public func ba_asset_pack_manager_shared() -> UnsafeMutableRawPointer? {
    if #available(macOS 26.0, *) {
        return retained(ManagerBox(AssetPackManager.shared))
    }
    return nil
}

@_cdecl("ba_asset_pack_manager_asset_pack_is_available_locally")
public func ba_asset_pack_manager_asset_pack_is_available_locally(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackID: UnsafePointer<CChar>?
) -> Bool {
    guard let ptr, let assetPackID else { return false }
    guard #available(macOS 26.4, *) else { return false }
    return manager(from: ptr).assetPackIsAvailableLocally(withID: String(cString: assetPackID))
}

@_cdecl("ba_asset_pack_manager_contents")
public func ba_asset_pack_manager_contents(
    _ ptr: UnsafeMutableRawPointer?,
    _ rawPath: UnsafePointer<CChar>?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ lengthOut: UnsafeMutablePointer<Int>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let ptr, let rawPath else {
        writeErrorOut(errorOut, "manager pointer and path are required")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }

    do {
        let data = try manager(from: ptr).contents(
            at: FilePath(String(cString: rawPath)),
            searchingInAssetPackWithID: assetPackID.map { String(cString: $0) }
        )
        return copyDataToHeap(data, lengthOut)
    } catch {
        writeErrorOut(errorOut, error)
        return nil
    }
}

@_cdecl("ba_asset_pack_manager_descriptor")
public func ba_asset_pack_manager_descriptor(
    _ ptr: UnsafeMutableRawPointer?,
    _ rawPath: UnsafePointer<CChar>?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let ptr, let rawPath else {
        writeErrorOut(errorOut, "manager pointer and path are required")
        return -1
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return -1
    }

    do {
        let descriptor = try manager(from: ptr).descriptor(
            for: FilePath(String(cString: rawPath)),
            searchingInAssetPackWithID: assetPackID.map { String(cString: $0) }
        )
        return descriptor.rawValue
    } catch {
        writeErrorOut(errorOut, error)
        return -1
    }
}

@_cdecl("ba_asset_pack_manager_url")
public func ba_asset_pack_manager_url(
    _ ptr: UnsafeMutableRawPointer?,
    _ rawPath: UnsafePointer<CChar>?,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard let ptr, let rawPath else {
        writeErrorOut(errorOut, "manager pointer and path are required")
        return nil
    }
    guard #available(macOS 26.0, *) else {
        writeErrorOut(errorOut, unavailableMessage)
        return nil
    }

    do {
        let url = try manager(from: ptr).url(for: FilePath(String(cString: rawPath)))
        return ffiString(url.absoluteString)
    } catch {
        writeErrorOut(errorOut, error)
        return nil
    }
}

@_cdecl("ba_asset_pack_manager_all_asset_packs_async")
public func ba_asset_pack_manager_all_asset_packs_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr else {
        messageErrorString("asset-pack manager pointer must not be null").withCString { cb(nil, $0, ctx) }
        return
    }

    if #available(macOS 26.0, *) {
        let manager = manager(from: ptr)
        let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
        Task.detached {
            do {
                let assetPacks = try await manager.allAssetPacks
                callbackBox.succeed(retained(AssetPackArrayBox(sortedAssetPacks(assetPacks))))
            } catch {
                callbackBox.fail(error: error)
            }
        }
    } else {
        unavailableMessage.withCString { cb(nil, $0, ctx) }
    }
}

@_cdecl("ba_asset_pack_manager_asset_pack_async")
public func ba_asset_pack_manager_asset_pack_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr, let assetPackID else {
        messageErrorString("asset-pack manager pointer and asset-pack identifier are required")
            .withCString { cb(nil, $0, ctx) }
        return
    }

    if #available(macOS 26.0, *) {
        let manager = manager(from: ptr)
        let identifier = String(cString: assetPackID)
        let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
        Task.detached {
            do {
                let assetPack = try await manager.assetPack(withID: identifier)
                callbackBox.succeed(retained(AssetPackBox(assetPack)))
            } catch {
                callbackBox.fail(error: error)
            }
        }
    } else {
        unavailableMessage.withCString { cb(nil, $0, ctx) }
    }
}

@_cdecl("ba_asset_pack_manager_status_relative_async")
public func ba_asset_pack_manager_status_relative_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackPtr: UnsafeMutableRawPointer?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr, let assetPackPtr else {
        messageErrorString("manager pointer and asset pack are required").withCString { cb(nil, $0, ctx) }
        return
    }
    guard #available(macOS 26.4, *) else {
        messageErrorString("AssetPackManager.status(relativeTo:) requires macOS 26.4 or newer")
            .withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = manager(from: ptr)
    let assetPack = assetPack(from: assetPackPtr)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    Task.detached {
        do {
            let status = try await manager.status(relativeTo: assetPack)
            callbackBox.succeed(string: String(status.rawValue))
        } catch {
            callbackBox.fail(error: error)
        }
    }
}

@_cdecl("ba_asset_pack_manager_local_status_async")
public func ba_asset_pack_manager_local_status_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr, let assetPackID else {
        messageErrorString("manager pointer and asset-pack identifier are required")
            .withCString { cb(nil, $0, ctx) }
        return
    }
    guard #available(macOS 26.4, *) else {
        messageErrorString("AssetPackManager.localStatus(ofAssetPackWithID:) requires macOS 26.4 or newer")
            .withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = manager(from: ptr)
    let identifier = String(cString: assetPackID)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    Task.detached {
        let status = await manager.localStatus(ofAssetPackWithID: identifier)
        callbackBox.succeed(string: String(status.rawValue))
    }
}

@_cdecl("ba_asset_pack_manager_ensure_local_availability_async")
public func ba_asset_pack_manager_ensure_local_availability_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackPtr: UnsafeMutableRawPointer?,
    _ requireLatestVersion: Bool,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr, let assetPackPtr else {
        messageErrorString("manager pointer and asset pack are required").withCString { cb(nil, $0, ctx) }
        return
    }
    guard #available(macOS 26.0, *) else {
        unavailableMessage.withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = manager(from: ptr)
    let assetPack = assetPack(from: assetPackPtr)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    Task.detached {
        do {
            if requireLatestVersion {
                if #available(macOS 26.4, *) {
                    try await manager.ensureLocalAvailability(of: assetPack, requireLatestVersion: true)
                } else {
                    throw NSError(
                        domain: "BackgroundAssetsBridge",
                        code: -1,
                        userInfo: [NSLocalizedDescriptionKey: "requireLatestVersion requires macOS 26.4 or newer"]
                    )
                }
            } else {
                try await manager.ensureLocalAvailability(of: assetPack)
            }

            callbackBox.succeed(string: "ok")
        } catch {
            callbackBox.fail(error: error)
        }
    }
}

@_cdecl("ba_asset_pack_manager_check_for_updates_async")
public func ba_asset_pack_manager_check_for_updates_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr else {
        messageErrorString("asset-pack manager pointer must not be null").withCString { cb(nil, $0, ctx) }
        return
    }
    guard #available(macOS 26.0, *) else {
        unavailableMessage.withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = manager(from: ptr)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    Task.detached {
        do {
            let result = try await manager.checkForUpdates()
            let payload = UpdateCheckPayload(
                updatingIDs: Array(result.updatingIDs).sorted(),
                removedIDs: Array(result.removedIDs).sorted()
            )
            let json = try bridgeJSON(payload)
            callbackBox.succeed(string: json)
        } catch {
            callbackBox.fail(error: error)
        }
    }
}

@_cdecl("ba_asset_pack_manager_remove_async")
public func ba_asset_pack_manager_remove_async(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    guard let ptr, let assetPackID else {
        messageErrorString("manager pointer and asset-pack identifier are required")
            .withCString { cb(nil, $0, ctx) }
        return
    }
    guard #available(macOS 26.0, *) else {
        unavailableMessage.withCString { cb(nil, $0, ctx) }
        return
    }

    let manager = manager(from: ptr)
    let identifier = String(cString: assetPackID)
    let callbackBox = AsyncCallbackBox(callback: cb, context: ctx)
    Task.detached {
        do {
            try await manager.remove(assetPackWithID: identifier)
            callbackBox.succeed(string: "ok")
        } catch {
            callbackBox.fail(error: error)
        }
    }
}

@_cdecl("ba_asset_pack_manager_status_updates_stream_create")
public func ba_asset_pack_manager_status_updates_stream_create(
    _ ptr: UnsafeMutableRawPointer?,
    _ assetPackID: UnsafePointer<CChar>?,
    _ ctx: UnsafeMutableRawPointer?,
    _ cb: @convention(c) (UnsafeMutableRawPointer?, UnsafeMutablePointer<CChar>?, Bool) -> Void
) -> UnsafeMutableRawPointer? {
    guard let ptr else { return nil }
    guard #available(macOS 26.0, *) else { return nil }

    let manager = manager(from: ptr)
    let assetPackIdentifier = assetPackID.map { String(cString: $0) }
    let callbackBox = StreamCallbackBox(callback: cb, context: ctx)

    let task = Task.detached {
        if let assetPackIdentifier {
            for await update in manager.statusUpdates(forAssetPackWithID: assetPackIdentifier) {
                if Task.isCancelled { break }
                if let json = try? bridgeJSON(statusUpdatePayload(update)) {
                    callbackBox.push(json: json)
                }
            }
        } else {
            let updates = await manager.statusUpdates
            for await update in updates {
                if Task.isCancelled { break }
                if let json = try? bridgeJSON(statusUpdatePayload(update)) {
                    callbackBox.push(json: json)
                }
            }
        }
        callbackBox.finish()
    }

    return retained(StatusUpdatesBridge(task))
}
