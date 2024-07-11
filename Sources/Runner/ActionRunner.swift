import Foundation
import Shellac

class ActionRunner: EventListener {
    let config: Config
    var queues: [String: ActionQueue] = [:]

    init(with config: Config) {
        self.config = config
        for (name, group) in config.groups {
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
        guard let action = config.find(
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
