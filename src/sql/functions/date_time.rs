use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Utc};

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::config::global::get_date_notation;
use crate::data::datatable::DataValue;

// Helper function for parsing dates with multiple format support
pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    // Try parsing as ISO 8601 with timezone first (most unambiguous)
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }

    // ISO formats (most common and unambiguous)
    // With T separator
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f") {
        return Ok(Utc.from_utc_datetime(&dt));
    }
    // With space separator
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.3f") {
        return Ok(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
        return Ok(Utc.from_utc_datetime(&dt));
    }
    // Without milliseconds
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
    }

    // Get date notation preference
    let date_notation = get_date_notation();

    // Date notation preference for ambiguous formats like 04/09/2025
    if date_notation == "european" {
        // European formats (DD/MM/YYYY) - try first
        // Date only formats
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        // With time formats (with milliseconds)
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S%.3f") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d-%m-%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }

        // US formats (MM/DD/YYYY) - fallback
        // Date only formats
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m-%d-%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        // With time formats (with milliseconds)
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S%.3f") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m-%d-%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
    } else {
        // US formats (MM/DD/YYYY) - default, try first
        // Date only formats
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%m-%d-%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        // With time formats (with milliseconds)
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S%.3f") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m-%d-%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }

        // European formats (DD/MM/YYYY) - fallback
        // Date only formats
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
            return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
        }
        // With time formats (with milliseconds)
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S%.3f") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d/%m/%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d-%m-%Y %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
    }

    // Excel/Windows format: DD-MMM-YYYY (e.g., 15-Jan-2024)
    if let Ok(dt) = NaiveDate::parse_from_str(s, "%d-%b-%Y") {
        return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
    }

    // Full month names: January 15, 2024 or 15 January 2024
    if let Ok(dt) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
        return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
    }
    if let Ok(dt) = NaiveDate::parse_from_str(s, "%d %B %Y") {
        return Ok(Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()));
    }

    // RFC3339 (e.g., 2024-01-15T10:30:00Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    Err(anyhow!("Could not parse date: {}. Supported formats: YYYY-MM-DD, MM/DD/YYYY, DD/MM/YYYY, DD-MMM-YYYY, Month DD YYYY", s))
}

/// NOW function - Returns current datetime
pub struct NowFunction;

impl SqlFunction for NowFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NOW",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(0),
            description: "Returns the current date and time",
            returns: "DATETIME",
            examples: vec![
                "SELECT NOW()",
                "SELECT * FROM orders WHERE created_at > NOW() - 7",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let now = Utc::now();
        Ok(DataValue::DateTime(
            now.format("%Y-%m-%d %H:%M:%S").to_string(),
        ))
    }
}

/// TODAY function - Returns current date
pub struct TodayFunction;

impl SqlFunction for TodayFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TODAY",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(0),
            description: "Returns today's date",
            returns: "DATE",
            examples: vec![
                "SELECT TODAY()",
                "SELECT * FROM events WHERE event_date = TODAY()",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;
        let today = Utc::now().date_naive();
        Ok(DataValue::String(today.format("%Y-%m-%d").to_string()))
    }
}

/// DATEDIFF function - Calculate difference between dates
pub struct DateDiffFunction;

