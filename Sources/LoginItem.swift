import Foundation

let kDevNull = "/dev/null"

struct LoginItem {
    let label: String
    let arguments: [String]
    let standardOutput: String
    let standardError: String
	let keepAlive: Bool
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
        standardOutput: String = kDevNull,
        standardError: String = kDevNull,
        keepAlive: Bool = false
    ) {
        self.label = label
        self.arguments = arguments
        self.standardOutput = standardOutput
        self.standardError = standardError
		self.keepAlive = keepAlive
    }

    func write() {
        // swiftlint:disable legacy_objc_type
        let loginItem: NSDictionary = [
            "KeepAlive": keepAlive,
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
