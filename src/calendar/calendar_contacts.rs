//! Contact lookup logic for DuckTape calendar module.
//
// This module provides functions to look up contacts and their emails.

use crate::calendar::calendar_types::EventConfig;
use crate::calendar::calendar_validation::validate_email;
use anyhow::{Result, anyhow};
use log::{debug, error, info};

/// Lookup a contact by name and return their email addresses
pub async fn lookup_contact(name: &str) -> Result<Vec<String>> {
    debug!("Looking up contact: '{}'", name);
    let script = format!(
        r#"tell application "Contacts"
            set the_emails to {{}}
            try
                set the_people to (every person whose name contains "{}")
                repeat with the_person in the_people
                    if exists email of the_person then
                        repeat with the_email in (get every email of the_person)
                            if value of the_email is not missing value then
                                set the end of the_emails to (value of the_email as text)
                            end if
                        end repeat
                    end if
                end repeat
                return the_emails
            on error errMsg
                log "Error looking up contact: " & errMsg
                return {{}}
            end try
        end tell"#,
        name.replace("\"", "\\\"")
    );

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;

    if output.status.success() {
        let emails = String::from_utf8_lossy(&output.stdout);
        debug!("Raw contact lookup output: {}", emails);
        let email_list: Vec<String> = emails
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty() && !s.contains("missing value"))
            .map(|s| s.trim_matches('"').trim().to_string())
            .collect();
        if email_list.is_empty() {
            debug!("No emails found for contact '{}'", name);
        } else {
            debug!("Found {} email(s) for '{}': {:?}", email_list.len(), name, email_list);
        }
        Ok(email_list)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Contact lookup error: {}", error);
        Ok(Vec::new())
    }
}

/// Helper to search by a specific part of the name (first or last)
async fn lookup_by_name_part(name_part: &str, part_type: &str) -> Result<Vec<String>> {
    debug!("Looking up contacts by {} name: '{}'", part_type, name_part);

    let script = format!(
        r#"tell application "Contacts"
            set the_emails to {{}}
            try
                set search_term to "{}"
                if "{}" is "first" then
                    set the_people to (every person whose first name contains search_term)
                else
                    set the_people to (every person whose last name contains search_term)
                end if
                
                repeat with the_person in the_people
                    if exists email of the_person then
                        repeat with the_email in (get every email of the_person)
                            if value of the_email is not missing value then
                                set the end of the_emails to (value of the_email as text)
                            end if
                        end repeat
                    end if
                end repeat
                return the_emails
            on error errMsg
                log "Error looking up contact by {} name: " & errMsg
                return {{}}
            end try
        end tell"#,
        name_part.replace("\"", "\\\""),
        part_type,
        part_type
    );

    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .await
        .map_err(|e| {
            anyhow!("Failed to execute AppleScript for {} name search: {}", part_type, e)
        })?;

    if output.status.success() {
        let emails = String::from_utf8_lossy(&output.stdout);
        debug!("Raw contact lookup output ({} name search): '{}'", part_type, emails);

        let email_list: Vec<String> = emails
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty() && !s.contains("missing value"))
            .map(|s| s.trim_matches('"').trim().to_string())
            .filter(|email| validate_email(email))
            .collect();

        Ok(email_list)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Contact lookup error ({} name search): {}", part_type, error);
        Ok(Vec::new())
    }
}

/// Enhanced event creation with contact lookup
pub async fn create_event_with_contacts(
    mut config: EventConfig,
    contact_names: &[&str],
) -> anyhow::Result<()> {
    use crate::calendar::create_event;

    info!("Creating event with {} contact names: {:?}", contact_names.len(), contact_names);
    let mut found_emails = Vec::new();

    for name in contact_names {
        info!("Looking up contact: '{}'", name);
        match lookup_contact(name).await {
            Ok(emails) => {
                if emails.is_empty() {
                    info!("No email found for contact: '{}'", name);
                } else {
                    info!("Found {} email(s) for contact '{}': {:?}", emails.len(), name, emails);
                    // Directly add all emails to found_emails collection
                    found_emails.extend(emails.into_iter().map(|e| e.trim().to_string()));
                }
            }
            Err(e) => {
                error!("Failed to lookup contact '{}': {}", name, e);
            }
        }
    }

    // Log the found emails
    info!("Adding {} found email(s): {:?}", found_emails.len(), found_emails);

    // Create a completely fresh email list with proper capacity
    let mut all_emails = Vec::with_capacity(config.emails.len() + found_emails.len());

    // Add existing emails first
    all_emails.extend(config.emails.iter().cloned());

    // Then add found emails
    all_emails.extend(found_emails);

    // Sort and deduplicate
    all_emails.sort();
    all_emails.dedup();

    info!("Final email list after deduplication: {} emails", all_emails.len());

    // Set the emails in the config
    config.emails = all_emails;

    // Create the event with the updated config
    create_event(config).await
}
