use anyhow::{anyhow, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

use crate::calendar::{EventConfig, create_event_with_contacts};

/// Represents a group of contacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    /// Display name for this group
    pub name: String,
    /// List of contact names to include (these will be looked up in Apple Contacts)
    pub contacts: Vec<String>,
    /// Optional description of what this group is for
    pub description: Option<String>,
}

/// Storage for all contact groups
#[derive(Debug, Serialize, Deserialize)]
pub struct ContactGroups {
    /// Map of group_id to ContactGroup
    pub groups: HashMap<String, ContactGroup>,
}

impl ContactGroups {
    /// Create a new empty ContactGroups instance
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add a new contact group
    pub fn add_group(&mut self, id: String, group: ContactGroup) {
        self.groups.insert(id, group);
    }

    /// Get a contact group by ID
    pub fn get_group(&self, id: &str) -> Option<&ContactGroup> {
        self.groups.get(id)
    }

    /// Remove a contact group by ID
    pub fn remove_group(&mut self, id: &str) -> Option<ContactGroup> {
        self.groups.remove(id)
    }

    /// Load contact groups from file
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            info!("Contact groups file doesn't exist, creating a default one");
            let groups = Self::new();
            groups.save()?;
            return Ok(groups);
        }
        
        let contents = fs::read_to_string(&config_path)?;
        let groups: ContactGroups = serde_json::from_str(&contents)
            .map_err(|e| anyhow!("Failed to parse contact groups: {}", e))?;
        
        debug!("Loaded {} contact groups", groups.groups.len());
        Ok(groups)
    }

    /// Save contact groups to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&config_path)?;
        file.write_all(json.as_bytes())?;
        
        debug!("Saved {} contact groups", self.groups.len());
        Ok(())
    }
    
    /// Get the path to the contact groups config file
    fn get_config_path() -> Result<std::path::PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
        Ok(home_dir.join(".ducktape").join("contact_groups.json"))
    }
    
    /// List all available contact groups
    pub fn list_groups(&self) {
        println!("Available contact groups:");
        if self.groups.is_empty() {
            println!("  No contact groups defined");
            println!("\nTo create a group: ducktape contacts add <group_id> <name> <contact1,contact2,...>");
            return;
        }
        
        for (id, group) in &self.groups {
            println!("  {} - {} ({} contacts)", id, group.name, group.contacts.len());
            if let Some(desc) = &group.description {
                println!("    Description: {}", desc);
            }
            println!("    Contacts: {}", group.contacts.join(", "));
        }
    }
}

/// Create an event with a predefined contact group
pub async fn create_event_with_group(
    config: EventConfig, 
    group_id: &str
) -> Result<()> {
    // Load contact groups
    let groups = ContactGroups::load()?;
    
    // Find the requested group
    let group = groups.get_group(group_id)
        .ok_or_else(|| anyhow!("Contact group '{}' not found", group_id))?;
    
    info!("Using contact group '{}' with {} contacts", group.name, group.contacts.len());
    
    // Convert contacts to str slice reference format
    let contacts: Vec<&str> = group.contacts.iter().map(AsRef::as_ref).collect();
    
    // Create the event with the contacts
    create_event_with_contacts(config, &contacts).await
}