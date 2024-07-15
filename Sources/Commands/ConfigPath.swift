import ArgumentParser

extension RunOn {
    struct ConfigPath: ParsableCommand {
        static var configuration =
            CommandConfiguration(
                commandName: "config-path",
                abstract: """
                Print the absolute path of the configuration file.
                """.trimmingCharacters(in: [" "])
            )

        mutating func run() {
			print(kDefaultConfigPath)
        }
    }
}
