import ArgumentParser

extension RunOn {
    struct Run: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Start event observer.")

        @Flag(
            name: .shortAndLong,
            help: "Print more information for debugging."
        )
        var verbose = false
        @Option(
            name: [.customLong("config"), .customShort("c")],
            help: "Configuration file path."
        )
        var configPath: String?

        mutating func run() throws {
            kLogger.verbose = verbose
            guard let config = ConfigLoader.read(handlersOf: configPath) else {
                throw ValidationError("Can't open config file.")
            }
            let activeSources = sources.filter { source in
                config.keys.contains(source.name)
            }
            let runner = CommandRunner(with: config)
            let observer = EventObserver(activeSources)
            observer.listener = runner
            let sourceNames = activeSources.map { source in source.name.cyan }
            kLogger.info("observed sources: \(sourceNames.joined(separator: ", "))")
            observer.runLoop()
        }
    }
}
