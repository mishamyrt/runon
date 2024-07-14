import Foundation
@testable import runon
import Testing

@Suite
struct StringDropPrefixSuite {
    @Test("check prefix dropping")
    func testDropPrefix() throws {
        #expect("kValue".dropPrefix("k") == "Value")
        #expect("longPrefixValue".dropPrefix("longPrefix") == "Value")
		#expect("kValue".dropPrefix("kValue").isEmpty)
    }

    @Test("check without prefix")
    func testEmptyPrefix() throws {
        #expect("kValue".dropPrefix("m") == "kValue")
        #expect("kValue".dropPrefix("longPrefix") == "kValue")
        #expect("kValue".dropPrefix("") == "kValue")
    }

    @Test("check drop schema")
    func testDropSchema() throws {
        #expect("http://myrt.co".dropSchema("http") == "myrt.co")
        #expect("ftp://myrt.co".dropSchema("ftp") == "myrt.co")
        #expect("ws://myrt.co".dropSchema("ws") == "myrt.co")
    }
}
