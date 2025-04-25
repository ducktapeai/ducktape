/// Extract contact names from natural language input
///
/// This function attempts to identify contact names in a natural language input
/// using different patterns like "with Person", "invite Person", etc.
pub fn extract_contact_names(input: &str) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let mut contacts = Vec::new();

    debug!("Extracting contact names from: '{}'", input);
    
    // Pattern 1: Handle "with Person" pattern
    if let Some(idx) = input_lower.find(" with ") {
        if idx + 6 < input.len() {
            let text_to_parse = &input[idx + 6..]; // Skip " with "
            debug!("Found 'with' keyword for contact extraction");
            debug!("Text to parse for contacts: '{}'", text_to_parse);
            
            // Extract until the end of the string or a keyword that would end the contact name
            let end_markers = [" about ", " at ", " on ", " from ", " for ", " in ", " to ", " regarding ", " re: "];
            let mut end_pos = text_to_parse.len();
            
            for marker in &end_markers {
                if let Some(pos) = text_to_parse.to_lowercase().find(marker) {
                    if pos < end_pos {
                        end_pos = pos;
                    }
                }
            }
            
            let contact_text = text_to_parse[..end_pos].trim();
            
            // Process multiple contacts separated by "and" or commas
            extract_multiple_contacts(contact_text, &mut contacts);
        }
    }
    
    // Pattern 2: Handle "invite Person" pattern
    if let Some(idx) = input_lower.find(" invite ") {
        if idx + 8 < input.len() {
            let text_to_parse = &input[idx + 8..]; // Skip " invite "
            debug!("Found 'invite' keyword for contact extraction");
            debug!("Text to parse for contacts: '{}'", text_to_parse);
            
            // Extract until the end of the string or a keyword that would end the contact name
            let end_markers = [" about ", " at ", " on ", " from ", " for ", " in ", " to ", " regarding ", " re: "];
            let mut end_pos = text_to_parse.len();
            
            for marker in &end_markers {
                if let Some(pos) = text_to_parse.to_lowercase().find(marker) {
                    if pos < end_pos {
                        end_pos = pos;
                    }
                }
            }
            
            let contact_text = text_to_parse[..end_pos].trim();
            
            // Process multiple contacts separated by "and" or commas
            extract_multiple_contacts(contact_text, &mut contacts);
        }
    }
    
    // Pattern 3: Handle "and invite Person" pattern
    if let Some(idx) = input_lower.find(" and invite ") {
        if idx + 12 < input.len() {
            let text_to_parse = &input[idx + 12..]; // Skip " and invite "
            debug!("Found 'and invite' keyword for contact extraction");
            debug!("Text to parse for contacts: '{}'", text_to_parse);
            
            // Extract until the end of the string or a keyword that would end the contact name
            let end_markers = [" about ", " at ", " on ", " from ", " for ", " in ", " to ", " regarding ", " re: "];
            let mut end_pos = text_to_parse.len();
            
            for marker in &end_markers {
                if let Some(pos) = text_to_parse.to_lowercase().find(marker) {
                    if pos < end_pos {
                        end_pos = pos;
                    }
                }
            }
            
            let contact_text = text_to_parse[..end_pos].trim();
            
            // Process multiple contacts separated by "and" or commas
            extract_multiple_contacts(contact_text, &mut contacts);
        }
    }

    debug!("Extracted contact names: {:?}", contacts);
    contacts
}

/// Helper function to extract multiple contacts from a text string
/// that might contain names separated by commas or "and"
fn extract_multiple_contacts(text: &str, contacts: &mut Vec<String>) {
    debug!("Extracting multiple contacts from: '{}'", text);
    
    // First, split by commas
    let comma_parts: Vec<&str> = text.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    
    for part in comma_parts {
        // For each comma-separated part, also split by " and "
        let and_parts: Vec<&str> = part.split(" and ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        
        for name in and_parts {
            if !name.is_empty() {
                debug!("Adding contact: '{}'", name);
                contacts.push(name.to_string());
            }
        }
    }
}