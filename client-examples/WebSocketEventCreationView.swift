import SwiftUI

struct WebSocketEventCreationView: View {
    @State private var title = ""
    @State private var date = Date()
    @State private var startTime = Date()
    @State private var endTime = Date().addingTimeInterval(3600) // 1 hour later
    @State private var location = ""
    @State private var description = ""
    @State private var isConnected = false
    @State private var message = ""
    @State private var availableCalendars: [String] = []
    @State private var selectedCalendar = ""
    
    private let webSocketClient = WebSocketClient()
    private let dateFormatter = DateFormatter()
    private let timeFormatter = DateFormatter()
    
    init() {
        dateFormatter.dateFormat = "yyyy-MM-dd"
        timeFormatter.dateFormat = "HH:mm"
    }
    
    var body: some View {
        VStack {
            Text("WebSocket Event Creation")
                .font(.headline)
            
            Form {
                Section(header: Text("WebSocket Status")) {
                    HStack {
                        Image(systemName: isConnected ? "circle.fill" : "circle")
                            .foregroundColor(isConnected ? .green : .red)
                        Text(isConnected ? "Connected" : "Disconnected")
                        
                        Spacer()
                        
                        Button(isConnected ? "Disconnect" : "Connect") {
                            if isConnected {
                                webSocketClient.disconnect()
                                isConnected = false
                            } else {
                                connectWebSocket()
                            }
                        }
                    }
                }
                
                Section(header: Text("Event Details")) {
                    TextField("Title", text: $title)
                    
                    DatePicker("Date", selection: $date, displayedComponents: .date)
                    
                    DatePicker("Start Time", selection: $startTime, displayedComponents: .hourAndMinute)
                    
                    DatePicker("End Time", selection: $endTime, displayedComponents: .hourAndMinute)
                    
                    TextField("Location", text: $location)
                    
                    Picker("Calendar", selection: $selectedCalendar) {
                        Text("Select a calendar").tag("")
                        ForEach(availableCalendars, id: \.self) { calendar in
                            Text(calendar).tag(calendar)
                        }
                    }
                    .disabled(availableCalendars.isEmpty)
                }
                
                Section(header: Text("Description")) {
                    TextEditor(text: $description)
                        .frame(height: 100)
                }
                
                Button("Create Event via WebSocket") {
                    createEvent()
                }
                .disabled(!isConnected || title.isEmpty || selectedCalendar.isEmpty)
                
                if !message.isEmpty {
                    Text(message)
                        .foregroundColor(message.contains("success") ? .green : .red)
                        .padding()
                }
            }
        }
        .padding()
    }
    
    private func connectWebSocket() {
        webSocketClient.connect { connected in
            isConnected = connected
            if connected {
                webSocketClient.getCalendars { calendars in
                    self.availableCalendars = calendars
                    if !calendars.isEmpty {
                        self.selectedCalendar = calendars[0]
                    }
                }
            }
        } messageHandler: { responseMessage in
            DispatchQueue.main.async {
                self.message = responseMessage
            }
        }
    }
    
    private func createEvent() {
        let eventData: [String: Any] = [
            "title": title,
            "date": dateFormatter.string(from: date),
            "start_time": timeFormatter.string(from: startTime),
            "end_time": timeFormatter.string(from: endTime),
            "location": location,
            "description": description,
            "calendars": [selectedCalendar]
        ]
        
        webSocketClient.createEvent(eventData: eventData)
    }
}

class WebSocketClient {
    private var webSocket: URLSessionWebSocketTask?
    private var session: URLSession!
    private var messageHandler: ((String) -> Void)?
    private var calendarsCallback: (([String]) -> Void)?
    
    func connect(completion: @escaping (Bool) -> Void, messageHandler: @escaping (String) -> Void) {
        self.messageHandler = messageHandler
        session = URLSession(configuration: .default)
        
        let url = URL(string: "ws://127.0.0.1:3000/chat")!
        webSocket = session.webSocketTask(with: url)
        webSocket?.resume()
        
        // Start receiving messages
        receiveMessage()
        
        // Notify connection success
        completion(true)
    }
    
    func getCalendars(completion: @escaping ([String]) -> Void) {
        self.calendarsCallback = completion
        
        let message = ["action": "GetCalendars"]
        sendJSON(message)
    }
    
    func createEvent(eventData: [String: Any]) {
        let message: [String: Any] = [
            "action": "CreateEvent",
            "data": eventData
        ]
        
        sendJSON(message)
    }
    
    private func receiveMessage() {
        webSocket?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    print("Received: \(text)")
                    
                    // Parse the response
                    if let data = text.data(using: .utf8),
                       let response = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
                        
                        // Handle success or error message
                        if let success = response["success"] as? Bool,
                           let message = response["message"] as? String {
                            DispatchQueue.main.async {
                                self?.messageHandler?(success ? "Success: \(message)" : "Error: \(message)")
                            }
                        }
                        
                        // Handle calendars list
                        if let responseData = response["data"] as? [String: Any],
                           let calendars = responseData["calendars"] as? [String] {
                            DispatchQueue.main.async {
                                self?.calendarsCallback?(calendars)
                            }
                        }
                    }
                    
                case .data:
                    print("Received binary data")
                @unknown default:
                    break
                }
                // Continue receiving messages
                self?.receiveMessage()
            case .failure(let error):
                print("WebSocket receive error: \(error)")
                DispatchQueue.main.async {
                    self?.messageHandler?("Connection error: \(error.localizedDescription)")
                }
            }
        }
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
        webSocket = nil
    }
}
