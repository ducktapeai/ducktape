fn extract_multiple_contacts(text: &str) -> Vec<String> {
    let mut contacts = Vec::new();
    
    // First, split by commas
    let comma_parts: Vec<&str> = text.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    
    for part in comma_parts {
        // For each comma-separated part, also split by " and "
        let and_parts: Vec<&str> = part.split(" and ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        
        for name in and_parts {
            if !name.is_empty() {
                println!("Adding contact: '{}'", name);
                contacts.push(name.to_string());
            }
        }
    }
    
    contacts
}

fn main() {
    println!("Test 1: Single name");
    let contacts = extract_multiple_contacts("Shaun Stuart");
    println!("Extracted: {:?}", contacts);
    
    println!("\nTest 2: Multiple names with 'and'");
    let contacts = extract_multiple_contacts("Shaun Stuart and Joe Buck");
    println!("Extracted: {:?}", contacts);
    
    println!("\nTest 3: Multiple names with comma");
    let contacts = extract_multiple_contacts("Shaun Stuart, Joe Buck");
    println!("Extracted: {:?}", contacts);
    
    println!("\nTest 4: Multiple names with both comma and 'and'");
    let contacts = extract_multiple_contacts("Shaun Stuart, Joe Buck and Jane Doe");
    println!("Extracted: {:?}", contacts);
}
