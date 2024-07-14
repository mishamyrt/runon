import Shellac

struct ActionRunnerLogger {
	private let logger: Logger

	init(with logger: Logger) {
		self.logger = logger
	}

	func actionNotFound() {
		self.logger.info("action not found, skipping")
	}

	func queueNotFound(_ name: String) {
		self.logger.error("queue not found: \(name)")
	}

	func actionSuccess(_ output: String) {
		self.logger.info("command successfully finished".green)
		if !output.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
			self.logger.debug("output: \(output)")
		}
	}

	func actionStarted(_ action: Action) {
		var message = "starting action \(action.source.cyan):\(action.kind.blue)"
		if let target = action.target {
			message += " with \(target.yellow)"
		}
		message += " on \(action.group.magenta)"
		self.logger.info(message)
	}

	func actionFailed(_ error: Error) {
		if let error = error as? ShellError {
			self.logger.error("the process exited with a non-zero status code: \(error.code).")
			if !error.error.isEmpty {
				self.logger.error("output: \(error.error)")
			}
		} else if let error = error as? ActionError {
			switch error {
			case .busy:
				self.logger.info("queue busy, skipping")
			}
		} else {
			self.logger.error(String(describing: error))
		}
	}

	func eventReceived(_ event: Event) {
		self.logger.info("received event \(event.source.cyan):\(event.kind.blue) with \(event.target.yellow)")
	}
}

let kActionLogger = ActionRunnerLogger(with: logger.child(prefix: "Runner"))
