// quicklook/HostApp/RecentEntry.swift
import Foundation

struct RecentEntry: Codable, Equatable {
    var bookmark: Data
    var displayPath: String
    var lastOpened: Date
}
