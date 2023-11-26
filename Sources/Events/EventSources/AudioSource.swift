import Cocoa
import SimplyCoreAudio

class AudioSource: EventSource {
    var name = "audio"
    let coreAudio = SimplyCoreAudio()
    var listener: EventListener?
    var lastScreens: [String]?
    var updating = false

    func emitAll(kind: String, devices: [AudioDevice]) {
        var names: [String] = []
        for device in devices where !names.contains(device.name) {
            emit(kind: kind, target: device.name)
            names.append(device.name)
        }
    }

    @objc
    func handleDeviceListChanged(notification: Notification) {
        if let addedDevices =
            notification.userInfo?["addedDevices"] as? [AudioDevice], !addedDevices.isEmpty {
            emitAll(kind: "connected", devices: addedDevices)
        }

        if let removedDevices =
            notification.userInfo?["removedDevices"] as? [AudioDevice], !removedDevices.isEmpty {
            emitAll(kind: "disconnected", devices: removedDevices)
        }
    }

    func subscribe() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(self.handleDeviceListChanged),
            name: Notification.Name.deviceListChanged,
            object: nil
        )
    }
}
