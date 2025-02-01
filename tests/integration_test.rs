use anyhow::Result;

#[test]
fn test_calendar_list() -> Result<()> {
    // This test requires Calendar.app to be running and accessible
    ducktape::calendar::list_calendars()?;
    Ok(())
}

#[test]
fn test_create_basic_event() -> Result<()> {
    let config = ducktape::calendar::EventConfig {
        title: "Integration Test Event",
        date: "2024-02-21",
        time: "14:30",
        calendar: Some("Calendar"),
        all_day: false,
        location: None,
        description: None,
        email: None,
    };

    ducktape::calendar::create_event(config)?;
    Ok(())
}

#[test]
fn test_create_all_day_event() -> Result<()> {
    let config = ducktape::calendar::EventConfig {
        title: "Integration Test All-Day Event",
        date: "2024-02-21",
        time: "00:00",
        calendar: Some("Calendar"),
        all_day: true,
        location: None,
        description: None,
        email: None,
    };

    ducktape::calendar::create_event(config)?;
    Ok(())
}
