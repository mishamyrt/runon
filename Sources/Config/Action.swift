import Foundation

struct ActionGroup {
    let debounce: TimeInterval
}

struct Action {
    let source: String
    let kind: String
    let commands: [String]
    let target: String?
    let timeout: TimeInterval
    let group: String
}
