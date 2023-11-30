import Darwin.C
import Foundation
import Rainbow

struct Logger {
    var verbose = false

    func now() -> String {
        let date = Date()
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return dateFormatter.string(from: date)
    }

    func printMessage(_ message: String) {
        let timestamp = now().dim
        fputs("\(timestamp) \(message)\n", stdout)
        fflush(stdout)
    }

    func info(_ message: String) {
        printMessage(message)
    }

    func debug(_ message: String) {
        if verbose {
            printMessage(message)
        }
    }

    func warning(_ message: String) {
        printMessage(message.yellow)
    }

    func error(_ message: String) {
        printMessage(message.red)
    }
}
