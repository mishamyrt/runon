import Foundation
import Shellac

class ActionRunner: EventListener {
    let handler: ConfigHandler
    var queues: [String: ActionQueue] = [:]

    init(with handler: ConfigHandler) {
        self.handler = handler
        for (name, group) in handler.groupMap {
            queues[name] = ActionQueue(
				name: name,
				interval: group.debounce
			)
        }
    }

    func handle(_ event: Event) {
        guard let action = handler.findAction(
            source: event.source,
            kind: event.kind,
            target: event.target
        ) else {
            logger.debug("handler not found")
            return
        }
        queues[action.group]?.run(action)
    }
}
