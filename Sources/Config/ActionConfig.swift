struct ActionConfig: Codable {
    // swiftlint:disable identifier_name
    let on: String
    // swiftlint:enable identifier_name
    let with: String?
    let run: String
    let group: String?
    let timeout: String?
}

struct ActionGroupConfig: Codable {
    let name: String
    let debounce: String?
}
