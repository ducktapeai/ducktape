# DuckTape WebSocket API Documentation

## Overview

DuckTape provides a real-time WebSocket API for bidirectional communication. This API enables instant updates for calendar events, reminders, and command responses.

## Connection

### Endpoint
```
ws://localhost:3000/ws
wss://your-server.com/ws  # Production
```

### Authentication
Include your API key in the connection request headers:
```
Authorization: Bearer your-api-key
```

## Message Format

All messages are JSON objects with the following structure:
```json
{
    "type": "string",     // Message type
    "payload": {},        // Message payload
    "id": "string",       // Optional message ID for correlation
    "timestamp": "string" // ISO 8601 timestamp
}
```

### Message Types

#### Client → Server

1. **Command**
```json
{
    "type": "command",
    "payload": {
        "command": "schedule meeting tomorrow at 2pm",
        "timezone": "America/New_York"
    },
    "id": "cmd-123"
}
```

2. **Calendar Query**
```json
{
    "type": "calendar.query",
    "payload": {
        "start": "2025-03-24T00:00:00Z",
        "end": "2025-03-24T23:59:59Z"
    },
    "id": "cal-456"
}
```

3. **Subscribe**
```json
{
    "type": "subscribe",
    "payload": {
        "topics": ["calendar", "reminders"]
    },
    "id": "sub-789"
}
```

4. **Ping**
```json
{
    "type": "ping",
    "payload": {},
    "id": "ping-101112"
}
```

#### Server → Client

1. **Command Response**
```json
{
    "type": "command.response",
    "payload": {
        "success": true,
        "result": {
            "eventId": "evt-123",
            "title": "Meeting",
            "start": "2025-03-25T14:00:00Z",
            "end": "2025-03-25T15:00:00Z"
        }
    },
    "id": "cmd-123"
}
```

2. **Calendar Update**
```json
{
    "type": "calendar.update",
    "payload": {
        "action": "create",
        "event": {
            "id": "evt-123",
            "title": "Meeting",
            "start": "2025-03-25T14:00:00Z",
            "end": "2025-03-25T15:00:00Z"
        }
    }
}
```

3. **Reminder**
```json
{
    "type": "reminder",
    "payload": {
        "id": "rem-123",
        "title": "Meeting in 5 minutes",
        "eventId": "evt-123",
        "triggerTime": "2025-03-25T13:55:00Z"
    }
}
```

4. **Error**
```json
{
    "type": "error",
    "payload": {
        "code": "invalid_command",
        "message": "Unable to parse command"
    },
    "id": "cmd-123"
}
```

5. **Pong**
```json
{
    "type": "pong",
    "payload": {},
    "id": "ping-101112"
}
```

## Error Codes

| Code | Description |
|------|-------------|
| invalid_command | Command could not be parsed |
| auth_error | Authentication failed |
| rate_limit | Rate limit exceeded |
| invalid_request | Invalid request format |
| server_error | Internal server error |

## Connection Lifecycle

1. **Connection**
   - Client initiates WebSocket connection
   - Server validates authentication
   - On success, server sends welcome message

2. **Keep-Alive**
   - Client should send ping every 30 seconds
   - Server responds with pong
   - Connection closed after 90 seconds of inactivity

3. **Disconnection**
   - Either party can close connection
   - Client should attempt reconnection with exponential backoff
   - Server preserves subscription state for 5 minutes

## Examples

### JavaScript
```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
    // Subscribe to updates
    ws.send(JSON.stringify({
        type: 'subscribe',
        payload: {
            topics: ['calendar', 'reminders']
        },
        id: 'sub-1'
    }));
    
    // Send a command
    ws.send(JSON.stringify({
        type: 'command',
        payload: {
            command: 'schedule meeting tomorrow at 2pm',
            timezone: 'America/New_York'
        },
        id: 'cmd-1'
    }));
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    switch (message.type) {
        case 'command.response':
            console.log('Command result:', message.payload.result);
            break;
        case 'calendar.update':
            console.log('Calendar update:', message.payload);
            break;
        case 'reminder':
            console.log('Reminder:', message.payload);
            break;
        case 'error':
            console.error('Error:', message.payload);
            break;
    }
};

// Keep-alive
setInterval(() => {
    ws.send(JSON.stringify({
        type: 'ping',
        payload: {},
        id: `ping-${Date.now()}`
    }));
}, 30000);
```

