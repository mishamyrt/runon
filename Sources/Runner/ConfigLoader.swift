import Foundation
import Yams

enum ConfigLoader {
    static var defaultPath: URL {
        let homeDir = FileManager.default.homeDirectoryForCurrentUser
        return homeDir
            .appendingPathComponent(".config")
            .appendingPathComponent("runif")
            .appendingPathComponent("config")
            .appendingPathExtension("yaml")
    }

    static func read(contentsOf: URL) -> String? {
        var value = ""
        do {
            value = try String(contentsOf: contentsOf)
        } catch {
            return nil
        }
        return value
    }

    static func read(handlersOf: String?) -> [String: [Handler]]? {
        let contentOfUrl: URL
        if let handlersOf {
            contentOfUrl = URL(filePath: handlersOf)
        } else {
            contentOfUrl = defaultPath
        }

        guard
            let contents = read(contentsOf: contentOfUrl),
            let items = load(yaml: contents) else {
            return nil
        }
        return parseHandlers(from: items)
    }

    static func load(yaml: String) -> [[AnyHashable: String]]? {
        try? Yams.load(yaml: yaml) as? [[AnyHashable: String]]
    }

    static func parseHandlers(from: [[AnyHashable: String]]) -> [String: [Handler]]? {
        var config: [String: [Handler]] = [:]
        for handler in from {
            guard
                let condition = handler["on"],
                let command = handler["run"],
                let target = handler["with"] else {
                    return nil
                }
            let conditionParts = condition.components(separatedBy: ":")
            if conditionParts.count != 2 {
                return nil
            }
            guard
                let provider = conditionParts[safe: 0],
                let event = conditionParts[safe: 1] else {
                    return nil
                }

            let handler = Handler(
                kind: event,
                target: target,
                command: command
            )

            if config[provider] == nil {
                config[provider] = [handler]
            } else {
                config[provider]!.append(handler)
            }
        }
        return config
    }
}
