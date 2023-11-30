import Foundation
import Yams

enum ConfigParsingError: Error {
    case invalidFormat
    case invalidValue
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
    var handlersMap: [String: [Handler]] = [:]
    var sources: [String] {
        Array(handlersMap.keys)
    }

    init(from path: String?) throws {
        let pathUrl = getUrl(from: path)
        let contents = try String(contentsOf: pathUrl)
        guard let descriptions = try Yams.load(yaml: contents) as? [[AnyHashable: String]] else {
            throw ConfigParsingError.invalidFormat
        }
        for description in descriptions {
            let handler = try parseHandler(description)
            if handlersMap[handler.source] == nil {
                handlersMap[handler.source] = [handler]
            } else {
                handlersMap[handler.source]?.append(handler)
            }
        }
    }

    func findHandler(source: String, kind: String, target: String?) -> Handler? {
        guard let sourceHandler = handlersMap[source] else {
            return nil
        }

        for handler in sourceHandler where (
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
                throw ConfigParsingError.invalidFormat
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

    private func parseHandler(_ handler: [AnyHashable: String]) throws -> Handler {
        guard
            let condition = handler["on"],
            let command = handler["run"] else {
                throw ConfigParsingError.invalidFormat
            }
        let conditionParts = condition.components(separatedBy: ":")
        if conditionParts.count != 2 {
            throw ConfigParsingError.invalidFormat
        }
        guard
            let provider = conditionParts[safe: 0],
            let event = conditionParts[safe: 1] else {
                throw ConfigParsingError.invalidFormat
            }

        let timeoutString = handler["timeout"] ?? "30s"
        let timeout = try parseInterval(timeoutString)

        return Handler(
            source: provider,
            kind: event,
            command: command,
            target: handler["with"],
            timeout: timeout
        )
    }
}
