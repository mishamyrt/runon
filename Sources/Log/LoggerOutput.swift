import Darwin.C
import Foundation

class LogOutput {
	let handle: UnsafeMutablePointer<FILE>
	private let shouldClose: Bool

	init(stream: UnsafeMutablePointer<FILE>) {
		handle = stream
		shouldClose = false
	}

	init(file: String) {
		handle = fopen(file, "wb")
		shouldClose = true
	}

	deinit {
		if shouldClose {
			fclose(handle)
		}
	}
}

let kStdOutput = LogOutput(stream: stdout)
