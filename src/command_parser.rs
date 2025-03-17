use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Serialize)]
pub struct ParsedCommand {
    pub command_type: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UserMessage {
    pub content: String,
    pub timestamp: String,
    pub id: String, 
    pub sender: String,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub content: String,
    pub success: bool,
    pub command_id: String,
}

pub fn parse_command(message: &str) -> Option<ParsedCommand> {
    // Match schedule command: "schedule a <type> <what> with <who> <when> at <time>"
    if let Some(schedule) = parse_schedule(message) {
        return Some(ParsedCommand {
            command_type: "schedule".to_string(),
            details: serde_json::to_value(schedule).unwrap(),
        });
    }

    None
}

#[derive(Debug, Serialize)]
struct ScheduleCommand {
    event_type: String,
    event_name: String,
    person: String,
    day: String,
    time: String,
}

fn parse_schedule(message: &str) -> Option<ScheduleCommand> {
    // Use regex to parse the schedule command
    let re = Regex::new(r"schedule a (\w+) (\w+) with (\w+) (\w+) at (\d+(?::\d+)?(?:am|pm)?)").ok()?;
    
    if let Some(caps) = re.captures(message) {
        return Some(ScheduleCommand {
            event_type: caps.get(1)?.as_str().to_string(),
            event_name: caps.get(2)?.as_str().to_string(),
            person: caps.get(3)?.as_str().to_string(),
            day: caps.get(4)?.as_str().to_string(),
            time: caps.get(5)?.as_str().to_string(),
        });
    }
    
    None
}

pub fn process_command(message: UserMessage) -> CommandResponse {
    let parsed = parse_command(&message.content);
    
    match parsed {
        Some(cmd) => {
            let response = format!("Processing command: {}. Details: {}", 
                cmd.command_type, cmd.details.to_string());
                
            CommandResponse {
                content: response,
                success: true,
                command_id: message.id,
            }
        },
        None => CommandResponse {
            content: "Sorry, I didn't understand that command.".to_string(),
            success: false,
            command_id: message.id,
        }
    }
}
