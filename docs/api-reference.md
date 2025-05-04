# DuckTape API Reference

This document provides a complete reference for the DuckTape API, which enables frontend applications to integrate with DuckTape.

## Base URL

```
http://127.0.0.1:3000
```

## Authentication

Currently, the API does not require authentication. Future versions may implement authentication using API keys or OAuth.

## Response Format

All API responses follow a common structure:

```json
{
  "success": true,
  "message": "Operation completed successfully",
  "data": {} // Optional, depends on the endpoint
}
```

Error responses include:

```json
{
  "success": false,
  "message": "Error description"
}
```

## API Endpoints

### Health Check

```
GET /health
```

Returns a simple string to check if the API is running.

**Example Response:**
```
DuckTape API is running
```

### System Status

```
GET /status
```

Returns detailed information about the API server status.

**Example Response:**
```json
{
  "version": "0.13.0",
  "uptime": "0d 0h 5m 32s",
  "status": "online",
  "calendars_available": true
}
```

### List Calendars

```
GET /calendars
```

Returns a list of available calendars.

**Example Response:**
```json
{
  "success": true,
  "message": "Calendars retrieved successfully",
  "calendars": ["Work", "Personal", "Family"]
}
```

### Create Calendar Event

```
POST /calendar/event
```

Creates a new calendar event.

**Request Body:**
```json
{
  "title": "Meeting with Team",
  "date": "2025-04-21",
  "start_time": "14:00",
  "end_time": "15:00",
  "calendars": ["Work"],
  "location": "Conference Room A",
  "description": "Weekly status update",
  "emails": ["team@example.com", "manager@example.com"],
  "reminder": 15,
  "create_zoom_meeting": true
}
```

**Required Fields:**
- `title`: Event title
- `date`: Event date in YYYY-MM-DD format
- `start_time`: Event start time in HH:MM format

**Optional Fields:**
- `end_time`: Event end time in HH:MM format
- `calendars`: Array of calendar names (defaults to default calendar)
- `location`: Event location
- `description`: Event description
- `emails`: Array of email addresses for attendees
- `reminder`: Reminder time in minutes before event
- `create_zoom_meeting`: Boolean to create a Zoom meeting for this event

**Example Response:**
```json
{
  "success": true,
  "message": "Event created successfully"
}
```

### Create Reminder Item

```
POST /reminder
```

Creates a new reminder item.

**Request Body:**
```json
{
  "title": "Submit project proposal",
  "lists": ["Work", "Projects"],
  "reminder_time": "2025-04-25 15:00",
  "notes": "Include budget estimates and timeline"
}
```

**Required Fields:**
- `title`: Reminder item title

**Optional Fields:**
- `lists`: Array of list names to add the item to
- `reminder_time`: Reminder time in YYYY-MM-DD HH:MM format
- `notes`: Additional notes for the reminder item

**Example Response:**
```json
{
  "success": true,
  "message": "Reminder created successfully"
}
```

### Create Note

```
POST /note
```

Creates a new note.

**Request Body:**
```json
{
  "title": "Project Ideas",
  "content": "Here are some ideas for the new project...",
  "folder": "Work"
}
```

**Required Fields:**
- `title`: Note title
- `content`: Note content

**Optional Fields:**
- `folder`: Folder to save the note in

**Example Response:**
```json
{
  "success": true,
  "message": "Note created successfully"
}
```

## WebSocket API

DuckTape also provides a WebSocket endpoint for real-time communication and natural language commands.

### Connection

```
WebSocket: ws://127.0.0.1:3000/chat
```

### Message Format

Messages can be sent as either text or binary messages in JSON format.

#### Natural Language Commands

```json
{
  "content": "Schedule a meeting with the team tomorrow at 2pm"
}
```

#### Structured Commands

```json
{
  "type": "create",
  "action": "event",
  "data": {
    "title": "Team Meeting",
    "date": "2025-04-21", 
    "start_time": "14:00",
    "end_time": "15:00",
    "location": "Conference Room"
  }
}
```

### Response Format

Responses are sent in JSON format:

```json
{
  "sender": "ducktape",
  "content": "✅ Created event \"Team Meeting\" for 2025-04-21 at 14:00",
  "timestamp": "2025-04-20T14:32:17.123Z",
  "type": "chat"
}
```

Error responses:

```json
{
  "sender": "ducktape",
  "content": "❌ Failed to create event: invalid date format",
  "timestamp": "2025-04-20T14:32:17.123Z",
  "type": "error"
}
```

## OpenAPI Documentation

For a more interactive experience, the API also provides OpenAPI documentation at:

```
GET /docs
```

This endpoint returns a JSON OpenAPI specification that can be used with tools like Swagger UI.