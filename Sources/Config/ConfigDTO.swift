struct ActionGroupDTO: Codable {
    let name: String
    let debounce: String?
}

struct ActionDTO: Codable {
    // swiftlint:disable identifier_name
    let on: String
    // swiftlint:enable identifier_name
    let with: String?
    let run: String
    let group: String?
    let timeout: String?
}

struct ConfigDTO: Codable {
    let actions: [ActionDTO]
    let groups: [ActionGroupDTO]?
}
