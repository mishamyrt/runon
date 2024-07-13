import Testing
import Darwin.C
@testable import runon

extension [Action] {
	func contains(action needle: Action) -> Bool {
		self.contains { action in
			action.source == needle.source &&
			action.kind == needle.kind &&
			action.target == needle.target &&
			action.commands == needle.commands &&
			action.timeout == needle.timeout &&
			action.group == needle.group &&
			action.timeout == needle.timeout
		}
	}
}

class CStream {
    var size: UnsafeMutablePointer<size_t>?
    var handle: UnsafeMutablePointer<FILE>!
	var buffer: UnsafeMutablePointer<Int8>?

	var content: String { read() }
	var lines: [String] { content.components(separatedBy: "\n") }
	var meaningfulLines: [String] { lines.filter { !$0.isEmpty } }
	var isEmpty: Bool { content.isEmpty }

	init() {
		allocate()
	}

	deinit {
		deallocate()
	}

	func clear() {
		deallocate()
		allocate()
	}

	private func read() -> String {
  		String(cString: buffer!)
	}

	private func allocate() {
		buffer = UnsafeMutablePointer<Int8>.allocate(capacity: 1)
        size = UnsafeMutablePointer<size_t>.allocate(capacity: 1)
        handle = open_memstream(&buffer, size)
	}

	private func deallocate() {
		fclose(handle)
        buffer?.deallocate()
        size?.deallocate()
	}
}

func expectEqualActions(_ a: Action?, _ b: Action?) {
	if a == nil && b == nil {
		return
	}
	#expect(a != nil && b != nil, "expected action to be not nil")
	guard let a = a else { return }
	guard let b = b else { return }

	#expect(a.source == b.source)
	#expect(a.kind == b.kind)
	#expect(a.commands == b.commands)
	#expect(a.target == b.target)
	#expect(a.timeout == b.timeout)
	#expect(a.group == b.group)
}
