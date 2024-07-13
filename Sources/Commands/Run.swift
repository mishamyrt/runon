import ArgumentParser
import Foundation

extension RunOn {
    struct Run: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Start event observer.")

        @Option(
            name: .shortAndLong,
            help: "Logging level."
        )
        var log = LogLevel.error
        @Option(
            name: [.customLong("config"), .customShort("c")],
            help: "Configuration file path."
        )
        var configPath: String?

        var configUrl: URL? {
            configPath.map { URL(filePath: $0) }
        }

        mutating func run() throws {
			logger.setLevel(log)
            let config = try Config(fromContentsOf: configUrl)
			let handler = ConfigHandler(with: config)
            let activeSources = sources.filter { source in
                handler.actionSources.contains(source.name)
            }
            let runner = ActionRunner(with: handler)
            let observer = EventObserver(activeSources)
            observer.listener = runner
            let sourceNames = activeSources.map { source in source.name.cyan }
            logger.info("observed sources: \(sourceNames.joined(separator: ", "))")
            observer.runLoop()
        }
    }
}
