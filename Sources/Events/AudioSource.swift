import Cocoa
import SimplyCoreAudio

class AudioSource: EventSource {
    let coreAudio = SimplyCoreAudio()
    var name = "audio"
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
        let addedDevices = notification.userInfo?["addedDevices"] as? [AudioDevice]
        if addedDevices != nil && !addedDevices!.isEmpty {
            emitAll(kind: "connected", devices: addedDevices!)
        }

        let removedDevices = notification.userInfo?["removedDevices"] as? [AudioDevice]
        if removedDevices != nil && !removedDevices!.isEmpty {
            emitAll(kind: "disconnected", devices: removedDevices!)
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
