import Foundation
import Yams

enum ConfigError: Error {
    case invalidFormat(String)
    case invalidGroup(String)
}

extension ConfigError: CustomStringConvertible {
    var description: String {
        switch self {
        case .invalidFormat(let problem):
            return "The config has format problem: \(problem)"
        case .invalidGroup(let group):
            return "Requested group is not found: '\(group)'"
        }
    }
}

let kDefaultGroup = "common"
var kDefaultConfigURL: URL {
    let homeDir = FileManager.default.homeDirectoryForCurrentUser
    return homeDir
        .appendingPathComponent(".config")
        .appendingPathComponent("runon")
        .appendingPathComponent("config")
        .appendingPathExtension("yaml")
}
var kDefaultConfigPath: String {
    kDefaultConfigURL.absoluteString.dropSchema("file")
}

struct Config {
	var actionMap: ActionMap = [:]
	var groupMap: GroupMap = [:]

	init(from config: ConfigDTO) throws {
		(actionMap, groupMap) = try Self.parseDTO(config)
    }

    init(fromContentsOf url: URL?) throws {
        let contents = try String(contentsOf: url ?? kDefaultConfigURL)
        let decoder = YAMLDecoder()
        let spec = try decoder.decode(ConfigDTO.self, from: contents)
        (actionMap, groupMap) = try Self.parseDTO(spec)
    }

	/// Parse the config file and return `actions` and `groups`
    static func parseDTO(_ config: ConfigDTO) throws -> (ActionMap, GroupMap) {
        let userGroups = config.groups ?? []
		var actionMap: ActionMap = [:]
		var groupMap: GroupMap = [kDefaultGroup: ActionGroup(debounce: 0)]
        for group in userGroups {
			let interval: TimeInterval
			if let debounce = group.debounce {
				interval = try TimeInterval(fromTimeString: debounce)
			} else {
				interval = 0
			}
            groupMap[group.name] = ActionGroup(debounce: interval)
        }

        for action in config.actions {
            let actionGroup = action.group ?? kDefaultGroup
            if groupMap[actionGroup] == nil {
                throw ConfigError.invalidGroup(actionGroup)
            }
            let childActions = try parseActions(action)
			for handler in childActions {
				if actionMap[handler.source] == nil {
					actionMap[handler.source] = [handler]
				} else {
					actionMap[handler.source]?.append(handler)
				}
			}
        }

		return (actionMap, groupMap)
    }

	/// Parse the lines and return an array of trimmed and splitted strings
    static func parseMultiline(_ lines: String) -> [String] {
		lines
			.components(separatedBy: "\n")
			.map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
			.filter { !$0.isEmpty }
    }

	/// Parse the condition and return an Action
	static func parseConditionAction(
		_ condition: String,
		_ target: String?,
		_ action: ActionDTO
	) throws -> Action {
		let conditionParts = condition.components(separatedBy: ":")
		if conditionParts.count != 2 {
            throw ConfigError.invalidFormat("invalid parts count on '\(condition)'")
        }
        guard
            let provider = conditionParts[safe: 0],
            let event = conditionParts[safe: 1] else {
                throw ConfigError.invalidFormat("invalid parts on '\(action.on)'")
            }

        let timeoutValue = action.timeout ?? "30s"
        let timeout = try TimeInterval(fromTimeString: timeoutValue)
        let commands = parseMultiline(action.run)

        return Action(
            source: provider,
            kind: event,
            commands: commands,
            target: target,
            timeout: timeout,
            group: action.group ?? kDefaultGroup
        )
	}

	/// Parse the config and return an array of Actions
    static func parseActions(_ spec: ActionDTO) throws -> [Action] {
        let conditions = parseMultiline(spec.on)
		let targets: [String]
		if let target = spec.with {
			targets = parseMultiline(target)
		} else {
			targets = []
		}

		let actions: [Action]
		if targets.isEmpty {
			actions = try conditions.map { condition in
				try parseConditionAction(condition, nil, spec)
			}
		} else {
			actions = try conditions.flatMap { condition in
				try targets.map { target in
					try parseConditionAction(condition, target, spec)
				}
			}
		}
		return actions
    }
}
