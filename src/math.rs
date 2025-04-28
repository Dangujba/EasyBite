/****************************************************************************************
 * File: math.rs
 * Author: Muhammad Baba Goni
 * Created: March 24, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides mathematical utilities and common algorithms.
 *
 * Responsibilities:
 * -----------------
 * - Perform arithmetic, algebraic, and statistical computations.
 * - Provide utility functions (e.g., min, max, abs, pow, sqrt).
 * - Support random number generation if needed.
 *
 * Usage:
 * ------
 * Useful for scientific calculations, games, simulations, and more.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::collections::HashMap;
use rand::Rng;

// abs(x) Returns the absolute value of x.
pub fn math_abs(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("abs expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err("abs expects a number".to_string()),
    }
}

// pow(x, y) Returns x raised to the power of y.
pub fn math_pow(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("pow expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
        _ => Err("pow expects two numbers".to_string()),
    }
}

// sqrt(x) Returns the square root of x.
pub fn math_sqrt(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqrt expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) if n >= 0.0 => Ok(Value::Number(n.sqrt())),
        Value::Number(_) => Err("sqrt expects a non-negative number".to_string()),
        _ => Err("sqrt expects a number".to_string()),
    }
}

// sin(x) Returns the sine of x.
pub fn math_sin(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sin expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err("sin expects a number".to_string()),
    }
}

// cos(x) Returns the cosine of x.
pub fn math_cos(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("cos expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err("cos expects a number".to_string()),
    }
}

// tan(x) Returns the tangent of x.
pub fn math_tan(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tan expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.tan())),
        _ => Err("tan expects a number".to_string()),
    }
}

// round(x) Rounds x to the nearest integer.
pub fn math_round(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("round expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.round())),
        _ => Err("round expects a number".to_string()),
    }
}

// random()
// Returns a random floating-point number between 0 (inclusive) and 1 (exclusive).
// random(start, end)
// Returns a random integer between start (inclusive) and end (exclusive) if both are provided.
pub fn math_random(args: Vec<Value>) -> Result<Value, String> {
    let mut rng = rand::thread_rng();
    match args.len() {
        0 => Ok(Value::Number(rng.gen_range(0.0..1.0))),
        2 => {
            match (&args[0], &args[1]) {
                (Value::Number(start), Value::Number(end)) => {
                    if start.fract() != 0.0 || end.fract() != 0.0 {
                        return Err("random(start, end) expects integer values".to_string());
                    }
                    let start_int = *start as i64;
                    let end_int = *end as i64;
                    if start_int >= end_int {
                        return Err("random(start, end): start must be less than end".to_string());
                    }
                    let result = rng.gen_range(start_int..end_int);
                    Ok(Value::Number(result as f64))
                }
                _ => Err("random expects two numbers for start and end".to_string()),
            }
        }
        _ => Err("random expects either 0 or 2 arguments".to_string()),
    }
}

// max(x, y, ...) Returns the maximum value among the given arguments.
pub fn math_max(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max expects at least one argument".to_string());
    }
    let mut max_val = match &args[0] {
        Value::Number(n) => *n,
        _ => {
            return Err("max expects numbers".to_string());
        }
    };
    for arg in args.iter().skip(1) {
        match arg {
            Value::Number(n) => {
                if *n > max_val {
                    max_val = *n;
                }
            }
            _ => {
                return Err("max expects numbers".to_string());
            }
        }
    }
    Ok(Value::Number(max_val))
}

// min(x, y, ...) Returns the minimum value among the given arguments.
pub fn math_min(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("min expects at least one argument".to_string());
    }
    let mut min_val = match &args[0] {
        Value::Number(n) => *n,
        _ => {
            return Err("min expects numbers".to_string());
        }
    };
    for arg in args.iter().skip(1) {
        match arg {
            Value::Number(n) => {
                if *n < min_val {
                    min_val = *n;
                }
            }
            _ => {
                return Err("max expects numbers".to_string());
            }
        }
    }
    Ok(Value::Number(min_val))
}

// sum(arr) Returns the sum of all elements in the given array.
pub fn math_sum(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sum expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut total = 0.0;
            for item in arr {
                let item_val = item.lock().unwrap();
                match &*item_val {
                    Value::Number(n) => {
                        total += *n;
                    }
                    _ => {
                        return Err("sum expects an array of numbers".to_string());
                    }
                }
            }
            Ok(Value::Number(total))
        }
        _ => Err("sum expects an array".to_string()),
    }
}

// ceiling(x) Returns the smallest integer greater than or equal to x.
pub fn math_ceiling(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("ceiling expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err("ceiling expects a number".to_string()),
    }
}

// floor(x) Returns the largest integer less than or equal to x.
pub fn math_floor(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("floor expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err("floor expects a number".to_string()),
    }
}

// log10(x) Returns the base 10 logarithm of x.
pub fn math_log10(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("log10 expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.log10())),
        _ => Err("log10 expects a number".to_string()),
    }
}

// average(arr) Returns the average of all elements in the given array.
pub fn math_average(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("average expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err("average expects a non-empty array".to_string());
            }
            let mut total = 0.0;
            for item in arr {
                let item_val = item.lock().unwrap();
                match &*item_val {
                    Value::Number(n) => {
                        total += *n;
                    }
                    _ => {
                        return Err("average expects an array of numbers".to_string());
                    }
                }
            }
            Ok(Value::Number(total / (arr.len() as f64)))
        }
        _ => Err("average expects an array".to_string()),
    }
}

// log(x) Returns the natural logarithm (base e) of x.
pub fn math_log(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("log expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.ln())),
        _ => Err("log expects a number".to_string()),
    }
}

// exp(x) Returns e raised to the power of x.
pub fn math_exp(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("exp expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.exp())),
        _ => Err("exp expects a number".to_string()),
    }
}

// mean(arr) Returns the mean of all elements in the given array. (Alias for average)
pub fn math_mean(args: Vec<Value>) -> Result<Value, String> {
    math_average(args)
}

// mode(arr) Returns the mode (most frequent value) of the given array.
pub fn math_mode(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("mode expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err("mode expects a non-empty array".to_string());
            }
            let mut frequency: HashMap<String, (Value, usize)> = HashMap::new();
            for item in arr {
                let item_val = item.lock().unwrap();
                let key = format!("{:?}", *item_val);
                let cloned_val = (*item_val).clone();
                let entry = frequency.entry(key).or_insert((cloned_val, 0));
                entry.1 += 1;
            }
            let mut mode_val = None;
            let mut max_count = 0;
            for (_, (val, count)) in frequency {
                if count > max_count {
                    max_count = count;
                    mode_val = Some(val);
                }
            }
            mode_val.ok_or_else(|| "Could not determine mode".to_string())
        }
        _ => Err("mode expects an array".to_string()),
    }
}

// sign(x) Returns the sign of x (-1 for negative, 0 for zero, 1 for positive).
pub fn math_sign(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sign expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => {
            let s = if n < 0.0 { -1.0 } else if n > 0.0 { 1.0 } else { 0.0 };
            Ok(Value::Number(s))
        }
        _ => Err("sign expects a number".to_string()),
    }
}

// log2(x) Returns the base 2 logarithm of x.
pub fn math_log2(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("log2 expects 1 argument, got {}", args.len()));
    }
    match args[0] {
        Value::Number(n) => Ok(Value::Number(n.log2())),
        _ => Err("log2 expects a number".to_string()),
    }
}
