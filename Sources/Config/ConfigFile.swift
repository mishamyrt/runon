struct ConfigFile: Codable {
    let actions: [ActionConfig]
    let groups: [ActionGroupConfig]?

    func find(group name: String) -> ActionGroupConfig? {
        if let group = groups?.first(where: { $0.name == name }) {
            return group
        }
        return nil
    }
}
