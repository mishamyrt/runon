import ArgumentParser
import Foundation

let kLoginItem = LoginItem(
	for: kAppId,
	arguments: [
		"/usr/local/bin/runon-daemon",
		"run",
		"--config",
		kDefaultConfigPath
	],
	keepAlive: true
)

extension RunOn {
    struct Autostart: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: "Controls autostart status.",
                subcommands: [Enable.self, Disable.self]
            )

        mutating func run() throws {
            printAutostartStatus(enabled: kLoginItem.exists)
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

extension RunOn.Autostart {
    struct Enable: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Enable autostart.")

        mutating func run() throws {
            if !kLoginItem.exists {
                kLoginItem.write()
            }
            RunOn.printAutostartStatus(enabled: true)
        }
    }

    struct Disable: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Disable autostart.")

        mutating func run() throws {
            if kLoginItem.exists {
                try kLoginItem.remove()
            }
            RunOn.printAutostartStatus(enabled: false)
        }
    }
}