impl SqlFunction for DateDiffFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DATEDIFF",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(3),
            description: "Calculate the difference between two dates in the specified unit",
            returns: "INTEGER",
            examples: vec![
                "SELECT DATEDIFF('day', '2024-01-01', '2024-01-15')",
                "SELECT DATEDIFF('month', start_date, end_date) FROM projects",
                "SELECT DATEDIFF('year', birth_date, TODAY()) as age",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // First argument: unit
        let unit = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::InternedString(s) => s.to_lowercase(),
            _ => return Err(anyhow!("DATEDIFF unit must be a string")),
        };

        // Second argument: date1
        let date1 = match &args[1] {
            DataValue::String(s) | DataValue::DateTime(s) => parse_datetime(s)?,
            DataValue::InternedString(s) => parse_datetime(s.as_str())?,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DATEDIFF requires date/datetime values")),
        };

        // Third argument: date2
        let date2 = match &args[2] {
            DataValue::String(s) | DataValue::DateTime(s) => parse_datetime(s)?,
            DataValue::InternedString(s) => parse_datetime(s.as_str())?,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DATEDIFF requires date/datetime values")),
        };

        // Calculate difference based on unit
        let diff = match unit.as_str() {
            "day" | "days" => {
                let duration = date2.signed_duration_since(date1);
                duration.num_days()
            }
            "month" | "months" => {
                // Approximate months as 30.44 days
                let duration = date2.signed_duration_since(date1);
                duration.num_days() / 30
            }
            "year" | "years" => {
                // Approximate years as 365.25 days
                let duration = date2.signed_duration_since(date1);
                duration.num_days() / 365
            }
            "hour" | "hours" => {
                let duration = date2.signed_duration_since(date1);
                duration.num_hours()
            }
            "minute" | "minutes" => {
                let duration = date2.signed_duration_since(date1);
                duration.num_minutes()
            }
            "second" | "seconds" => {
                let duration = date2.signed_duration_since(date1);
                duration.num_seconds()
            }
            _ => {
                return Err(anyhow!(
                    "Unknown DATEDIFF unit: {}. Use: day, month, year, hour, minute, second",
                    unit
                ))
            }
        };

        Ok(DataValue::Integer(diff))
    }
}

/// DATEADD function - Add interval to date
pub struct DateAddFunction;

impl SqlFunction for DateAddFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DATEADD",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(3),
            description: "Add a specified interval to a date",
            returns: "DATETIME",
            examples: vec![
                "SELECT DATEADD('day', 7, '2024-01-01')",
                "SELECT DATEADD('month', -1, NOW())",
                "SELECT DATEADD('year', 1, hire_date) FROM employees",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // First argument: unit
        let unit = match &args[0] {
            DataValue::String(s) => s.to_lowercase(),
            DataValue::InternedString(s) => s.to_lowercase(),
            _ => return Err(anyhow!("DATEADD unit must be a string")),
        };

        // Second argument: amount to add
        let amount = match &args[1] {
            DataValue::Integer(i) => *i,
            DataValue::Float(f) => *f as i64,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DATEADD amount must be a number")),
        };

        // Third argument: base date
        let base_date = match &args[2] {
            DataValue::String(s) | DataValue::DateTime(s) => parse_datetime(s)?,
            DataValue::InternedString(s) => parse_datetime(s.as_str())?,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DATEADD requires date/datetime values")),
        };

        // Add the specified amount based on unit
        let result_date = match unit.as_str() {
            "day" | "days" => base_date + chrono::Duration::days(amount),
            "month" | "months" => {
                // For months, we need to be careful about month boundaries
                let naive = base_date.naive_utc();
                let mut year = naive.year();
                let mut month = naive.month() as i32;
                let day = naive.day();

                month += amount as i32;

                // Handle month overflow/underflow
                while month > 12 {
                    month -= 12;
                    year += 1;
                }
                while month < 1 {
                    month += 12;
                    year -= 1;
                }

                // Create new date, handling day overflow (e.g., Jan 31 + 1 month = Feb 28/29)
                let target_date =
                    NaiveDate::from_ymd_opt(year, month as u32, day).unwrap_or_else(|| {
                        // If day doesn't exist in target month, use the last day of that month
                        // Try decreasing days until we find a valid one
                        for test_day in (1..=day).rev() {
                            if let Some(date) =
                                NaiveDate::from_ymd_opt(year, month as u32, test_day)
                            {
                                return date;
                            }
                        }
                        NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap()
                    });

                Utc.from_utc_datetime(&target_date.and_time(base_date.naive_utc().time()))
            }
            "year" | "years" => {
                let naive = base_date.naive_utc();
                let new_year = naive.year() + amount as i32;
                let month = naive.month();
                let day = naive.day();

                // Handle leap year edge case (Feb 29 -> Feb 28 in non-leap year)
                let target_date =
                    NaiveDate::from_ymd_opt(new_year, month, day).unwrap_or_else(|| {
                        // If the date doesn't exist (e.g., Feb 29 in non-leap year), use Feb 28
                        NaiveDate::from_ymd_opt(new_year, month, day - 1).unwrap()
                    });

                Utc.from_utc_datetime(&target_date.and_time(base_date.naive_utc().time()))
            }
            "hour" | "hours" => base_date + chrono::Duration::hours(amount),
            "minute" | "minutes" => base_date + chrono::Duration::minutes(amount),
            "second" | "seconds" => base_date + chrono::Duration::seconds(amount),
            _ => {
                return Err(anyhow!(
                    "Unknown DATEADD unit: {}. Use: day, month, year, hour, minute, second",
                    unit
                ))
            }
        };

        // Return as datetime string
        Ok(DataValue::DateTime(
            result_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        ))
    }
}

