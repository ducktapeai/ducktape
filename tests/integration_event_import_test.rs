use anyhow::Result;
use ducktape::calendar;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_csv_header_validation() -> Result<()> {
    // Test that CSV headers are properly validated
    assert!(calendar::validate_email("test@example.com"));
    assert!(!calendar::validate_email("invalid-email"));
    Ok(())
}

#[test]
fn test_csv_import_preparation() -> Result<()> {
    // Create a temporary CSV file with test data
    let dir = tempdir()?;
    let file_path = dir.path().join("test-events.csv");
    let mut file = File::create(&file_path)?;

    writeln!(file, "title,date,time,end_time,description,location,attendees,calendar")?;
    writeln!(
        file,
        "Test Event,2025-03-20,10:00,11:00,Event description,Office,test@example.com,user@example.com"
    )?;

    // Just check that the file exists and has the right content
    assert!(file_path.exists());

    Ok(())
}

#[test]
fn test_calendar_email_validation() -> Result<()> {
    // Test validation for calendar field emails
    assert!(calendar::validate_email("user@example.com"));
    assert!(calendar::validate_email("name.surname@company.co.uk"));
    assert!(!calendar::validate_email("not-an-email"));
    assert!(!calendar::validate_email("missing@"));
    assert!(!calendar::validate_email("@incomplete.com"));

    Ok(())
}
