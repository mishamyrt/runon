import Darwin.C
import Shellac

/// launchctl swift adapter
enum LaunchControl {
	static func bootstrap(_ agent: LaunchAgent) throws {
		try shell(with: format(agent: agent, command: "bootstrap"))
	}

	static func bootOut(_ agent: LaunchAgent) throws {
		try shell(with: format(agent: agent, command: "bootout"))
	}

	static func isRunning(_ agent: LaunchAgent) -> Bool {
		do {
			let command = "launchctl print gui/\(getUserID())/co.myrt.runon"
			let output = try shell(with: command)
			return output.contains("state = running")
		} catch {
			return false
		}
	}

	private static func getUserID() -> Int {
		Int(geteuid())
	}

	private static func format(
		agent: LaunchAgent,
		command: String
	) -> String {
		"launchctl \(command) gui/\(getUserID()) '\(agent.path)'"
	}
}
