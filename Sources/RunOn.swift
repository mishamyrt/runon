import ArgumentParser
import ServiceManagement

let kAppId = "co.myrt.runon"

@main
struct RunOn: ParsableCommand {
    static let sources: [EventSource] = [
        ScreenSource(),
        AudioSource(),
        ApplicationSource()
    ]

    static var configuration = CommandConfiguration(
        commandName: "runon-daemon",
        abstract: "A utility for automating actions on system events.",
        version: "1.0.0",
        subcommands: [Run.self, Autostart.self, Print.self],
        defaultSubcommand: Run.self
    )
}
