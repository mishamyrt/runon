import Darwin.C
@testable import runon
import Testing

enum CStreamError: Error {
	case notOpened
}

/// CStream represents unix writable memory stream.
/// It's meant to be used in tests to simulate stdout.
class CStream {
    let handle: UnsafeMutablePointer<FILE>
	private var sizePtr: UnsafeMutablePointer<size_t>
	private var bufferPtr: UnsafeMutablePointer<CChar>!
	private var head: Int = 0

	var content: String {
		let decoded = String(cString: bufferPtr)
		if head >= decoded.count {
			return ""
		}
		return String(decoded.dropLast())
	}
	var lines: [String] { content.components(separatedBy: "\n") }
	var isEmpty: Bool { content.isEmpty }
	var count: Int { sizePtr.pointee }

	init() throws {
		bufferPtr = UnsafeMutablePointer<CChar>.allocate(capacity: 1)
		sizePtr = UnsafeMutablePointer<size_t>.allocate(capacity: 1)
        let handle = open_memstream(&bufferPtr, sizePtr)

		guard let handle = handle else {
			throw CStreamError.notOpened
		}
		self.handle = handle
	}

	/// Clear buffer.
	/// Actually, just set head pointer to get content substring.
	func clear() {
		head = count
	}

	deinit {
		fclose(handle)
        bufferPtr.deallocate()
        sizePtr.deallocate()
	}
}
