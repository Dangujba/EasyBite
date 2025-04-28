/****************************************************************************************
 * File: dictionary.rs
 * Author: Muhammad Baba Goni
 * Created: March 20, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides key-value pair data structure implementations (dictionaries or maps).
 *
 * Responsibilities:
 * -----------------
 * - Create, update, retrieve, and delete key-value pairs.
 * - Handle nested dictionaries and complex structures.
 * - Offer methods for iteration, searching, and serialization.
 *
 * Usage:
 * ------
 * Used for dynamic and structured storage of related data.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::fs;
use serde_json;

// add(dictionary: dict, key: any, value: any) -> dict
pub fn add(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("add() expects 3 arguments, got {}", args.len()));
    }
    let key_clone = match &args[1] {
        Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    };
    let value_clone = args[2].clone();
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            dict.insert(key_clone, Arc::new(Mutex::new(value_clone)));
            Ok(args[0].clone())
        }
        _ => Err("add() expects a dictionary as the first argument".to_string()),
    }
}

// get(dictionary: dict, key: any) -> any
pub fn get(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("get() expects 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let key = match &args[1] {
                Value::String(s) => s.clone(),
                other => format!("{:?}", other),
            };
            match dict.get(&key) {
                Some(val) => Ok(val.lock().unwrap().clone()),
                None => Err(format!("Key '{}' not found", key)),
            }
        }
        _ => Err("get() expects a dictionary as the first argument".to_string()),
    }
}

// remove(dictionary: dict, key: any) -> dict
pub fn remove(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("remove() expects 2 arguments, got {}", args.len()));
    }
    let key_clone = match &args[1] {
        Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    };
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            dict.remove(&key_clone);
            Ok(args[0].clone())
        }
        _ => Err("remove() expects a dictionary as the first argument".to_string()),
    }
}

// containskey(dictionary: dict, key: any) -> bool
pub fn containskey(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("containskey() expects 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let key = match &args[1] {
                Value::String(s) => s.clone(),
                other => format!("{:?}", other),
            };
            Ok(Value::Bool(dict.contains_key(&key)))
        }
        _ => Err("containskey() expects a dictionary as the first argument".to_string()),
    }
}

// containsvalue(dictionary: dict, value: any) -> bool
pub fn containsvalue(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("containsvalue() expects 2 arguments, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let target = args[1].clone();
            let found = dict.values().any(|v| *v.lock().unwrap() == target);
            Ok(Value::Bool(found))
        }
        _ => Err("containsvalue() expects a dictionary as the first argument".to_string()),
    }
}

// size(dictionary: dict) -> int
pub fn size(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("size() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => Ok(Value::Number(dict.len() as f64)),
        _ => Err("size() expects a dictionary".to_string()),
    }
}

// keys(dictionary: dict) -> list
pub fn keys(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("keys() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let key_list: Vec<Value> = dict
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect();
            let wrapped = key_list
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("keys() expects a dictionary".to_string()),
    }
}

// values(dictionary: dict) -> list
pub fn values(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("values() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let vals: Vec<Value> = dict
                .values()
                .map(|v| v.lock().unwrap().clone())
                .collect();
            let wrapped = vals
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("values() expects a dictionary".to_string()),
    }
}

// isempty(dictionary: dict) -> bool
pub fn isempty(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("isempty() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => Ok(Value::Bool(dict.is_empty())),
        _ => Err("isempty() expects a dictionary".to_string()),
    }
}

// clear(dictionary: dict) -> dict
pub fn clear(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("clear() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            dict.clear();
            Ok(args[0].clone())
        }
        _ => Err("clear() expects a dictionary".to_string()),
    }
}

// update(dictionary: dict, key: any, value: any) -> dict
pub fn update(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("update() expects 3 arguments, got {}", args.len()));
    }
    let key_clone = match &args[1] {
        Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    };
    let value_clone = args[2].clone();
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            dict.insert(key_clone, Arc::new(Mutex::new(value_clone)));
            Ok(args[0].clone())
        }
        _ => Err("update() expects a dictionary as the first argument".to_string()),
    }
}

// merge(dictionary: dict, otherDictionary: dict) -> dict
pub fn merge(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("merge() expects 2 arguments, got {}", args.len()));
    }
    let second = args[1].clone();
    match args.get_mut(0) {
        Some(Value::Dictionary(dict1)) => {
            match second {
                Value::Dictionary(dict2) => {
                    for (k, v) in dict2.iter() {
                        dict1.insert(k.clone(), v.clone());
                    }
                    Ok(args[0].clone())
                }
                _ => Err("merge() expects two dictionaries".to_string()),
            }
        }
        _ => Err("merge() expects two dictionaries".to_string()),
    }
}

// copy(dictionary: dict) -> dict
pub fn copy(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("copy() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let new_dict = dict.clone(); // Shallow copy
            Ok(Value::Dictionary(new_dict))
        }
        _ => Err("copy() expects a dictionary".to_string()),
    }
}

// tojson(dictionary: dict) -> str
pub fn tojson(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tojson() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let temp: HashMap<_, _> = dict
                .iter()
                .map(|(k, v)| (k.clone(), format!("{:?}", v.lock().unwrap())))
                .collect();
            match serde_json::to_string(&temp) {
                Ok(s) => Ok(Value::String(s)),
                Err(e) => Err(format!("tojson() error: {}", e)),
            }
        }
        _ => Err("tojson() expects a dictionary".to_string()),
    }
}

// tofile(dictionary: dict, filename: str)
pub fn tofile(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tofile() expects 2 arguments, got {}", args.len()));
    }
    let filename = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tofile() expects filename as a string".to_string());
        }
    };
    match &args[0] {
        Value::Dictionary(_) => {
            let json_val = tojson(vec![args[0].clone()])?;
            if let Value::String(s) = json_val {
                match fs::write(filename, s) {
                    Ok(_) => Ok(Value::Null),
                    Err(e) => Err(format!("tofile() error: {}", e)),
                }
            } else {
                Err("tofile() conversion error".to_string())
            }
        }
        _ => Err("tofile() expects a dictionary".to_string()),
    }
}

// pop(dictionary: dict, key: any) -> any
pub fn pop(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("pop() expects 2 arguments, got {}", args.len()));
    }
    let key_clone = match &args[1] {
        Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    };
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            if let Some(val) = dict.remove(&key_clone) {
                Ok(val.lock().unwrap().clone())
            } else {
                Err(format!("pop(): Key '{}' not found", key_clone))
            }
        }
        _ => Err("pop() expects a dictionary as the first argument".to_string()),
    }
}

// popitem(dictionary: dict) -> list
pub fn popitem(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("popitem() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            let key_opt = dict.keys().next().cloned();
            if let Some(key) = key_opt {
                if let Some(val) = dict.remove(&key) {
                    let pair = vec![
                        Arc::new(Mutex::new(Value::String(key))),
                        Arc::new(Mutex::new(val.lock().unwrap().clone()))
                    ];
                    return Ok(Value::Array(pair));
                }
            }
            Err("popitem(): dictionary is empty".to_string())
        }
        _ => Err("popitem() expects a dictionary as the first argument".to_string()),
    }
}

// items(dictionary: dict) -> list
pub fn items(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("items() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::Dictionary(dict) => {
            let items: Vec<Value> = dict
                .iter()
                .map(|(k, v)| {
                    let pair = vec![
                        Arc::new(Mutex::new(Value::String(k.clone()))),
                        Arc::new(Mutex::new(v.lock().unwrap().clone()))
                    ];
                    Value::Array(pair)
                })
                .collect();
            let wrapped = items
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("items() expects a dictionary".to_string()),
    }
}

// setdefault(dictionary: dict, key: any, default: any) -> any
pub fn setdefault(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("setdefault() expects 3 arguments, got {}", args.len()));
    }
    let key_clone = match &args[1] {
        Value::String(s) => s.clone(),
        other => format!("{:?}", other),
    };
    let value_clone = args[2].clone();
    match args.get_mut(0) {
        Some(Value::Dictionary(dict)) => {
            if let Some(val) = dict.get(&key_clone) {
                Ok(val.lock().unwrap().clone())
            } else {
                dict.insert(key_clone, Arc::new(Mutex::new(value_clone.clone())));
                Ok(value_clone)
            }
        }
        _ => Err("setdefault() expects a dictionary as the first argument".to_string()),
    }
}
