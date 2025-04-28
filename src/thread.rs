/****************************************************************************************
 * File: thread.rs
 * Author: Muhammad Baba Goni
 * Created: March 27, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides multi-threading and concurrent execution utilities.
 *
 * Responsibilities:
 * -----------------
 * - Create and manage lightweight threads.
 * - Synchronize shared resources safely.
 * - Support parallel execution to improve performance.
 *
 * Usage:
 * ------
 * Useful for building responsive, scalable, and high-performance applications.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use std::sync::{ Arc, Mutex };
use std::thread::{ self, JoinHandle };
use std::time::Duration;

use crate::evaluation::{ Value, Environment, ControlFlow }; // Adjust path as needed
use crate::evaluation::Interpreter; // Adjust path as needed

// ThreadHandle to manage thread results
#[derive(Debug, Clone)]
pub struct ThreadHandle(pub Arc<Mutex<Option<JoinHandle<Value>>>>);

impl ThreadHandle {
    pub fn join(&self) -> Result<Value, String> {
        let mut handle_opt = self.0.lock().unwrap();
        if let Some(handle) = handle_opt.take() {
            handle.join().map_err(|e| format!("Thread join error: {:?}", e))
        } else {
            Err("Thread already joined".to_string())
        }
    }
}

/// Spawns a new thread to execute a function and returns a ThreadHandle.
pub fn thread_spawn(
    args: Vec<Value>,
    interpreter: Arc<Mutex<Interpreter>>
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("thread.spawn expects 1 argument, got {}", args.len()));
    }

    let func_body = match &args[0] {
        Value::Function { body, .. } => body.clone(),
        _ => {
            return Err("thread.spawn expects a function".to_string());
        }
    };

    let thread_env = Arc::new(Mutex::new(Environment::new(None)));

    // Clone the Arc pointer (not the entire interpreter)
    let interpreter_shared = interpreter.clone();

    let handle = thread::spawn(move || {
        for node in &func_body {
            // Lock the interpreter for mutable access
            let mut interpreter_locked = interpreter_shared.lock().unwrap();
            match interpreter_locked.visit(node, &mut thread_env.clone()) {
                Ok(ControlFlow::Return(value)) => {
                    return value;
                }
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    return Value::Error(e);
                }
            }
        }
        Value::Null
    });

    let thread_handle = ThreadHandle(Arc::new(Mutex::new(Some(handle))));
    Ok(Value::ThreadHandle(thread_handle))
}

/// Waits for a thread to finish and returns its result.
pub fn thread_join(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("thread.join expects 1 argument, got {}", args.len()));
    }

    match &args[0] {
        Value::ThreadHandle(handle) => handle.join(),
        _ => Err("thread.join expects a ThreadHandle".to_string()),
    }
}

/// Pauses the current thread for a specified duration.
pub fn thread_sleep(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("thread.sleep expects 1 argument, got {}", args.len()));
    }

    let duration_ms = match &args[0] {
        Value::Number(n) => *n as u64,
        _ => {
            return Err("thread.sleep expects a number (milliseconds)".to_string());
        }
    };

    thread::sleep(Duration::from_millis(duration_ms));
    Ok(Value::Null)
}
