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

    @Test("check actions parsing and finding")
    func testActions() throws {
        let spec: Specification = .init(
            actions: [
                .init(
                    on: "screen:on",
                    with: "Apple Pro Display",
                    run: "echo 'yey!'",
                    group: nil,
                    timeout: nil
                ),
                .init(
                    on: "screen:off",
                    with: "Apple Pro Display",
                    run: "echo 'oh no!'",
                    group: nil,
                    timeout: nil
                ),
            ],
            groups: nil
        )
        let config = try Config(fromSpec: spec)

        let action = config.find(
            source: "screen",
            kind: "off",
            target: "Apple Pro Display"
        )

        #expect(action?.source == "screen")
        #expect(action?.kind == "off")
        #expect(action?.target == "Apple Pro Display")
        #expect(action?.group == "common")
        #expect(action?.commands == ["echo 'oh no!'"])
    }

    @Test("check groups parsing and finding")
    func testGroups() throws {
        let spec: Specification = .init(
            actions: [
                .init(
                    on: "screen:on",
                    with: "Apple Pro Display",
                    run: "brightness 50%",
                    group: "brightness",
                    timeout: nil
                ),
                .init(
                    on: "screen:off",
                    with: "Apple Pro Display",
                    run: "brightness 80%",
                    group: "brightness",
                    timeout: nil
                ),
            ],
            groups: [
                .init(
                    name: "brightness",
                    debounce: "1s"
                )
            ]
        )
        let config = try Config(fromSpec: spec)
        #expect(config.groups["brightness"]?.debounce == 1.0)

        let action = config.find(
            source: "screen",
            kind: "on",
            target: "Apple Pro Display"
        )
        #expect(action?.target == "Apple Pro Display")
        #expect(action?.group == "brightness")
        #expect(action?.commands == ["brightness 50%"])
    }
}