struct EventLogger {
	private let logger: Logger

	init(with logger: Logger) {
		self.logger = logger
	}

	func eventReceived(_ event: Event) {
		self.logger.info("received event \(event.source.cyan):\(event.kind.blue) with \(event.target.yellow)")
	}

	func sourcesSubscribed(_ sources: [EventSource]) {
		let names = sources
			.map { source in source.name.cyan }
			.joined(separator: ", ")
		self.logger.info("subscribed to: \(names)")
	}

	func listenerIsNil() {
		self.logger.error("event skipped, because listener is not set")
	}
}

let kEventLogger = EventLogger(with: logger.child(prefix: "Events"))
