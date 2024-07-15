import Foundation

let kAppName = "runon"
let kAppId = "co.myrt.\(kAppName)"
let kLaunchAgentLogPath = "/tmp/\(kAppId).log"
let kLaunchAgent = LaunchAgent(
	for: kAppId,
	arguments: [
		"/usr/local/bin/\(kAppName)",
		"run",
		"--config",
		kDefaultConfigPath,
		"--log",
		"info",
		"--output",
		kLaunchAgentLogPath
	],
	keepAlive: true
)
var kDefaultConfigURL: URL {
    let homeDir = FileManager.default.homeDirectoryForCurrentUser
    return homeDir
        .appendingPathComponent(".config")
        .appendingPathComponent(kAppName)
        .appendingPathComponent("config")
        .appendingPathExtension("yaml")
}
var kDefaultConfigPath: String {
    kDefaultConfigURL.absoluteString.dropSchema("file")
}
