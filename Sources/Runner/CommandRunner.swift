import ShellOut

class CommandRunner: EventListener {
    let handlers: [String: [Handler]]

    init(with: [String: [Handler]]) {
        handlers = with
    }

    func handleEvent(with: Event) {
        guard let handlersWithSource = handlers[with.source] else {
            return
        }
        for handler in handlersWithSource {
            if handler.kind == with.kind, handler.target == nil || handler.target == with.target {
                do {
                    print("Running", handler.command)
                    try shellOut(to: handler.command)
                } catch {
                    if let error = error as? ShellOutError {
                        print(error.message)
                        print(error.output)
                    } else {
                        debugPrint(String(describing: error))
                    }
                }
            }
        }
    }
}
