import Foundation
import Shellac

enum ActionError: Error {
	/// Queue is currently busy
	case busy
}

/// ActionGroupQueue represents a queue of actions.
/// Helps ensure that groups of commands are executed correctly
/// and allows for fine tuning of debounce.
class ActionGroupQueue {
    let name: String
    let interval: TimeInterval

	private let semaphore = DispatchSemaphore(value: 1)

    init(name: String, interval: TimeInterval? = nil) {
        self.name = name
		guard let interval = interval else {
			self.interval = 0
			return
		}
		self.interval = interval
    }

	/// Run action and return output
    func run(_ action: Action) throws -> String {
		if semaphore.wait(timeout: .now()) == .timedOut {
			throw ActionError.busy
		}
		let output = try shell(with: action.commands, timeout: action.timeout)
		if interval == 0 {
			semaphore.signal()
			return output
		}
        DispatchQueue.main.asyncAfter(deadline: .now() + interval) {
			self.semaphore.signal()
        }
		return output
    }
}
