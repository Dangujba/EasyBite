use crate::evaluation::Value;
use mysql::*;
use mysql::prelude::*;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::io;

// Type alias for our result type.
pub type Result<T> = std::result::Result<T, String>;

// Thread-local storage for the last MySQL connection.
// This is used by mysql_insertid() which takes no arguments.
thread_local! {
    static LAST_MYSQL_CONNECTION: Arc<Mutex<Option<Arc<Mutex<PooledConn>>>>> = Arc::new(
        Mutex::new(None)
    );
}

/// mysqlconnect(url)
/// Establishes a connection to the MySQL database specified by the connection URL.
/// Example URL: "mysql://user:password@localhost:3306/dbname"
pub fn mysqlconnect(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysqlconnect() expects 1 argument, got {}", args.len()));
    }
    let url = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("mysqlconnect() expects a string argument".to_string());
        }
    };

    // Convert the String to &str before passing to Pool::new.
    let pool = Pool::new(url.as_str()).map_err(|e| format!("mysqlconnect() error: {}", e))?;
    let conn = pool.get_conn().map_err(|e| format!("mysqlconnect() error: {}", e))?;
    let conn_arc = Arc::new(Mutex::new(conn));

    // Store the connection globally.
    LAST_MYSQL_CONNECTION.with(|global_conn| {
        let mut guard = global_conn.lock().unwrap();
        *guard = Some(conn_arc.clone());
    });
    Ok(Value::MySQLConnection(conn_arc))
}

/// mysql_query(connection, query)
/// Executes the SQL query on the given connection and returns all rows as a result.
/// For non-SELECT queries (INSERT, UPDATE, DELETE), returns a Boolean indicating success.
pub fn mysql_query(args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(format!("mysql_query() expects 2 arguments, got {}", args.len()));
    }

    let conn_arc = match &args[0] {
        Value::MySQLConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err(
                "mysql_query() expects a MySQL connection as the first argument".to_string()
            );
        }
    };
    let query = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("mysql_query() expects the query as a string".to_string());
        }
    };

    let mut conn = conn_arc.lock().unwrap();
    let upper_query = query.trim().to_uppercase();
    if
        upper_query.starts_with("INSERT") ||
        upper_query.starts_with("UPDATE") ||
        upper_query.starts_with("DELETE")
    {
        conn.exec_drop(query, ()).map_err(|e| format!("mysql_query() error: {}", e))?;
        let affected = conn.affected_rows();
        return Ok(Value::Bool(affected > 0));
    }

    // For SELECT queries, execute and collect rows.
    let result: Vec<Row> = conn.query(query).map_err(|e| format!("mysql_query() error: {}", e))?;
    let mut rows_vec: Vec<HashMap<String, Value>> = Vec::new();

    for row in result {
        let mut row_map = HashMap::new();
        for col in row.columns().iter() {
            let col_name = col.name_str().to_string();
            let mysql_val = row
                .get::<mysql::Value, &str>(col_name.as_str())
                .unwrap_or(mysql::Value::NULL);
            let value = match mysql_val {
                mysql::Value::NULL => Value::Null,
                mysql::Value::Int(i) => Value::Number(i as f64),
                mysql::Value::UInt(u) => Value::Number(u as f64),
                mysql::Value::Float(f) => Value::Number(f as f64),
                mysql::Value::Double(d) => Value::Number(d),
                mysql::Value::Bytes(ref bytes) => {
                    match String::from_utf8(bytes.clone()) {
                        Ok(s) => Value::String(s),
                        Err(_) => Value::String("<invalid utf8>".to_string()),
                    }
                }
                _ => Value::Null,
            };
            row_map.insert(col_name, value);
        }
        rows_vec.push(row_map);
    }

    if rows_vec.is_empty() {
        return Ok(Value::Bool(false));
    }
    Ok(Value::MySQLResult(rows_vec))
}

/// mysql_fetchall(result)
/// Repackages the MySQL result as an Array of Dictionaries.
pub fn mysql_fetchall(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_fetchall() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::MySQLResult(rows) => {
            let list_of_dicts: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let converted: HashMap<String, Arc<Mutex<Value>>> = row
                        .iter()
                        .map(|(k, v)| (k.clone(), Arc::new(Mutex::new(v.clone()))))
                        .collect();
                    Value::Dictionary(converted)
                })
                .collect();
            let wrapped = list_of_dicts
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect();
            Ok(Value::Array(wrapped))
        }
        _ => Err("mysql_fetchall() expects a MySQLResult".to_string()),
    }
}

