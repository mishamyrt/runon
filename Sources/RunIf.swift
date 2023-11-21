import ArgumentParser

@main
struct RunIf: ParsableCommand {
    static var configuration = CommandConfiguration(
        commandName: "runif-daemon",
        abstract: "A utility for automating actions on system events.",
        version: "1.0.0"
    )

    @Option(
        name: [.customLong("config"), .customShort("c")],
        help: "Configuration file path."
    )
    var configPath: String?

    mutating func run() throws {
        let config = ConfigLoader.read(handlersOf: configPath)
        if config == nil {
            throw ValidationError("Can't open config file.")
        }
        let runner = CommandRunner(with: config!)
        let sources: [EventSource] = [
            ScreenSource()
        ]
        let observer = EventObserver(sources: sources)
        observer.listener = runner
        print("starting with \(sources.count) event sources")
        observer.runLoop()
    }
}
