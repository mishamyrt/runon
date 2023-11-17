import ShellOut

class CommandRunner: EventListener {
    let handlers: [String: [Handler]]

    init(with: [String: [Handler]]) {
        self.handlers = with
    }

    func handleEvent(with: Event) {
        if self.handlers[with.source] == nil {
            return
        }
        for handler in handlers[with.source]! {
            if handler.kind == with.kind && (handler.target == nil || handler.target == with.target) {
                do {
                    print("Running", handler.command)
                    try shellOut(to: handler.command)
                } catch {
                    let error = error as! ShellOutError
                    print(error.message) // Prints STDERR
                    print(error.output) // Prints STDOUT
                }
            } 
        }
    }
}