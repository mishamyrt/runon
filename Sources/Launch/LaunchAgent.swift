import Foundation

let kDevNull = "/dev/null"

struct LaunchAgent {
    let bundleID: String
    let arguments: [String]
    let standardOutput: String
    let standardError: String
	let keepAlive: Bool
    var url: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library")
            .appendingPathComponent("LaunchAgents")
            .appendingPathComponent(bundleID)
            .appendingPathExtension("plist")
    }
	var path: String {
		url.absoluteString.dropSchema("file")
	}
    var exists: Bool {
        FileManager.default.fileExists(atPath: url.path())
    }

    init(
        for bundleID: String,
        arguments: [String],
        standardOutput: String = kDevNull,
        standardError: String = kDevNull,
        keepAlive: Bool = false
    ) {
        self.bundleID = bundleID
        self.arguments = arguments
        self.standardOutput = standardOutput
        self.standardError = standardError
		self.keepAlive = keepAlive
    }

    func write() {
        // swiftlint:disable legacy_objc_type
        let launchAgent: NSDictionary = [
            "KeepAlive": keepAlive,
            "Label": bundleID,
            "ProgramArguments": arguments,
            "StandardErrorPath": standardError,
            "StandardOutPath": standardOutput,
            "RunAtLoad": true
        ]
        // swiftlint:enable legacy_objc_type
        launchAgent.write(to: url, atomically: true)
    }

    func remove() throws {
        try FileManager.default.removeItem(at: url)
    }
}
