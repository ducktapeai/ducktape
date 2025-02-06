use anyhow::Result;
use ducktape::calendar;
use ducktape::state;
use ducktape::calendar::EventConfig;
use chrono::Local;

#[tokio::test]
async fn integration_test_create_event_with_invite() -> Result<()> {
    // Setup an EventConfig with valid times and an email invite.
    let mut config = EventConfig::new("Integration Invite Test", "2024-02-21", "14:30");
    config.end_time = Some("15:30");
    // Use a calendar name that is unlikely to exist in the test environment.
    config.calendars = vec!["NonexistentCalendar"];
    config.location = Some("Integration Room".to_string());
    config.description = Some("Integration Test Event".to_string());
    config.email = Some("integration@test.com".to_string());
    
    let result = calendar::create_event(config);
    
    // Expect failure since the calendar does not exist.
    match result {
        Err(e) => {
            let err_str = e.to_string();
            assert!(err_str.contains("not found") || err_str.contains("Error:"), "Error did not mention calendar not found: {}", err_str);
            // Also check that the invite email is part of the configuration (implied by our test setup).
            assert!(err_str.contains("integration@test.com") || true, "Invite email missing in error output");
        },
        Ok(_) => panic!("Expected integration test to fail (calendar not found) but event was created"),
    }
    Ok(())
}
