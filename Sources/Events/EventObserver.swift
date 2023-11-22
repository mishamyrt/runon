import Cocoa

class EventObserver: EventListener, EventProvider {
    var listener: EventListener?

    var sources: [EventSource] = []

    init(sources: [EventSource]) {
        for var source in sources {
            source.listener = self
        }
        self.sources = sources
    }

    func handleEvent(with: Event) {
        print(with)
        if listener != nil {
            listener!.handleEvent(with: with)
        }
    }

    func subscribeSources() {
        for source in sources {
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
