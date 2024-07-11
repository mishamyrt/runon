import Testing
import Foundation
@testable import runon

@Suite struct ConfigSuite {
    @Test("check empty config creation")
    func testEmpty() throws {
        let spec: Specification = .init(actions: [], groups: [])
        let config = try Config(fromSpec: spec)
        #expect(config.count == 0)
    }

    @Test("check basic actions parsing and finding")
    func testBasic() throws {
        let spec: Specification = .init(
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
					with: nil,
                    run: "eq-correction -disable",
                    group: nil,
                    timeout: nil
                ),
            ],
            groups: [
				.init(name: "test-group", debounce: nil),
			]
        )
        let config = try Config(fromSpec: spec)

		let homeAction = config.find(
            source: "audio",
            kind: "connected",
			target: "Home Speakers"
        )
		#expect(homeAction != nil)
		#expect(homeAction?.source == "audio")
        #expect(homeAction?.kind == "connected")
		#expect(homeAction?.target == "Home Speakers")
		#expect(
			homeAction?.group == "common",
			"group must be 'common' if no group is specified"
		)

        let workAction = config.find(
            source: "audio",
            kind: "connected",
            target: "Work Speakers"
        )
		#expect(workAction != nil)
        #expect(workAction?.source == "audio")
        #expect(workAction?.kind == "connected")
        #expect(workAction?.target == "Work Speakers")
        #expect(workAction?.commands == ["eq-correction -preset work"])
		#expect(workAction?.group == "test-group")
		#expect(workAction?.timeout == 5.0)

		let disconnectedAction = config.find(
			source: "audio",
			kind: "disconnected",
			target: nil
		)
		#expect(disconnectedAction != nil)
		#expect(disconnectedAction?.source == "audio")
		#expect(disconnectedAction?.kind == "disconnected")
		#expect(disconnectedAction?.target == nil)
		#expect(disconnectedAction?.group == "common")
		#expect(disconnectedAction?.commands == ["eq-correction -disable"])
    }

	@Test("check multiline actions")
	func testMultiline() throws {
		let spec: Specification = .init(
			actions: [
				.init(
					on: "audio:connected \n audio:disconnected",
					with: "first \n second",
					run: "eq-correction -preset work",
					group: nil,
					timeout: nil
				),
			],
			groups: [
				.init(name: "test-group", debounce: nil),
			]
		)
		let config = try Config(fromSpec: spec)
		#expect(config.count == 4)

		#expect(config.find(
			source: "audio",
			kind: "connected",
			target: "first"
		) != nil)
		#expect(config.find(
			source: "audio",
			kind: "disconnected",
			target: "first"
		) != nil)
		#expect(config.find(
			source: "audio",
			kind: "connected",
			target: "second"
		) != nil)
		#expect(config.find(
			source: "audio",
			kind: "disconnected",
			target: "second"
		) != nil)
	}
}
