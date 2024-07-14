import Foundation
@testable import runon
import Testing

@Suite
struct ConfigHandlerSuite {
    @Test("check find action")
    func testFind() throws {
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
					on: "audio:disconnected",
					with: "Home Speakers",
					run: "eq-correction -disable",
					group: nil,
					timeout: nil
				)
			],
			groups: [
				.init(name: "test-group", debounce: nil)
			]
		))
        let handler = ConfigHandler(with: config)
		#expect(handler.findAction(
			source: "audio",
			kind: "connected",
			target: "Work Speakers"
		) == Action(
			source: "audio",
			kind: "connected",
			commands: ["eq-correction -preset work"],
			target: "Work Speakers",
			timeout: 5.0,
			group: "test-group"
		))

		#expect(handler.findAction(
			source: "audio",
			kind: "connected",
			target: "Home Speakers"
		) == Action(
			source: "audio",
			kind: "connected",
			commands: ["eq-correction -preset home"],
			target: "Home Speakers",
			timeout: 30.0,
			group: "common"
		))

		let disconnectAction = Action(
			source: "audio",
			kind: "disconnected",
			commands: ["eq-correction -disable"],
			target: "Home Speakers",
			timeout: 30.0,
			group: "common"
		)

		#expect(handler.findAction(
			source: "audio",
			kind: "disconnected",
			target: "Home Speakers"
		) == disconnectAction)

		// Search with nil target
		#expect(handler.findAction(
			source: "audio",
			kind: "disconnected",
			target: nil
		) == disconnectAction)

		#expect(handler.findAction(
			source: "screen",
			kind: "disconnected",
			target: nil
		) == nil)
    }

	@Test("check action sources")
	func testSources() throws {
		let config = try Config(from: .init(
			actions: [
				.init(
					on: "screen:connected",
					with: nil,
					run: "echo 1",
					group: nil,
					timeout: nil
				),
				.init(
					on: "audio:connected",
					with: nil,
					run: "echo 1",
					group: nil,
					timeout: nil
				),
				.init(
					on: "app:activated",
					with: nil,
					run: "echo 1",
					group: nil,
					timeout: nil
				)
			],
			groups: nil
		))
		let handler = ConfigHandler(with: config)
		let sources = handler.actionSources
		#expect(sources.count == 3)
		#expect(sources.contains("screen") == true)
		#expect(sources.contains("audio") == true)
		#expect(sources.contains("app") == true)

		let emptyHandler = ConfigHandler(with: try Config(
			from: .init(actions: [], groups: nil)
		))

		#expect(emptyHandler.actionSources.isEmpty)
	}
}

//     @Test("check basic actions parsing and finding")
//     func testBasic() throws {
//         let spec: Specification = .init(
//             actions: [
//                 .init(
//                     on: "audio:connected",
//                     with: "Work Speakers",
//                     run: "eq-correction -preset work",
//                     group: "test-group",
//                     timeout: "5s"
//                 ),
//                 .init(
//                     on: "audio:connected",
//                     with: "Home Speakers",
//                     run: "eq-correction -preset home",
//                     group: nil,
//                     timeout: nil
//                 ),
// 				.init(
//                     on: "audio:disconnected",
// 					with: nil,
//                     run: "eq-correction -disable",
//                     group: nil,
//                     timeout: nil
//                 ),
//             ],
//             groups: [
// 				.init(name: "test-group", debounce: nil),
// 			]
//         )
//         let config = try Config(fromSpec: spec)

// 		let homeAction = config.find(
//             source: "audio",
//             kind: "connected",
// 			target: "Home Speakers"
//         )
// 		#expect(homeAction != nil)
// 		#expect(homeAction?.source == "audio")
//         #expect(homeAction?.kind == "connected")
// 		#expect(homeAction?.target == "Home Speakers")
// 		#expect(
// 			homeAction?.group == "common",
// 			"group must be 'common' if no group is specified"
// 		)

//         let workAction = config.find(
//             source: "audio",
//             kind: "connected",
//             target: "Work Speakers"
//         )
// 		#expect(workAction != nil)
//         #expect(workAction?.source == "audio")
//         #expect(workAction?.kind == "connected")
//         #expect(workAction?.target == "Work Speakers")
//         #expect(workAction?.commands == ["eq-correction -preset work"])
// 		#expect(workAction?.group == "test-group")
// 		#expect(workAction?.timeout == 5.0)

// 		let disconnectedAction = config.find(
// 			source: "audio",
// 			kind: "disconnected",
// 			target: nil
// 		)
// 		#expect(disconnectedAction != nil)
// 		#expect(disconnectedAction?.source == "audio")
// 		#expect(disconnectedAction?.kind == "disconnected")
// 		#expect(disconnectedAction?.target == nil)
// 		#expect(disconnectedAction?.group == "common")
// 		#expect(disconnectedAction?.commands == ["eq-correction -disable"])
//     }

// 	@Test("check multiline actions")
// 	func testMultiline() throws {
// 		let spec: Specification = .init(
// 			actions: [
// 				.init(
// 					on: "audio:connected \n audio:disconnected",
// 					with: "first \n second",
// 					run: "eq-correction -preset work",
// 					group: nil,
// 					timeout: nil
// 				),
// 			],
// 			groups: [
// 				.init(name: "test-group", debounce: nil),
// 			]
// 		)
// 		let config = try Config(fromSpec: spec)
// 		#expect(config.count == 4)

// 		#expect(config.find(
// 			source: "audio",
// 			kind: "connected",
// 			target: "first"
// 		) != nil)
// 		#expect(config.find(
// 			source: "audio",
// 			kind: "disconnected",
// 			target: "first"
// 		) != nil)
// 		#expect(config.find(
// 			source: "audio",
// 			kind: "connected",
// 			target: "second"
// 		) != nil)
// 		#expect(config.find(
// 			source: "audio",
// 			kind: "disconnected",
// 			target: "second"
// 		) != nil)
// 	}
// }
