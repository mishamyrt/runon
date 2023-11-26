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

    func subscribe()
}

extension EventSource {
    func emit(kind: String, target: String) {
        guard let listener else {
            return
        }
        listener.handleEvent(with: Event(
            source: name,
            kind: kind,
            target: target
        ))
    }
}
