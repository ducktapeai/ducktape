use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use log::debug;
use phf::phf_map;
use regex::Regex;

/// Parse a time string like "8pm" into a 24-hour format time
///
/// # Arguments
///
/// * `time_str` - The time string to parse (e.g., "8pm", "10:30am")
///
/// # Returns
///
/// * `Option<(u32, u32)>` - The parsed hour and minute in 24-hour format
pub fn parse_time_with_ampm(time_str: &str) -> Option<(u32, u32)> {
    // Create regex pattern to extract hour, minute, and am/pm
    let re = Regex::new(r"(?i)^(\d{1,2})(?::(\d{2}))?\s*([ap]\.?m\.?)?$").unwrap();

    let time_lower = time_str.to_lowercase();
    let am_pm_present = time_lower.contains("am") || time_lower.contains("pm");

    if let Some(caps) = re.captures(&time_lower) {
        let hour_str = caps.get(1).unwrap().as_str();
        let minute_str = caps.get(2).map_or("0", |m| m.as_str()); // Default to 0 if no minutes
        let ampm_str_opt = caps.get(3).map(|m| m.as_str().to_lowercase());

        if let (Ok(h_val), Ok(m_val)) = (hour_str.parse::<u32>(), minute_str.parse::<u32>()) {
            // Convert to 24-hour format
            let hour_24 = if let Some(ampm) = ampm_str_opt {
                if ampm.starts_with('p') && h_val < 12 {
                    h_val + 12
                } else if ampm.starts_with('a') && h_val == 12 {
                    0
                } else {
                    h_val
                }
            } else if am_pm_present {
                // If am/pm is present in string but not captured by regex
                if time_lower.contains("pm") && h_val < 12 {
                    h_val + 12
                } else if time_lower.contains("am") && h_val == 12 {
                    0
                } else {
                    h_val
                }
            } else {
                h_val
            };

            // Return parsed time if valid
            if hour_24 < 24 && m_val < 60 {
                return Some((hour_24, m_val));
            }
        }
    }

    None
}

/// Extract time and timezone from string
///
/// # Arguments
///
/// * `input` - The input string containing time information (e.g., "at 8pm PST")
///
/// # Returns
///
/// * `Option<(String, String, String)>` - A tuple with (date, start_time, end_time) in standard format
pub fn extract_time_info(input: &str) -> Option<(String, String, String)> {
    // Look for common time patterns with potential timezone abbreviations
    let re_time_with_tz =
        Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?)(?:\s+[A-Z]{3,4})?)\b").unwrap();

    if let Some(time_match) = re_time_with_tz.find(input) {
        let time_str = time_match.as_str();
        debug!("Found time string: {}", time_str);

        // Parse the time with potential timezone
        if let Some((hour, minute, timezone)) = parse_time_with_timezone(time_str) {
            // Create date string (today)
            let today = Local::now().format("%Y-%m-%d").to_string();

            // Get the start time adjusted for timezone if needed
            let (adjusted_hour, adjusted_minute) = if let Some(tz) = timezone {
                adjust_time_for_timezone(hour, minute, tz)
            } else {
                (hour, minute)
            };

            // Create formatted start and end times
            let start_time = format!("{:02}:{:02}", adjusted_hour, adjusted_minute);

            // Set end time 1 hour later
            let end_hour = if adjusted_hour == 23 { 0 } else { adjusted_hour + 1 };
            let end_time = format!("{:02}:{:02}", end_hour, adjusted_minute);

            debug!("Extracted time with timezone adjustment: {} -> {}", time_str, start_time);
            return Some((today, start_time, end_time));
        }
    }

    // Fall back to original implementation without timezone support
    let re_time = Regex::new(r"(?i)\b(\d{1,2}(?::\d{2})?\s*(?:[ap]\.?m\.?))\b").unwrap();

    if let Some(time_match) = re_time.find(input) {
        let time_str = time_match.as_str();

        // Parse the time
        if let Some((hour, minute)) = parse_time_with_ampm(time_str) {
            // Create date string (today)
            let today = Local::now().format("%Y-%m-%d").to_string();

            // Create formatted start and end times
            let start_time = format!("{:02}:{:02}", hour, minute);

            // Set end time 1 hour later
            let end_hour = if hour == 23 { 0 } else { hour + 1 };
            let end_time = format!("{:02}:{:02}", end_hour, minute);

            debug!("Extracted time without timezone: {} -> {}", time_str, start_time);
            return Some((today, start_time, end_time));
        }
    }

    None
}

