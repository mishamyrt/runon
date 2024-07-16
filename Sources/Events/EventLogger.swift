struct EventLogger {
	private let logger: Logger

	init(with logger: Logger) {
		self.logger = logger
	}

	func eventReceived(_ event: Event) {
		var message = "received event \(event.source.cyan):\(event.kind.blue)"
		if let target = event.target {
			message += " with \(target.yellow)"
		}
		self.logger.info(message)
	}

	func sourcesSubscribed(_ sources: [EventSource]) {
		let names = sources
			.map { source in source.name.cyan }
			.joined(separator: ", ")
		self.logger.info("subscribed to: \(names)")
	}
}

let kEventLogger = EventLogger(with: logger.child(prefix: "Events"))
