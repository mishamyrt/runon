import Foundation

struct Action {
    let source: String
    let kind: String
    let commands: [String]
    let target: String?
    let timeout: TimeInterval
    let group: String
}

struct ActionGroup {
    let debounce: TimeInterval
}

typealias ActionMap = [String: [Action]]
typealias GroupMap = [String: ActionGroup]

extension Action: Equatable {
	static func == (lhs: Action, rhs: Action) -> Bool {
		lhs.source == rhs.source &&
		lhs.kind == rhs.kind &&
		lhs.commands == rhs.commands &&
		lhs.target == rhs.target &&
		lhs.timeout == rhs.timeout &&
		lhs.group == rhs.group
	}
}
