import Foundation

extension Pipe {
    var stringValue: String {
        let data = fileHandleForReading.availableData
        return String(data: data, encoding: .utf8) ?? ""
    }
}
