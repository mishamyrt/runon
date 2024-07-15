import Darwin.C

/// LogConfig is used to pass configuration between sub-loggers.
class LogConfig {
	var level: LogLevel
	var showTimestamp: Bool
	var out: LogOutput

	init(
		level: LogLevel = .error,
		showTimestamp: Bool = true,
		out: LogOutput = kStdOutput
	) {
		self.level = level
		self.showTimestamp = showTimestamp
		self.out = out
	}
}
