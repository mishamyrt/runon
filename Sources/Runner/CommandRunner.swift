import ShellOut

class CommandRunner: EventListener {
    let handlers: [String: [Handler]]

    init(with: [String: [Handler]]) {
        handlers = with
    }

    func handleEvent(_ with: Event) {
        guard let handlersWithSource = handlers[with.source] else {
            return
        }
        for handler in handlersWithSource {
            if handler.kind == with.kind, handler.target == nil || handler.target == with.target {
                do {
                    kLogger.info("running '\(handler.command.green)'")
                    try shellOut(to: handler.command)
                    kLogger.info("command successfully executed")
                } catch {
                    if let error = error as? ShellOutError {
                        kLogger.warning(error.message)
                        kLogger.error(error.output)
                    } else {
                        kLogger.error(String(describing: error))
                    }
                }
            }
        }
    }
}
