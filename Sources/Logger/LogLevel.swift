import ArgumentParser

/// Logger log level
enum LogLevel {
	case debug
	case disabled
	case error
	case info
	case warning
}

extension LogLevel: Comparable, Equatable, ExpressibleByArgument {
	/// Return log level as integer
	var integerValue: Int {
		switch self {
		case .disabled:
			return 0

		case .error:
			return 1

		case .warning:
			return 2

		case .info:
			return 3

		case .debug:
			return 4
		}
	}

	init?(argument: String) {
		switch argument.lowercased() {
		case "debug":
			self = .debug

		case "disabled":
			self = .disabled

		case "error":
			self = .error

		case "info":
			self = .info

		case "warning":
			self = .warning

		default:
			return nil
		}
	}

	static func < (lhs: LogLevel, rhs: LogLevel) -> Bool {
        lhs.integerValue < rhs.integerValue
    }

	static func == (lhs: LogLevel, rhs: LogLevel) -> Bool {
		lhs.integerValue == rhs.integerValue
	}
}
