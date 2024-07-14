import Darwin.C
import Foundation
import Rainbow

struct Logger {
	var level: LogLevel
	var out: UnsafeMutablePointer<FILE>
	var showTimestamp: Bool
	var prefix: String?

	init(
		level: LogLevel = .error,
		out: UnsafeMutablePointer<FILE> = stdout,
		showTimestamp: Bool = true,
		prefix: String? = nil
	) {
		self.level = level
		self.out = out
		self.showTimestamp = showTimestamp
		self.prefix = prefix
	}

	mutating func setLevel(_ level: LogLevel) {
		self.level = level
	}

	func child(prefix: String) -> Self {
		let childPrefix: String
		if let currentPrefix = self.prefix {
			childPrefix = "\(currentPrefix): \(prefix)"
		} else {
			childPrefix = prefix
		}

		return Self(
			level: level,
			out: out,
			showTimestamp: showTimestamp,
			prefix: childPrefix
		)
	}

	/// Print message to stdout.
	/// This function is uses low level API to print correct output colors.
    func print(_ message: String) {
		var line = ""
		if showTimestamp {
			line += timestamp().dim + " "
		}
		if let prefix = prefix {
			line += prefix + ": "
		}
		line += message
		fputs(line + "\n", out)
		fflush(out)
	}

	/// Print information message to stdout.
    func info(_ message: String) {
		if level >= LogLevel.info {
			self.print(message)
		}
    }

	/// Print debug message to stdout.
    func debug(_ message: String) {
        if level == .debug {
            self.print(message)
        }
    }

	/// Print warning message to stdout.
    func warning(_ message: String) {
		if level >= LogLevel.warning {
			self.print(message.yellow)
		}
	}

	/// Print error message to stdout.
    func error(_ message: String) {
		if level >= LogLevel.error {
			self.print(message.red)
		}
	}

	private func timestamp() -> String {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return dateFormatter.string(from: Date())
    }
}
