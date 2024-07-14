import Darwin.C
import Foundation
import Rainbow

struct Logger {
	var prefix: String?
	private var out: UnsafeMutablePointer<FILE>

	var config: LogConfig
	var level: LogLevel {
		get { config.level }
		set { config.level = newValue }
	}
	var showTimestamp: Bool {
		get { config.showTimestamp }
		set { config.showTimestamp = newValue }
	}

	init(
		config: LogConfig? = nil,
		out: UnsafeMutablePointer<FILE> = stdout,
		prefix: String? = nil
	) {
		if let config = config {
			self.config = config
		} else {
			self.config = LogConfig(
				level: .error,
				showTimestamp: true
			)
		}
		self.out = out
		self.prefix = prefix
	}

	mutating func child(prefix: String) -> Self {
		let childPrefix: String
		if let currentPrefix = self.prefix {
			childPrefix = "\(currentPrefix) \(prefix)"
		} else {
			childPrefix = prefix
		}

		return Self(
			config: config,
			out: out,
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
			line += prefix.dim + " "
		}
		line += message
		fputs(line + "\n", out)
		fflush(out)
	}

	/// Print information message to stdout.
    func info(_ message: String) {
		if level >= .info {
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
		if level >= .warning {
			self.print(message.yellow)
		}
	}

	/// Print error message to stdout.
    func error(_ message: String) {
		if level >= .error {
			self.print(message.red)
		}
	}

	private func timestamp() -> String {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return dateFormatter.string(from: Date())
    }
}
