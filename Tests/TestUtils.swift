import Testing
@testable import runon

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