/// mysql_insertid()
/// Returns the last inserted row ID using the global connection.
pub fn mysql_insertid(args: Vec<Value>) -> Result<Value> {
    if args.len() != 0 {
        return Err(format!("mysql_insertid() expects 0 arguments, got {}", args.len()));
    }
    LAST_MYSQL_CONNECTION.with(|global_conn| {
        let guard = global_conn.lock().unwrap();
        if let Some(conn_arc) = &*guard {
            let conn = conn_arc.lock().unwrap();
            let last_id = conn.last_insert_id();
            Ok(Value::Number(last_id as f64))
        } else {
            Err("No MySQL connection available".to_string())
        }
    })
}

/// mysql_close(connection)
/// Closes the MySQL connection.
pub fn mysql_close(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_close() expects 1 argument, got {}", args.len()));
    }
    match args[0].clone() {
        Value::MySQLConnection(conn_arc) => {
            drop(conn_arc);
            Ok(Value::Null)
        }
        _ => Err("mysql_close() expects a MySQL connection".to_string()),
    }
}

/// mysql_commit(connection)
/// Commits the current transaction.
pub fn mysql_commit(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_commit() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::MySQLConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("mysql_commit() expects a MySQL connection".to_string());
        }
    };
    let mut conn = conn_arc.lock().unwrap();
    conn.query_drop("COMMIT").map_err(|e| format!("mysql_commit() error: {}", e))?;
    Ok(Value::Null)
}

/// mysql_begin_transaction(connection)
/// Begins a new transaction.
pub fn mysql_begin_transaction(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_begin_transaction() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::MySQLConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("mysql_begin_transaction() expects a MySQL connection".to_string());
        }
    };
    let mut conn = conn_arc.lock().unwrap();
    conn.query_drop("BEGIN").map_err(|e| format!("mysql_begin_transaction() error: {}", e))?;
    Ok(Value::Null)
}

/// mysql_rollback(connection)
/// Rolls back the current transaction.
pub fn mysql_rollback(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_rollback() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::MySQLConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("mysql_rollback() expects a MySQL connection".to_string());
        }
    };
    let mut conn = conn_arc.lock().unwrap();
    conn.query_drop("ROLLBACK").map_err(|e| format!("mysql_rollback() error: {}", e))?;
    Ok(Value::Null)
}

/// mysql_escape_string(input)
/// Escapes special characters in the input string for safe MySQL queries.
pub fn mysql_escape_string(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_escape_string() expects 1 argument, got {}", args.len()));
    }
    let input = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("mysql_escape_string() expects a string".to_string());
        }
    };
    let escaped = input.replace("'", "''");
    Ok(Value::String(escaped))
}

/// mysql_error()
/// Returns the last MySQL error message (simplified as an empty string).
pub fn mysql_error(_args: Vec<Value>) -> Result<Value> {
    Ok(Value::String("".to_string()))
}

/// mysql_version(connection)
/// Returns the MySQL server version.
pub fn mysql_version(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_version() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::MySQLConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("mysql_version() expects a MySQL connection as its argument".to_string());
        }
    };
    let mut conn = conn_arc.lock().unwrap();
    let version: String = conn
        .query_first("SELECT VERSION()")
        .map_err(|e| format!("mysql_version() error: {}", e))?
        .ok_or_else(|| "mysql_version() error: no version returned".to_string())?;
    Ok(Value::String(version))
}

/// mysql_create(dbname)
/// Creates a new MySQL database with the specified name.
pub fn mysql_create(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_create() expects 1 argument, got {}", args.len()));
    }
    let dbname = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("mysql_create() expects a string".to_string());
        }
    };
    LAST_MYSQL_CONNECTION.with(|global_conn| {
        let guard = global_conn.lock().unwrap();
        if let Some(conn_arc) = &*guard {
            let mut conn = conn_arc.lock().unwrap();
            conn
                .query_drop(format!("CREATE DATABASE IF NOT EXISTS {}", dbname))
                .map_err(|e| format!("mysql_create() error: {}", e))?;
            Ok(Value::Null)
        } else {
            Err("No MySQL connection available".to_string())
        }
    })
}

/// mysql_numrows(result)
/// Returns the number of rows in the MySQL result set.
pub fn mysql_numrows(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_numrows() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::MySQLResult(rows) => Ok(Value::Number(rows.len() as f64)),
        _ => Err("mysql_numrows() expects a MySQLResult".to_string()),
    }
}

