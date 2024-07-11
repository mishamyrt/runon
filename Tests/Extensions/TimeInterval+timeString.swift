import Testing
import Foundation
@testable import runon

@Suite struct TimeIntervalSuite {
    @Test("check single units parsing")
    func testSingle() throws {
        #expect(try TimeInterval(fromTimeString: "10s") == 10)
        #expect(try TimeInterval(fromTimeString: "10ms") == 0.01)
        #expect(try TimeInterval(fromTimeString: "10m") == 600.0)
        #expect(try TimeInterval(fromTimeString: "10h") == 36000.0)
        #expect(try TimeInterval(fromTimeString: "10d") == 864000.0)
    }

    @Test("check units combining")
    func testMultiple() throws {
        #expect(try TimeInterval(fromTimeString: "10s10ms") == 10.01)
        #expect(try TimeInterval(fromTimeString: "10m10s") == 610.0)
        #expect(try TimeInterval(fromTimeString: "10h10s5ms") == 36010.005)
    }

    @Test("check duplicated units")
    func testDuplicatedUnits() throws {
        #expect(throws: IntervalParsingError.duplicatedUnit(Substring("h"))) {
            try TimeInterval(fromTimeString: "10h10h")
        }
    }

    @Test("check invalid format")
    func testInvalidFormat() throws {
        #expect(throws: IntervalParsingError.invalidFormat) {
            try TimeInterval(fromTimeString: "10")
        }
    }

    @Test("check invalid unit")
    func testInvalidUnit() throws {
        #expect(throws: IntervalParsingError.invalidUnit(Substring("a"))) {
            try TimeInterval(fromTimeString: "10a")
        }
    }
}
