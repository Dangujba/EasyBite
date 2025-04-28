/****************************************************************************************
 * File: datetime.rs
 * Author: Muhammad Baba Goni
 * Created: March 20, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides functionality for working with dates, times, and durations.
 * 
 * It supports operations like getting the current time, formatting dates, and 
 * performing date arithmetic.
 *
 * Responsibilities:
 * -----------------
 * - Fetch system date and time.
 * - Parse and format datetime strings.
 * - Support basic date-time manipulations.
 *
 * Usage:
 * ------
 * Useful for logging, scheduling, timestamps, and time-based computations.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use chrono::{ Local, NaiveDate, NaiveTime, Duration, Timelike };

/// today() -> date
/// Returns the current date as a string in "YYYY-MM-DD" format.
pub fn today(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("today() expects no arguments".to_string());
    }
    let date = Local::now().naive_local().date();
    let s = date.format("%Y-%m-%d").to_string();
    Ok(Value::String(s))
}

/// timenow() -> time
/// Returns the current time as a string in "HH:MM:SS" format.
pub fn timenow(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("timenow() expects no arguments".to_string());
    }
    let time = Local::now().naive_local().time();
    let s = time.format("%H:%M:%S").to_string();
    Ok(Value::String(s))
}

/// datediff(date1, date2) -> int
/// Calculates the difference in days between date1 and date2.
/// Expects both dates as strings in "YYYY-MM-DD" format.
pub fn datediff(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("datediff() expects 2 arguments".to_string());
    }
    let date1_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("datediff(): first argument must be a date string".to_string());
        }
    };
    let date2_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("datediff(): second argument must be a date string".to_string());
        }
    };
    let date1 = NaiveDate::parse_from_str(date1_str, "%Y-%m-%d").map_err(|e|
        format!("datediff(): error parsing date1: {}", e)
    )?;
    let date2 = NaiveDate::parse_from_str(date2_str, "%Y-%m-%d").map_err(|e|
        format!("datediff(): error parsing date2: {}", e)
    )?;
    let diff = (date2 - date1).num_days();
    Ok(Value::Number(diff as f64))
}

/// dateadd(date, days) -> date
/// Adds the specified number of days (as a number) to date (a string "YYYY-MM-DD")
/// and returns the new date as a string.
pub fn dateadd(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("dateadd() expects 2 arguments".to_string());
    }
    let date_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("dateadd(): first argument must be a date string".to_string());
        }
    };
    let days = match &args[1] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err("dateadd(): second argument must be a number".to_string());
        }
    };
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e|
        format!("dateadd(): error parsing date: {}", e)
    )?;
    let new_date = date + Duration::days(days);
    Ok(Value::String(new_date.format("%Y-%m-%d").to_string()))
}

/// dateformat(date, format) -> date
/// Formats the given date (as a string "YYYY-MM-DD") according to the specified format.
pub fn dateformat(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("dateformat() expects 2 arguments".to_string());
    }
    let date_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("dateformat(): first argument must be a date string".to_string());
        }
    };
    let format_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("dateformat(): second argument must be a format string".to_string());
        }
    };
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e|
        format!("dateformat(): error parsing date: {}", e)
    )?;
    Ok(Value::String(date.format(format_str).to_string()))
}

/// dateparse(dateString, format) -> date
/// Parses the dateString using the specified format and returns the date in "YYYY-MM-DD" format.
pub fn dateparse(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("dateparse() expects 2 arguments".to_string());
    }
    let date_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("dateparse(): first argument must be a date string".to_string());
        }
    };
    let format_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("dateparse(): second argument must be a format string".to_string());
        }
    };
    let date = NaiveDate::parse_from_str(date_str, format_str).map_err(|e|
        format!("dateparse(): error parsing date: {}", e)
    )?;
    Ok(Value::String(date.format("%Y-%m-%d").to_string()))
}

/// timediff(time1, time2) -> int
/// Calculates the difference in seconds between time1 and time2.
/// Expects both time strings in "HH:MM:SS" format.
pub fn timediff(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("timediff() expects 2 arguments".to_string());
    }
    let time1_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("timediff(): first argument must be a time string".to_string());
        }
    };
    let time2_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("timediff(): second argument must be a time string".to_string());
        }
    };
    let time1 = NaiveTime::parse_from_str(time1_str, "%H:%M:%S").map_err(|e|
        format!("timediff(): error parsing time1: {}", e)
    )?;
    let time2 = NaiveTime::parse_from_str(time2_str, "%H:%M:%S").map_err(|e|
        format!("timediff(): error parsing time2: {}", e)
    )?;
    // Calculate the difference in seconds (may be negative if time2 is earlier)
    let diff =
        (time2.num_seconds_from_midnight() as i64) - (time1.num_seconds_from_midnight() as i64);
    Ok(Value::Number(diff as f64))
}

/// timeadd(time, unit, interval) -> time
/// Adds an interval (a number) to the time (string "HH:MM:SS") based on the unit (seconds, minutes, or hours).
pub fn timeadd(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("timeadd() expects 3 arguments".to_string());
    }
    let time_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("timeadd(): first argument must be a time string".to_string());
        }
    };
    let unit = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => {
            return Err("timeadd(): second argument must be a unit string".to_string());
        }
    };
    let interval = match &args[2] {
        Value::Number(n) => *n as i64,
        _ => {
            return Err("timeadd(): third argument must be a number".to_string());
        }
    };
    let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").map_err(|e|
        format!("timeadd(): error parsing time: {}", e)
    )?;
    let duration = match unit {
        "seconds" => Duration::seconds(interval),
        "minutes" => Duration::minutes(interval),
        "hours" => Duration::hours(interval),
        _ => {
            return Err(
                "timeadd(): unsupported unit. Use 'seconds', 'minutes', or 'hours'.".to_string()
            );
        }
    };
    let new_time = time + duration;
    Ok(Value::String(new_time.format("%H:%M:%S").to_string()))
}

/// timeformat(time, format) -> time
/// Formats the given time (string "HH:MM:SS") according to the specified format.
pub fn timeformat(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("timeformat() expects 2 arguments".to_string());
    }
    let time_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("timeformat(): first argument must be a time string".to_string());
        }
    };
    let format_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("timeformat(): second argument must be a format string".to_string());
        }
    };
    let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").map_err(|e|
        format!("timeformat(): error parsing time: {}", e)
    )?;
    Ok(Value::String(time.format(format_str).to_string()))
}

/// timeparse(timeString, format) -> time
/// Parses the timeString using the specified format and returns the time in "HH:MM:SS" format.
pub fn timeparse(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("timeparse() expects 2 arguments".to_string());
    }
    let time_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("timeparse(): first argument must be a time string".to_string());
        }
    };
    let format_str = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("timeparse(): second argument must be a format string".to_string());
        }
    };
    let time = NaiveTime::parse_from_str(time_str, format_str).map_err(|e|
        format!("timeparse(): error parsing time: {}", e)
    )?;
    Ok(Value::String(time.format("%H:%M:%S").to_string()))
}
