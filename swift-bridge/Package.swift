// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "BackgroundAssetsBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(name: "BackgroundAssetsBridge", type: .static, targets: ["BackgroundAssetsBridge"])
    ],
    targets: [
        .target(
            name: "BackgroundAssetsObjCBridge",
            path: "Sources/BackgroundAssetsObjCBridge",
            publicHeadersPath: "include",
            linkerSettings: [
                .linkedFramework("BackgroundAssets"),
                .linkedFramework("Foundation")
            ]
        ),
        .target(
            name: "BackgroundAssetsBridge",
            dependencies: ["BackgroundAssetsObjCBridge"],
            path: "Sources/BackgroundAssetsBridge",
            publicHeadersPath: "include",
            cSettings: [
                .headerSearchPath("include")
            ],
            linkerSettings: [
                .linkedFramework("BackgroundAssets"),
                .linkedFramework("Foundation"),
                .linkedFramework("ExtensionFoundation"),
                .linkedFramework("Security")
            ]
        )
    ]
)
