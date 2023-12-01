import Foundation

struct Handler {
    let source: String
    let kind: String
    let commands: [String]
    let target: String?
    let timeout: TimeInterval
}
