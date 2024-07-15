import ArgumentParser
import Foundation
import Shellac

extension RunOn {
    struct Start: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Start launchd agent.")

        mutating func run() throws {
			if !kLaunchAgent.exists {
				kLaunchAgent.write()
			}
			if LaunchControl.isRunning(kLaunchAgent) {
				print("agent is already running")
				return
			}
			try LaunchControl.bootstrap(kLaunchAgent)
			print("agent started")
        }
    }
	struct Stop: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Stop launchd agent.")

        mutating func run() throws {
			if !kLaunchAgent.exists {
				kLaunchAgent.write()
			}
			if !LaunchControl.isRunning(kLaunchAgent) {
				print("agent is not running")
				return
			}
			try LaunchControl.bootOut(kLaunchAgent)
			print("agent stopped")
        }
    }
	struct Status: ParsableCommand {
        static var configuration =
            CommandConfiguration(abstract: "Print launchd agent status.")

        mutating func run() throws {
			if !kLaunchAgent.exists {
				kLaunchAgent.write()
			}
			var message = "agent is "
			if LaunchControl.isRunning(kLaunchAgent) {
				message += "running"
			} else {
				message += "not running"
			}
			print(message)
        }
    }
	struct Log: ParsableCommand {
        static var configuration =
            CommandConfiguration(
				commandName: "log-path",
				abstract: "Print agent log path."
			)

        mutating func run() throws {
			print(kLaunchAgentLogPath)
        }
    }
}