/// UNIX_TIMESTAMP - Convert datetime to Unix epoch timestamp
pub struct UnixTimestamp;

impl SqlFunction for UnixTimestamp {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "UNIX_TIMESTAMP",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description:
                "Convert datetime to Unix epoch timestamp (seconds since 1970-01-01 00:00:00 UTC)",
            returns: "INTEGER (seconds since epoch)",
            examples: vec![
                "SELECT UNIX_TIMESTAMP('2024-01-01 00:00:00')",
                "SELECT UNIX_TIMESTAMP('2024-01-01T12:30:45')",
                "SELECT UNIX_TIMESTAMP(trade_time) FROM trades",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("UNIX_TIMESTAMP expects exactly 1 argument"));
        }

        match &args[0] {
            DataValue::DateTime(dt_str) | DataValue::String(dt_str) => {
                let dt = parse_datetime(dt_str)?;
                Ok(DataValue::Integer(dt.timestamp()))
            }
            DataValue::InternedString(dt_str) => {
                let dt = parse_datetime(dt_str)?;
                Ok(DataValue::Integer(dt.timestamp()))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!(
                "UNIX_TIMESTAMP expects a datetime or string argument"
            )),
        }
    }
}

/// FROM_UNIXTIME - Convert Unix epoch timestamp to datetime
pub struct FromUnixTime;

impl SqlFunction for FromUnixTime {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "FROM_UNIXTIME",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description: "Convert Unix epoch timestamp to datetime string",
            returns: "DATETIME string in ISO format",
            examples: vec![
                "SELECT FROM_UNIXTIME(1704067200)",
                "SELECT FROM_UNIXTIME(timestamp_col) FROM data",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("FROM_UNIXTIME expects exactly 1 argument"));
        }

        match &args[0] {
            DataValue::Integer(timestamp) => {
                let dt = DateTime::<Utc>::from_timestamp(*timestamp, 0)
                    .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?;
                Ok(DataValue::DateTime(
                    dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                ))
            }
            DataValue::Float(timestamp) => {
                let secs = timestamp.floor() as i64;
                let nanos = ((timestamp - timestamp.floor()) * 1_000_000_000.0) as u32;
                let dt = DateTime::<Utc>::from_timestamp(secs, nanos)
                    .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?;
                Ok(DataValue::DateTime(
                    dt.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                ))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("FROM_UNIXTIME expects a numeric timestamp")),
        }
    }
}

/// TIME_BUCKET - Round timestamps down to bucket boundaries
pub struct TimeBucket;

impl SqlFunction for TimeBucket {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TIME_BUCKET",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(2),
            description: "Round timestamp down to bucket boundary (for time-based grouping)",
            returns: "INTEGER (bucket timestamp)",
            examples: vec![
                "SELECT TIME_BUCKET(300, UNIX_TIMESTAMP(trade_time)) as bucket FROM trades -- 5 minute buckets",
                "SELECT TIME_BUCKET(3600, UNIX_TIMESTAMP(trade_time)) as hour FROM trades -- 1 hour buckets",
                "SELECT TIME_BUCKET(60, timestamp_col) as minute FROM data -- 1 minute buckets",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!(
                "TIME_BUCKET expects exactly 2 arguments: bucket_size, timestamp"
            ));
        }

        let bucket_size = match &args[0] {
            DataValue::Integer(size) => *size,
            DataValue::Float(size) => *size as i64,
            _ => return Err(anyhow!("TIME_BUCKET bucket_size must be numeric")),
        };

        if bucket_size <= 0 {
            return Err(anyhow!("TIME_BUCKET bucket_size must be positive"));
        }

        match &args[1] {
            DataValue::Integer(timestamp) => {
                let bucket = (timestamp / bucket_size) * bucket_size;
                Ok(DataValue::Integer(bucket))
            }
            DataValue::Float(timestamp) => {
                let ts = *timestamp as i64;
                let bucket = (ts / bucket_size) * bucket_size;
                Ok(DataValue::Integer(bucket))
            }
            DataValue::Null => Ok(DataValue::Null),
            _ => Err(anyhow!("TIME_BUCKET timestamp must be numeric")),
        }
    }
}

