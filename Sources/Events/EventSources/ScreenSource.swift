import Cocoa
import CoreGraphics

class ScreenSource: EventSource {
	enum EventID: String {
		case locked = "com.apple.screenIsLocked"
		case unlocked = "com.apple.screenIsUnlocked"

		init(of value: Notification) {
			self = Self(rawValue: value.name.rawValue) ?? .locked
		}
	}

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
	func handleLockChange(notification: Notification) {
		switch EventID(of: notification) {
		case .locked:
			emit(kind: "locked")

		case .unlocked:
			emit(kind: "unlocked")
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
		let events = ["com.apple.screenIsLocked", "com.apple.screenIsUnlocked"]
		for event in events {
			DistributedNotificationCenter.default().addObserver(
				self,
				selector: #selector(handleLockChange),
				name: .init(event),
				object: nil
			)
		}

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(refreshScreens),
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )
        refreshScreens()
    }
}
