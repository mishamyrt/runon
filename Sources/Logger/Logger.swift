import Darwin.C
import Foundation
import Rainbow

var kLoggerVerbose = false

enum Logger {
    static func now() -> String {
        let date = Date()
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return dateFormatter.string(from: date)
    }

    static func printMessage(_ message: String) {
        let timestamp = now().dim
        fputs("\(timestamp) \(message)\n", stdout)
        fflush(stdout)
    }

    static func info(_ message: String) {
        printMessage(message)
    }

    static func debug(_ message: String) {
        if kLoggerVerbose {
            printMessage(message)
        }
    }

    static func warning(_ message: String) {
        printMessage(message.yellow)
    }

    static func error(_ message: String) {
        printMessage(message.red)
    }
}
