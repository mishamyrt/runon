import ArgumentParser
import Foundation

extension RunIf {
    struct Autostart: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: "Controls autostart status.",
                subcommands: [Enable.self, Disable.self]
            )

        mutating func run() throws {
            printAutostartStatus(enabled: LoginItem.exists)
        }
    }

    static func printAutostartStatus(enabled: Bool) {
        var message = "Autostart is "
        if enabled {
            message += "enabled"
        } else {
            message += "disabled"
        }
        print(message)
    }
}

extension RunIf.Autostart {
    struct Enable: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Enable autostart.")

        mutating func run() throws {
            if !LoginItem.exists {
                LoginItem.create()
            }
            RunIf.printAutostartStatus(enabled: true)
        }
    }

    struct Disable: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Disable autostart.")

        mutating func run() throws {
            if LoginItem.exists {
                try LoginItem.remove()
            }
            RunIf.printAutostartStatus(enabled: false)
        }
    }
}
