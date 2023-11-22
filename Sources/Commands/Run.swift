import ArgumentParser

extension RunIf {
    struct Run: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Start event observer.")

        @Option(
            name: [.customLong("config"), .customShort("c")],
            help: "Configuration file path."
        )
        var configPath: String?

        mutating func run() throws {
            let sources: [EventSource] = [
                ScreenSource(),
                AudioSource()
            ]
            let config = ConfigLoader.read(handlersOf: configPath)
            if config == nil {
                throw ValidationError("Can't open config file.")
            }
            let activeSources = sources.filter { source in
                config!.keys.contains(source.name)
            }
            let runner = CommandRunner(with: config!)
            let observer = EventObserver(sources: activeSources)
            observer.listener = runner
            print("starting with \(activeSources.count) event sources")
            observer.runLoop()
        }
    }
}
