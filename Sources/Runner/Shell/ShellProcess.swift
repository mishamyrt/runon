import Foundation

public struct ShellError: Swift.Error {
    public let code: Int32
    public let message: String
}

struct ShellProcess {
    private let process: Process
    private let outputPipe: Pipe
    private let errorPipe: Pipe
    private let timeInterval: TimeInterval

    init(with command: String, timeout: TimeInterval = 30) {
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

    func launch() throws {
        var timeoutReached = false
        process.launch()
        Logger.debug("command was started".yellow)

        let timer = Timer(timeInterval: timeInterval, repeats: false) { _ in
            Logger.debug("command timeout, terminating".red)
            timeoutReached = true
            process.terminate()
        }

        RunLoop.current.add(timer, forMode: .default)

        process.waitUntilExit()
        Logger.debug("command finished".yellow)
        timer.invalidate()

        if timeoutReached {
            throw ShellError(
                code: -1,
                message: "command execution has exceeded the allowed time."
            )
        }

        if process.terminationStatus != 0 {
            throw ShellError(
                code: process.terminationStatus,
                message: errorPipe.stringValue
            )
        }
    }
}
