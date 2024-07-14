import Foundation
@testable import runon
import Testing

@Test("check safe accessing collection elements")
func testDropPrefix() throws {
    let collection = ["first", "second"]
    #expect(collection[safe: 0] == "first")
    #expect(collection[safe: 1] == "second")
    #expect(collection[safe: 2] == nil)
    #expect(collection[safe: -1] == nil)
    #expect(collection[safe: 9999] == nil)
}
