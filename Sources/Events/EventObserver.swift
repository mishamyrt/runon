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

    func handleEvent(_ with: Event) {
        var message = "\(with.source.cyan) emitted \(with.kind.blue) event"
        if !with.target.isEmpty {
            message += " with \(with.target.yellow)"
        }
        kLogger.debug(message)
        guard let listener else {
            return
        }
        listener.handleEvent(with)
    }

    func subscribeSources() {
        for source in sources {
            kLogger.debug("subscribing to \(source.name.cyan)")
            source.subscribe()
        }
    }

    func runLoop() {
        subscribeSources()
        _ = NSApplication.shared
        let ticket = Timer.publish(every: 5, on: .main, in: .common)
            .autoconnect()
            .sink { _ in }
        RunLoop.main.run()
        ticket.cancel()
    }
}
