/****************************************************************************************
 * File: fetcher.rs (requester)
 * Author: Muhammad Baba Goni
 * Created: March 24, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides HTTP client functionality to send and receive network requests.
 *
 * Responsibilities:
 * -----------------
 * - Perform HTTP GET, POST, PUT, DELETE operations.
 * - Handle headers, timeouts, and response parsing.
 * - Support JSON and form data communication.
 *
 * Usage:
 * ------
 * Useful for web scraping, REST APIs, and server communications.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::Duration;
use lazy_static::lazy_static;
use serde_json;

// Global storage for the last HTTP client using lazy_static.
lazy_static! {
    static ref LAST_HTTP_CLIENT: Arc<Mutex<Option<Arc<Mutex<reqwest::blocking::Client>>>>> =
        Arc::new(Mutex::new(None));
}

// Global storage for the base URL (if provided in connect)
lazy_static! {
    static ref LAST_BASE_URL: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

// Global cookie jar storage using lazy_static.
lazy_static! {
    static ref COOKIE_JAR: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// fetch_connect(base_url: string) -> HTTPClient
///
/// Creates an HTTP client using reqwest and stores it as the last connection.
/// The base_url parameter is stored as the default URL for subsequent requests.
pub fn fetch_connect(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("fetch_connect() expects 1 argument, got {}", args.len()));
    }
    let base_url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("fetch_connect() expects a string argument".to_string());
        }
    };

    let client = reqwest::blocking::Client
        ::builder()
        .build()
        .map_err(|e| format!("fetch_connect() error: {}", e))?;
    let client_arc = Arc::new(Mutex::new(client));

    {
        let mut client_guard = LAST_HTTP_CLIENT.lock().unwrap();
        *client_guard = Some(client_arc.clone());
    }
    {
        let mut url_guard = LAST_BASE_URL.lock().unwrap();
        *url_guard = Some(base_url);
    }
    Ok(Value::HTTPClient(client_arc))
}

/// fetch_settimeout([client,] timeout_seconds)
///
/// Sets the timeout for the HTTP client. If a client is provided as the first argument,
/// that client is updated; otherwise the last HTTP client is used.
pub fn fetch_settimeout(args: Vec<Value>) -> Result<Value, String> {
    let (client_arc, timeout_secs) = match args.len() {
        1 => {
            let timeout_secs = match &args[0] {
                Value::Number(n) => *n as u64,
                _ => {
                    return Err("fetch_settimeout() expects a number for timeout".to_string());
                }
            };
            let client_arc = {
                let guard = LAST_HTTP_CLIENT.lock().unwrap();
                guard.clone()
            };
            if client_arc.is_none() {
                return Err("No HTTPClient provided and no last connection available".to_string());
            }
            (client_arc.unwrap(), timeout_secs)
        }
        2 => {
            let client_arc = match &args[0] {
                Value::HTTPClient(arc_client) => arc_client.clone(),
                _ => {
                    return Err(
                        "fetch_settimeout() expects a HTTPClient as the first argument".to_string()
                    );
                }
            };
            let timeout_secs = match &args[1] {
                Value::Number(n) => *n as u64,
                _ => {
                    return Err("fetch_settimeout() expects a number for timeout".to_string());
                }
            };
            (client_arc, timeout_secs)
        }
        _ => {
            return Err(format!("fetch_settimeout() expects 1 or 2 arguments, got {}", args.len()));
        }
    };

    let mut client = client_arc.lock().unwrap();
    let new_client = reqwest::blocking::Client
        ::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("fetch_settimeout() error: {}", e))?;
    *client = new_client;
    Ok(Value::Null)
}

/// Helper function to ensure a URL starts with "http://" or "https://".
/// If a relative path is provided and no base URL is available, the URL is returned as-is.
fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string() // Already absolute.
    } else if url.starts_with("/") {
        let base_url = {
            let guard = LAST_BASE_URL.lock().unwrap();
            guard.clone()
        };
        match base_url {
            Some(base) => format!("{}/{}", base.trim_end_matches('/'), &url[1..]),
            None => url.to_string(), // Fallback.
        }
    } else {
        format!("http://{}", url)
    }
}

/// fetch_request([client,] method, [url], [body?], [options?])
///
/// Makes an HTTP request with the given method and URL. The client parameter is optional;
/// if the first argument is not a HTTPClient then the last stored client is used (or a default client is created).
/// If no URL is provided, the default base URL is used.
/// Optionally, you can pass an options dictionary as the last argument:
/// - "headers": a dictionary of header key-value pairs.
/// - "params": a dictionary of query parameters (for GET requests).
///
/// Returns a dictionary containing keys "status", "headers", and "body".
pub fn fetch_request(mut args: Vec<Value>) -> Result<Value, String> {
    // Determine the HTTP client.
    let client_arc = match args.get(0) {
        Some(Value::HTTPClient(_)) => {
            if let Value::HTTPClient(client) = args.remove(0) { client } else { unreachable!() }
        }
        _ => {
            let maybe_client = {
                let guard = LAST_HTTP_CLIENT.lock().unwrap();
                guard.clone()
            };
            match maybe_client {
                Some(client) => client,
                None => {
                    let client = reqwest::blocking::Client
                        ::builder()
                        .build()
                        .map_err(|e| format!("Error creating default HTTP client: {}", e))?;
                    let client_arc = Arc::new(Mutex::new(client));
                    {
                        let mut client_guard = LAST_HTTP_CLIENT.lock().unwrap();
                        *client_guard = Some(client_arc.clone());
                    }
                    client_arc
                }
            }
        }
    };

    // Determine method.
    if args.is_empty() {
        return Err("fetch_request() expects at least a method argument".to_string());
    }
    let method = match &args[0] {
        Value::String(s) => s.to_uppercase(),
        _ => {
            return Err("fetch_request() expects the method as a string".to_string());
        }
    };

    // Determine URL.
    let url = if args.len() >= 2 {
        match &args[1] {
            Value::String(s) => normalize_url(s),
            _ => {
                return Err("fetch_request() expects the URL as a string".to_string());
            }
        }
    } else {
        let default_url = {
            let guard = LAST_BASE_URL.lock().unwrap();
            guard.clone()
        };
        match default_url {
            Some(url) => normalize_url(&url),
            None => {
                return Err("No URL provided and no default base URL set".to_string());
            }
        }
    };

    // Optional body.
    let mut body: Option<String> = None;
    if args.len() >= 3 {
        if let Value::String(s) = &args[2] {
            body = Some(s.clone());
            args.remove(2); // Remove body so that subsequent optional argument is options.
        }
    }

    // Check if an options dictionary was provided.
    let options = if !args.is_empty() && matches!(args[0], Value::Dictionary(_)) {
        if let Value::Dictionary(dict) = args.remove(0) { Some(dict) } else { None }
    } else {
        None
    };

    // Build the request.
    let client = client_arc.lock().unwrap();
    let mut req_builder = client.request(
        reqwest::Method
            ::from_bytes(method.as_bytes())
            .map_err(|e| format!("Invalid HTTP method: {}", e))?,
        &url
    );
    if let Some(b) = body {
        req_builder = req_builder.body(b);
    }
    // If options are provided, handle headers and query parameters.
    if let Some(opts) = options {
        // Handle headers.
        if let Some(headers_val_arc) = opts.get("headers") {
            let headers_val = headers_val_arc.lock().unwrap();
            if let Value::Dictionary(headers_dict) = &*headers_val {
                for (key, val_arc) in headers_dict {
                    let val = val_arc.lock().unwrap();
                    if let Value::String(s) = &*val {
                        req_builder = req_builder.header(key.as_str(), s.as_str());
                    }
                }
            }
        }
        // Handle query parameters (for GET requests).
        if let Some(params_val_arc) = opts.get("params") {
            let params_val = params_val_arc.lock().unwrap();
            if let Value::Dictionary(params_dict) = &*params_val {
                let mut query_params = Vec::new();
                for (key, val_arc) in params_dict {
                    let val = val_arc.lock().unwrap();
                    if let Value::String(s) = &*val {
                        query_params.push((key.clone(), s.clone())); // Clone both key and value
                    }
                }
                req_builder = req_builder.query(&query_params);
            }
        }
    }

    let resp = req_builder.send().map_err(|e| format!("fetch_request() error: {}", e))?;

    let status = resp.status().as_u16();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                Arc::new(Mutex::new(Value::String(v.to_str().unwrap_or("").to_string()))),
            )
        })
        .collect::<HashMap<_, _>>();

    let body_text = resp.text().map_err(|e| format!("fetch_request() error reading body: {}", e))?;

    let mut response_dict = HashMap::new();
    response_dict.insert("status".to_string(), Arc::new(Mutex::new(Value::Number(status as f64))));
    response_dict.insert("headers".to_string(), Arc::new(Mutex::new(Value::Dictionary(headers))));
    response_dict.insert("body".to_string(), Arc::new(Mutex::new(Value::String(body_text))));

    Ok(Value::Dictionary(response_dict))
}

/// fetch_getresponse([client,] [url_or_response])
///
/// Makes a GET request and returns the full HTTP response (status, headers, body).
/// If the first argument is already a response (either as HTTPResponse or a dictionary with "status", "headers", "body"),
/// it is returned as-is. Otherwise, if a URL string is passed, a GET request is performed.
pub fn fetch_getresponse(mut args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        match &args[0] {
            Value::HTTPResponse(_) => {
                return Ok(args.remove(0));
            }
            Value::Dictionary(dict) => {
                if
                    dict.contains_key("status") &&
                    dict.contains_key("headers") &&
                    dict.contains_key("body")
                {
                    return Ok(args.remove(0));
                }
            }
            _ => {}
        }
    }
    // For GET, simply call fetch_request with "GET" and remaining arguments.
    let mut new_args = vec![Value::String("GET".to_string())];
    new_args.append(&mut args);
    fetch_request(new_args)
}

/// fetch_get([client,] [url], [options?])
pub fn fetch_get(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("GET".to_string()));
    } else {
        args.insert(0, Value::String("GET".to_string()));
    }
    fetch_request(args)
}

/// fetch_post([client,] [url], body, [options?])
pub fn fetch_post(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("POST".to_string()));
    } else {
        args.insert(0, Value::String("POST".to_string()));
    }
    fetch_request(args)
}

/// fetch_put([client,] [url], body, [options?])
pub fn fetch_put(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("PUT".to_string()));
    } else {
        args.insert(0, Value::String("PUT".to_string()));
    }
    fetch_request(args)
}

/// fetch_delete([client,] [url], [options?])
pub fn fetch_delete(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("DELETE".to_string()));
    } else {
        args.insert(0, Value::String("DELETE".to_string()));
    }
    fetch_request(args)
}

/// fetch_head([client,] [url], [options?])
pub fn fetch_head(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("HEAD".to_string()));
    } else {
        args.insert(0, Value::String("HEAD".to_string()));
    }
    fetch_request(args)
}

/// fetch_patch([client,] [url], body, [options?])
pub fn fetch_patch(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("PATCH".to_string()));
    } else {
        args.insert(0, Value::String("PATCH".to_string()));
    }
    fetch_request(args)
}

/// fetch_options([client,] [url], [options?])
pub fn fetch_options(mut args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::HTTPClient(_)) = args.get(0) {
        args.insert(1, Value::String("OPTIONS".to_string()));
    } else {
        args.insert(0, Value::String("OPTIONS".to_string()));
    }
    fetch_request(args)
}

/// fetch_setcookies(key: string, value: string)
pub fn fetch_setcookies(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("fetch_setcookies() expects 2 arguments, got {}", args.len()));
    }
    let key = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("fetch_setcookies() expects a string for cookie key".to_string());
        }
    };
    let value = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("fetch_setcookies() expects a string for cookie value".to_string());
        }
    };
    {
        let mut cookies = COOKIE_JAR.lock().unwrap();
        cookies.insert(key, value);
    }
    Ok(Value::Null)
}

/// fetch_getcookies() -> Dictionary
pub fn fetch_getcookies(_args: Vec<Value>) -> Result<Value, String> {
    let cookies = COOKIE_JAR.lock().unwrap();
    let dict = cookies
        .iter()
        .map(|(k, v)| (k.clone(), Arc::new(Mutex::new(Value::String(v.clone())))))
        .collect();
    Ok(Value::Dictionary(dict))
}

/// fetch_clearcookies()
pub fn fetch_clearcookies(_args: Vec<Value>) -> Result<Value, String> {
    {
        let mut cookies = COOKIE_JAR.lock().unwrap();
        cookies.clear();
    }
    Ok(Value::Null)
}

/// fetch_close([client])
///
/// Closes the HTTP client. If a client is provided as the first argument, it closes that client;
/// otherwise, it will close the last stored client.
pub fn fetch_close(args: Vec<Value>) -> Result<Value, String> {
    let client_arc = match args.len() {
        0 => {
            (
                {
                    let guard = LAST_HTTP_CLIENT.lock().unwrap();
                    guard.clone()
                }
            ).ok_or_else(|| "No HTTPClient available".to_string())?
        }
        1 => {
            match &args[0] {
                Value::HTTPClient(client) => client.clone(),
                _ => {
                    return Err(
                        "fetch_close() expects a HTTPClient as the first argument".to_string()
                    );
                }
            }
        }
        _ => {
            return Err(format!("fetch_close() expects 0 or 1 argument, got {}", args.len()));
        }
    };
    drop(client_arc);
    Ok(Value::Null)
}

/// Helper: convert a serde_json::Value into our custom Value.
fn json_to_value(j: serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) =>
            Value::Array(
                arr
                    .into_iter()
                    .map(|v| Arc::new(Mutex::new(json_to_value(v))))
                    .collect()
            ),
        serde_json::Value::Object(map) => {
            let mut dict = HashMap::new();
            for (k, v) in map {
                dict.insert(k, Arc::new(Mutex::new(json_to_value(v))));
            }
            Value::Dictionary(dict)
        }
    }
}

/// fetch_getjson([client,] [url_or_response], [options?])
///
/// Issues a GET request (or accepts an existing response) and attempts to parse
/// the response body as JSON. Returns the parsed JSON as a Value.
pub fn fetch_getjson(mut args: Vec<Value>) -> Result<Value, String> {
    let response = fetch_getresponse(args)?;
    if let Value::Dictionary(dict) = response {
        if let Some(body_arc) = dict.get("body") {
            let body_lock = body_arc.lock().unwrap();
            if let Value::String(body_str) = &*body_lock {
                let json_value: serde_json::Value = serde_json
                    ::from_str(body_str)
                    .map_err(|e| format!("JSON parsing error: {}", e))?;
                return Ok(json_to_value(json_value));
            } else {
                return Err("Response body is not a string".to_string());
            }
        }
        Err("No body in response".to_string())
    } else {
        Err("Expected a dictionary response".to_string())
    }
}

/// fetch_gettext([client,] [url_or_response], [options?])
///
/// Issues a GET request (or accepts an existing response) and returns
/// the response body as plain text.
pub fn fetch_gettext(mut args: Vec<Value>) -> Result<Value, String> {
    let response = fetch_getresponse(args)?;
    if let Value::Dictionary(dict) = response {
        if let Some(body_arc) = dict.get("body") {
            let body_lock = body_arc.lock().unwrap();
            if let Value::String(body_str) = &*body_lock {
                return Ok(Value::String(body_str.clone()));
            } else {
                return Err("Response body is not a string".to_string());
            }
        }
        Err("No body in response".to_string())
    } else {
        Err("Expected a dictionary response".to_string())
    }
}
