import Cocoa

class ApplicationSource: EventSource {
    var listener: EventListener?

    var name = "app"

    @objc
    func handleAppActivate(notification: Notification) {
        guard let bundleId = appIdentifier(from: notification) else {
            return
        }
        emit(kind: "activated", target: bundleId)
    }

    @objc
    func handleAppDeactivate(notification: Notification) {
        guard let bundleId = appIdentifier(from: notification) else {
            return
        }
        emit(kind: "deactivated", target: bundleId)
    }

	@objc
    func handleAppLaunch(notification: Notification) {
        guard let bundleId = appIdentifier(from: notification) else {
            return
        }
        emit(kind: "launched", target: bundleId)
    }

	@objc
    func handleAppTerminate(notification: Notification) {
        guard let bundleId = appIdentifier(from: notification) else {
            return
        }
        emit(kind: "terminated", target: bundleId)
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
		NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(handleAppLaunch),
            name: NSWorkspace.didLaunchApplicationNotification,
            object: nil
        )
		NSWorkspace.shared.notificationCenter.addObserver(
			self,
			selector: #selector(handleAppTerminate),
			name: NSWorkspace.didTerminateApplicationNotification,
			object: nil
		)
    }

    private func appIdentifier(from: Notification) -> String? {
        let app = from.userInfo?["NSWorkspaceApplicationKey"] as? NSRunningApplication
        return app?.bundleIdentifier
			?? app?.localizedName
			?? app?.executableURL?.lastPathComponent
    }
}
