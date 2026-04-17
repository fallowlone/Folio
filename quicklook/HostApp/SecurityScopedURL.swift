// quicklook/HostApp/SecurityScopedURL.swift
import Foundation

/// Balances startAccessingSecurityScopedResource / stop calls.
/// Sandbox enforces that every successful start MUST be paired with a stop.
final class SecurityScopedURL {
    let url: URL
    private var didStart: Bool = false

    init(url: URL) {
        self.url = url
    }

    /// Returns true if access was granted. Safe to call multiple times; only the first start is honoured.
    @discardableResult
    func start() -> Bool {
        guard !didStart else { return true }
        didStart = url.startAccessingSecurityScopedResource()
        return didStart
    }

    func stop() {
        guard didStart else { return }
        url.stopAccessingSecurityScopedResource()
        didStart = false
    }

    deinit {
        stop()
    }
}

#if DEBUG
extension SecurityScopedURL {
    /// Sanity self-test: writes a tmp file, takes a security-scoped bookmark on it,
    /// resolves the bookmark, and verifies start/stop balance. Logs result.
    static func runSelfTest() {
        let log = { (msg: String) in LuraDebugLog.log("SecurityScopedURL.selfTest: \(msg)") }
        let tmpURL = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("lura-scope-selftest-\(UUID().uuidString).txt")
        do {
            try "hello".write(to: tmpURL, atomically: true, encoding: .utf8)
            let bookmark = try tmpURL.bookmarkData(
                options: [.withSecurityScope],
                includingResourceValuesForKeys: nil,
                relativeTo: nil
            )
            var stale = false
            let resolved = try URL(
                resolvingBookmarkData: bookmark,
                options: [.withSecurityScope],
                relativeTo: nil,
                bookmarkDataIsStale: &stale
            )
            let scope = SecurityScopedURL(url: resolved)
            let started = scope.start()
            let data = try? Data(contentsOf: resolved)
            scope.stop()
            try? FileManager.default.removeItem(at: tmpURL)
            if started, data == "hello".data(using: .utf8) {
                log("PASS (stale=\(stale))")
            } else {
                log("FAIL started=\(started) dataNil=\(data == nil)")
            }
        } catch {
            log("ERROR \(error.localizedDescription)")
        }
    }
}
#endif
