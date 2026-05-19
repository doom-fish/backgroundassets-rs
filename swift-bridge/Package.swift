// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "BackgroundAssetsBridge",
    platforms: [
        .macOS(.v26)
    ],
    products: [
        .library(name: "BackgroundAssetsBridge", type: .static, targets: ["BackgroundAssetsBridge"])
    ],
    targets: [
        .target(
            name: "BackgroundAssetsBridge",
            path: "Sources/BackgroundAssetsBridge",
            publicHeadersPath: "include",
            cSettings: [
                .headerSearchPath("include")
            ],
            linkerSettings: [
                .linkedFramework("BackgroundAssets"),
                .linkedFramework("Foundation"),
                .linkedFramework("ExtensionFoundation")
            ]
        )
    ]
)
