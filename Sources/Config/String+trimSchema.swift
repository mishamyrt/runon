extension String {
    func trimSchema(_ schema: String) -> String {
        let schemaPrefix = "\(schema)://"
        guard self.hasPrefix(schemaPrefix) else {
            return self
        }
        return String(self.dropFirst(schemaPrefix.count))
    }
}