/// mysql_fetchassoc(result)
/// Fetches the next row from the result set as a dictionary.
pub fn mysql_fetchassoc(mut args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_fetchassoc() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::MySQLResult(rows)) => {
            if rows.is_empty() {
                Ok(Value::Null)
            } else {
                let row = rows.remove(0);
                let converted: HashMap<String, Arc<Mutex<Value>>> = row
                    .into_iter()
                    .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                    .collect();
                Ok(Value::Dictionary(converted))
            }
        }
        _ => Err("mysql_fetchassoc() expects a MySQLResult".to_string()),
    }
}

/// mysql_fetchrow(result, index)
/// Fetches the row at the given index as a numeric array of values.
pub fn mysql_fetchrow(mut args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(format!("mysql_fetchrow() expects 2 arguments, got {}", args.len()));
    }
    let index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("mysql_fetchrow() expects a number for index".to_string());
        }
    };
    match args.get_mut(0) {
        Some(Value::MySQLResult(rows)) => {
            if rows.is_empty() {
                Ok(Value::Null)
            } else if index >= rows.len() {
                return Err("Index out of bounds in mysql_fetchrow".to_string());
            } else {
                let row = rows.remove(index);
                let mut keys: Vec<&String> = row.keys().collect();
                keys.sort();
                let mut arr: Vec<Value> = Vec::new();
                for key in keys {
                    if let Some(val) = row.get(key) {
                        arr.push(val.clone());
                    }
                }
                let wrapped: Vec<Arc<Mutex<Value>>> = arr
                    .into_iter()
                    .map(|v| Arc::new(Mutex::new(v)))
                    .collect();
                Ok(Value::Array(wrapped))
            }
        }
        _ => Err("mysql_fetchrow() expects a MySQLResult".to_string()),
    }
}

/// mysql_fetchone(result)
/// Fetches the first row (index 0) as a numeric array.
pub fn mysql_fetchone(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(format!("mysql_fetchone() expects 1 argument, got {}", args.len()));
    }
    mysql_fetchrow(vec![args[0].clone(), Value::Number(0.0)])
}

/// mysql_fetcharray(result, mode)
/// Fetches the next row in one of three modes:
///   - mode 0: associative array (dictionary)
///   - mode 1: numeric array
///   - mode 2: both (dictionary with column names and numeric indices)
pub fn mysql_fetcharray(mut args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(format!("mysql_fetcharray() expects 2 arguments, got {}", args.len()));
    }
    let mode = match &args[1] {
        Value::Number(n) => *n as i32,
        _ => {
            return Err("mysql_fetcharray() expects a number for mode".to_string());
        }
    };
    match args.get_mut(0) {
        Some(Value::MySQLResult(rows)) => {
            if rows.is_empty() {
                Ok(Value::Null)
            } else {
                let row = rows.remove(0);
                match mode {
                    0 => {
                        let converted: HashMap<String, Arc<Mutex<Value>>> = row
                            .into_iter()
                            .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                            .collect();
                        Ok(Value::Dictionary(converted))
                    }
                    1 => {
                        let mut keys: Vec<&String> = row.keys().collect();
                        keys.sort();
                        let mut arr: Vec<Value> = Vec::new();
                        for key in keys {
                            if let Some(val) = row.get(key) {
                                arr.push(val.clone());
                            }
                        }
                        let wrapped: Vec<Arc<Mutex<Value>>> = arr
                            .into_iter()
                            .map(|v| Arc::new(Mutex::new(v)))
                            .collect();
                        Ok(Value::Array(wrapped))
                    }
                    2 => {
                        let mut combined: HashMap<String, Arc<Mutex<Value>>> = HashMap::new();
                        let mut keys: Vec<&String> = row.keys().collect();
                        keys.sort();
                        for (i, key) in keys.iter().enumerate() {
                            if let Some(val) = row.get(*key) {
                                combined.insert((*key).clone(), Arc::new(Mutex::new(val.clone())));
                                combined.insert(i.to_string(), Arc::new(Mutex::new(val.clone())));
                            }
                        }
                        Ok(Value::Dictionary(combined))
                    }
                    _ =>
                        Err(
                            "Invalid mode for mysql_fetcharray. Use 0 for associative, 1 for numeric, or 2 for both.".to_string()
                        ),
                }
            }
        }
        _ => Err("mysql_fetcharray() expects a MySQLResult".to_string()),
    }
}
