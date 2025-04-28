/****************************************************************************************
 * File: array.rs
 * Author: Muhammad Baba Goni
 * Created: March 20, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides array manipulation utilities for dynamic lists.
 * 
 * It offers methods for creating, accessing, modifying, sorting, and iterating arrays.
 *
 * Responsibilities:
 * -----------------
 * - Perform basic array operations (push, pop, insert, remove).
 * - Support functional operations (map, filter, reduce).
 * - Handle multidimensional arrays if required.
 *
 * Usage:
 * ------
 * Core support for array-based data structures and algorithms.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use std::{ sync::{ Arc, Mutex }, cmp::Ordering };
use crate::evaluation::Value;

pub fn array_length(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_length() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
        _ => Err("array_length() expects an array".to_string()),
    }
}

pub fn array_append(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_append() expects 2 arguments, got {}", args.len()));
    }
    // Clone the second argument before mutably borrowing args
    let value_to_append = args[1].clone();
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            arr.push(Arc::new(Mutex::new(value_to_append)));
            Ok(args[0].clone())
        }
        _ => Err("array_append() expects an array".to_string()),
    }
}

pub fn array_copy(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_copy() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            let new_arr = arr.clone();
            Ok(Value::Array(new_arr))
        }
        _ => Err("array_copy() expects an array".to_string()),
    }
}

pub fn array_clear(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_clear() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            arr.clear();
            Ok(args[0].clone())
        }
        _ => Err("array_clear() expects an array".to_string()),
    }
}

pub fn array_remove(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_remove() expects 2 arguments, got {}", args.len()));
    }
    // Clone the target value before mutably borrowing args
    let target = args[1].clone(); // No borrow, just take ownership
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            if let Some(index) = arr.iter().position(|item| *item.lock().unwrap() == target) {
                arr.remove(index);
            }
            Ok(args[0].clone())
        }
        _ => Err("array_remove() expects an array".to_string()),
    }
}

pub fn array_reverse(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_reverse() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            arr.reverse();
            Ok(args[0].clone())
        }
        _ => Err("array_reverse() expects an array".to_string()),
    }
}

pub fn array_insert(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("array_insert() expects 3 arguments, got {}", args.len()));
    }
    // Extract values before mutably borrowing args
    let index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("array_insert(): Index must be a number".to_string());
        }
    };
    let value = args[2].clone();
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            if index <= arr.len() {
                arr.insert(index, Arc::new(Mutex::new(value)));
                Ok(args[0].clone())
            } else {
                Err("array_insert(): Index out of bounds".to_string())
            }
        }
        _ => Err("array_insert() expects an array".to_string()),
    }
}

pub fn array_sort(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_sort() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            arr.sort_by(|a, b| {
                let a_val = a.lock().unwrap();
                let b_val = b.lock().unwrap();
                match (&*a_val, &*b_val) {
                    (Value::Number(a), Value::Number(b)) =>
                        a.partial_cmp(b).unwrap_or(Ordering::Equal),
                    (Value::String(a), Value::String(b)) => a.cmp(b),
                    _ => Ordering::Equal,
                }
            });
            Ok(args[0].clone())
        }
        _ => Err("array_sort() expects an array".to_string()),
    }
}

pub fn array_indexof(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_indexof() expects 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            let target = &args[1];
            let position = arr.iter().position(|item| *item.lock().unwrap() == *target);
            Ok(Value::Number(position.map_or(-1.0, |i| i as f64)))
        }
        _ => Err("array_indexof() expects an array".to_string()),
    }
}

pub fn array_pop(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_pop() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            if let Some(item) = arr.pop() {
                Ok(item.lock().unwrap().clone())
            } else {
                Ok(Value::Null)
            }
        }
        _ => Err("array_pop() expects an array".to_string()),
    }
}

pub fn array_shift(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("array_shift() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            if let Some(item) = arr.drain(0..1).next() {
                Ok(item.lock().unwrap().clone())
            } else {
                Ok(Value::Null)
            }
        }
        _ => Err("array_shift() expects an array".to_string()),
    }
}

pub fn array_unshift(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_unshift() expects 2 arguments, got {}", args.len()));
    }
    // Clone the value to prepend before mutably borrowing args
    let value = args[1].clone();
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            arr.insert(0, Arc::new(Mutex::new(value)));
            Ok(args[0].clone())
        }
        _ => Err("array_unshift() expects an array".to_string()),
    }
}

pub fn array_slice(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("array_slice() expects 3 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Array(arr), Value::Number(start), Value::Number(end)) => {
            let s = *start as usize;
            let e = (*end as usize).min(arr.len());
            if s <= e && s <= arr.len() {
                let new_arr = arr[s..e].to_vec();
                Ok(Value::Array(new_arr))
            } else {
                Err("array_slice(): Invalid slice indices".to_string())
            }
        }
        _ => Err("array_slice() expects an array and two numbers".to_string()),
    }
}

pub fn array_splice(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 3 {
        return Err(format!("array_splice() expects at least 3 arguments, got {}", args.len()));
    }
    // Extract values before mutably borrowing args
    let start = match args[1].clone() {
        Value::Number(n) => n as usize,
        _ => {
            return Err("array_splice(): Start must be a number".to_string());
        }
    };
    let count = match args[2].clone() {
        Value::Number(n) => n as usize,
        _ => {
            return Err("array_splice(): Count must be a number".to_string());
        }
    };
    let extra_values = if args.len() > 3 {
        args[3..].to_vec() // Clone the slice into a new Vec
    } else {
        vec![]
    };
    match args.get_mut(0) {
        Some(Value::Array(arr)) => {
            if start <= arr.len() {
                let removed: Vec<_> = arr.drain(start..(start + count).min(arr.len())).collect();
                for (i, item) in extra_values.iter().enumerate() {
                    arr.insert(start + i, Arc::new(Mutex::new(item.clone())));
                }
                Ok(Value::Array(removed))
            } else {
                Err("array_splice(): Start index out of bounds".to_string())
            }
        }
        _ => Err("array_splice() expects an array".to_string()),
    }
}

pub fn array_concat(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_concat() expects 2 arguments, got {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Array(arr1), Value::Array(arr2)) => {
            let mut new_arr = arr1.clone();
            new_arr.extend(arr2.clone());
            Ok(Value::Array(new_arr))
        }
        _ => Err("array_concat() expects two arrays".to_string()),
    }
}

pub fn array_includes(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("array_includes() expects 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Value::Array(arr) => {
            let target = &args[1];
            let includes = arr.iter().any(|item| *item.lock().unwrap() == *target);
            Ok(Value::Bool(includes))
        }
        _ => Err("array_includes() expects an array".to_string()),
    }
}
