import ArgumentParser
import ServiceManagement

let kAppId = "co.myrt.runon"
var logger = Logger(config: .init(
	level: .error,
	showTimestamp: true
))

@main
struct RunOn: ParsableCommand {
    static let sources: [EventSource] = [
        ScreenSource(),
        AudioSource(),
        ApplicationSource()
    ]

    static var configuration = CommandConfiguration(
        commandName: "runon",
        abstract: "A utility for automating actions on system events.",
        discussion: "VERSION: \(kAppVersion) (\(kBuildCommit))",
        version: kAppVersion,
        subcommands: [Run.self, Autostart.self, Print.self],
        defaultSubcommand: Run.self
    )
}
