import Cocoa

class ApplicationSource: EventSource {
    var listener: EventListener?

    var name = "app"

    @objc
    func handleAppActivate(notification: Notification) {
        guard let bundleId = bundleIdentifier(from: notification) else {
            return
        }
        emit(kind: "activated", target: bundleId)
    }

    @objc
    func handleAppDeactivate(notification: Notification) {
        guard let bundleId = bundleIdentifier(from: notification) else {
            return
        }
        emit(kind: "deactivated", target: bundleId)
    }

    func subscribe() {
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(handleAppActivate),
            name: NSWorkspace.didActivateApplicationNotification,
            object: nil
        )
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(handleAppDeactivate),
            name: NSWorkspace.didDeactivateApplicationNotification,
            object: nil
        )
    }

    private func bundleIdentifier(from: Notification) -> String? {
        let app = from.userInfo?["NSWorkspaceApplicationKey"] as? NSRunningApplication
        return app?.bundleIdentifier
    }
}
