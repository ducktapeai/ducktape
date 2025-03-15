import Foundation

class DucktapeWebSocketClient {
    private var webSocket: URLSessionWebSocketTask?
    private var session: URLSession!
    
    init() {
        session = URLSession(configuration: .default)
        connect()
    }
    
    func connect() {
        let url = URL(string: "ws://127.0.0.1:3000/chat")!
        webSocket = session.webSocketTask(with: url)
        webSocket?.resume()
        
        print("Connecting to DuckTape WebSocket server...")
        receiveMessage()
        
        // First, get available calendars
        getCalendars()
    }
    
    func receiveMessage() {
        webSocket?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    print("Received: \(text)")
                    
                    // Parse the response
                    if let data = text.data(using: .utf8),
                       let response = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                       let success = response["success"] as? Bool,
                       success == true,
                       let responseData = response["data"] as? [String: Any],
                       let calendars = responseData["calendars"] as? [String],
                       !calendars.isEmpty {
                        
                        // Create an event in the first available calendar
                        self?.createEvent(calendar: calendars[0])
                    }
                    
                case .data(let data):
                    print("Received binary data: \(data)")
                @unknown default:
                    break
                }
                // Continue receiving messages
                self?.receiveMessage()
            case .failure(let error):
                print("WebSocket receive error: \(error)")
            }
        }
    }
    
    func getCalendars() {
        let message = ["action": "GetCalendars"]
        sendJSON(message)
    }
    
    func createEvent(calendar: String) {
        let eventData: [String: Any] = [
            "title": "Swift WebSocket Meeting",
            "date": "2025-04-02",
            "start_time": "10:00",
            "end_time": "11:00",
            "calendars": [calendar],
            "description": "Meeting created via Swift WebSocket",
            "location": "Conference Room B",
            "reminder": 10,
            "create_zoom_meeting": false
        ]
        
        let message: [String: Any] = [
            "action": "CreateEvent",
            "data": eventData
        ]
        
        sendJSON(message)
    }
    
    private func sendJSON(_ object: [String: Any]) {
        do {
            let data = try JSONSerialization.data(withJSONObject: object)
            if let jsonString = String(data: data, encoding: .utf8) {
                webSocket?.send(.string(jsonString)) { error in
                    if let error = error {
                        print("WebSocket send error: \(error)")
                    }
                }
            }
        } catch {
            print("Failed to serialize JSON: \(error)")
        }
    }
    
    func disconnect() {
        webSocket?.cancel(with: .goingAway, reason: nil)
    }
}

// Example usage
let client = DucktapeWebSocketClient()

// Keep the program running to maintain the WebSocket connection
RunLoop.main.run(until: Date(timeIntervalSinceNow: 60))
client.disconnect()
