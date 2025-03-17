use crate::grok_parser_new::{GrokParser, DucktapeCommand};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Command {
    pub content: String,
    pub timestamp: String,
    pub id: String,
    pub sender: String,
}

#[derive(Serialize)]
pub struct Response {
    pub content: String,
    pub success: bool,
    pub command_id: String,
}

pub struct CommandProcessor {
    grok_parser: GrokParser,
}

impl CommandProcessor {
    pub fn new() -> Self {
        Self {
            grok_parser: GrokParser::new(),
        }
    }

    pub fn process_command(&self, command: Command) -> Response {
        let parsed_command = self.grok_parser.parse(&command.content);
        
        let response_content = match parsed_command {
            Some(ducktape_cmd) => {
                // Execute the Ducktape command
                format!("Processing command: {:?}", ducktape_cmd)
            },
            None => "Could not parse command. Please try again.".to_string(),
        };

        Response {
            content: response_content,
            success: parsed_command.is_some(),
            command_id: command.id,
        }
    }
}
