import ArgumentParser
import ServiceManagement

@main
struct RunIf: ParsableCommand {
    static let sources: [EventSource] = [
        ScreenSource(),
        AudioSource(),
        ApplicationSource()
    ]

    static var configuration = CommandConfiguration(
        commandName: "runif-daemon",
        abstract: "A utility for automating actions on system events.",
        version: "1.0.0",
        subcommands: [Run.self, Autostart.self, Print.self],
        defaultSubcommand: Run.self
    )
}
