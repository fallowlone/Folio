import Foundation

/// NDJSON debug lines for Cursor debug mode (session 8cc234). Best-effort append; sandbox may block workspace path.
enum LuraAgentSessionLog {
    private static let sessionId = "8cc234"
    private static let workspacePath = "/Users/artemmac/programming/personal/lura/.cursor/debug-8cc234.log"

    static func append(
        hypothesisId: String,
        location: String,
        message: String,
        data: [String: Any],
        siblingToDocument: URL? = nil
    ) {
        let ts = Int(Date().timeIntervalSince1970 * 1000)
        let payload: [String: Any] = [
            "sessionId": sessionId,
            "hypothesisId": hypothesisId,
            "location": location,
            "message": message,
            "data": data,
            "timestamp": ts,
        ]
        guard let json = try? JSONSerialization.data(withJSONObject: payload),
              var line = String(data: json, encoding: .utf8) else { return }
        line += "\n"
        guard let bytes = line.data(using: .utf8) else { return }

        var urls: [URL] = [URL(fileURLWithPath: workspacePath)]
        if let base = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first {
            urls.append(base.appendingPathComponent("lura-debug-8cc234.ndjson"))
        }
        if let doc = siblingToDocument {
            urls.append(doc.deletingLastPathComponent().appendingPathComponent("lura-ql-debug-8cc234.ndjson"))
        }

        for url in urls {
            if !FileManager.default.fileExists(atPath: url.path) {
                FileManager.default.createFile(atPath: url.path, contents: nil, attributes: nil)
            }
            do {
                let h = try FileHandle(forWritingTo: url)
                defer { try? h.close() }
                try h.seekToEnd()
                try h.write(contentsOf: bytes)
            } catch {
                // Sandbox or read-only: skip this sink
            }
        }
    }
}
