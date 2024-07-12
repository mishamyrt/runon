import Foundation
import Yams

struct ConfigHandler {
	let config: Config

	var groupMap: GroupMap {
		config.groupMap
	}

	var actionSources: [String] {
        Array(config.actionMap.keys)
    }

    init (with config: Config) {
		self.config = config
	}

    func findAction(source: String, kind: String, target: String?) -> Action? {
        guard let sourceAction = config.actionMap[source] else {
            return nil
        }

        for action in sourceAction where action.kind == kind {
			if target != nil {
				if action.target != nil && action.target == target {
					return action
				}
				continue
			}
            return action
        }
        return nil
    }
}
