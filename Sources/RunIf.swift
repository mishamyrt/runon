import ArgumentParser
import Foundation
import Cocoa

struct Options: ParsableArguments {
    @Option(
        name: [.customLong("config"), .customShort("c")],
        help: "Configuration file path.")
    var configPath: String? = nil
}

@main
struct RunIf: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "runif",
        abstract: "A utility for automating actions on system events.",
        version: "1.0.0",
        subcommands: [Run.self],
        defaultSubcommand: Run.self
    )
    static let configError = ValidationError("Can't open config file.")
}

extension RunIf {
    struct Run: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Print the sum of the values.")

        @OptionGroup var options: Options

        mutating func run() throws {
            let config = ConfigLoader.read(handlersOf: options.configPath)
            if (config == nil) {
                throw configError
            }
            let runner = CommandRunner(with: config!)
            let observer = EventObserver(providers: [
                ScreenProvider(),
            ])
            observer.listener = runner
            observer.runLoop()
        }
    }
}