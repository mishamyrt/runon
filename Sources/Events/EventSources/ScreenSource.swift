import Cocoa
import CoreGraphics

class ScreenSource: EventSource {
    let semaphore = DispatchSemaphore(value: 1)
    var name = "screen"
    var listener: EventListener?
    var lastScreens: [String]

    init() {
        lastScreens = Self.getScreenNames()
    }

    private static func getScreenNames() -> [String] {
        NSScreen.screens.map { screen in
            screen.localizedName
        }
    }

    @objc
    func refreshScreens() {
        semaphore.wait()
        let screens = Self.getScreenNames()
        for screen in screens where !lastScreens.contains(where: { $0 == screen }) {
            emit(kind: "connected", target: screen)
        }
        for screen in lastScreens where !screens.contains(where: { $0 == screen }) {
            emit(kind: "disconnected", target: screen)
        }
        lastScreens = screens
        semaphore.signal()
    }

    func subscribe() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(refreshScreens),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )
        refreshScreens()
    }
}
