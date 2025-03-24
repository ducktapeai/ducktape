use anyhow::Result;
use ducktape::calendar::{
    self, CalendarError, EventConfig, RecurrenceFrequency, RecurrencePattern,
};

#[tokio::test]
async fn integration_test_create_event_with_invite() -> Result<()> {
    // Setup an EventConfig with valid times and an email invite.
    let mut config = EventConfig::new("Integration Invite Test", "2024-02-21", "14:30");
    config.end_time = Some("15:30".to_string());
    // Use a calendar name that is unlikely to exist in the test environment.
    config.calendars = vec!["NonexistentCalendar".to_string()];
    config.location = Some("Integration Room".to_string());
    config.description = Some("Integration Test Event".to_string());
    config.emails = vec!["integration@test.com".to_string()];

    let result = calendar::create_event(config).await;

    // Expect failure since the calendar does not exist.
    match result {
        Err(e) => {
            // Try to downcast to CalendarError
            if let Some(calendar_err) = e.downcast_ref::<CalendarError>() {
                match calendar_err {
                    CalendarError::CalendarNotFound(_) => (), // Expected error
                    _ => panic!("Unexpected calendar error: {:?}", calendar_err),
                }
            } else {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("not found") || err_str.contains("Calendar"),
                    "Error did not mention calendar not found: {}",
                    err_str
                );
            }
        }
        Ok(_) => {
            panic!("Expected integration test to fail (calendar not found) but event was created")
        }
    }
    Ok(())
}

/// This test verifies the recurrence pattern builder for different recurrence scenarios
#[test]
fn test_recurrence_pattern_scenarios() -> Result<()> {
    // Daily recurrence
    let daily = RecurrencePattern::new(RecurrenceFrequency::Daily)
        .with_interval(1)
        .with_count(10);
    assert_eq!(daily.frequency, RecurrenceFrequency::Daily);
    assert_eq!(daily.count, Some(10));

    // Weekly recurrence on specific days
    let weekly = RecurrencePattern::new(RecurrenceFrequency::Weekly)
        .with_interval(2) // Bi-weekly
        .with_days_of_week(&[1, 3, 5]); // Monday, Wednesday, Friday
    assert_eq!(weekly.frequency, RecurrenceFrequency::Weekly);
    assert_eq!(weekly.interval, 2);
    assert_eq!(weekly.days_of_week, vec![1, 3, 5]);

    // Monthly recurrence with end date
    let monthly = RecurrencePattern::new(RecurrenceFrequency::Monthly)
        .with_interval(1)
        .with_end_date("2025-12-31");
    assert_eq!(monthly.frequency, RecurrenceFrequency::Monthly);
    assert_eq!(monthly.end_date, Some("2025-12-31".to_string()));

    // Yearly recurrence
    let yearly = RecurrencePattern::new(RecurrenceFrequency::Yearly);
    assert_eq!(yearly.frequency, RecurrenceFrequency::Yearly);
    assert_eq!(yearly.interval, 1); // Default interval

    Ok(())
}

/// Test the creation of event configurations with different recurrence patterns
#[test]
fn test_event_config_with_recurrence() -> Result<()> {
    // Create an event that repeats daily with end date
    let daily_recurrence =
        RecurrencePattern::new(RecurrenceFrequency::Daily).with_end_date("2024-12-31");
    let mut config = EventConfig::new("Daily Meeting", "2024-05-01", "10:00");
    config = config.with_recurrence(daily_recurrence.clone());

    assert!(config.recurrence.is_some());
    let rec = config.recurrence.unwrap();
    assert_eq!(rec.frequency, RecurrenceFrequency::Daily);
    assert_eq!(rec.end_date, Some("2024-12-31".to_string()));

    // Create a weekly meeting on Monday and Thursday
    let weekly_recurrence =
        RecurrencePattern::new(RecurrenceFrequency::Weekly).with_days_of_week(&[1, 4]); // Monday and Thursday
    let mut config = EventConfig::new("Team Sync", "2024-05-06", "14:00");
    config.end_time = Some("15:00".to_string());
    config = config.with_recurrence(weekly_recurrence);

    assert!(config.recurrence.is_some());
    let rec = config.recurrence.unwrap();
    assert_eq!(rec.frequency, RecurrenceFrequency::Weekly);
    assert_eq!(rec.days_of_week, vec![1, 4]);

    // Create a monthly meeting limited to 6 occurrences
    let monthly_recurrence = RecurrencePattern::new(RecurrenceFrequency::Monthly).with_count(6);
    let mut config = EventConfig::new("Monthly Review", "2024-05-15", "09:00");
    config.end_time = Some("10:30".to_string());
    config = config.with_recurrence(monthly_recurrence);

    assert!(config.recurrence.is_some());
    let rec = config.recurrence.unwrap();
    assert_eq!(rec.frequency, RecurrenceFrequency::Monthly);
    assert_eq!(rec.count, Some(6));

    // Create a yearly anniversary event
    let yearly_recurrence = RecurrencePattern::new(RecurrenceFrequency::Yearly);
    let mut config = EventConfig::new("Company Anniversary", "2024-06-01", "12:00");
    config.all_day = true;
    config = config.with_recurrence(yearly_recurrence);

    assert!(config.recurrence.is_some());
    let rec = config.recurrence.unwrap();
    assert_eq!(rec.frequency, RecurrenceFrequency::Yearly);

    Ok(())
}

