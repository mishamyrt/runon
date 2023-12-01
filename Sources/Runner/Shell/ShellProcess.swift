import Darwin.C
import Foundation

@_silgen_name("kill")
private func kill(pid: Int32, signal: Int32) -> Int32

enum ProcessSignal {
    case check
    case kill
}

public struct ShellError: Swift.Error {
    public let code: Int32
    public let message: String
}

struct ShellProcess {
    private let process: Process
    private let outputPipe: Pipe
    private let errorPipe: Pipe
    private let timeInterval: TimeInterval
    private var deadline: Date?

    var output: String {
        outputPipe.stringValue
    }

    init(with command: String, timeout: TimeInterval) {
        process = Process()
        // Set stderr/stdout
        outputPipe = Pipe()
        process.standardOutput = outputPipe
        errorPipe = Pipe()
        process.standardError = errorPipe
        // Set command
        process.launchPath = "/bin/bash"
        process.arguments = ["-c", command]
        timeInterval = timeout
    }

    func waitUntilExit() {
        guard let deadline else {
            return
        }
        while true {
            if send(signal: .check) == -1 {
                return
            }
            if deadline.timeIntervalSinceNow < 0 {
                send(signal: .kill)
                continue
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    mutating func run() throws {
        process.launch()
        Logger.debug("command was started".yellow)
        deadline = Date().advanced(by: timeInterval)

        waitUntilExit()

        Logger.debug("command finished".yellow)

        if process.terminationStatus != 0 {
            throw ShellError(
                code: process.terminationStatus,
                message: errorPipe.stringValue
            )
        }
    }

    @discardableResult
    private func send(signal: ProcessSignal) -> Int32 {
        let signalCode: Int32
        switch signal {
        case .check:
            signalCode = 0

        case .kill:
            signalCode = 9
        }
        return kill(process.processIdentifier, signalCode)
    }
}

func shell(with command: String, timeout: TimeInterval = 30) throws -> String {
    var process = ShellProcess(with: command, timeout: timeout)
    try process.run()
    return process.output
}
