import Foundation

struct ChatMessage: Codable {
    let sender: String
    let content: String
    var timestamp: Date?
    
    init(sender: String, content: String, timestamp: Date? = Date()) {
        self.sender = sender
        self.content = content
        self.timestamp = timestamp
    }
}