/// DAYOFWEEK function - returns day of week as number (0=Sunday, 6=Saturday)
pub struct DayOfWeekFunction;

impl SqlFunction for DayOfWeekFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DAYOFWEEK",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description: "Returns day of week as number (0=Sunday, 6=Saturday)",
            returns: "INTEGER",
            examples: vec![
                "SELECT DAYOFWEEK('2024-01-01')", // Returns 1 (Monday)
                "SELECT DAYOFWEEK(NOW())",
                "SELECT DAYOFWEEK(date_column) FROM table",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("DAYOFWEEK expects exactly 1 argument"));
        }

        let date_str = match &args[0] {
            DataValue::String(s) | DataValue::DateTime(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DAYOFWEEK expects a date/datetime string")),
        };

        let dt = parse_datetime(date_str)?;
        // chrono weekday: Mon=0, Tue=1, ..., Sun=6
        // We want: Sun=0, Mon=1, ..., Sat=6
        let chrono_weekday = dt.weekday().num_days_from_monday(); // 0=Mon, 6=Sun
        let our_weekday = (chrono_weekday + 1) % 7; // Convert to 0=Sun, 6=Sat

        Ok(DataValue::Integer(our_weekday as i64))
    }
}

/// DAYNAME function - returns full day name (Monday, Tuesday, etc.)
pub struct DayNameFunction;

impl SqlFunction for DayNameFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DAYNAME",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Range(1, 2),
            description: "Returns day name. Optional second arg: 'full' (default) or 'short'",
            returns: "STRING",
            examples: vec![
                "SELECT DAYNAME('2024-01-01')",          // Returns 'Monday'
                "SELECT DAYNAME('2024-01-01', 'short')", // Returns 'Mon'
                "SELECT DAYNAME(NOW(), 'full')",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!("DAYNAME expects 1 or 2 arguments"));
        }

        let date_str = match &args[0] {
            DataValue::String(s) | DataValue::DateTime(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DAYNAME expects a date/datetime string")),
        };

        let format = if args.len() == 2 {
            match &args[1] {
                DataValue::String(s) => s.as_str(),
                DataValue::InternedString(s) => s.as_str(),
                DataValue::Null => "full",
                _ => return Err(anyhow!("DAYNAME format must be 'full' or 'short'")),
            }
        } else {
            "full"
        };

        let dt = parse_datetime(date_str)?;

        let day_name = match format {
            "short" => dt.format("%a").to_string(), // Mon, Tue, Wed, etc.
            "full" | _ => dt.format("%A").to_string(), // Monday, Tuesday, etc.
        };

        Ok(DataValue::String(day_name))
    }
}

/// ISLEAPYEAR function - returns true if the year is a leap year
pub struct IsLeapYearFunction;

impl SqlFunction for IsLeapYearFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "ISLEAPYEAR",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description: "Returns true if the year is a leap year",
            returns: "BOOLEAN",
            examples: vec![
                "SELECT ISLEAPYEAR('2024-01-01')", // Returns true
                "SELECT ISLEAPYEAR(2024)",         // Returns true
                "SELECT ISLEAPYEAR(2023)",         // Returns false
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("ISLEAPYEAR expects exactly 1 argument"));
        }

        let year = match &args[0] {
            DataValue::Integer(y) => *y as i32,
            DataValue::Float(f) => *f as i32,
            DataValue::String(s) | DataValue::DateTime(s) => {
                // Try to parse as a year number first
                if let Ok(y) = s.parse::<i32>() {
                    y
                } else {
                    // Try to parse as a date
                    let dt = parse_datetime(s.as_str())?;
                    dt.year()
                }
            }
            DataValue::InternedString(s) => {
                // Try to parse as a year number first
                if let Ok(y) = s.parse::<i32>() {
                    y
                } else {
                    // Try to parse as a date
                    let dt = parse_datetime(s.as_str())?;
                    dt.year()
                }
            }
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("ISLEAPYEAR expects a year number or date")),
        };

        // Leap year calculation
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

        Ok(DataValue::Boolean(is_leap))
    }
}

