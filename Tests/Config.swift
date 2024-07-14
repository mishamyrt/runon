import Foundation
@testable import runon
import Testing

// swiftlint:disable force_unwrapping
@Suite
struct ConfigSuite {
    @Test("check config creation")
    func testBasicConstructor() throws {
        let config = try Config(from: .init(actions: [
			.init(
				on: "audio:connected",
				with: nil,
				run: "echo 1",
				group: nil,
				timeout: nil
			),
			.init(
				on: "screen:connected",
				with: nil,
				run: "echo 1",
				group: nil,
				timeout: nil
			)
        ], groups: []))
        #expect(config.actionMap.count == 2)
    }

	@Test("check config creation from file")
	func testFileConstructor() throws {
		let config = try Config(
			fromContentsOf: URL(filePath: "testdata/basic.yaml")
		)
		#expect(!config.actionMap.isEmpty)
	}

	@Test("check empty config creation")
	func testEmpty() throws {
		let config = try Config(from: .init(actions: [], groups: []))
		#expect(config.actionMap.isEmpty)
	}

    @Test("check config parsing")
    func testParsing() throws {
        let config = try Config(from: .init(
            actions: [
                .init(
                    on: "audio:connected",
                    with: "Work Speakers",
                    run: "eq-correction -preset work",
                    group: "test-group",
                    timeout: "5s"
                ),
                .init(
                    on: "audio:connected",
                    with: "Home Speakers",
                    run: "eq-correction -preset home",
                    group: nil,
                    timeout: nil
                ),
                .init(
                    on: "screen:disconnected",
                    with: nil,
                    run: "eq-correction -disable",
                    group: nil,
                    timeout: nil
                )
            ],
            groups: [
				.init(name: "test-group", debounce: nil)
            ]
        ))

		let actions = config.actionMap
		let groups = config.groupMap

		#expect(groups.count == 2, "should have 2 groups, but has \(groups.count)")
		#expect(actions.count == 2, "should have 2 actions sources, but has \(actions.count)")

		#expect(
			actions["audio"]?.count == 2,
			"should have 2 actions for audio, but has \(actions["audio"]?.count as Int?)")

		let audioActions = actions["audio"]!
		let screenActions = actions["screen"]!

		#expect(audioActions[0] == Action(
			source: "audio",
			kind: "connected",
			commands: ["eq-correction -preset work"],
			target: "Work Speakers",
			timeout: 5.0,
			group: "test-group"
		))
		#expect(audioActions[1] == Action(
			source: "audio",
			kind: "connected",
			commands: ["eq-correction -preset home"],
			target: "Home Speakers",
			timeout: 30.0,
			group: "common"
		))
		#expect(screenActions[0] == .init(
			source: "screen",
			kind: "disconnected",
			commands: ["eq-correction -disable"],
			target: nil,
			timeout: 30.0,
			group: "common"
		))
    }

	@Test("check config multiline parsing")
	func testMultiline() throws {
		let config = try Config(from: .init(
			actions: [
				.init(
					on: "audio:connected \n audio:disconnected",
					with: "first \n second",
					run: "echo 1",
					group: nil,
					timeout: nil
				)
			],
			groups: nil
		))

		let actions = config.actionMap["audio"]!

		#expect(actions.count == 4)

		#expect(actions.contains { action in
			action == .init(
				source: "audio",
				kind: "connected",
				commands: ["echo 1"],
				target: "first",
				timeout: 30.0,
				group: "common"
			)
		})
		#expect(actions.contains { action in
			action == .init(
				source: "audio",
				kind: "connected",
				commands: ["echo 1"],
				target: "second",
				timeout: 30.0,
				group: "common"
			)
		})
		#expect(actions.contains { action in
			action == .init(
				source: "audio",
				kind: "disconnected",
				commands: ["echo 1"],
				target: "first",
				timeout: 30.0,
				group: "common"
			)
		})
		#expect(actions.contains { action in
			action == .init(
				source: "audio",
				kind: "disconnected",
				commands: ["echo 1"],
				target: "second",
				timeout: 30.0,
				group: "common"
			)
		})
	}
}
// swiftlint:enable force_unwrapping
