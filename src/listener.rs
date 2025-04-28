/****************************************************************************************
 * File: listener.rs
 * Author: Muhammad Baba Goni
 * Created: March 24, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides server-side socket listening and event-driven networking capabilities.
 *
 * Responsibilities:
 * -----------------
 * - Accept incoming socket connections.
 * - Manage client connections and messaging.
 * - Handle network events asynchronously.
 *
 * Usage:
 * ------
 * Foundation for building TCP servers, real-time applications, or custom protocols.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use std::collections::HashMap;
use std::net::{ SocketAddr, TcpListener, TcpStream };
use std::sync::{ Arc, Mutex };
use std::sync::mpsc::{ self, Sender, Receiver };
use tokio::sync::oneshot;
use std::thread::JoinHandle;
use std::io::{ Read, Write };
use crate::evaluation::{ Environment, Value, GLOBAL_INTERPRETER };
use lazy_static::lazy_static;
#[cfg(unix)]
use std::os::unix::io::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

/// A helper function that accepts one connection from the given listener.
/// Returns the accepted TcpStream wrapped in an Arc<Mutex<_>>.
fn accept_single_connection(listener: Arc<TcpListener>) -> Result<Arc<Mutex<TcpStream>>, String> {
    match listener.accept() {
        Ok((stream, _)) => Ok(Arc::new(Mutex::new(stream))),
        Err(e) => Err(format!("Accept error: {}", e)),
    }
}

/// A helper function that ensures there is a connection available in the server.
/// If not, it performs a synchronous accept using the bound listener.
/// Returns the connection.
fn ensure_connection(server: &mut ListenerServer) -> Result<Arc<Mutex<TcpStream>>, String> {
    if let Some(conn) = &server.last_connection {
        return Ok(conn.clone());
    }
    accept_single_connection(server.listener.clone()).map(|conn| {
        server.last_connection = Some(conn.clone());
        conn
    })
}

/// The ListenerServer struct holds the TCP listener (wrapped in an Arc),
/// channels for accepted connections, a join handle for the accept loop,
/// a shutdown channel, and a pointer to the last accepted connection.
#[derive(Debug)]
pub struct ListenerServer {
    pub addr: SocketAddr,
    // Instead of Option<TcpListener>, we store the listener in an Arc
    // so that it can be shared for auto-accept operations.
    pub listener: Arc<TcpListener>,
    pub accepted_tx: Option<Sender<Arc<Mutex<TcpStream>>>>,
    pub accepted_rx: Option<Receiver<Arc<Mutex<TcpStream>>>>,
    pub server_handle: Option<JoinHandle<()>>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub last_connection: Option<Arc<Mutex<TcpStream>>>,
}

impl ListenerServer {
    pub fn new(addr: SocketAddr, listener: TcpListener) -> Self {
        // Create a synchronous channel with capacity 100 for accepted connections.
        let (tx, rx) = mpsc::channel();
        ListenerServer {
            addr,
            listener: Arc::new(listener),
            accepted_tx: Some(tx),
            accepted_rx: Some(rx),
            server_handle: None,
            shutdown_tx: None,
            last_connection: None,
        }
    }
}

// Global storage for the last listener using lazy_static.
lazy_static! {
    static ref LAST_HTTP_LISTENER: Arc<Mutex<Option<Arc<Mutex<ListenerServer>>>>> = Arc::new(
        Mutex::new(None)
    );
}

/// listener_bind(addr: string)
/// Binds to the specified address and stores the ListenerServer in global storage.
pub fn listener_bind(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("listener_bind() expects 1 argument, got {}", args.len()));
    }
    let addr_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("listener_bind() expects a string argument for address".to_string());
        }
    };
    let addr: SocketAddr = addr_str.parse().map_err(|e| format!("Invalid address: {}", e))?;
    let listener = TcpListener::bind(addr).map_err(|e| format!("listener_bind() error: {}", e))?;
    let server = ListenerServer::new(addr, listener);
    let server_arc = Arc::new(Mutex::new(server));
    {
        let mut guard = LAST_HTTP_LISTENER.lock().unwrap();
        *guard = Some(server_arc.clone());
    }
    Ok(Value::HTTPListener(server_arc))
}

/// listener_listen([max_clients: number])
/// Starts an accept loop that listens for incoming connections.
/// If a maximum number of clients is provided, stops after that many connections;
/// otherwise runs until a shutdown signal is received.
pub fn listener_listen(args: Vec<Value>) -> Result<Value, String> {
    // Retrieve the last listener.
    let server_arc = (
        {
            let guard = LAST_HTTP_LISTENER.lock().unwrap();
            guard.clone()
        }
    ).ok_or("No listener bound. Call listener_bind() first.".to_string())?;

    // Take a clone of the listener so we can use it both inside and outside the thread.
    let listener_clone;
    {
        let server = server_arc.lock().unwrap();
        listener_clone = server.listener.clone();
    }
    let accepted_tx = {
        let mut server = server_arc.lock().unwrap();
        server.accepted_tx.take().ok_or("Accepted channel not available".to_string())?
    };

    // Optional max_clients: if provided as a number, use that as limit.
    let max_clients = if !args.is_empty() {
        match &args[0] {
            Value::Number(n) => Some(*n as usize),
            _ => {
                return Err("listener_listen() expects a number as max_clients".to_string());
            }
        }
    } else {
        None
    };

    // Create a shutdown channel.
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    {
        let mut server = server_arc.lock().unwrap();
        server.shutdown_tx = Some(shutdown_tx);
    }

    // Spawn the accept loop in a separate thread.
    let handle = std::thread::spawn(move || {
        let mut accepted_count = 0;
        loop {
            // Check for shutdown signal non-blocking.
            if shutdown_rx.try_recv().is_ok() {
                break;
            }
            // Accept a connection (this blocks).
            match listener_clone.accept() {
                Ok((stream, _peer)) => {
                    let arc_stream = Arc::new(Mutex::new(stream));
                    accepted_count += 1;
                    let _ = accepted_tx.send(arc_stream.clone());
                    // Update the last connection.
                    {
                        let mut guard = LAST_HTTP_LISTENER.lock().unwrap();
                        if let Some(ref server_arc) = *guard {
                            let mut srv = server_arc.lock().unwrap();
                            srv.last_connection = Some(arc_stream.clone());
                        }
                    }
                    if let Some(max) = max_clients {
                        if accepted_count >= max {
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
    });
    {
        let mut server = server_arc.lock().unwrap();
        server.server_handle = Some(handle);
    }
    Ok(Value::String(format!("Listening on {}", server_arc.lock().unwrap().addr)))
}

/// listener_start_server([server: HTTPListener])
/// Starts the server using the provided ListenerServer; if not provided,
/// uses the last bound listener. If no accepted connection is available,
/// performs a one-shot accept to obtain one.
pub fn listener_start_server(args: Vec<Value>) -> Result<Value, String> {
    let server_arc = if !args.is_empty() {
        match &args[0] {
            Value::HTTPListener(arc) => arc.clone(),
            _ => {
                return Err(
                    "listener_start_server() expects a HTTPListener as argument".to_string()
                );
            }
        }
    } else {
        (
            {
                let guard = LAST_HTTP_LISTENER.lock().unwrap();
                guard.clone()
            }
        ).ok_or("No listener bound. Call listener_bind() first.".to_string())?
    };

    {
        let mut server = server_arc.lock().unwrap();
        // If no connection exists, auto accept one.
        if server.last_connection.is_none() {
            let conn = accept_single_connection(server.listener.clone())?;
            server.last_connection = Some(conn);
        }
    }
    Ok(Value::String(format!("Server started on {}", server_arc.lock().unwrap().addr)))
}

/// listener_serve_forever([server: HTTPListener], [callback: function])
/// Runs an infinite loop accepting connections. For each new connection, if a callback function
/// is provided, it is invoked on a separate thread.
pub fn listener_serve_forever(args: Vec<Value>) -> Result<Value, String> {
    // Retrieve the server to use.
    let server_arc = if !args.is_empty() {
        match &args[0] {
            Value::HTTPListener(arc) => arc.clone(),
            _ => {
                return Err(
                    "listener_serve_forever() expects a HTTPListener as first argument".to_string()
                );
            }
        }
    } else {
        (
            {
                let guard = LAST_HTTP_LISTENER.lock().unwrap();
                guard.clone()
            }
        ).ok_or("No listener bound. Call listener_bind() first.".to_string())?
    };

    // Optional callback provided as second argument.
    let callback = if args.len() >= 2 { Some(args[1].clone()) } else { None };

    // Take the accepted_rx channel from the server.
    let accepted_rx = {
        let mut server = server_arc.lock().unwrap();
        server.accepted_rx.take().ok_or("Accepted channel not available".to_string())?
    };

    loop {
        match accepted_rx.recv() {
            Ok(conn) => {
                {
                    let mut server = server_arc.lock().unwrap();
                    server.last_connection = Some(conn.clone());
                }
                if let Some(ref cb) = callback {
                    match cb {
                        Value::Function { body, closure, object, .. } => {
                            let body = body.clone();
                            let closure = closure.clone();
                            let object = object.clone();
                            std::thread::spawn(move || {
                                let mut interpreter = GLOBAL_INTERPRETER.lock().unwrap();
                                let mut local_env = Arc::new(
                                    Mutex::new(Environment::new(Some(closure)))
                                );
                                if let Some(obj) = object {
                                    let value = obj.lock().unwrap().clone();
                                    local_env.lock().unwrap().define("this".to_string(), value);
                                }
                                let _ = interpreter.visit_block(&body, &mut local_env);
                            });
                        }
                        _ => {}
                    }
                } else {
                    println!("New connection accepted");
                }
            }
            Err(_) => {
                break;
            } // Channel closed or error
        }
    }
    Ok(Value::Null)
}

/// listener_accept([server: HTTPListener])
/// Accepts one connection from the listener. If no server is provided,
/// uses the last bound listener. If no accepted connection is already available,
/// it performs an accept on demand.
pub fn listener_accept(args: Vec<Value>) -> Result<Value, String> {
    let server_arc = if !args.is_empty() {
        match &args[0] {
            Value::HTTPListener(arc) => arc.clone(),
            _ => {
                return Err("listener_accept() expects a HTTPListener as argument".to_string());
            }
        }
    } else {
        (
            {
                let guard = LAST_HTTP_LISTENER.lock().unwrap();
                guard.clone()
            }
        ).ok_or("No listener bound. Call listener_bind() first.".to_string())?
    };

    let mut server = server_arc.lock().unwrap();
    // Use the accepted_rx channel if available.
    if let Some(ref rx) = server.accepted_rx {
        if let Ok(conn) = rx.recv() {
            server.last_connection = Some(conn.clone());
            return Ok(Value::TCPConnection(conn));
        }
    }
    // Otherwise, perform an on-demand accept.
    let conn = ensure_connection(&mut server)?;
    Ok(Value::TCPConnection(conn))
}

/// listener_join()
/// Awaits the server's accept loop to finish.
pub fn listener_join(_args: Vec<Value>) -> Result<Value, String> {
    let server_arc = (
        {
            let guard = LAST_HTTP_LISTENER.lock().unwrap();
            guard.clone()
        }
    ).ok_or("No listener bound.".to_string())?;
    let mut server = server_arc.lock().unwrap();
    if let Some(handle) = server.server_handle.take() {
        handle.join().map_err(|e| format!("Join error: {:?}", e))?;
        Ok(Value::Null)
    } else {
        Err("No running server to join.".to_string())
    }
}

/// listener_client_info([connection])
///
/// Returns a dictionary with detailed client connection information.
/// If no TCPConnection is provided, it uses the last accepted connection.
/// The returned dictionary contains:
///   - "peer_addr": Remote address as a string.
///   - "local_addr": Local address as a string.
///   - "raw_fd" (Unix) or "raw_socket" (Windows): The underlying file descriptor/socket handle.
pub fn listener_client_info(mut args: Vec<Value>) -> Result<Value, String> {
    // Get the connection: either passed as an argument or use the last accepted connection.
    let conn = if !args.is_empty() {
        match args.remove(0) {
            Value::TCPConnection(conn) => conn,
            _ => {
                return Err(
                    "listener_client_info() expects a TCPConnection as argument".to_string()
                );
            }
        }
    } else {
        let server_arc = {
            let guard = LAST_HTTP_LISTENER.lock().unwrap();
            guard.clone().ok_or("No listener bound.".to_string())?
        };
        let mut server = server_arc.lock().unwrap();
        if server.last_connection.is_none() {
            // Auto-accept if needed.
            server.last_connection = Some(accept_single_connection(server.listener.clone())?);
        }
        server.last_connection.clone().ok_or("No connection available".to_string())?
    };

    let stream = conn.lock().unwrap();
    let peer_addr = stream
        .peer_addr()
        .map_err(|e| format!("peer_addr error: {}", e))?
        .to_string();
    let local_addr = stream
        .local_addr()
        .map_err(|e| format!("local_addr error: {}", e))?
        .to_string();

    let mut info = HashMap::new();
    info.insert("peer_addr".to_string(), Arc::new(Mutex::new(Value::String(peer_addr))));
    info.insert("local_addr".to_string(), Arc::new(Mutex::new(Value::String(local_addr))));

    // Add platform-specific info.
    #[cfg(unix)]
    {
        let raw_fd = stream.as_raw_fd();
        info.insert("raw_fd".to_string(), Arc::new(Mutex::new(Value::Number(raw_fd as f64))));
    }
    #[cfg(windows)]
    {
        let raw_socket = stream.as_raw_socket();
        info.insert(
            "raw_socket".to_string(),
            Arc::new(Mutex::new(Value::Number(raw_socket as f64)))
        );
    }

    Ok(Value::Dictionary(info))
}

/// listener_send_response([connection,] message, [options?])
///
/// Sends an HTTP response over the given connection. The connection argument is optional.
/// If no connection is provided, the last accepted connection is used (auto-accepted if needed).
/// The message is any string, and the optional options dictionary may contain:
///   - "status": a string (e.g., "200 OK", "404 Not Found")
///   - "headers": a dictionary of header key–value pairs
///
/// If no options are provided, defaults are used ("200 OK", "Content-Type: text/plain").
/// The function automatically sets the Content-Length header based on the message length.
pub fn listener_send_response(mut args: Vec<Value>) -> Result<Value, String> {
    // Determine the connection.
    let conn = if !args.is_empty() {
        // Check if the first argument is a TCPConnection.
        match args.get(0) {
            Some(Value::TCPConnection(_)) => {
                if let Value::TCPConnection(c) = args.remove(0) {
                    c
                } else {
                    return Err("Expected a TCPConnection as first argument".to_string());
                }
            }
            _ => {
                // No valid connection passed; use (or auto-accept) the last accepted connection.
                let server_arc = {
                    let guard = LAST_HTTP_LISTENER.lock().unwrap();
                    guard.clone().ok_or("No listener bound.".to_string())?
                };
                let mut server = server_arc.lock().unwrap();

                ensure_connection(&mut server)?
            }
        }
    } else {
        let server_arc = {
            let guard = LAST_HTTP_LISTENER.lock().unwrap();
            guard.clone().ok_or("No listener bound.".to_string())?
        };
        let mut server = server_arc.lock().unwrap();

        ensure_connection(&mut server)?
    };

    // The next argument must be the message string.
    if args.is_empty() {
        return Err("listener_send_response() expects a message as an argument".to_string());
    }
    let message = match args.remove(0) {
        Value::String(s) => s,
        _ => {
            return Err("listener_send_response() expects the message as a string".to_string());
        }
    };

    // Optional options dictionary.
    let (status_line, extra_headers) = if !args.is_empty() {
        if let Value::Dictionary(opts) = args.remove(0) {
            let status = if let Some(val_arc) = opts.get("status") {
                let val = val_arc.lock().unwrap();
                if let Value::String(s) = &*val {
                    s.clone()
                } else {
                    "200 OK".to_string()
                }
            } else {
                "200 OK".to_string()
            };
            let mut headers = HashMap::new();
            if let Some(headers_arc) = opts.get("headers") {
                let headers_val = headers_arc.lock().unwrap();
                if let Value::Dictionary(dict) = &*headers_val {
                    for (key, val_arc) in dict {
                        let val = val_arc.lock().unwrap();
                        if let Value::String(s) = &*val {
                            headers.insert(key.clone(), s.clone());
                        }
                    }
                }
            }
            (status, headers)
        } else {
            ("200 OK".to_string(), HashMap::new())
        }
    } else {
        ("200 OK".to_string(), HashMap::new())
    };

    // Always set Content-Length automatically.
    let content_length = message.len();
    let content_type = extra_headers
        .get("Content-Type")
        .cloned()
        .unwrap_or_else(|| "text/plain".to_string());

    // Construct the HTTP response.
    let mut response = format!("HTTP/1.1 {}\r\n", status_line);
    response.push_str(&format!("Content-Length: {}\r\n", content_length));
    response.push_str(&format!("Content-Type: {}\r\n", content_type));
    for (key, value) in extra_headers {
        if key == "Content-Length" || key == "Content-Type" {
            continue;
        }
        response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str("\r\n");
    response.push_str(&message);

    // Write the response to the connection.
    let mut stream = conn.lock().unwrap();
    stream.write_all(response.as_bytes()).map_err(|e| format!("Write error: {}", e))?;
    Ok(Value::Null)
}

/// listener_read_request([connection])
/// Reads data from the given connection. If no connection is provided, uses (or auto-accepts)
/// the last accepted connection.
pub fn listener_read_request(mut args: Vec<Value>) -> Result<Value, String> {
    let conn = if !args.is_empty() {
        match args.remove(0) {
            Value::TCPConnection(conn) => conn,
            _ => {
                return Err(
                    "listener_read_request() expects a TCPConnection as first argument".to_string()
                );
            }
        }
    } else {
        let server_arc = {
            let guard = LAST_HTTP_LISTENER.lock().unwrap();
            guard.clone().ok_or("No listener bound.".to_string())?
        };
        let mut server = server_arc.lock().unwrap();

        ensure_connection(&mut server)?
    };
    let mut stream = conn.lock().unwrap();
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).map_err(|e| format!("Read error: {}", e))?;
    if n == 0 {
        return Err("Connection closed".to_string());
    }
    let request_str = String::from_utf8_lossy(&buffer[..n]).to_string();
    Ok(Value::String(request_str))
}

/// listener_shutdown([server: HTTPListener])
/// Shuts down the server. If no server is provided, uses the last bound listener.
pub fn listener_shutdown(args: Vec<Value>) -> Result<Value, String> {
    let server_arc = if !args.is_empty() {
        match &args[0] {
            Value::HTTPListener(arc) => arc.clone(),
            _ => {
                return Err("listener_shutdown() expects a HTTPListener as argument".to_string());
            }
        }
    } else {
        (
            {
                let guard = LAST_HTTP_LISTENER.lock().unwrap();
                guard.clone()
            }
        ).ok_or("No listener bound.".to_string())?
    };
    let mut server = server_arc.lock().unwrap();
    if let Some(tx) = server.shutdown_tx.take() {
        let _ = tx.send(());
        Ok(Value::Null)
    } else {
        Err("Server is not running or already shut down".to_string())
    }
}
