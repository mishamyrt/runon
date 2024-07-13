import Foundation
import Shellac

class ActionQueue {
    var name: String
    var isRunning = false
    var interval: TimeInterval?

    init(forGroup name: String, after interval: TimeInterval?) {
        self.name = name
        self.interval = interval
    }

    func run(_ action: Action) {
        guard let debounceInterval = self.interval else {
            self.runAction(action)
            return
        }
        if isRunning {
            logger.info("debounced action \(formatMessage(action))")
            return
        }
        isRunning = true
        self.runAction(action)
        DispatchQueue.main.asyncAfter(deadline: .now() + debounceInterval) {
            logger.debug("unlock \(self.name.magenta)")
            self.isRunning = false
        }
    }

    private func formatMessage(_ action: Action) -> String {
        let result = "\(action.source.cyan):\(action.kind.blue)"
        guard let target = action.target else {
            return result
        }
        return "on \(name.magenta) - \(result) with \(target.yellow)"
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
