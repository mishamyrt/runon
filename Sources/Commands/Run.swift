import ArgumentParser
import Foundation

extension RunOn {
    struct Run: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Start action runner on foreground.")

        @Option(
            name: [.customLong("log"), .customShort("l")],
            help: "Logging level."
        )
        var logLevel = LogLevel.error
        @Option(
            name: [.customLong("config"), .customShort("c")],
            help: "Configuration file path."
        )
        var configPath: String?

		@Option(
            name: [.customLong("output"), .customShort("o")],
            help: "."
        )
        var output: String = "stdout"

        var configUrl: URL? {
            configPath.map { URL(filePath: $0) }
        }

        mutating func run() throws {
			logger.config.level = logLevel
			if output != "stdout" {
				logger.out = LogOutput(file: output)
			}
            let config = try Config(fromContentsOf: configUrl)
			let handler = ConfigHandler(with: config)
            let activeSources = sources.filter { source in
                handler.actionSources.contains(source.name)
            }
            let runner = ActionRunner(with: handler)
            let observer = EventObserver(activeSources)
            observer.listener = runner
            observer.runLoop()
        }
    }
}
