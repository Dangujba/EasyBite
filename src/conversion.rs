/****************************************************************************************
 * File: conversion.rs
 * Author: Muhammad Baba Goni
 * Created: March 21, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides utilities for converting data types between formats, such as 
 * string-to-integer, float-to-string, boolean conversions, and custom casting functions.
 *
 * Responsibilities:
 * -----------------
 * - Implement type-safe data conversions.
 * - Handle edge cases and errors during conversion.
 * - Support common primitive type conversions.
 *
 * Usage:
 * ------
 * Used by the interpreter and user programs to perform explicit and implicit conversions.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::str::FromStr;

/// Converts the value to an integer.
/// For numbers, returns the truncated integer part.
/// For strings, attempts to parse as an integer.
/// For booleans, false → 0, true → 1.
pub fn toint(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("toint() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.trunc())),
        Value::String(s) => {
            match i64::from_str(s) {
                Ok(i) => Ok(Value::Number(i as f64)),
                Err(e) => Err(format!("toint(): cannot convert string '{}' to integer: {}", s, e)),
            }
        }
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err("toint(): unsupported type".to_string()),
    }
}

/// Converts the value to a double (f64).
/// If already a number, returns it. If a string, attempts to parse it.
pub fn todouble(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("todouble() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(s) => {
            match f64::from_str(s) {
                Ok(d) => Ok(Value::Number(d)),
                Err(e) =>
                    Err(format!("todouble(): cannot convert string '{}' to double: {}", s, e)),
            }
        }
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        _ => Err("todouble(): unsupported type".to_string()),
    }
}

/// Converts any value to a string.
pub fn tostring(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tostring() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.clone())),
        Value::Number(n) => Ok(Value::String(n.to_string())),
        Value::Bool(b) => Ok(Value::String(b.to_string())),
        Value::Array(arr) => {
            let mut s = String::from("[");
            let mut first = true;
            for item in arr {
                if !first {
                    s.push_str(", ");
                }
                first = false;
                s.push_str(&format!("{:?}", item.lock().unwrap()));
            }
            s.push(']');
            Ok(Value::String(s))
        }
        Value::Dictionary(dict) => {
            let mut s = String::from("{");
            let mut first = true;
            for (k, v) in dict {
                if !first {
                    s.push_str(", ");
                }
                first = false;
                s.push_str(&format!("{}: {:?}", k, v.lock().unwrap()));
            }
            s.push('}');
            Ok(Value::String(s))
        }
        Value::BuiltinFunction(_) => Ok(Value::String("<function>".to_string())),
        Value::Null => Ok(Value::String("null".to_string())),
        // Cover any other variants not explicitly handled
        other => Ok(Value::String(format!("{:?}", other))),
    }
}

/// Checks if the value is an integer (i.e. a Number with no fractional part).
pub fn isint(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isint() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Bool(n.fract() == 0.0)),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value (if a string) is alphanumeric.
pub fn isalnum(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isalnum() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(s.chars().all(|c| c.is_alphanumeric()))),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value (if a string) consists only of digits.
pub fn isdigit(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isdigit() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(s.chars().all(|c| c.is_digit(10)))),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value is a double (i.e. a Number with a fractional part).
pub fn isdouble(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isdouble() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Bool(n.fract() != 0.0)),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value is a string.
pub fn isstring(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isstring() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::String(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value is a list (an Array).
pub fn islist(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("islist() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

/// Checks if the value is a dictionary.
pub fn isdict(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isdict() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

/// Returns a string representing the type of the value.
pub fn typeof_(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("typeof() expects 1 argument, got {}", args.len()));
    }
    let type_str = match &args[0] {
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::Array(_) => "list",
        Value::Dictionary(_) => "dict",
        Value::BuiltinFunction(_) => "function",
        Value::Null => "null",
        _ => "unknown", // covers any additional variants, e.g. Class, Object, Function, etc.
    };
    Ok(Value::String(type_str.to_string()))
}
