import Foundation

@MainActor
final class LuraFileDocument: ObservableObject {
    let url: URL

    @Published var text: String

    /// True when buffer differs from last saved or reverted content.
    var isDirty: Bool { text != savedText }

    private var savedText: String

    /// Optional. Set when the URL was resolved from a security-scoped bookmark
    /// (Recents). When set, `deinit` calls `stop()`. Powerbox-granted URLs
    /// (Open / Save panel, Finder double-click) do NOT need this — system
    /// manages their scope.
    private var scope: SecurityScopedURL?

    init(url: URL, scope: SecurityScopedURL? = nil) throws {
        self.url = url
        self.scope = scope
        let loaded = try String(contentsOf: url, encoding: .utf8)
        self.savedText = loaded
        self.text = loaded
    }

    func save() throws {
        try text.write(to: url, atomically: true, encoding: .utf8)
        savedText = text
        objectWillChange.send()
    }

    func revert() throws {
        let loaded = try String(contentsOf: url, encoding: .utf8)
        savedText = loaded
        text = loaded
    }

    deinit {
        scope?.stop()
    }
}
