import Cocoa
import CoreGraphics

class ScreenSource: EventSource {
    var name = "screen"
    var listener: EventListener?
    var lastScreens: [String]?
    var updating = false

    init() {
        subscribeChange()
        refreshScreens()
    }

    @objc
    func handleDisplayConnection(notification _: Notification) {
        refreshScreens()
    }

    func subscribeChange() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleDisplayConnection),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )
    }

    func getScreenNames() -> [String] {
        var names: [String] = []
        for screen in NSScreen.screens {
            names.append(screen.localizedName)
        }
        return names
    }

    func refreshScreens() {
        if updating {
            return
        }
        updating = true
        let screens = getScreenNames()
        if lastScreens == nil {
            lastScreens = screens
            updating = false
            return
        }
        for screen in screens where !lastScreens!.contains(where: { $0 == screen }) {
            emit(kind: "connected", target: screen)
        }
        for screen in lastScreens! where !screens.contains(where: { $0 == screen }) {
            emit(kind: "disconnected", target: screen)
        }
        lastScreens = screens
        updating = false
    }
}
