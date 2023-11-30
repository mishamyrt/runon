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
            kLogger.verbose = true
            let observer = EventObserver(sources)
            kLogger.info("printing all supported events")
            observer.runLoop()
        }
    }
}
