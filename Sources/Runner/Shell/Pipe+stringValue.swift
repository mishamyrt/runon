import Foundation

extension Pipe {
    var stringValue: String {
        let data = fileHandleForReading.readDataToEndOfFile()
        return String(data: data, encoding: .utf8) ?? ""
    }
}
