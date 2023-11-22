import Foundation

enum LoginItem {
    private static var loginPath: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library")
            .appendingPathComponent("LaunchAgents")
            .appendingPathComponent("runif")
            .appendingPathExtension("plist")
    }

    static var exists: Bool {
        FileManager.default.fileExists(atPath: loginPath.path())
    }

    static func create() {
        // swiftlint:disable legacy_objc_type
        let loginItem: NSDictionary = [
            "KeepAlive": false,
            "Label": "co.myrt.runif",
            "ProgramArguments": [
                "/usr/local/bin/runif-daemon",
                "run"
            ],
            "StandardErrorPath": "/dev/null",
            "StandardOutPath": "/dev/null"
        ]
        // swiftlint:enable legacy_objc_type
        loginItem.write(to: loginPath, atomically: true)
    }

    static func remove() throws {
        try FileManager.default.removeItem(at: loginPath)
    }
}
