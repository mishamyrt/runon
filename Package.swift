// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "RunOn",
    platforms: [
        .macOS(.v14)
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.2.0"),
        .package(url: "https://github.com/jpsim/Yams.git", from: "5.0.6"),
        .package(url: "https://github.com/rnine/SimplyCoreAudio.git", from: "4.1.1"),
        .package(url: "https://github.com/onevcat/Rainbow", .upToNextMajor(from: "4.0.0")),
        .package(url: "https://github.com/mishamyrt/swift-shellac.git", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-testing", from: "0.10.0"),
    ],
    targets: [
        .executableTarget(
            name: "runon",
            dependencies: [
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
                .product(name: "Yams", package: "Yams"),
                .product(name: "SimplyCoreAudio", package: "SimplyCoreAudio"),
                .product(name: "Rainbow", package: "Rainbow"),
                .product(name: "Shellac", package: "swift-shellac"),
            ]
        ),
        .testTarget(
            name: "runonTests",
            dependencies: [
                "runon",
                .product(name: "Testing", package: "swift-testing"),
            ]
        ),
    ]
)
