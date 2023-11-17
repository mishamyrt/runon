import Cocoa

class EventObserver: EventListener, EventProvider {
    var listener: EventListener?

    var sources: [EventSource] = []

    init(sources: [EventSource]) {
        for var provider in sources {
            provider.listener = self
        }
        self.sources = sources
    }

    func handleEvent(with: Event) {
        print(with)
        if listener != nil {
            listener!.handleEvent(with: with)
        }
    }

    func runLoop() {
        _ = NSApplication.shared
        let ticket = Timer.publish(every: 5, on: .main, in: .common)
            .autoconnect()
            .sink { _ in
                self.run()
            }
        RunLoop.main.run()
        ticket.cancel()
    }

    func run() {
        for provider in sources {
            provider.handle()
        }
    }
}
