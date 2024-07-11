extension String {
    func dropSchema(_ schema: String) -> String {
        dropPrefix("\(schema)://")
    }

    func dropPrefix(_ prefix: String) -> String {
        guard self.hasPrefix(prefix) else {
            return self
        }
        return String(self.dropFirst(prefix.count))
    }
}
