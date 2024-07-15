import ArgumentParser
import Foundation
import Shellac

extension RunOn {
    struct Service: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                abstract: "Control launchd service status.",
                subcommands: [ServiceStart.self, ServiceStop.self, ServiceLog.self]
            )

        mutating func run() throws {
			var message = "service is "
			if LaunchControl.isRunning(kLaunchAgent) {
				message += "running"
			} else {
				message += "not running"
			}
			print(message)
        }
    }
}

extension RunOn.Service {
    struct ServiceStart: ParsableCommand {
        static var configuration =
            CommandConfiguration(
				commandName: "start",
				abstract: "Start background service."
			)

        mutating func run() throws {
			if LaunchControl.isRunning(kLaunchAgent) {
				print("service is already running")
				return
			}
			try LaunchControl.bootstrap(kLaunchAgent)
			print("service started")
        }
    }

	struct ServiceStop: ParsableCommand {
        static var configuration =
            CommandConfiguration(
				commandName: "stop",
				abstract: "Stop background service."
			)

        mutating func run() throws {
			if !LaunchControl.isRunning(kLaunchAgent) {
				print("service is not running")
				return
			}
			try LaunchControl.bootOut(kLaunchAgent)
			print("service stopped")
        }
    }

	struct ServiceLog: ParsableCommand {
        static var configuration =
            CommandConfiguration(
				commandName: "log",
				abstract: "Print service log path."
			)

        mutating func run() throws {
			print(kLaunchAgentLogPath)
        }
    }
}
