import Cocoa

class EventObserver: EventListener, EventProvider {
    var listener: EventListener?

    var sources: [EventSource] = []

    init(_ sources: [EventSource]) {
        for var source in sources {
            source.listener = self
        }
        self.sources = sources
    }

    func handle(_ event: Event) {
        var message = "\(event.source.cyan) emitted \(event.kind.blue) event"
        if !event.target.isEmpty {
            message += " with \(event.target.yellow)"
        }
        logger.debug(message)
        guard let listener else {
            return
        }
        listener.handle(event)
    }

    func subscribeSources() {
        for source in sources {
            logger.debug("subscribing to \(source.name.cyan)")
            source.subscribe()
        }
    }

    func runLoop() {
        subscribeSources()
        _ = NSApplication.shared
        let ticket = Timer.publish(every: 60, on: .main, in: .common)
            .autoconnect()
            .sink { _ in }
        RunLoop.main.run()
        ticket.cancel()
    }
}
