import ArgumentParser

extension RunOn {
    struct Print: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: """
                Starts the observer in a special mode that does not execute commands, \
                but prints all supported events.
                """.trimmingCharacters(in: [" "])
            )

        mutating func run() throws {
            kLoggerVerbose = true
            let observer = EventObserver(sources)
            Logger.info("printing all supported events")
            observer.runLoop()
        }
    }
}
