import Foundation
import Yams

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
let kDefaultGroup = "common"

enum ConfigError: Error {
    case invalidFormat(String)
    case invalidGroup(String)
}

extension ConfigError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .invalidFormat(let problem):
            return "The config has format problem: \(problem)"
        case .invalidGroup(let group):
            return "Requested group is not found: '\(group)'"
        }
    }
}

struct Action {
    let source: String
    let kind: String
    let commands: [String]
    let target: String?
    let timeout: TimeInterval
    let group: String
}

struct ActionGroup {
    let debounce: TimeInterval
}

struct ActionConfig: Codable {
    // swiftlint:disable identifier_name
    let on: String
    // swiftlint:enable identifier_name
    let with: String?
    let run: String
    let group: String?
    let timeout: String?
}

struct ActionGroupConfig: Codable {
    let name: String
    let debounce: String?
}

struct Specification: Codable {
    let actions: [ActionConfig]
    let groups: [ActionGroupConfig]?
}

struct Config {
    var handlersMap: [String: [Action]] = [:]
    var sources: [String] {
        Array(handlersMap.keys)
    }
    var groups: [String: ActionGroup] = [
        "common": ActionGroup(debounce: 0)
    ]
    var count: Int {
        var count = 0
        for (_, actions) in handlersMap {
            count += actions.count
        }
        return count
    }

    init(fromSpec spec: Specification) throws {
        try parseSpec(spec)
    }

    init(fromFile specFile: URL?) throws {
        let contents = try String(contentsOf: specFile ?? kDefaultConfigURL)
        let decoder = YAMLDecoder()
        let decoded = try decoder.decode(Specification.self, from: contents)
        try parseSpec(decoded)
    }

    func find(source: String, kind: String, target: String?) -> Action? {
        guard let sourceAction = handlersMap[source] else {
            return nil
        }

        for handler in sourceAction where handler.kind == kind {
			if target != nil {
				if handler.target != nil && handler.target == target {
					return handler
				}
				continue
			}
            return handler
        }
        return nil
    }

    private mutating func parseSpec(_ spec: Specification) throws {
        let userGroups = spec.groups ?? []
        for group in userGroups {
            let interval = try TimeInterval(fromTimeString: group.debounce ?? "0s")
            groups[group.name] = ActionGroup(debounce: interval)
        }

        for action in spec.actions {
            let actionGroup = action.group ?? kDefaultGroup
            if groups[actionGroup] == nil {
                throw ConfigError.invalidGroup(actionGroup)
            }
            let handlers = try parseActions(action)
			for handler in handlers {
				if handlersMap[handler.source] == nil {
					handlersMap[handler.source] = [handler]
				} else {
					handlersMap[handler.source]?.append(handler)
				}
			}
        }
    }

    private func parseCommands(_ script: String) -> [String] {
        let commands = script.components(separatedBy: "\n")
        return commands.filter { command in
            !command.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        }
    }

	private func parseConditionAction(
		_ condition: String,
		_ target: String?,
		_ action: ActionConfig
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
        let commands = parseCommands(action.run)

        return Action(
            source: provider,
            kind: event,
            commands: commands,
            target: target,
            timeout: timeout,
            group: action.group ?? kDefaultGroup
        )
	}

	private func parseMultipleOptions(_ options: String) -> [String] {
		options
			.components(separatedBy: "\n")
			.map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
			.filter { !$0.isEmpty }
	}

    private func parseActions(_ actionConfig: ActionConfig) throws -> [Action] {
        let conditions = parseMultipleOptions(actionConfig.on)
		var actions: [Action] = []
		var targets: [String] = []
		if let with = actionConfig.with {
			targets += parseMultipleOptions(with)
		}
		for condition in conditions {
			if targets.isEmpty {
				let action = try parseConditionAction(condition, nil, actionConfig)
				actions.append(action)
				continue
			}
			for target in targets {
				let action = try parseConditionAction(condition, target, actionConfig)
				actions.append(action)
			}
		}
		return actions
    }
}
