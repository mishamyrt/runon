import Foundation
import Shellac

class ActionRunner: EventListener {
    let handler: ConfigHandler
    var queues: [String: ActionQueue] = [:]

    init(with handler: ConfigHandler) {
        self.handler = handler
        for (name, group) in handler.groupMap {
            queues[name] = ActionQueue(forGroup: name, after: group.debounce)
        }
    }

    func format(handler: Action) -> String {
        let result = "\(handler.source.cyan):\(handler.kind.blue)"
        guard let target = handler.target else {
            return result
        }
        return result + " with \(target.yellow)"
    }

    func handle(_ event: Event) {
        guard let action = handler.findAction(
            source: event.source,
            kind: event.kind,
            target: event.target
        ) else {
            Logger.debug("handler not found")
            return
        }
        queues[action.group]?.run(action)
    }
}