/// This test mocks the string generation for recurrence rules to validate the format
#[test]
fn test_recurrence_rule_generation() -> Result<()> {
    // Build RFC 5545 rule parts for daily recurrence
    let daily = RecurrencePattern::new(RecurrenceFrequency::Daily).with_count(5);
    let parts = vec![
        format!("FREQ={}", daily.frequency.to_rfc5545()),
        format!("INTERVAL={}", daily.interval),
        format!("COUNT={}", daily.count.unwrap()),
    ];
    let rule = parts.join(";");
    assert_eq!(rule, "FREQ=DAILY;INTERVAL=1;COUNT=5");

    // Build RFC 5545 rule for weekly recurrence
    let weekly = RecurrencePattern::new(RecurrenceFrequency::Weekly)
        .with_interval(2)
        .with_days_of_week(&[1, 3, 5]); // Monday, Wednesday, Friday
    let mut parts = vec![
        format!("FREQ={}", weekly.frequency.to_rfc5545()),
        format!("INTERVAL={}", weekly.interval),
    ];
    let byday = "BYDAY=MO,WE,FR";
    parts.push(byday.to_string());
    let rule = parts.join(";");
    assert_eq!(rule, "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE,FR");

    // Test with end date
    let monthly = RecurrencePattern::new(RecurrenceFrequency::Monthly).with_end_date("2025-12-31");
    let mut parts = vec![
        format!("FREQ={}", monthly.frequency.to_rfc5545()),
        format!("INTERVAL={}", monthly.interval),
    ];
    // In a real scenario, end_date would be converted to a proper UNTIL format
    // This is just testing the string composition
    parts.push(format!("UNTIL=20251231T235900Z"));
    let rule = parts.join(";");
    assert_eq!(rule, "FREQ=MONTHLY;INTERVAL=1;UNTIL=20251231T235900Z");

    Ok(())
}

#[tokio::test]
async fn test_create_calendar_event() -> Result<()> {
    use ducktape::calendar::{create_event, EventConfig};

    let mut config = EventConfig::new("Test Event", "2024-02-01", "14:30");
    config.end_time = Some("15:30".to_string());

    let result = create_event(config).await;
    assert!(
        result.is_ok(),
        "Failed to create calendar event: {:?}",
        result
    );
    Ok(())
}

#[tokio::test]
async fn test_create_event_with_contacts() -> Result<()> {
    use ducktape::calendar::{create_event, EventConfig};

    let mut config = EventConfig::new("Meeting with Team", "2024-02-01", "14:00");
    config.end_time = Some("15:00".to_string());
    config.calendars = vec!["Work".to_string()];
    config.emails = vec!["john.doe@example.com".to_string()];

    let result = create_event(config).await;
    assert!(
        result.is_ok(),
        "Failed to create calendar event with contacts: {:?}",
        result
    );
    Ok(())
}

#[tokio::test]
async fn test_create_short_event() -> Result<()> {
    use ducktape::calendar::{create_event, EventConfig};

    let mut config = EventConfig::new("Quick Sync", "2024-02-01", "10:00");
    config.end_time = Some("10:30".to_string());

    let result = create_event(config).await;
    assert!(
        result.is_ok(),
        "Failed to create short calendar event: {:?}",
        result
    );
    Ok(())
}
