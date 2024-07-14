import Foundation
import Shellac

typealias ActionGroupQueueMap = [String: ActionGroupQueue]

class ActionRunner: EventListener {
    let handler: ConfigHandler
    var queues: ActionGroupQueueMap = [:]

    init(with handler: ConfigHandler) {
        self.handler = handler
        for (name, group) in handler.groupMap {
            queues[name] = ActionGroupQueue(
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
            kActionLogger.actionNotFound()
            return
        }
		guard let queue = queues[action.group] else {
			kActionLogger.queueNotFound(action.group)
			return
		}
		do {
			kActionLogger.actionStarted(action)
			let output = try queue.run(action)
			kActionLogger.actionSuccess(output)
		} catch {
			kActionLogger.actionFailed(error)
		}
    }
}
