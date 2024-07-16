struct Event {
    let source: String
    let kind: String
    let target: String?
}

protocol EventListener {
    func handle(_ event: Event)
}

protocol EventProvider {
    var listener: EventListener? { get set }
}

protocol EventSource: EventProvider {
    var name: String { get }

    func subscribe()
}

extension EventSource {
    func emit(kind: String, target: String? = nil) {
        guard let listener else {
            return
        }
        listener.handle(Event(
            source: name,
            kind: kind,
            target: target
        ))
    }
}
