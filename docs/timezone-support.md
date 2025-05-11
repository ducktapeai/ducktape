# Timezone Support in Ducktape

Ducktape now supports timezone handling in natural language commands. This feature allows you to schedule events in different timezones, and Ducktape will automatically convert the times to your local timezone.

## How to Use Timezone Support

When creating events or meetings, you can specify the timezone along with the time:

```
schedule a meeting at 9pm PST called West Coast Sync
```

Ducktape will recognize the timezone (PST in this example) and convert the time to your local timezone before creating the event.

## Supported Timezone Abbreviations

Ducktape supports the following timezone abbreviations:

| Abbreviation | Timezone |
|--------------|----------|
| PST/PDT | Pacific Time (US & Canada) |
| MST/MDT | Mountain Time (US & Canada) |
| CST/CDT | Central Time (US & Canada) |
| EST/EDT | Eastern Time (US & Canada) |
| AKST/AKDT | Alaska Time |
| HST/HDT | Hawaii Time |
| GMT | Greenwich Mean Time |
| BST | British Summer Time |
| IST | Indian Standard Time |
| CET/CEST | Central European Time |
| EET/EEST | Eastern European Time |
| MSK | Moscow Time |
| AEST/AEDT | Australian Eastern Time |
| ACST/ACDT | Australian Central Time |
| AWST | Australian Western Time |
| NZST/NZDT | New Zealand Time |
| JST | Japan Standard Time |
| KST | Korea Standard Time |
| UTC | Coordinated Universal Time |

## Examples

Here are some examples of using timezone support in Ducktape:

1. **Schedule a meeting in Pacific Time:**
   ```
   schedule a meeting at 3pm PST called Team Sync
   ```

2. **Set up a call in Eastern Time:**
   ```
   create a call at 9am EST with marketing team
   ```

3. **Add an event in Japan Time:**
   ```
   add event at 7pm JST called Tokyo Office Hours
   ```

4. **Schedule a Zoom meeting in GMT:**
   ```
   schedule a zoom meeting at 2pm GMT called International Sync
   ```

## How It Works

When you specify a time with a timezone abbreviation, Ducktape:

1. Identifies the timezone abbreviation in your command
2. Parses the time expression (e.g., "3pm PST")
3. Converts the time from the specified timezone to your local timezone
4. Creates the event at the correct local time

If no timezone is specified, Ducktape assumes the time is in your local timezone.

## Testing Timezone Support

The repository includes several test scripts to verify timezone functionality:

- `test_timezone.sh` - A basic test for timezone parsing and conversion
- `test_timezone_comprehensive.sh` - A more comprehensive test suite
- `test_timezone_ducktape.sh` - Integration tests with the Ducktape application

## Known Limitations

- Timezone conversion relies on the system's timezone settings
- Only timezone abbreviations are supported (not full names like "Pacific Standard Time")
- In ambiguous cases (like during DST transitions), the most common interpretation is used
