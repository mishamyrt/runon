struct Event {
    let source: String
    let kind: String
    let target: String
}

protocol EventListener {
    func handleEvent(with: Event)
}

protocol EventProvider {
    var listener: EventListener? { get set }
}

protocol EventSource: EventProvider {
    var name: String { get }
    func handle()
}

extension EventSource {
    func handle() {
        // Do nothing by default
    }

    func emit(kind: String, target: String) {
        if self.listener == nil {
            return
        }
        self.listener!.handleEvent(with: Event(
            source: self.name,
            kind: kind,
            target: target
        ))
    }
}
