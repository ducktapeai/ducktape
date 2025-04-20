//! Contact lookup logic for DuckTape calendar module.
//
// This module provides functions to look up contacts and their emails.

use crate::calendar::calendar_types::EventConfig;
use crate::calendar::calendar_validation::validate_email;
use anyhow::{Result, anyhow};
use log::{debug, error, info};

/// Lookup a contact by name and return their email addresses
pub async fn lookup_contact(name: &str) -> Result<Vec<String>> {
    debug!("Looking up contact by name: '{}'", name);
    let clean_name = name.trim().to_lowercase();

    // Try exact match first (highest priority)
    let exact_script = format!(
        r#"tell application "Contacts"
            set the_emails to {{}}
            try
                set the_people to (every person whose name is "{}")
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

    debug!("Executing AppleScript for exact match");
    let output = tokio::process::Command::new("osascript")
        .arg("-e")
        .arg(&exact_script)
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;

    if output.status.success() {
        let emails = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            debug!("AppleScript stderr (exact match): '{}'", stderr);
        }

        let email_list: Vec<String> = emails
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty() && !s.contains("missing value"))
            .map(|s| s.trim_matches('"').trim().to_string())
            .filter(|email| validate_email(email))
            .collect();

        if !email_list.is_empty() {
            info!("Found {} email(s) for contact '{}': {:?}", email_list.len(), name, email_list);
            return Ok(email_list);
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Contact lookup error (exact match): {}", error);
    }

    // Try to search by full name with both first name AND last name (second priority)
    debug!("Getting contacts with full name matching '{}'", name);
    let name_parts: Vec<&str> = name.split_whitespace().collect();

    if name_parts.len() >= 2 {
        let first_name = name_parts[0];
        let last_name = name_parts[name_parts.len() - 1];

        // Look for contacts with BOTH first AND last name match
        let combined_script = format!(
            r#"tell application "Contacts"
                set the_emails to {{}}
                try
                    set the_people to (every person whose first name contains "{}" and last name contains "{}")
                    repeat with the_person in the_people
                        if exists email of the_person then
                            log "Found matching contact: " & (name of the_person)
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
            first_name.replace("\"", "\\\""),
            last_name.replace("\"", "\\\"")
        );

        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&combined_script)
            .output()
            .await
            .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;

        if output.status.success() {
            let emails = String::from_utf8_lossy(&output.stdout);
            debug!("Raw contact lookup output (first+last name match): '{}'", emails);

            let email_list: Vec<String> = emails
                .trim_matches('{')
                .trim_matches('}')
                .split(", ")
                .filter(|s| !s.is_empty() && !s.contains("missing value"))
                .map(|s| s.trim_matches('"').trim().to_string())
                .filter(|email| validate_email(email))
                .collect();

            if !email_list.is_empty() {
                info!(
                    "Found {} email(s) for '{}' with first+last name match: {:?}",
                    email_list.len(),
                    name,
                    email_list
                );
                return Ok(email_list);
            }
        }
    }

    // If exact match returned no results, try contains match
    debug!("No exact match found for '{}', trying contains match", name);
    let contains_script = format!(
        r#"tell application "Contacts"
            set the_emails to {{}}
            try
                set searchText to "{}"
                set the_people to (every person whose name contains searchText)
                repeat with the_person in the_people
                    set personName to name of the_person
                    if personName contains searchText then
                        if exists email of the_person then
                            log "Found contains match: " & personName
                            repeat with the_email in (get every email of the_person)
                                if value of the_email is not missing value then
                                    set the end of the_emails to (value of the_email as text)
                                end if
                            end repeat
                        end if
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
        .arg(&contains_script)
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute AppleScript: {}", e))?;

    let mut email_list = Vec::new();

    if output.status.success() {
        let emails = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            debug!("AppleScript stderr (contains match): '{}'", stderr);
        }

        email_list = emails
            .trim_matches('{')
            .trim_matches('}')
            .split(", ")
            .filter(|s| !s.is_empty() && !s.contains("missing value"))
            .map(|s| s.trim_matches('"').trim().to_string())
            .filter(|email| validate_email(email))
            .collect();

        if !email_list.is_empty() {
            info!(
                "Found {} email(s) for '{}' with contains match: {:?}",
                email_list.len(),
                name,
                email_list
            );
            return Ok(email_list);
        } else {
            debug!("No emails found via contains match for contact '{}'", name);
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        error!("Contact lookup error (contains match): {}", error);
    }

    if email_list.is_empty() {
        info!("No emails found for contact '{}' after all search attempts", name);
    }

    Ok(email_list)
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
    use crate::calendar::calendar_validation::validate_email;
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
                    found_emails.extend(emails.into_iter().filter(|e| validate_email(e)));
                }
            }
            Err(e) => {
                error!("Failed to lookup contact '{}': {}", name, e);
            }
        }
    }

    // Merge and deduplicate emails
    let mut all_emails = config.emails.clone();
    all_emails.extend(found_emails);
    all_emails.sort();
    all_emails.dedup();
    config.emails = all_emails;

    create_event(config).await
}