/// Adjust time from source timezone to local timezone
///
/// # Arguments
///
/// * `hour` - Hour in source timezone
/// * `minute` - Minute in source timezone
/// * `source_tz` - Source timezone
///
/// # Returns
///
/// * `(u32, u32)` - Hour and minute adjusted to local timezone
fn adjust_time_for_timezone(hour: u32, minute: u32, source_tz: Tz) -> (u32, u32) {
    // Get today's date
    let today = Local::now().date_naive();

    // Create a naive datetime in the source timezone
    let naive_dt = match NaiveTime::from_hms_opt(hour, minute, 0) {
        Some(nt) => NaiveDateTime::new(today, nt),
        None => return (hour, minute), // Invalid time, return unchanged
    };

    // Convert to source timezone
    let source_dt = match source_tz.from_local_datetime(&naive_dt).single() {
        Some(dt) => dt,
        None => return (hour, minute), // Ambiguous time, return unchanged
    };

    // Convert to UTC first (to handle DST and other timezone complexities)
    let utc_dt = source_dt.with_timezone(&Utc);

    // Then convert to local timezone
    let local_dt = utc_dt.with_timezone(&Local);

    debug!(
        "Timezone conversion: {}:{:02} {} -> {}:{:02} local",
        hour,
        minute,
        source_tz.name(),
        local_dt.hour(),
        local_dt.minute()
    );

    (local_dt.hour(), local_dt.minute())
}

/// Process a natural language command to extract time information
///
/// # Arguments
///
/// * `command` - The original command string
/// * `input` - The natural language input
///
/// # Returns
///
/// * `String` - The updated command with correct time information
pub fn process_time_in_command(command: &str, input: &str) -> String {
    if let Some((date, start_time, end_time)) = extract_time_info(input) {
        // Replace placeholders in command
        let mut processed = command.to_string();

        // If the command contains "today", replace it with the date
        if processed.contains("today") {
            processed = processed.replace("today", &date);
        }

        // Replace time placeholders (assumes 00:00 and 01:00 are placeholders)
        if processed.contains("00:00") {
            processed = processed.replace("00:00", &start_time);

            // Also replace end time if present
            if processed.contains("01:00") {
                processed = processed.replace("01:00", &end_time);
            }
        }

        return processed;
    }

    // Return original command if no time info found
    command.to_string()
}

// Timezone abbreviation map - copied from time_extractor.rs to maintain consistency
const TIMEZONE_ABBR_MAP: phf::Map<&'static str, Tz> = phf_map! {
    "PST" => Tz::America__Los_Angeles, // Pacific Standard Time
    "PDT" => Tz::America__Los_Angeles, // Pacific Daylight Time
    "MST" => Tz::America__Denver,     // Mountain Standard Time
    "MDT" => Tz::America__Denver,     // Mountain Daylight Time
    "CST" => Tz::America__Chicago,    // Central Standard Time
    "CDT" => Tz::America__Chicago,    // Central Daylight Time
    "EST" => Tz::America__New_York,   // Eastern Standard Time
    "EDT" => Tz::America__New_York,   // Eastern Daylight Time
    "AKST" => Tz::America__Anchorage, // Alaska Standard Time
    "AKDT" => Tz::America__Anchorage, // Alaska Daylight Time
    "HST" => Tz::Pacific__Honolulu,   // Hawaii Standard Time
    "HDT" => Tz::Pacific__Honolulu,   // Hawaii Daylight Time (though Hawaii doesn't observe DST)
    "GMT" => Tz::Etc__GMT,            // Greenwich Mean Time
    "BST" => Tz::Europe__London,      // British Summer Time
    "IST" => Tz::Asia__Kolkata,       // Indian Standard Time
    "CET" => Tz::Europe__Berlin,      // Central European Time
    "CEST" => Tz::Europe__Berlin,     // Central European Summer Time
    "EET" => Tz::Europe__Helsinki,    // Eastern European Time
    "EEST" => Tz::Europe__Helsinki,   // Eastern European Summer Time
    "MSK" => Tz::Europe__Moscow,      // Moscow Time
    "AEST" => Tz::Australia__Sydney,  // Australian Eastern Standard Time
    "AEDT" => Tz::Australia__Sydney,  // Australian Eastern Daylight Time
    "ACST" => Tz::Australia__Adelaide,// Australian Central Standard Time
    "ACDT" => Tz::Australia__Adelaide,// Australian Central Daylight Time
    "AWST" => Tz::Australia__Perth,   // Australian Western Standard Time
    "NZST" => Tz::Pacific__Auckland,  // New Zealand Standard Time
    "NZDT" => Tz::Pacific__Auckland,  // New Zealand Daylight Time
    "JST" => Tz::Asia__Tokyo,         // Japan Standard Time
    "KST" => Tz::Asia__Seoul,         // Korea Standard Time
    "UTC" => Tz::UTC,                 // Coordinated Universal Time
};

/// Helper function to map timezone abbreviation to chrono_tz timezone
pub fn map_timezone_abbr(abbr: &str) -> Option<Tz> {
    let upper_abbr = abbr.to_uppercase();
    debug!("Mapping timezone abbreviation: {}", upper_abbr);
    TIMEZONE_ABBR_MAP.get(&upper_abbr).cloned()
}