### Python
```python
import websockets
import json
import asyncio
import datetime

async def ducktape_client():
    uri = "ws://localhost:3000/ws"
    
    async with websockets.connect(uri) as ws:
        # Subscribe to updates
        await ws.send(json.dumps({
            "type": "subscribe",
            "payload": {
                "topics": ["calendar", "reminders"]
            },
            "id": "sub-1"
        }))
        
        # Send a command
        await ws.send(json.dumps({
            "type": "command",
            "payload": {
                "command": "schedule meeting tomorrow at 2pm",
                "timezone": "America/New_York"
            },
            "id": "cmd-1"
        }))
        
        while True:
            message = json.loads(await ws.recv())
            
            if message["type"] == "command.response":
                print("Command result:", message["payload"]["result"])
            elif message["type"] == "calendar.update":
                print("Calendar update:", message["payload"])
            elif message["type"] == "reminder":
                print("Reminder:", message["payload"])
            elif message["type"] == "error":
                print("Error:", message["payload"])

asyncio.get_event_loop().run_until_complete(ducktape_client())
```

### Rust
```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::{json, Value};
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let url = "ws://localhost:3000/ws";
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    
    // Subscribe to updates
    ws_stream.send(Message::Text(json!({
        "type": "subscribe",
        "payload": {
            "topics": ["calendar", "reminders"]
        },
        "id": "sub-1"
    }).to_string())).await.expect("Failed to subscribe");
    
    // Send a command
    ws_stream.send(Message::Text(json!({
        "type": "command",
        "payload": {
            "command": "schedule meeting tomorrow at 2pm",
            "timezone": "America/New_York"
        },
        "id": "cmd-1"
    }).to_string())).await.expect("Failed to send command");
    
    while let Some(msg) = ws_stream.next().await {
        if let Ok(Message::Text(text)) = msg {
            let message: Value = serde_json::from_str(&text)
                .expect("Failed to parse message");
            
            match message["type"].as_str() {
                Some("command.response") => {
                    println!("Command result: {:?}", 
                        message["payload"]["result"]);
                }
                Some("calendar.update") => {
                    println!("Calendar update: {:?}", 
                        message["payload"]);
                }
                Some("reminder") => {
                    println!("Reminder: {:?}", 
                        message["payload"]);
                }
                Some("error") => {
                    println!("Error: {:?}", 
                        message["payload"]);
                }
                _ => {}
            }
        }
    }
}
```

## Rate Limits

- Maximum 100 commands per minute per client
- Maximum 10 simultaneous connections per API key
- Maximum message size: 64KB
- Maximum subscription topics: 10 per connection

## Best Practices

1. **Connection Management**
   - Implement exponential backoff for reconnections
   - Handle connection errors gracefully
   - Monitor connection health with ping/pong
   - Close connections when not in use

2. **Message Handling**
   - Validate message format before sending
   - Handle all message types
   - Process messages asynchronously
   - Implement proper error handling

3. **Performance**
   - Batch updates when possible
   - Limit subscription topics
   - Implement message queuing
   - Monitor message latency

4. **Security**
   - Use TLS in production (wss://)
   - Protect API keys
   - Validate message input
   - Monitor for suspicious activity

## Testing

### Test WebSocket Client
We provide a test client at `test-websocket.html` that you can use to:
- Test connections
- Send commands
- Monitor updates
- Debug messages

### Command Line Testing
```bash
# Using websocat
websocat ws://localhost:3000/ws

# Using wscat
wscat -c ws://localhost:3000/ws
```

## Debugging

1. **Enable Debug Logging**
```bash
export RUST_LOG=debug
ducktape --websocket-debug
```

2. **Monitor Connections**
```bash
ducktape --show-connections
```

3. **Test Latency**
```bash
ducktape --websocket-latency-test
```

## Support

For API support:
- GitHub issues for bugs
- Discord community for questions
- Documentation PRs welcome