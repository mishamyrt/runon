protocol SourceEventProvider: EventProvider {
    var name: String { get }
    func handle()
}

extension SourceEventProvider {
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
