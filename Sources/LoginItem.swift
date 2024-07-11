import Foundation

struct LoginItem {
    let label: String
    let arguments: [String]
    let standardOutput: String
    let standardError: String
    var path: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library")
            .appendingPathComponent("LaunchAgents")
            .appendingPathComponent(label)
            .appendingPathExtension("plist")
    }
    var exists: Bool {
        FileManager.default.fileExists(atPath: path.path())
    }

    init(
        for label: String,
        arguments: [String],
        standardOutput: String = "/dev/null",
        standardError: String = "/dev/null"
    ) {
        self.label = label
        self.arguments = arguments
        self.standardOutput = standardOutput
        self.standardError = standardError
    }

    func write() {
        // swiftlint:disable legacy_objc_type
        let loginItem: NSDictionary = [
            "KeepAlive": true,
            "Label": label,
            "ProgramArguments": arguments,
            "StandardErrorPath": standardError,
            "StandardOutPath": standardOutput,
            "RunAtLoad": true
        ]
        // swiftlint:enable legacy_objc_type
        loginItem.write(to: path, atomically: true)
    }

    func remove() throws {
        try FileManager.default.removeItem(at: path)
    }
}
