use crate::evaluation::Value;
use rusqlite::Connection;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;

// Use a thread-local variable to store the last connection, now thread-safe with Arc<Mutex<>>.
thread_local! {
    static LAST_SQLITE_CONNECTION: Arc<Mutex<Option<Arc<Mutex<Connection>>>>> = Arc::new(
        Mutex::new(None)
    );
}

/// sqliteconnect(dbname)
/// Establishes a connection to the SQLite database specified by dbname.
pub fn sqliteconnect(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqliteconnect() expects 1 argument, got {}", args.len()));
    }
    let dbname = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("sqliteconnect() expects a string argument".to_string());
        }
    };
    match Connection::open(dbname) {
        Ok(conn) => {
            let conn_arc = Arc::new(Mutex::new(conn));
            LAST_SQLITE_CONNECTION.with(|global_conn| {
                let mut guard = global_conn.lock().unwrap();
                *guard = Some(conn_arc.clone());
            });
            Ok(Value::SQLiteConnection(conn_arc))
        }
        Err(e) => Err(format!("sqliteconnect() error: {}", e)),
    }
}

/// sqlite_query(connection, query)
/// Executes the SQL query on the given connection and returns all rows as a result.
pub fn sqlite_query(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("sqlite_query() expects 2 arguments, got {}", args.len()));
    }

    let conn_arc = match &args[0] {
        Value::SQLiteConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err(
                "sqlite_query() expects a SQLite connection as the first argument".to_string()
            );
        }
    };
    let query = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("sqlite_query() expects the query as a string".to_string());
        }
    };

    let conn = conn_arc.lock().map_err(|_| "Failed to lock connection".to_string())?;

    if
        query.trim().to_uppercase().starts_with("INSERT") ||
        query.trim().to_uppercase().starts_with("UPDATE") ||
        query.trim().to_uppercase().starts_with("DELETE")
    {
        let affected_rows = conn
            .execute(query, [])
            .map_err(|e| format!("sqlite_query() error: {}", e))?;
        return Ok(Value::Bool(affected_rows > 0));
    }

    let mut stmt = conn.prepare(query).map_err(|e| format!("sqlite_query() error: {}", e))?;
    let columns = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut rows_vec = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| format!("sqlite_query() error: {}", e))?;

    while let Some(row) = rows.next().map_err(|e| format!("sqlite_query() error: {}", e))? {
        let mut row_map = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let value = match row.get_ref_unwrap(i) {
                rusqlite::types::ValueRef::Null => Value::Null,
                rusqlite::types::ValueRef::Integer(i) => Value::Number(i as f64),
                rusqlite::types::ValueRef::Real(r) => Value::Number(r),
                rusqlite::types::ValueRef::Text(t) =>
                    match std::str::from_utf8(t) {
                        Ok(s) => Value::String(s.to_string()),
                        Err(_) => Value::String("<invalid utf8>".to_string()),
                    }
                rusqlite::types::ValueRef::Blob(b) => Value::String(format!("{:?}", b)),
            };
            row_map.insert(col_name.clone(), value);
        }
        rows_vec.push(row_map);
    }

    if rows_vec.is_empty() {
        return Ok(Value::Bool(false));
    }

    Ok(Value::SQLiteResult(rows_vec))
}

/// sqlite_fetchall(result)
/// Repackages the SQLite result as an Array of Dictionaries.
pub fn sqlite_fetchall(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_fetchall() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::SQLiteResult(rows) => {
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
        _ => Err("sqlite_fetchall() expects a SQLite result".to_string()),
    }
}

/// sqlite_insertid()
/// Returns the last inserted row id using the global connection.
pub fn sqlite_insertid(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 0 {
        return Err(format!("sqlite_insertid() expects 0 arguments, got {}", args.len()));
    }
    LAST_SQLITE_CONNECTION.with(|global_conn| {
        let guard = global_conn.lock().unwrap();
        if let Some(conn_arc) = &*guard {
            let conn = conn_arc.lock().unwrap();
            let last_id = conn.last_insert_rowid();
            Ok(Value::Number(last_id as f64))
        } else {
            Err("No SQLite connection available".to_string())
        }
    })
}

/// sqlite_close(connection)
/// Closes the SQLite connection.
pub fn sqlite_close(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_close() expects 1 argument, got {}", args.len()));
    }
    match args[0].clone() {
        Value::SQLiteConnection(conn_arc) => {
            drop(conn_arc);
            Ok(Value::Null)
        }
        _ => Err("sqlite_close() expects a SQLite connection".to_string()),
    }
}

/// sqlite_commit(connection)
/// Commits the current transaction.
pub fn sqlite_commit(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_commit() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::SQLiteConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("sqlite_commit() expects a SQLite connection".to_string());
        }
    };
    let conn = conn_arc.lock().map_err(|_| "Failed to lock connection".to_string())?;
    conn.execute("COMMIT", []).map_err(|e| format!("sqlite_commit() error: {}", e))?;
    Ok(Value::Null)
}

/// sqlite_begin_transaction(connection)
/// Begins a new transaction.
pub fn sqlite_begin_transaction(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_begin_transaction() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::SQLiteConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("sqlite_begin_transaction() expects a SQLite connection".to_string());
        }
    };
    let conn = conn_arc.lock().map_err(|_| "Failed to lock connection".to_string())?;
    conn
        .execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("sqlite_begin_transaction() error: {}", e))?;
    Ok(Value::Null)
}

