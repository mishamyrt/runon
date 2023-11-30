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
            let process = ShellProcess(with: handler.command, timeout: handler.timeout)
            Logger.info("running '\(handler.command.green)'")
            try process.launch()
            Logger.info("command successfully executed".green)
        } catch {
            if let error = error as? ShellError {
                Logger.error("The process exited with a non-zero status code: \(error.code).")
                Logger.error("Message: \(error.message)")
            } else {
                Logger.error(String(describing: error))
            }
        }
    }
}
