import Darwin.C
import Foundation
import Rainbow

struct Logger {
	var level: LogLevel
	var out: UnsafeMutablePointer<FILE>

	init(level: LogLevel, out: UnsafeMutablePointer<FILE> = stdout) {
		self.level = level
		self.out = out
	}

	mutating func setLevel(_ level: LogLevel) {
		self.level = level
	}

	/// Print message to stdout.
	/// This function is uses low level API to print correct output colors.
    func print(_ message: String) {
        let timestamp = now().dim
        fputs("\(timestamp) \(message)\n", stdout)
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

	private func now() -> String {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return dateFormatter.string(from: Date())
    }
}
