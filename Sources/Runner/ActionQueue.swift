import Foundation
import Shellac

class ActionQueue {
    var name: String
    var isRunning = false
    var interval: TimeInterval?

    init(name: String, interval: TimeInterval? = nil) {
        self.name = name
        self.interval = interval
    }

    func run(_ action: Action) {
        guard let debounceInterval = self.interval else {
            self.runAction(action)
            return
        }
        if isRunning {
            logger.debug("debounced action \(formatMessage(action))")
            return
        }
        isRunning = true
		logger.debug("lock \(self.name.magenta) queue")
        self.runAction(action)
        DispatchQueue.main.asyncAfter(deadline: .now() + debounceInterval) {
            logger.debug("unlock \(self.name.magenta) queue")
            self.isRunning = false
        }
    }

    private func formatMessage(_ action: Action) -> String {
        let event = "\(action.source.cyan):\(action.kind.blue)"
        guard let target = action.target else {
            return event
        }
        return "on \(name.magenta) - \(event) with \(target.yellow)"
    }

    private func runAction(_ action: Action) {
        do {
            logger.info("running \(formatMessage(action)) action")
            let output = try shell(with: action.commands, timeout: action.timeout)
            logger.info("command successfully finished".green)
            if !output.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                logger.debug("output:\n\(output)")
            }
        } catch {
            if let error = error as? ShellError {
                logger.error("The process exited with a non-zero status code: \(error.code).")
                if !error.error.isEmpty {
                    logger.error("Message: \(error.error)")
                }
            } else {
                logger.error(String(describing: error))
            }
        }
    }
}
