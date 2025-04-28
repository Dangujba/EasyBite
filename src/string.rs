/****************************************************************************************
 * File: string.rs
 * Author: Muhammad Baba Goni
 * Created: March 24, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides string manipulation and text processing utilities.
 *
 * Responsibilities:
 * -----------------
 * - Modify, split, join, search, and replace strings.
 * - Handle encoding, decoding, and formatting.
 * - Provide regular expressions or pattern matching support if needed.
 *
 * Usage:
 * ------
 * Essential for almost every application that deals with user input, parsing, or output.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::sync::{ Arc, Mutex };
use std::cmp::Ordering;
use base64::{encode as base64_encode, decode as base64_decode};

/// count(str) - Returns the number of characters in the string.
pub fn string_count(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("count expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
        _ => Err("count expects a string".to_string()),
    }
}

/// contains(str, sub) - Returns true if 'sub' is found in 'str', otherwise false.
pub fn string_contains(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("contains expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => Ok(Value::Bool(s.contains(sub))),
        _ => Err("contains expects two strings".to_string()),
    }
}

/// replace(str, old, new) - Replaces all occurrences of 'old' with 'new' in 'str'.
pub fn string_replace(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("replace expects 3 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::String(old), Value::String(new)) => {
            Ok(Value::String(s.replace(old, new)))
        }
        _ => Err("replace expects three strings".to_string()),
    }
}

/// substring(str, start, end) - Returns the substring of 'str' from index start to end.
/// (Indices are interpreted in terms of characters.)
pub fn string_substring(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("substring expects 3 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::Number(start), Value::Number(end)) => {
            let char_count = s.chars().count();
            let start_idx = *start as usize;
            let end_idx = *end as usize;
            if start_idx > char_count || end_idx > char_count || start_idx > end_idx {
                return Err("Invalid substring indices".to_string());
            }
            let substring: String = s
                .chars()
                .skip(start_idx)
                .take(end_idx - start_idx)
                .collect();
            Ok(Value::String(substring))
        }
        _ => Err("substring expects a string and two numbers".to_string()),
    }
}

/// uppercase(str) - Converts the string to uppercase.
pub fn string_uppercase(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("uppercase expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err("uppercase expects a string".to_string()),
    }
}

/// lowercase(str) - Converts the string to lowercase.
pub fn string_lowercase(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("lowercase expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err("lowercase expects a string".to_string()),
    }
}

/// capitalize(str) - Capitalizes the first letter of each word in the string.
pub fn string_capitalize(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("capitalize expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => {
            let capitalized = s
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            Ok(Value::String(capitalized))
        }
        _ => Err("capitalize expects a string".to_string()),
    }
}

/// strreverse(str) - Reverses the characters in the string.
pub fn string_reverse(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("strreverse expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::String(reversed))
        }
        _ => Err("strreverse expects a string".to_string()),
    }
}

/// join(arr, sep) - Joins the elements of the array into a single string separated by 'sep'.
pub fn string_join(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("join expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Array(arr), Value::String(sep)) => {
            let mut pieces = Vec::new();
            for item in arr {
                let item_guard = item
                    .lock()
                    .map_err(|_| "Failed to lock array element".to_string())?;
                match &*item_guard {
                    Value::String(s) => pieces.push(s.clone()),
                    _ => pieces.push(format!("{:?}", item_guard)),
                }
            }
            Ok(Value::String(pieces.join(sep)))
        }
        _ => Err("join expects an array and a string".to_string()),
    }
}

/// tolist(str, sep) - Splits the string into a list of strings using 'sep' as the delimiter.
pub fn string_tolist(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tolist expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            let parts: Vec<Value> = s
                .split(sep)
                .map(|p| Value::String(p.to_string()))
                .collect();
            let wrapped = parts
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("tolist expects a string and a string as separator".to_string()),
    }
}

/// compare(str1, str2) - Compares two strings and returns -1 if str1 is less, 0 if equal, 1 if greater.
pub fn string_compare(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("compare expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s1), Value::String(s2)) => {
            let result = match s1.cmp(s2) {
                Ordering::Less => -1.0,
                Ordering::Equal => 0.0,
                Ordering::Greater => 1.0,
            };
            Ok(Value::Number(result))
        }
        _ => Err("compare expects two strings".to_string()),
    }
}

/// trim(str) - Removes leading and trailing whitespace from the string.
pub fn string_trim(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("trim expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err("trim expects a string".to_string()),
    }
}

/// startswith(str, prefix) - Returns true if the string starts with the given prefix.
pub fn string_startswith(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("startswith expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
        _ => Err("startswith expects two strings".to_string()),
    }
}

/// endswith(str, suffix) - Returns true if the string ends with the given suffix.
pub fn string_endswith(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("endswith expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
        _ => Err("endswith expects two strings".to_string()),
    }
}

/// strremove(str, sub) - Removes all occurrences of 'sub' from the string.
pub fn string_remove(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("strremove expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => Ok(Value::String(s.replace(sub, ""))),
        _ => Err("strremove expects two strings".to_string()),
    }
}

/// split(str, sep) - Splits the string into an array of strings using 'sep' as the delimiter.
pub fn string_split(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("split expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sep)) => {
            let parts: Vec<Value> = s
                .split(sep)
                .map(|p| Value::String(p.to_string()))
                .collect();
            let wrapped = parts
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("split expects a string and a string as separator".to_string()),
    }
}

/// find(str, sub) - Returns the index of the first occurrence of 'sub' in the string, or -1 if not found.
pub fn string_find(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("find expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(sub)) => {
            match s.find(sub) {
                Some(idx) => Ok(Value::Number(idx as f64)),
                None => Ok(Value::Number(-1.0)),
            }
        }
        _ => Err("find expects two strings".to_string()),
    }
}

/// frombytes(bytes) - Converts a byte array into a string.
pub fn string_frombytes(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("frombytes expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            let bytes: Result<Vec<u8>, _> = arr.iter().map(|v| {
                v.lock()
                    .map_err(|_| "Failed to lock byte".to_string())
                    .and_then(|inner| match &*inner {
                        Value::Number(n) => Ok(*n as u8),
                        _ => Err("frombytes expects array of numbers".to_string()),
                    })
            }).collect();

            match bytes {
                Ok(b) => match String::from_utf8(b) {
                    Ok(s) => Ok(Value::String(s)),
                    Err(_) => Err("Invalid UTF-8 sequence".to_string()),
                },
                Err(e) => Err(e),
            }
        },
        _ => Err("frombytes expects an array".to_string()),
    }
}

/// format(template, value1, value2, ...) - Formats a template string with the provided values.
pub fn string_format(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("format expects at least 1 argument".to_string());
    }
    let template = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("format expects a string as the first argument".to_string()),
    };
    let mut formatted = template.clone();
    for (i, value) in args.iter().skip(1).enumerate() {
        let placeholder = format!("{{{}}}", i);
        let replacement = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => format!("{:?}", value),
        };
        formatted = formatted.replace(&placeholder, &replacement);
    }
    Ok(Value::String(formatted))
}

/// encode(str) - Encodes a string into base64.
pub fn string_encode(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("encode expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(base64_encode(s))),
        _ => Err("encode expects a string".to_string()),
    }
}

/// decode(str) - Decodes a base64 encoded string.
pub fn string_decode(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("decode expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => match base64_decode(s) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => Ok(Value::String(decoded)),
                Err(_) => Err("Decoded bytes are not valid UTF-8".to_string()),
            },
            Err(_) => Err("Invalid base64 string".to_string()),
        },
        _ => Err("decode expects a string".to_string()),
    }
}