import Cocoa
import CoreGraphics

class ScreenSource: EventSource {
    let semaphore = DispatchSemaphore(value: 1)
    var name = "screen"
    var listener: EventListener?
    var lastScreens: [String]?
    var updating = false

    @objc
    func handleDisplayConnection(notification _: Notification) {
        refreshScreens()
    }

    func subscribe() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleDisplayConnection),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )
        refreshScreens()
    }

    func getScreenNames() -> [String] {
        NSScreen.screens.map { screen in
            screen.localizedName
        }
    }

    func refreshScreens() {
        semaphore.wait()
        let screens = getScreenNames()
        guard var lastScreens else {
            semaphore.signal()
            return
        }
        for screen in screens where !lastScreens.contains(where: { $0 == screen }) {
            emit(kind: "connected", target: screen)
        }
        for screen in lastScreens where !screens.contains(where: { $0 == screen }) {
            emit(kind: "disconnected", target: screen)
        }
        lastScreens = screens
        semaphore.signal()
    }
}
