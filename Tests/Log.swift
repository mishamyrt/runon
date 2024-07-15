import Foundation
@testable import runon
import Testing

@Suite
struct LoggerSuite {
    @Test("check basic logger creation")
	func testBasicConstructor() {
		let logger = Logger(config: .init( level: LogLevel.debug))
		#expect(logger.level == .debug)
	}

	@Test("check logger set level")
	func testSetLevel() {
		var logger = Logger(config: .init( level: LogLevel.debug))
		#expect(logger.level == .debug)
		logger.level = .info
		#expect(logger.level == .info)
	}

	@Test("check logger print")
	func testPrint() throws {
		let stream = try CStream()
		let logger = Logger(
			config: .init(
				showTimestamp: false,
				out: stream.output
			)
		)
		logger.print("test")
		#expect(stream.content == "test")
	}

	@Test("check logger print levels")
	func testPrintLevels() throws {
		let stream = try CStream()
		var logger = Logger(
			config: .init(
				level: .error,
				showTimestamp: false,
				out: stream.output
			)
		)
		logger.warning("should not be printed")
		logger.error("should be printed")
		#expect(stream.lines.count == 1)
		logger.level = .info
		logger.debug("should not be printed")
		logger.info("should be printed")
		logger.warning("should be printed")
		#expect(stream.lines.count == 3)
		logger.level = .debug
		logger.debug("should be printed")
		logger.info("should be printed")
		logger.warning("should be printed")
		#expect(stream.lines.count == 6)
	}

	@Test("check logger prints timestamp")
	func testPrintsTimestamp() throws {
		let stream = try CStream()
		let logger = Logger(
			config: .init(out: stream.output)
		)
		logger.print("test")
		let parts = stream.lines[0].components(separatedBy: " ")
		#expect(parts[0].components(separatedBy: "-").count == 3)
		#expect(parts[1].components(separatedBy: ":") .count == 3)
		#expect(parts[2] == "test")
	}

	@Test("check logger child")
	func testChild() throws {
		let stream = try CStream()
		var logger = Logger(
			config: .init(
				showTimestamp: false,
				out: stream.output
			)
		)
		var fooLogger = logger.child(prefix: "foo")
		#expect(fooLogger.prefix == "foo")

		fooLogger.print("test")
		#expect(stream.content == "foo test")
		stream.clear()

		let barLogger = fooLogger.child(prefix: "bar")
		barLogger.print("test")
		#expect(stream.content == "foo bar test")
	}

	@Test("check logger child level")
	func testChildLevel() throws {
		var logger = Logger()
		var fooLogger = logger.child(prefix: "foo")
		let barLogger = fooLogger.child(prefix: "bar")

		#expect(logger.level == fooLogger.level)
		#expect(fooLogger.level == barLogger.level)

		barLogger.config.level = .debug
		#expect(logger.level == .debug)
	}
}
