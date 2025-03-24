use anyhow::{anyhow, Result};
use std::future::Future;
use std::pin::Pin;

use crate::calendar::EventConfig;
use crate::commands::{CommandArgs, CommandExecutor};
use crate::contact_groups::{create_event_with_group, ContactGroup, ContactGroups};

/// Handle contact group commands
pub async fn handle_contacts_command(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        // Display available contact groups if no subcommand provided
        let groups = ContactGroups::load()?;
        groups.list_groups();
        return Ok(());
    }

    match args[0] {
        "list" => {
            // List available contact groups
            let groups = ContactGroups::load()?;
            groups.list_groups();
        }
        "add" | "create" => {
            if args.len() < 4 {
                return Err(anyhow!(
                    "Usage: contacts add <group_id> <name> <contact1,contact2,...> [description]"
                ));
            }
            let group_id = args[1].to_string();
            let name = args[2].to_string();
            let contacts: Vec<String> = args[3].split(',').map(|s| s.trim().to_string()).collect();
            let description = if args.len() > 4 {
                Some(args[4].to_string())
            } else {
                None
            };

            // Create and add the new group
            let mut groups = ContactGroups::load()?;
            let group = ContactGroup {
                name,
                contacts,
                description,
            };
            groups.add_group(group_id, group);
            groups.save()?;
            println!("Contact group '{}' created successfully", args[1]);
        }
        "remove" | "delete" => {
            if args.len() < 2 {
                return Err(anyhow!("Usage: contacts remove <group_id>"));
            }
            let group_id = args[1];

            // Remove the specified group
            let mut groups = ContactGroups::load()?;
            if let Some(_group) = groups.remove_group(group_id) {
                groups.save()?;
                println!("Contact group '{}' removed successfully", group_id);
            } else {
                return Err(anyhow!("Contact group '{}' not found", group_id));
            }
        }
        "show" => {
            if args.len() < 2 {
                return Err(anyhow!("Usage: contacts show <group_id>"));
            }
            let group_id = args[1];

            // Show details for a specific group
            let groups = ContactGroups::load()?;
            if let Some(group) = groups.get_group(group_id) {
                println!("Contact Group: {} - {}", group_id, group.name);
                if let Some(desc) = &group.description {
                    println!("Description: {}", desc);
                }
                println!("Contacts:");
                for contact in &group.contacts {
                    println!("  - {}", contact);
                }
            } else {
                return Err(anyhow!("Contact group '{}' not found", group_id));
            }
        }
        _ => {
            return Err(anyhow!(
                "Unknown contact group command: {}. Valid commands are: list, add, remove, show",
                args[0]
            ));
        }
    }

    Ok(())
}

/// Create an event with contacts from a specified group
pub async fn create_calendar_event_with_group(
    event_config: EventConfig,
    group_id: &str,
) -> Result<()> {
    create_event_with_group(event_config, group_id).await
}

/// Command executor for contact groups
pub struct ContactGroupsCommand;

impl CommandExecutor for ContactGroupsCommand {
    fn execute(&self, args: CommandArgs) -> Pin<Box<dyn Future<Output = Result<()>> + '_>> {
        Box::pin(async move {
            let str_args: Vec<&str> = args.args.iter().map(|s| s.as_str()).collect();
            handle_contacts_command(&str_args).await
        })
    }

    fn can_handle(&self, command: &str) -> bool {
        command == "contacts" || command == "contact-groups" || command == "contactgroups"
    }
}
