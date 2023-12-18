import Foundation
import Yams

enum ConfigParsingError: Error {
    case invalidFormat(String)
    case invalidGroup(String)
    case invalidValue
}

extension ConfigParsingError: CustomStringConvertible {
    public var description: String {
        switch self {
        case .invalidFormat(let problem):
            return "The config has format problem: \(problem)"
        case .invalidValue:
            return "The specified item could not be found"
        case .invalidGroup(let group):
            return "Requested group is not found: '\(group)'"
        }
    }
}

private var defaultConfigPath: URL {
    let homeDir = FileManager.default.homeDirectoryForCurrentUser
    return homeDir
        .appendingPathComponent(".config")
        .appendingPathComponent("runon")
        .appendingPathComponent("config")
        .appendingPathExtension("yaml")
}

struct Config {
    var handlersMap: [String: [Action]] = [:]
    var sources: [String] {
        Array(handlersMap.keys)
    }
    var groups: [String: ActionGroup] = [
        "common": ActionGroup(debounce: 0)
    ]

    init(from path: String?) throws {
        let pathUrl = getUrl(from: path)
        let contents = try String(contentsOf: pathUrl)
        let decoder = YAMLDecoder()
        let decoded = try decoder.decode(ConfigFile.self, from: contents)
        try parseConfig(decoded)
    }

    mutating func parseConfig(_ file: ConfigFile) throws {
        let userGroups = file.groups ?? []
        for group in userGroups {
            let interval = try parseInterval(group.debounce ?? "0s")
            groups[group.name] = ActionGroup(debounce: interval)
        }

        for action in file.actions {
            let actionGroup = action.group ?? "common"
            if groups[actionGroup] == nil {
                throw ConfigParsingError.invalidGroup(actionGroup)
            }
            let handler = try parseAction(action)
            if handlersMap[handler.source] == nil {
                handlersMap[handler.source] = [handler]
            } else {
                handlersMap[handler.source]?.append(handler)
            }
        }
    }

    func findAction(source: String, kind: String, target: String?) -> Action? {
        guard let sourceAction = handlersMap[source] else {
            return nil
        }

        for handler in sourceAction where (
            handler.kind == kind && (handler.target != nil && handler.target == target)
        ) {
            return handler
        }
        return nil
    }

    private func getUrl(from path: String?) -> URL {
        guard let path else {
            return defaultConfigPath
        }
        return URL(filePath: path)
    }

    private func parseInterval(_ timeString: String) throws -> TimeInterval {
        let scanner = Scanner(string: timeString)

        guard
            let value = scanner.scanDouble(),
            let unit = scanner.scanCharacters(from: CharacterSet.letters) else {
                throw ConfigParsingError.invalidFormat("time unit is not found")
            }

        switch unit.lowercased() {
        case "ms":
            return value / 1000
        case "s":
            return value

        default:
            throw ConfigParsingError.invalidValue
        }
    }

    private func parseCommands(_ script: String) -> [String] {
        let commands = script.components(separatedBy: "\n")
        return commands.filter { command in
            !command.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        }
    }

    private func parseAction(_ action: ActionConfig) throws -> Action {
        let conditionParts = action.on.components(separatedBy: ":")
        if conditionParts.count != 2 {
            throw ConfigParsingError.invalidFormat("action format is invalid is not found at '\(action.on)'")
        }
        guard
            let provider = conditionParts[safe: 0],
            let event = conditionParts[safe: 1] else {
                throw ConfigParsingError.invalidFormat("action format is invalid is not found at '\(action.on)'")
            }

        let timeoutString = action.timeout ?? "30s"
        let timeout = try parseInterval(timeoutString)
        let commands = parseCommands(action.run)

        return Action(
            source: provider,
            kind: event,
            commands: commands,
            target: action.with,
            timeout: timeout,
            group: action.group ?? "common"
        )
    }
}