/// sqlite_rollback(connection)
/// Rolls back the current transaction.
pub fn sqlite_rollback(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_rollback() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::SQLiteConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("sqlite_rollback() expects a SQLite connection".to_string());
        }
    };
    let conn = conn_arc.lock().map_err(|_| "Failed to lock connection".to_string())?;
    conn.execute("ROLLBACK", []).map_err(|e| format!("sqlite_rollback() error: {}", e))?;
    Ok(Value::Null)
}

/// sqlite_escape_string(input)
/// Escapes special characters in the input string for safe SQLite queries.
pub fn sqlite_escape_string(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_escape_string() expects 1 argument, got {}", args.len()));
    }
    let input = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("sqlite_escape_string() expects a string".to_string());
        }
    };
    let escaped = input.replace("'", "''");
    Ok(Value::String(escaped))
}

/// sqlite_error()
/// Returns the last SQLite error message (simplified as an empty string).
pub fn sqlite_error(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::String("".to_string()))
}

/// sqlite_version(connection)
/// Returns the SQLite version.
pub fn sqlite_version(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_version() expects 1 argument, got {}", args.len()));
    }
    let conn_arc = match &args[0] {
        Value::SQLiteConnection(arc_conn) => arc_conn.clone(),
        _ => {
            return Err("sqlite_version() expects a SQLite connection as its argument".to_string());
        }
    };
    let conn = conn_arc.lock().map_err(|_| "Failed to lock connection".to_string())?;
    let version: String = conn
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .map_err(|e| format!("sqlite_version() error: {}", e))?;
    Ok(Value::String(version))
}

/// sqlite_create(dbname)
/// Creates a new SQLite database file with the specified dbname.
pub fn sqlite_create(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_create() expects 1 argument, got {}", args.len()));
    }
    let dbname = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("sqlite_create() expects a string".to_string());
        }
    };
    match Connection::open(dbname) {
        Ok(conn) => {
            drop(conn);
            Ok(Value::Null)
        }
        Err(e) => Err(format!("sqlite_create() error: {}", e)),
    }
}

/// sqlite_numrows(result)
/// Returns the number of rows in the SQLite result set.
pub fn sqlite_numrows(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_numrows() expects 1 argument, got {}", args.len()));
    }
    match &args[0] {
        Value::SQLiteResult(rows) => Ok(Value::Number(rows.len() as f64)),
        _ => Err("sqlite_numrows() expects a SQLiteResult".to_string()),
    }
}

/// sqlite_fetchassoc(result)
/// Fetches the next row from the result set as a dictionary.
pub fn sqlite_fetchassoc(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_fetchassoc() expects 1 argument, got {}", args.len()));
    }
    match args.get_mut(0) {
        Some(Value::SQLiteResult(rows)) => {
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
        _ => Err("sqlite_fetchassoc() expects a SQLiteResult".to_string()),
    }
}

/// sqlite_fetchrow(result, index)
/// Fetches the row at the given index as a numeric array of values.
pub fn sqlite_fetchrow(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("sqlite_fetchrow() expects 2 arguments, got {}", args.len()));
    }
    let index = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err("sqlite_fetchrow() expects a number for index".to_string());
        }
    };
    match args.get_mut(0) {
        Some(Value::SQLiteResult(rows)) => {
            if rows.is_empty() {
                Ok(Value::Null)
            } else if index >= rows.len() {
                return Err("Index out of bounds in sqlite_fetchrow".to_string());
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
        _ => Err("sqlite_fetchrow() expects a SQLiteResult".to_string()),
    }
}

/// sqlite_fetchone(result)
/// Fetches the first row (index 0) as a numeric array.
pub fn sqlite_fetchone(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sqlite_fetchone() expects 1 argument, got {}", args.len()));
    }
    sqlite_fetchrow(vec![args[0].clone(), Value::Number(0.0)])
}

/// sqlite_fetcharray(result, mode)
/// Fetches the next row in one of three modes:
///   - mode 0: associative array (dictionary)
///   - mode 1: numeric array
///   - mode 2: both (dictionary with column names and numeric indices)
pub fn sqlite_fetcharray(mut args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("sqlite_fetcharray() expects 2 arguments, got {}", args.len()));
    }
    let mode = match &args[1] {
        Value::Number(n) => *n as i32,
        _ => {
            return Err("sqlite_fetcharray() expects a number for mode".to_string());
        }
    };
    match args.get_mut(0) {
        Some(Value::SQLiteResult(rows)) => {
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
                                combined.insert(key.to_string(), Arc::new(Mutex::new(val.clone())));
                                combined.insert(i.to_string(), Arc::new(Mutex::new(val.clone())));
                            }
                        }
                        Ok(Value::Dictionary(combined))
                    }
                    _ =>
                        Err(
                            "Invalid mode for sqlite_fetcharray. Use 0 for associative, 1 for numeric, or 2 for both.".to_string()
                        ),
                }
            }
        }
        _ => Err("sqlite_fetcharray() expects a SQLiteResult".to_string()),
    }
}
