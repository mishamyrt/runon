import Foundation

class CommandRunner: EventListener {
    let config: Config

    init(with: Config) {
        config = with
    }

    func handle(_ event: Event) {
        guard let handler = config.findHandler(
            source: event.source,
            kind: event.kind,
            target: event.target
        ) else {
            Logger.debug("handler not found")
            return
        }
        do {
            Logger.info("running '\(handler.command.green)'")
            let output = try shell(with: handler.command, timeout: handler.timeout)
            Logger.info("command successfully finished".green)
            if !output.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                Logger.debug("output:\n\(output)")
            }
        } catch {
            if let error = error as? ShellError {
                Logger.error("The process exited with a non-zero status code: \(error.code).")
                if !error.message.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    Logger.error("Message: \(error.message)")
                }
            } else {
                Logger.error(String(describing: error))
            }
        }
    }
}
