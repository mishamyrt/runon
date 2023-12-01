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
            kLoggerVerbose = verbose
            let config = try Config(from: configPath)
            let activeSources = sources.filter { source in
                config.sources.contains(source.name)
            }
            let runner = CommandRunner(with: config)
            let observer = EventObserver(activeSources)
            observer.listener = runner
            let sourceNames = activeSources.map { source in source.name.cyan }
            Logger.info("observed sources: \(sourceNames.joined(separator: ", "))")
            observer.runLoop()
        }
    }
}