/// WEEKOFYEAR function - returns the ISO week number (1-53)
pub struct WeekOfYearFunction;

impl SqlFunction for WeekOfYearFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "WEEKOFYEAR",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the ISO week number of the year (1-53)",
            returns: "INTEGER",
            examples: vec![
                "SELECT WEEKOFYEAR('2024-01-01')", // Returns week number
                "SELECT WEEKOFYEAR(NOW())",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("WEEKOFYEAR expects exactly 1 argument"));
        }

        let date_str = match &args[0] {
            DataValue::String(s) | DataValue::DateTime(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("WEEKOFYEAR expects a date/datetime string")),
        };

        let dt = parse_datetime(date_str)?;
        let week = dt.iso_week().week();

        Ok(DataValue::Integer(week as i64))
    }
}

/// QUARTER function - returns the quarter of the year (1-4)
pub struct QuarterFunction;

impl SqlFunction for QuarterFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "QUARTER",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the quarter of the year (1-4)",
            returns: "INTEGER",
            examples: vec![
                "SELECT QUARTER('2024-01-15')", // Returns 1
                "SELECT QUARTER('2024-07-01')", // Returns 3
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 1 {
            return Err(anyhow!("QUARTER expects exactly 1 argument"));
        }

        let date_str = match &args[0] {
            DataValue::String(s) | DataValue::DateTime(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("QUARTER expects a date/datetime string")),
        };

        let dt = parse_datetime(date_str)?;
        let month = dt.month();
        let quarter = (month - 1) / 3 + 1;

        Ok(DataValue::Integer(quarter as i64))
    }
}

/// MONTHNAME function - returns the month name
pub struct MonthNameFunction;

impl SqlFunction for MonthNameFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "MONTHNAME",
            category: FunctionCategory::Date,
            arg_count: ArgCount::Range(1, 2),
            description: "Returns month name. Optional second arg: 'full' (default) or 'short'",
            returns: "STRING",
            examples: vec![
                "SELECT MONTHNAME('2024-01-15')",          // Returns 'January'
                "SELECT MONTHNAME('2024-01-15', 'short')", // Returns 'Jan'
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() || args.len() > 2 {
            return Err(anyhow!("MONTHNAME expects 1 or 2 arguments"));
        }

        let date_str = match &args[0] {
            DataValue::String(s) | DataValue::DateTime(s) => s.as_str(),
            DataValue::InternedString(s) => s.as_str(),
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("MONTHNAME expects a date/datetime string")),
        };

        let format = if args.len() == 2 {
            match &args[1] {
                DataValue::String(s) => s.as_str(),
                DataValue::InternedString(s) => s.as_str(),
                DataValue::Null => "full",
                _ => return Err(anyhow!("MONTHNAME format must be 'full' or 'short'")),
            }
        } else {
            "full"
        };

        let dt = parse_datetime(date_str)?;

        let month_name = match format {
            "short" => dt.format("%b").to_string(), // Jan, Feb, Mar, etc.
            "full" | _ => dt.format("%B").to_string(), // January, February, etc.
        };

        Ok(DataValue::String(month_name))
    }
}

/// Register all date/time functions
pub fn register_date_time_functions(registry: &mut super::FunctionRegistry) {
    registry.register(Box::new(NowFunction));
    registry.register(Box::new(TodayFunction));
    registry.register(Box::new(DateDiffFunction));
    registry.register(Box::new(DateAddFunction));
    registry.register(Box::new(UnixTimestamp));
    registry.register(Box::new(FromUnixTime));
    registry.register(Box::new(TimeBucket));

    // New date utility functions
    registry.register(Box::new(DayOfWeekFunction));
    registry.register(Box::new(DayNameFunction));
    registry.register(Box::new(IsLeapYearFunction));
    registry.register(Box::new(WeekOfYearFunction));
    registry.register(Box::new(QuarterFunction));
    registry.register(Box::new(MonthNameFunction));
}
