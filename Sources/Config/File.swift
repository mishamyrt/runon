import Foundation
import Yams

class ConfigLoader {
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

    static func read(handlersOf: String?) -> [String: Array<Handler>]? {
       let contents = self.read(
            contentsOf: handlersOf == nil
                ? self.defaultPath
                : URL(filePath: handlersOf!)
        )
        if contents == nil {
            return nil
        }
        let items = load(yaml: contents!)
        if items == nil {
            return nil
        }
        return parseHandlers(of: items!)
    }

    static func load(yaml: String) -> [[AnyHashable: String]]? {
        do {
            let handlers = try Yams.load(yaml: yaml) as? [[AnyHashable: String]]
            return handlers
        } catch {
            return nil
        }
    }

    static func parseHandlers(of: [[AnyHashable: String]]) -> [String: Array<Handler>]? {
        var config: [String: Array<Handler>] = [:]
        for handler in of {
            let condition = handler["if"]
            let command = handler["run"]
            let target = handler["with"]
            if condition == nil || command == nil {
                return nil
            }
            let conditionParts = condition!.components(separatedBy: ":")
            if conditionParts.count != 2 {
                return nil
            }
            let provider = conditionParts[0]
            let event = conditionParts[1]
            let handler = Handler(
                kind: event,
                target: target,
                command: command!
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