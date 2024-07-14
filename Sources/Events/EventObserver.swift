import Cocoa

class EventObserver: EventListener, EventProvider {
    var listener: EventListener?

    let sources: [EventSource]

    init(_ sources: [EventSource]) {
		self.sources = sources
        for var source in sources {
            source.listener = self
        }
    }

    func handle(_ event: Event) {
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