/// Parse time string with timezone support
///
/// This is an enhanced version of parse_time_with_ampm that also handles timezone abbreviations
///
/// # Arguments
///
/// * `time_str` - The time string to parse (e.g., "8pm PST")
///
/// # Returns
///
/// * `Option<(u32, u32, Option<Tz>)>` - The parsed hour, minute in 24-hour format, and optional timezone
pub fn parse_time_with_timezone(time_str: &str) -> Option<(u32, u32, Option<Tz>)> {
    // First, check if there's a timezone abbreviation at the end
    let mut time_parts = time_str.trim().split_whitespace().collect::<Vec<&str>>();
    let mut timezone = None;

    if time_parts.len() > 1 {
        // Last part might be a timezone abbreviation
        let last_part = time_parts.last().unwrap().to_uppercase();
        if let Some(tz) = map_timezone_abbr(&last_part) {
            debug!("Found timezone: {}", tz.name());
            timezone = Some(tz);
            // Remove timezone part from the time string
            time_parts.pop();
        }
    }

    // Join remaining parts and parse as regular time
    let time_without_tz = time_parts.join(" ");
    if let Some((hour, minute)) = parse_time_with_ampm(&time_without_tz) {
        return Some((hour, minute, timezone));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_time_with_ampm() {
        // Test basic time parsing
        assert_eq!(parse_time_with_ampm("8pm"), Some((20, 0)));
        assert_eq!(parse_time_with_ampm("10:30am"), Some((10, 30)));
        assert_eq!(parse_time_with_ampm("12pm"), Some((12, 0)));
        assert_eq!(parse_time_with_ampm("12am"), Some((0, 0)));
    }

    #[test]
    fn test_parse_time_with_timezone() {
        // Test timezone parsing
        let result = parse_time_with_timezone("8pm PST");
        assert!(result.is_some());
        let (hour, minute, tz) = result.unwrap();
        assert_eq!(hour, 20);
        assert_eq!(minute, 0);
        assert!(tz.is_some());
        assert_eq!(tz.unwrap().name(), "America/Los_Angeles");

        let result = parse_time_with_timezone("3:30pm EST");
        assert!(result.is_some());
        let (hour, minute, tz) = result.unwrap();
        assert_eq!(hour, 15);
        assert_eq!(minute, 30);
        assert!(tz.is_some());
        assert_eq!(tz.unwrap().name(), "America/New_York");
    }

    #[test]
    fn test_adjust_time_for_timezone() {
        // This test is timezone-dependent, so we'll just make sure it doesn't crash
        // and returns sensible values
        let pst = map_timezone_abbr("PST").unwrap();
        let (hour, minute) = adjust_time_for_timezone(20, 0, pst);

        // Results will vary depending on local timezone and DST status,
        // so just make sure they're valid time values
        assert!(hour < 24);
        assert!(minute < 60);

        // Test conversion from EST to local
        let est = map_timezone_abbr("EST").unwrap();
        let (hour, minute) = adjust_time_for_timezone(15, 30, est);
        assert!(hour < 24);
        assert!(minute < 60);
    }

    #[test]
    fn test_extract_time_info() {
        // Test with no timezone
        let input = "create an event at 8pm";
        let result = extract_time_info(input);
        assert!(result.is_some());
        let (date, start_time, end_time) = result.unwrap();
        assert_eq!(start_time, "20:00");
        assert_eq!(end_time, "21:00");

        // Test with timezone
        let input = "schedule a meeting at 9pm PST";
        let result = extract_time_info(input);
        assert!(result.is_some());
        let (date, start_time, end_time) = result.unwrap();

        // Local time will be adjusted from PST - can't predict exact values in test
        // So we just check that the time format is correct and end_time is 1 hour after start_time
        assert!(start_time.len() == 5); // format is xx:xx
        assert!(start_time.contains(":"));
        assert!(end_time.len() == 5);
        assert!(end_time.contains(":"));

        let start_parts: Vec<&str> = start_time.split(':').collect();
        let end_parts: Vec<&str> = end_time.split(':').collect();

        let start_hour: u32 = start_parts[0].parse().unwrap();
        let end_hour: u32 = end_parts[0].parse().unwrap();

        assert_eq!((start_hour + 1) % 24, end_hour);
    }

    #[test]
    fn test_process_time_in_command() {
        let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
        let input = "create meeting at 8pm";
        let result = process_time_in_command(command, input);
        assert!(result.contains("20:00"));
        assert!(result.contains("21:00"));

        let command = "ducktape calendar create \"Meeting\" today 00:00 01:00 \"Work\"";
        let input = "schedule a call at 9am PST";
        let result = process_time_in_command(command, input);

        // Times will be adjusted for timezone, so we just check format
        assert!(result.contains(":00"));
        assert!(!result.contains("00:00")); // placeholders should be replaced
    }
}
