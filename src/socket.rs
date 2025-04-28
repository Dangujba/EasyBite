/****************************************************************************************
 * File: socket.rs
 * Author: Muhammad Baba Goni
 * Created: March 23, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides lower-level socket communication utilities.
 *
 * Responsibilities:
 * -----------------
 * - Establish client connections over TCP/UDP.
 * - Send and receive raw data streams.
 * - Support basic socket options and configurations.
 *
 * Usage:
 * ------
 * Common in networking applications, peer-to-peer systems, or custom communication protocols.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::{ DebugTlsAcceptor, Value };
use std::net::{ TcpStream, TcpListener, UdpSocket, Shutdown, SocketAddr };
use std::io::{ Read, Write };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use socket2::{ Socket, Domain, Type, Protocol };
use std::fs::{ self, File };
use native_tls::{ TlsConnector, TlsAcceptor, TlsStream, Identity };

//
// Global variables for the TCP extended API.
// These allow a two-step process: first bind, then listen, and finally accept without passing the listener around.
//
thread_local! {
    static GLOBAL_TCP_SOCKET: Arc<Mutex<Option<Socket>>> = Arc::new(Mutex::new(None));
    static GLOBAL_TCP_LISTENER: Arc<Mutex<Option<Arc<Mutex<TcpListener>>>>> = Arc::new(
        Mutex::new(None)
    );
}

//
// ==================== TCP FUNCTIONS ====================
//

/// tcp_connect(host: String, port: Number)
/// Establishes a TCP connection to the given host and port.
pub fn tcp_connect(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tcp_connect() expects 2 arguments, got {}", args.len()));
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tcp_connect() expects a string for host".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("tcp_connect() expects a number for port".to_string());
        }
    };
    let addr = format!("{}:{}", host, port);
    match TcpStream::connect(&addr) {
        Ok(stream) => {
            stream.set_nonblocking(false).unwrap_or(());
            Ok(Value::TcpStream(Arc::new(Mutex::new(stream))))
        }
        Err(e) => Err(format!("tcp_connect() error: {}", e)),
    }
}

/// tcp_send(tcp_stream: TcpStream, data: String)
/// Sends data over an established TCP connection.
pub fn tcp_send(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tcp_send() expects 2 arguments, got {}", args.len()));
    }
    let stream_arc = match &args[0] {
        Value::TcpStream(arc) => arc.clone(),
        _ => {
            return Err("tcp_send() expects a TCP stream as the first argument".to_string());
        }
    };
    let data = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("tcp_send() expects data as a string".to_string());
        }
    };
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TCP stream".to_string())?;
    stream.write_all(data.as_bytes()).map_err(|e| format!("tcp_send() error: {}", e))?;
    Ok(Value::Bool(true))
}

/// tcp_receive(tcp_stream: TcpStream)
/// Receives data from a TCP connection.
pub fn tcp_receive(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tcp_receive() expects 1 argument, got {}", args.len()));
    }
    let stream_arc = match &args[0] {
        Value::TcpStream(arc) => arc.clone(),
        _ => {
            return Err("tcp_receive() expects a TCP stream".to_string());
        }
    };
    let mut buffer = [0u8; 1024];
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TCP stream".to_string())?;
    let bytes_read = stream.read(&mut buffer).map_err(|e| format!("tcp_receive() error: {}", e))?;
    if bytes_read == 0 {
        return Ok(Value::Null);
    }
    let received = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    Ok(Value::String(received))
}

/// tcp_close(tcp_stream: TcpStream)
/// Closes a TCP connection.
pub fn tcp_close(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tcp_close() expects 1 argument, got {}", args.len()));
    }
    let stream_arc = match args[0].clone() {
        Value::TcpStream(arc) => arc,
        _ => {
            return Err("tcp_close() expects a TCP stream".to_string());
        }
    };
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TCP stream".to_string())?;
    stream.shutdown(Shutdown::Both).map_err(|e| format!("tcp_close() error: {}", e))?;
    Ok(Value::Null)
}

/// tcp_bind(host: String, port: Number)
/// Binds a TCP socket to the specified host and port and stores it globally.
pub fn tcp_bind(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tcp_bind() expects 2 arguments, got {}", args.len()));
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tcp_bind() expects a string for host".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("tcp_bind() expects a number for port".to_string());
        }
    };
    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| format!("tcp_bind() invalid address: {}", e))?;
    let domain = if addr.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).map_err(|e|
        format!("tcp_bind() socket creation error: {}", e)
    )?;
    socket
        .set_reuse_address(true)
        .map_err(|e| format!("tcp_bind() set_reuse_address error: {}", e))?;
    socket.bind(&addr.into()).map_err(|e| format!("tcp_bind() bind error: {}", e))?;
    GLOBAL_TCP_SOCKET.with(|global| {
        let mut guard = global.lock().unwrap();
        *guard = Some(socket);
    });
    Ok(Value::Bool(true))
}

/// tcp_listen(backlog: Number)
/// Converts the previously bound socket into a listening socket using the specified backlog.
pub fn tcp_listen(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tcp_listen() expects 1 argument (backlog), got {}", args.len()));
    }
    let backlog = match &args[0] {
        Value::Number(n) => *n as i32,
        _ => {
            return Err("tcp_listen() expects a number for backlog".to_string());
        }
    };
    let listener = GLOBAL_TCP_SOCKET.with(|global| {
        let mut guard = global.lock().unwrap();
        if let Some(socket) = guard.take() {
            socket.listen(backlog).map_err(|e| format!("tcp_listen() listen error: {}", e))?;
            let std_listener: TcpListener = socket.into();
            Ok(Arc::new(Mutex::new(std_listener)))
        } else {
            Err("tcp_listen() error: No bound socket available. Call tcp_bind() first.".to_string())
        }
    })?;
    GLOBAL_TCP_LISTENER.with(|global| {
        let mut guard = global.lock().unwrap();
        *guard = Some(listener.clone());
    });
    Ok(Value::TcpListener(listener))
}

/// tcp_accept()
/// Accepts an incoming connection from the global listener and returns a dictionary.
pub fn tcp_accept(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("tcp_accept() expects no arguments, got {}", args.len()));
    }
    let listener_arc = GLOBAL_TCP_LISTENER.with(|global| {
        let guard = global.lock().unwrap();
        guard.clone()
    });
    let listener = if let Some(l) = listener_arc {
        l
    } else {
        return Err(
            "tcp_accept() error: No global listener available. Call tcp_listen() first.".to_string()
        );
    };
    let (stream, addr) = listener
        .lock()
        .map_err(|_| "tcp_accept() error: Failed to lock listener".to_string())?
        .accept()
        .map_err(|e| format!("tcp_accept() error: {}", e))?;
    let stream_arc = Arc::new(Mutex::new(stream));
    let mut info = HashMap::new();
    info.insert("connection".to_string(), Value::TcpStream(stream_arc));
    info.insert("client_addr".to_string(), Value::String(addr.to_string()));
    Ok(
        Value::Dictionary(
            info
                .into_iter()
                .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                .collect()
        )
    )
}

/// tcp_accept_by_addr(remote_ip: String, remote_port: Number)
/// Accepts an incoming connection only if the client's IP and port match.
pub fn tcp_accept_by_addr(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(
            format!(
                "tcp_accept_by_addr() expects 2 arguments (remote_ip, remote_port), got {}",
                args.len()
            )
        );
    }
    let remote_ip = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(
                "tcp_accept_by_addr() expects first argument as remote ip (string)".to_string()
            );
        }
    };
    let remote_port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err(
                "tcp_accept_by_addr() expects second argument as remote port (number)".to_string()
            );
        }
    };
    let listener_arc = GLOBAL_TCP_LISTENER.with(|global| {
        let guard = global.lock().unwrap();
        guard.clone()
    });
    let listener = if let Some(l) = listener_arc {
        l
    } else {
        return Err(
            "tcp_accept_by_addr() error: No global listener available. Call tcp_listen() first.".to_string()
        );
    };
    loop {
        let (stream, addr) = listener
            .lock()
            .map_err(|_| "tcp_accept_by_addr() error: Failed to lock listener".to_string())?
            .accept()
            .map_err(|e| format!("tcp_accept_by_addr() error: {}", e))?;
        if addr.ip().to_string() == remote_ip && addr.port() == remote_port {
            let stream_arc = Arc::new(Mutex::new(stream));
            let mut info = HashMap::new();
            info.insert("connection".to_string(), Value::TcpStream(stream_arc));
            info.insert("client_addr".to_string(), Value::String(addr.to_string()));
            return Ok(
                Value::Dictionary(
                    info
                        .into_iter()
                        .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                        .collect()
                )
            );
        } else {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}

/// tcp_listen_with_backlog(host: String, port: Number, backlog: Number)
/// Binds and listens with the specified backlog in one call.
pub fn tcp_listen_with_backlog(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("tcp_listen_with_backlog() expects 3 arguments, got {}", args.len()));
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tcp_listen_with_backlog() expects a string for host".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("tcp_listen_with_backlog() expects a number for port".to_string());
        }
    };
    let backlog = match &args[2] {
        Value::Number(n) => *n as i32,
        _ => {
            return Err("tcp_listen_with_backlog() expects a number for backlog".to_string());
        }
    };
    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| format!("Invalid address {}: {}", addr_str, e))?;
    let domain = if addr.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).map_err(|e|
        format!("Socket creation error: {}", e)
    )?;
    socket.set_reuse_address(true).map_err(|e| format!("set_reuse_address error: {}", e))?;
    socket.bind(&addr.into()).map_err(|e| format!("Socket bind error: {}", e))?;
    socket.listen(backlog).map_err(|e| format!("Socket listen error: {}", e))?;
    let listener: TcpListener = socket.into();
    Ok(Value::TcpListener(Arc::new(Mutex::new(listener))))
}

//
// ==================== UDP FUNCTIONS ====================
//

/// udp_bind(address: String)
/// Binds a UDP socket on the given address.
pub fn udp_bind(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("udp_bind() expects 1 argument, got {}", args.len()));
    }
    let addr = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("udp_bind() expects a string for address".to_string());
        }
    };
    match UdpSocket::bind(addr) {
        Ok(socket) => Ok(Value::UdpSocket(Arc::new(Mutex::new(socket)))),
        Err(e) => Err(format!("udp_bind() error: {}", e)),
    }
}

/// udp_send_to(udp_socket: UdpSocket, data: String, target: String)
/// Sends data to the specified target address via a UDP socket.
pub fn udp_send_to(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("udp_send_to() expects 3 arguments, got {}", args.len()));
    }
    let socket_arc = match &args[0] {
        Value::UdpSocket(arc) => arc.clone(),
        _ => {
            return Err("udp_send_to() expects a UDP socket as the first argument".to_string());
        }
    };
    let data = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("udp_send_to() expects data as a string".to_string());
        }
    };
    let target = match &args[2] {
        Value::String(s) => s,
        _ => {
            return Err("udp_send_to() expects a target address as a string".to_string());
        }
    };
    let mut socket = socket_arc.lock().map_err(|_| "Failed to lock UDP socket".to_string())?;
    let sent = socket
        .send_to(data.as_bytes(), target)
        .map_err(|e| format!("udp_send_to() error: {}", e))?;
    Ok(Value::Number(sent as f64))
}

/// udp_receive_from(udp_socket: UdpSocket)
/// Receives data and the sender’s address from a UDP socket.
pub fn udp_receive_from(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("udp_receive_from() expects 1 argument, got {}", args.len()));
    }
    let socket_arc = match &args[0] {
        Value::UdpSocket(arc) => arc.clone(),
        _ => {
            return Err("udp_receive_from() expects a UDP socket".to_string());
        }
    };
    let mut buffer = [0u8; 1024];
    let mut socket = socket_arc.lock().map_err(|_| "Failed to lock UDP socket".to_string())?;
    let (size, addr) = socket
        .recv_from(&mut buffer)
        .map_err(|e| format!("udp_receive_from() error: {}", e))?;
    let message = String::from_utf8_lossy(&buffer[..size]).to_string();
    let mut result = HashMap::new();
    result.insert("message".to_string(), Value::String(message));
    result.insert("address".to_string(), Value::String(addr.to_string()));
    Ok(
        Value::Dictionary(
            result
                .into_iter()
                .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                .collect()
        )
    )
}

//
// ==================== TLS/SSL FUNCTIONS ====================
//

/// tls_connect(host: String, port: Number, domain: String)
/// Establishes a TLS connection to the given host and port using the provided domain for certificate validation.
pub fn tls_connect(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("tls_connect() expects 3 arguments, got {}", args.len()));
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tls_connect() expects a string for host".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("tls_connect() expects a number for port".to_string());
        }
    };
    let domain = match &args[2] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tls_connect() expects a string for domain".to_string());
        }
    };
    let addr = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&addr).map_err(|e| format!("tls_connect() error: {}", e))?;
    tcp_stream.set_nonblocking(false).unwrap_or(());
    let connector = TlsConnector::new().map_err(|e| format!("tls_connect() error: {}", e))?;
    let tls_stream = connector
        .connect(&domain, tcp_stream)
        .map_err(|e| format!("tls_connect() error: {}", e))?;
    Ok(Value::TlsStream(Arc::new(Mutex::new(tls_stream))))
}

/// tls_listen(host: String, port: Number, pkcs12_path: String, password: String)
/// Sets up a TLS listener using a PKCS12 archive for the server certificate.
pub fn tls_listen(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 4 {
        return Err(
            "tls_listen() expects 4 arguments: host, port, pkcs12_path, password".to_string()
        );
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tls_listen() expects a string for host".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("tls_listen() expects a number for port".to_string());
        }
    };
    let pkcs12_path = match &args[2] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tls_listen() expects a string for pkcs12_path".to_string());
        }
    };
    let password = match &args[3] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("tls_listen() expects a string for password".to_string());
        }
    };
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).map_err(|e| format!("tls_listen() error: {}", e))?;
    let identity_bytes = fs
        ::read(&pkcs12_path)
        .map_err(|e| format!("tls_listen() error reading PKCS12 file: {}", e))?;
    let identity = Identity::from_pkcs12(&identity_bytes, &password).map_err(|e|
        format!("tls_listen() error creating identity: {}", e)
    )?;
    let acceptor = TlsAcceptor::new(identity).map_err(|e|
        format!("tls_listen() error creating TLS acceptor: {}", e)
    )?;
    let mut result = HashMap::new();
    result.insert("listener".to_string(), Value::TcpListener(Arc::new(Mutex::new(listener))));
    result.insert(
        "acceptor".to_string(),
        Value::TlsAcceptor(DebugTlsAcceptor(Arc::new(Mutex::new(acceptor))))
    );
    Ok(
        Value::Dictionary(
            result
                .into_iter()
                .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                .collect()
        )
    )
}

/// tls_accept({ "listener": TcpListener, "acceptor": TlsAcceptor })
/// Accepts a new TLS connection using the provided listener and TLS acceptor.
pub fn tls_accept(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            "tls_accept() expects 1 argument: a dictionary containing listener and acceptor".to_string()
        );
    }
    let dict = match &args[0] {
        Value::Dictionary(map) => map,
        _ => {
            return Err("tls_accept() expects a dictionary".to_string());
        }
    };
    let listener_arc = match dict.get("listener") {
        Some(val) => {
            let guard = val.lock().unwrap();
            if let Value::TcpListener(arc) = &*guard {
                arc.clone()
            } else {
                return Err(
                    "tls_accept() dictionary key 'listener' must be a TCP listener".to_string()
                );
            }
        }
        None => {
            return Err("tls_accept() missing 'listener' in dictionary".to_string());
        }
    };
    let acceptor_arc = match dict.get("acceptor") {
        Some(val) => {
            let guard = val.lock().unwrap();
            if let Value::TlsAcceptor(arc) = &*guard {
                arc.clone()
            } else {
                return Err(
                    "tls_accept() dictionary key 'acceptor' must be a TLS acceptor".to_string()
                );
            }
        }
        None => {
            return Err("tls_accept() missing 'acceptor' in dictionary".to_string());
        }
    };
    let listener = listener_arc.lock().map_err(|_| "Failed to lock TCP listener".to_string())?;
    let (tcp_stream, _addr) = listener
        .accept()
        .map_err(|e| format!("tls_accept() error accepting connection: {}", e))?;
    let acceptor = acceptor_arc.0.lock().map_err(|_| "Failed to lock TLS acceptor".to_string())?;
    let tls_stream = acceptor
        .accept(tcp_stream)
        .map_err(|e| format!("tls_accept() error upgrading connection to TLS: {}", e))?;
    Ok(Value::TlsStream(Arc::new(Mutex::new(tls_stream))))
}

/// tls_send(tls_stream: TlsStream, data: String)
/// Sends data over an established TLS connection.
pub fn tls_send(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("tls_send() expects 2 arguments, got {}", args.len()));
    }
    let stream_arc = match &args[0] {
        Value::TlsStream(arc) => arc.clone(),
        _ => {
            return Err("tls_send() expects a TLS stream".to_string());
        }
    };
    let data = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("tls_send() expects data as a string".to_string());
        }
    };
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TLS stream".to_string())?;
    stream.write_all(data.as_bytes()).map_err(|e| format!("tls_send() error: {}", e))?;
    Ok(Value::Bool(true))
}

/// tls_receive(tls_stream: TlsStream)
/// Receives data from a TLS connection.
pub fn tls_receive(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tls_receive() expects 1 argument, got {}", args.len()));
    }
    let stream_arc = match &args[0] {
        Value::TlsStream(arc) => arc.clone(),
        _ => {
            return Err("tls_receive() expects a TLS stream".to_string());
        }
    };
    let mut buffer = [0u8; 1024];
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TLS stream".to_string())?;
    let bytes_read = stream.read(&mut buffer).map_err(|e| format!("tls_receive() error: {}", e))?;
    if bytes_read == 0 {
        return Ok(Value::Null);
    }
    let received = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    Ok(Value::String(received))
}

/// tls_close(tls_stream: TlsStream)
/// Closes a TLS connection.
pub fn tls_close(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("tls_close() expects 1 argument, got {}", args.len()));
    }
    let stream_arc = match args[0].clone() {
        Value::TlsStream(arc) => arc,
        _ => {
            return Err("tls_close() expects a TLS stream".to_string());
        }
    };
    let mut stream = stream_arc.lock().map_err(|_| "Failed to lock TLS stream".to_string())?;
    drop(stream);
    Ok(Value::Null)
}

//
// ==================== ADDITIONAL UTILITY FUNCTIONS ====================
//

/// set_nonblocking(socket, mode: Bool)
/// Sets the non-blocking mode for a TCP stream, TCP listener, or UDP socket.
pub fn set_nonblocking(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_nonblocking() expects 2 arguments, got {}", args.len()));
    }
    let mode = match &args[1] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_nonblocking() expects a boolean for mode".to_string());
        }
    };
    match &args[0] {
        Value::TcpStream(arc) => {
            let mut stream = arc.lock().map_err(|_| "Failed to lock TCP stream".to_string())?;
            stream.set_nonblocking(mode).map_err(|e| format!("set_nonblocking error: {}", e))?;
            Ok(Value::Bool(true))
        }
        Value::TcpListener(arc) => {
            let mut listener = arc.lock().map_err(|_| "Failed to lock TCP listener".to_string())?;
            listener.set_nonblocking(mode).map_err(|e| format!("set_nonblocking error: {}", e))?;
            Ok(Value::Bool(true))
        }
        Value::UdpSocket(arc) => {
            let mut socket = arc.lock().map_err(|_| "Failed to lock UDP socket".to_string())?;
            socket.set_nonblocking(mode).map_err(|e| format!("set_nonblocking error: {}", e))?;
            Ok(Value::Bool(true))
        }
        _ => Err("set_nonblocking() expects a TCP stream, TCP listener, or UDP socket".to_string()),
    }
}

/// set_nonblocking_global(mode: Bool)
/// Sets the non-blocking mode on the global TCP listener (if available).
pub fn set_nonblocking_global(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(
            format!("set_nonblocking_global() expects 1 argument (a bool), got {}", args.len())
        );
    }
    let mode = match &args[0] {
        Value::Bool(b) => *b,
        _ => {
            return Err("set_nonblocking_global() expects a boolean".to_string());
        }
    };
    let mut result_set = false;
    GLOBAL_TCP_LISTENER.with(|global| {
        let guard = global.lock().unwrap();
        if let Some(listener_arc) = &*guard {
            if let Ok(mut listener) = listener_arc.lock() {
                if listener.set_nonblocking(mode).is_ok() {
                    result_set = true;
                }
            }
        }
    });
    if result_set {
        Ok(Value::Bool(true))
    } else {
        Err(
            "set_nonblocking_global() error: No global TCP listener available or failed to set".to_string()
        )
    }
}

/// gethostbyname(hostname: String)
/// Resolves the given hostname to a list of IP addresses.
pub fn gethostbyname(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("gethostbyname() expects 1 argument, got {}", args.len()));
    }
    let hostname = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("gethostbyname() expects a string".to_string());
        }
    };
    use std::net::ToSocketAddrs;
    let addrs_iter = (hostname.as_str(), 0)
        .to_socket_addrs()
        .map_err(|e| format!("gethostbyname() error: {}", e))?;
    let ips: Vec<Value> = addrs_iter.map(|addr| Value::String(addr.ip().to_string())).collect();
    let array = ips
        .into_iter()
        .map(|ip| Arc::new(Mutex::new(ip)))
        .collect();
    Ok(Value::Array(array))
}

/// sendfile(tcp_stream: TcpStream, file_path: String)
/// Sends the contents of a file over the given TCP stream.
pub fn sendfile(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("sendfile() expects 2 arguments, got {}", args.len()));
    }
    let socket = match &args[0] {
        Value::TcpStream(arc) => arc.clone(),
        _ => {
            return Err("sendfile() expects a TCP stream as the first argument".to_string());
        }
    };
    let file_path = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("sendfile() expects a string for file path".to_string());
        }
    };
    let mut file = File::open(file_path).map_err(|e|
        format!("sendfile() error opening file: {}", e)
    )?;
    let mut buffer = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|e| format!("sendfile() error reading file: {}", e))?;
        if n == 0 {
            break;
        }
        let mut stream = socket
            .lock()
            .map_err(|_| "sendfile() error locking TCP stream".to_string())?;
        stream
            .write_all(&buffer[..n])
            .map_err(|e| format!("sendfile() error writing to socket: {}", e))?;
    }
    Ok(Value::Bool(true))
}

/// receivefile(tcp_stream: TcpStream, file_path: String)
/// Receives data from the TCP stream and writes it to a file.
pub fn receivefile(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("receivefile() expects 2 arguments, got {}", args.len()));
    }
    let socket = match &args[0] {
        Value::TcpStream(arc) => arc.clone(),
        _ => {
            return Err("receivefile() expects a TCP stream as the first argument".to_string());
        }
    };
    let file_path = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("receivefile() expects a string for file path".to_string());
        }
    };
    let mut file = File::create(file_path).map_err(|e|
        format!("receivefile() error creating file: {}", e)
    )?;
    let mut buffer = [0u8; 8192];
    loop {
        let n = {
            let mut stream = socket
                .lock()
                .map_err(|_| "receivefile() error locking TCP stream".to_string())?;
            stream
                .read(&mut buffer)
                .map_err(|e| format!("receivefile() error reading from socket: {}", e))?
        };
        if n == 0 {
            break;
        }
        file
            .write_all(&buffer[..n])
            .map_err(|e| format!("receivefile() error writing to file: {}", e))?;
    }
    Ok(Value::Bool(true))
}
