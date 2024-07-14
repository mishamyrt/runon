/// LogConfig is used to pass configuration between sub-loggers.
class LogConfig {
	var level: LogLevel
	var showTimestamp: Bool

	init(level: LogLevel = .error, showTimestamp: Bool = true) {
		self.level = level
		self.showTimestamp = showTimestamp
	}
}
