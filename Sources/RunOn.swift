import ArgumentParser
import ServiceManagement

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
        subcommands: [
			Status.self,
			Start.self,
			Stop.self,
			Run.self,
			Print.self,
			Log.self,
			ConfigPath.self
        ],
        defaultSubcommand: Status.self
    )
}
