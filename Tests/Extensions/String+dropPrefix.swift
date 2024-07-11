import Testing
import Foundation
@testable import runon

@Suite struct StringDropPrefixSuite {
    @Test("check prefix dropping")
    func testDropPrefix() throws {
        #expect("kValue".dropPrefix("k") == "Value")
        #expect("kValue".dropPrefix("kValue") == "")
        #expect("longPrefixValue".dropPrefix("longPrefix") == "Value")
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