// ...existing code...

pub fn create_event(&self, event: &CalendarEvent) -> Result<(), CalendarError> {
    // ...existing code...

    // Format the location field properly for AppleScript
    let location = if let Some(loc) = &event.location {
        // Handle multiple addresses by joining them with commas
        if loc.contains(";") {
            let addresses = loc.split(";").collect::<Vec<&str>>();
            format!("\"{}\"", addresses.join(", "))
        } else {
            format!("\"{}\"", loc)
        }
    } else {
        "missing value".to_string()
    };

    // Handle attendees properly for AppleScript
    let attendees_script = if let Some(attendees) = &event.attendees {
        if attendees.is_empty() {
            "".to_string()
        } else {
            let mut script_parts = Vec::new();
            
            // Split multiple attendees by semicolon if present
            for attendee in attendees.split(";") {
                let trimmed = attendee.trim();
                if !trimmed.is_empty() {
                    script_parts.push(format!("make new attendee at end of attendees with properties {{email:\"{}\"}}",
                        trimmed));
                }
            }
            
            if script_parts.is_empty() {
                "".to_string()
            } else {
                script_parts.join("\n                ")
            }
        }
    } else {
        "".to_string()
    };

    // Construct the AppleScript
    let script = format!(
        r#"
        tell application "Calendar"
            tell calendar "{}"
                set newEvent to make new event at end with properties {{
                    summary: "{}",
                    start date: {},
                    end date: {},
                    location: {},
                    description: {},
                    all day event: {}
                }}
                
                tell newEvent
                    {}
                end tell
            end tell
        end tell
        "#,
        calendar_name,
        event.title,
        start_date,
        end_date,
        location,
        description,
        all_day,
        attendees_script
    );

    // ...existing code...
}

// ...existing code...
