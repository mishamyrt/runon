import Cocoa
import CoreGraphics

class ScreenSource: EventSource {
    var name = "screen"
    var listener: EventListener?
    var lastScreens: Array<String>?
    var updating = false

    init() {
        self.subscribeChange()
        self.refreshScreens()
    }

    @objc func handleDisplayConnection(notification: Notification) {
        self.refreshScreens()
    }

    func subscribeChange() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleDisplayConnection),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil)
    }

    func getScreenNames() -> Array<String> {
        var names: Array<String> = []
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
        for screen in screens {
            if !lastScreens!.contains(where: { $0 == screen }) {
                emit(kind: "connected", target: screen)
            }
        }
        for screen in lastScreens! {
            if !screens.contains(where: { $0 == screen }) {
                emit(kind: "disconnected", target: screen)
            }
        }
        lastScreens = screens
        updating = false
    }
}