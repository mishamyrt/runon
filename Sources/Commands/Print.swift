import ArgumentParser

extension RunIf {
    struct Print: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: """
                Starts the observer in a special mode that does not execute commands, \
                but prints all supported events.
                """.trimmingCharacters(in: [" "])
            )

        mutating func run() throws {
            let observer = EventObserver(sources: sources)
            print("printing all supported events")
            observer.runLoop()
        }
    }
}
