import ArgumentParser

extension RunOn {
    struct Print: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: """
                Print all observed events.
                """.trimmingCharacters(in: [" "])
            )

        mutating func run() throws {
			logger.level = .debug
            let observer = EventObserver(sources)
            logger.info("printing all supported events")
            observer.runLoop()
        }
    }
}
