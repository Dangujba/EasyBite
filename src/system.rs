/****************************************************************************************
 * File: system.rs
 * Author: Muhammad Baba Goni
 * Created: March 27, 2025
 * Last Updated: April 27, 2025
 *
 * Description:
 * ------------
 * This library provides system-level utilities to interact with the underlying operating system.
 *
 * Responsibilities:
 * -----------------
 * - Fetch system information (OS type, architecture, uptime).
 * - Manage environment variables and process-related tasks.
 * - Support system commands execution if necessary.
 *
 * Usage:
 * ------
 * Useful for system administration tools, monitoring, and automating tasks.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::evaluation::Value;
use std::env;
use std::fs;
use std::io::{ self, Read, Write };
use std::net::{ TcpStream, UdpSocket };
use std::path::{ Path, PathBuf };
use std::process::{ Command, Stdio };
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use mac_address::get_mac_address as mac_address;

#[cfg(unix)]
use nix::sys::signal::{ kill, Signal };
#[cfg(unix)]
use nix::unistd::Pid as NixPid;

use sysinfo::{Pid as sysPid, Users};
// For Windows drive detection:
#[cfg(windows)]
use winapi::um::fileapi::{ GetDriveTypeW, GetLogicalDrives };

//
// ==================== ENVIRONMENT & WORKING DIRECTORY ====================
//

pub fn get_env(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("get_env() expects 1 argument, got {}", args.len()));
    }
    let var = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("get_env() expects a string for the variable name".to_string());
        }
    };
    match env::var(&var) {
        Ok(val) => Ok(Value::String(val)),
        Err(e) => Err(format!("get_env() error: {}", e)),
    }
}

pub fn set_env(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("set_env() expects 2 arguments, got {}", args.len()));
    }
    let var = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_env() expects a string for the variable name".to_string());
        }
    };
    let value = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("set_env() expects a string for the variable value".to_string());
        }
    };
    unsafe {
        env::set_var(&var, &value);
    }
    Ok(Value::Bool(true))
}

pub fn current_dir(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("current_dir() expects no arguments, got {}", args.len()));
    }
    env::current_dir()
        .map(|path| Value::String(path.to_string_lossy().to_string()))
        .map_err(|e| format!("current_dir() error: {}", e))
}

pub fn change_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("change_dir() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("change_dir() expects a string for the path".to_string());
        }
    };
    env::set_current_dir(&path_str)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("change_dir() error: {}", e))
}

//
// ==================== FILE SYSTEM OPERATIONS ====================
//

pub fn list_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("list_dir() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("list_dir() expects a string for the path".to_string());
        }
    };
    let path = Path::new(&path_str);
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(path).map_err(|e| format!("list_dir() error: {}", e))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| format!("list_dir() error: {}", e))?;
        let filename = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| String::new());
        entries.push(Value::String(filename));
    }
    Ok(
        Value::Array(
            entries
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

pub fn read_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("read_file() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("read_file() expects a string for the path".to_string());
        }
    };
    fs::read_to_string(&path_str)
        .map(|contents| Value::String(contents))
        .map_err(|e| format!("read_file() error: {}", e))
}

pub fn write_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write_file() expects 2 arguments, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("write_file() expects a string for the file path".to_string());
        }
    };
    let data = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("write_file() expects a string for the data".to_string());
        }
    };
    fs::write(&path_str, data)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("write_file() error: {}", e))
}

pub fn delete_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("delete_file() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("delete_file() expects a string for the file path".to_string());
        }
    };
    fs::remove_file(&path_str)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("delete_file() error: {}", e))
}

pub fn create_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("create_dir() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("create_dir() expects a string for the directory path".to_string());
        }
    };
    fs::create_dir(&path_str)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("create_dir() error: {}", e))
}

pub fn delete_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("delete_dir() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("delete_dir() expects a string for the directory path".to_string());
        }
    };
    fs::remove_dir_all(&path_str)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("delete_dir() error: {}", e))
}

pub fn rename(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("rename() expects 2 arguments, got {}", args.len()));
    }
    let old_path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("rename() expects a string for the old path".to_string());
        }
    };
    let new_path = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("rename() expects a string for the new path".to_string());
        }
    };
    fs::rename(&old_path, &new_path)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("rename() error: {}", e))
}

pub fn copy_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("copy_file() expects 2 arguments, got {}", args.len()));
    }
    let source = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("copy_file() expects a string for the source path".to_string());
        }
    };
    let destination = match &args[1] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("copy_file() expects a string for the destination path".to_string());
        }
    };
    fs::copy(&source, &destination)
        .map(|_| Value::Bool(true))
        .map_err(|e| format!("copy_file() error: {}", e))
}

//
// ==================== DRIVE, VOLUME & OS DETECTION ====================
//

pub fn is_windows(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("is_windows() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Bool(cfg!(windows)))
}

pub fn is_linux(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("is_linux() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Bool(cfg!(target_os = "linux")))
}

pub fn is_macos(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("is_macos() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Bool(cfg!(target_os = "macos")))
}

pub fn is_android(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("is_android() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Bool(cfg!(target_os = "android")))
}

pub fn is_ios(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("is_ios() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Bool(cfg!(target_os = "ios")))
}

/// --- LIST VOLUMES ---
///
/// Windows: Return the drive letters (which represent volumes).
/// Linux: Parse `/proc/mounts` and return the mount points.
/// macOS: List the contents of the `/Volumes` directory.
#[cfg(windows)]
pub fn list_volumes(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("list_volumes() expects no arguments, got {}", args.len()));
    }
    // On Windows, volumes are essentially the drive letters.
    list_drives(vec![])
}

#[cfg(target_os = "linux")]
pub fn list_volumes(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("list_volumes() expects no arguments, got {}", args.len()));
    }
    let mounts = fs
        ::read_to_string("/proc/mounts")
        .map_err(|e| format!("list_volumes() error: {}", e))?;
    let mut volumes = Vec::new();
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let mount_point = parts[1].to_string();
            volumes.push(Value::String(mount_point));
        }
    }
    Ok(
        Value::Array(
            volumes
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

#[cfg(target_os = "macos")]
pub fn list_volumes(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("list_volumes() expects no arguments, got {}", args.len()));
    }
    let entries = fs::read_dir("/Volumes").map_err(|e| format!("list_volumes() error: {}", e))?;
    let mut volumes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("list_volumes() error: {}", e))?;
        if let Some(name) = entry.file_name().to_str() {
            volumes.push(Value::String(name.to_string()));
        }
    }
    Ok(
        Value::Array(
            volumes
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
pub fn list_volumes(args: Vec<Value>) -> Result<Value, String> {
    // Fallback for other Unix platforms.
    if !args.is_empty() {
        return Err(format!("list_volumes() expects no arguments, got {}", args.len()));
    }
    Ok(Value::Array(vec![]))
}

/// --- LIST DRIVES ---
///
/// On Windows, use WinAPI to get the list of drive letters. On non-Windows platforms,
/// we’ll simply reuse the `list_volumes()` function.
#[cfg(windows)]
pub fn list_drives(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("list_drives() expects no arguments, got {}", args.len()));
    }
    let drives_bitmask = unsafe { GetLogicalDrives() };
    let mut drives = Vec::new();
    for i in 0..26 {
        if (drives_bitmask & (1 << i)) != 0 {
            let drive_letter = (b'A' + (i as u8)) as char;
            let drive_str = format!("{}:\\", drive_letter);
            // Optionally, you could filter based on drive type using GetDriveTypeW.
            drives.push(Value::String(drive_str));
        }
    }
    Ok(
        Value::Array(
            drives
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

#[cfg(not(windows))]
pub fn list_drives(args: Vec<Value>) -> Result<Value, String> {
    // On non-Windows systems, list_drives is not applicable; return volumes instead.
    list_volumes(args)
}

pub fn is_app_installed(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("is_app_installed() expects 1 argument, got {}", args.len()));
    }
    let app_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("is_app_installed() expects a string for the application name".to_string());
        }
    };
    #[cfg(unix)]
    {
        let output = Command::new("which").arg(&app_name).output();
        match output {
            Ok(o) => Ok(Value::Bool(!o.stdout.is_empty())),
            Err(e) => Err(format!("is_app_installed() error: {}", e)),
        }
    }
    #[cfg(windows)]
    {
        let output = Command::new("where").arg(&app_name).output();
        match output {
            Ok(o) => Ok(Value::Bool(!o.stdout.is_empty())),
            Err(e) => Err(format!("is_app_installed() error: {}", e)),
        }
    }
}

//
// ==================== PATH OPERATIONS ====================
//

pub fn join_path(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("join_path() expects 1 argument (an array of strings)".to_string());
    }
    let parts_arc = match &args[0] {
        Value::Array(vec) => vec.clone(),
        _ => {
            return Err("join_path() expects an array of strings".to_string());
        }
    };
    let mut path_buf = PathBuf::new();
    for part_arc in parts_arc {
        let part = {
            let locked = part_arc
                .lock()
                .map_err(|_| "join_path() error: Failed to lock array element")?;
            match &*locked {
                Value::String(s) => s.clone(),
                _ => {
                    return Err("join_path() expects all elements to be strings".to_string());
                }
            }
        };
        path_buf.push(part);
    }
    Ok(Value::String(path_buf.to_string_lossy().to_string()))
}

pub fn split_drive(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("split_drive() expects 1 argument".to_string());
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("split_drive() expects a string".to_string());
        }
    };
    let path = Path::new(&path_str);
    #[cfg(windows)]
    {
        // On Windows, try to extract the drive prefix.
        if let Some(prefix) = path.components().next() {
            let drive = prefix.as_os_str().to_string_lossy().to_string();
            let rest = path.strip_prefix(&drive).unwrap_or(path).to_string_lossy().to_string();
            let mut dict = HashMap::new();
            dict.insert("drive".to_string(), Value::String(drive));
            dict.insert("path".to_string(), Value::String(rest));
            let arc_dict = dict
                .into_iter()
                .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
                .collect();
            return Ok(Value::Dictionary(arc_dict));
        }
    }
    // For non-Windows or if no drive is found, return an empty drive.
    let mut dict = HashMap::new();
    dict.insert("drive".to_string(), Value::String(String::new()));
    dict.insert("path".to_string(), Value::String(path_str));
    let arc_dict = dict
        .into_iter()
        .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
        .collect();
    Ok(Value::Dictionary(arc_dict))
}

pub fn split_path(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("split_path() expects 1 argument".to_string());
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("split_path() expects a string".to_string());
        }
    };
    let path = Path::new(&path_str);
    let components: Vec<Value> = path
        .components()
        .map(|c| Value::String(c.as_os_str().to_string_lossy().to_string()))
        .collect();
    Ok(
        Value::Array(
            components
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

//
// ==================== PROCESS INFORMATION & COMMAND EXECUTION ====================
//

pub fn get_process_id(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_process_id() expects no arguments, got {}", args.len()));
    }
    let pid = std::process::id();
    Ok(Value::Number(pid as f64))
}

pub fn run_command(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("run_command() expects at least 1 argument".to_string());
    }
    let command_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("run_command() expects the command as a string".to_string());
        }
    };
    let mut cmd = Command::new(&command_str);
    if args.len() == 2 {
        match &args[1] {
            Value::Array(arr) => {
                for arg_arc in arr {
                    let arg = {
                        let locked = arg_arc
                            .lock()
                            .map_err(|_| "run_command() error: Failed to lock argument")?;
                        match &*locked {
                            Value::String(s) => s.clone(),
                            _ => {
                                return Err(
                                    "run_command() expects command arguments as strings".to_string()
                                );
                            }
                        }
                    };
                    cmd.arg(arg);
                }
            }
            _ => {
                return Err(
                    "run_command() expects the second argument as an array of strings".to_string()
                );
            }
        }
    }
    let output = cmd.output().map_err(|e| format!("run_command() error: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let exit_status = output.status.code().unwrap_or(-1);
    let mut dict = HashMap::new();
    dict.insert("output".to_string(), Value::String(stdout));
    dict.insert("exit_status".to_string(), Value::Number(exit_status as f64));
    let arc_dict = dict
        .into_iter()
        .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
        .collect();
    Ok(Value::Dictionary(arc_dict))
}

pub fn spawn_process(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("spawn_process() expects at least 1 argument".to_string());
    }
    let command_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("spawn_process() expects the command as a string".to_string());
        }
    };
    let mut cmd = Command::new(&command_str);
    if args.len() == 2 {
        match &args[1] {
            Value::Array(arr) => {
                for arg_arc in arr {
                    let arg = {
                        let locked = arg_arc
                            .lock()
                            .map_err(|_| "spawn_process() error: Failed to lock argument")?;
                        match &*locked {
                            Value::String(s) => s.clone(),
                            _ => {
                                return Err(
                                    "spawn_process() expects command arguments as strings".to_string()
                                );
                            }
                        }
                    };
                    cmd.arg(arg);
                }
            }
            _ => {
                return Err(
                    "spawn_process() expects the second argument as an array of strings".to_string()
                );
            }
        }
    }
    let child = cmd.spawn().map_err(|e| format!("spawn_process() error: {}", e))?;
    // Return the process ID as a handle.
    Ok(Value::Number(child.id() as f64))
}

pub fn kill_process(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("kill_process() expects 1 argument (the process ID)".to_string());
    }
    let pid = match &args[0] {
        Value::Number(n) => *n as u32,
        _ => {
            return Err("kill_process() expects a number (PID)".to_string());
        }
    };
    #[cfg(unix)]
    {
        kill(Pid::from_raw(pid as i32), Signal::SIGKILL).map_err(|e|
            format!("kill_process() error: {}", e)
        )?;
        Ok(Value::Bool(true))
    }
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| format!("kill_process() error: {}", e))?;
        if output.status.success() {
            Ok(Value::Bool(true))
        } else {
            Err("kill_process() failed".to_string())
        }
    }
}

//
// ==================== FILE/DIRECTORY CHECKS & TREE DISPLAY ====================
//

pub fn is_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("is_file() expects 1 argument".to_string());
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("is_file() expects a string".to_string());
        }
    };
    let path = Path::new(&path_str);
    Ok(Value::Bool(path.is_file()))
}

pub fn is_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("is_dir() expects 1 argument".to_string());
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("is_dir() expects a string".to_string());
        }
    };
    let path = Path::new(&path_str);
    Ok(Value::Bool(path.is_dir()))
}

fn build_tree(path: &Path) -> Result<Value, String> {
    let mut dict = HashMap::new();
    let name = path
        .file_name()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    dict.insert("name".to_string(), Value::String(name));
    dict.insert("path".to_string(), Value::String(path.to_string_lossy().to_string()));
    if path.is_dir() {
        let mut children = Vec::new();
        for entry in fs
            ::read_dir(path)
            .map_err(|e| format!("get_tree() error reading directory: {}", e))? {
            let entry = entry.map_err(|e| format!("get_tree() error: {}", e))?;
            let child_path = entry.path();
            children.push(build_tree(&child_path)?);
        }
        dict.insert(
            "children".to_string(),
            Value::Array(
                children
                    .into_iter()
                    .map(|v| Arc::new(Mutex::new(v)))
                    .collect()
            )
        );
    }
    let arc_dict = dict
        .into_iter()
        .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
        .collect();
    Ok(Value::Dictionary(arc_dict))
}

pub fn get_tree(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_tree() expects 1 argument".to_string());
    }
    let path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("get_tree() expects a string".to_string());
        }
    };
    let path = Path::new(&path_str);
    build_tree(path)
}

//
// ==================== SYSTEM INFORMATION & UTILITY ====================
//

use sysinfo::{ System, Cpu, Disks, Networks, Pid, Process, User };

pub fn get_system_info(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(format!("get_system_info() expects no arguments, got {}", args.len()));
    }

    let mut dict = HashMap::new();

    // Basic OS and architecture info
    dict.insert("os".to_string(), Value::String(env::consts::OS.to_string()));
    dict.insert("architecture".to_string(), Value::String(env::consts::ARCH.to_string()));
    dict.insert("family".to_string(), Value::String(env::consts::FAMILY.to_string()));

    // Initialize sysinfo System
    let mut system = System::new_all();
    system.refresh_all();

    // Hostname and domain name
    if let Some(hostname) = sysinfo::System::host_name() {
        dict.insert("hostname".to_string(), Value::String(hostname));
    }
    if let Some(domain) = sysinfo::System::long_os_version() {
        dict.insert("domain_name".to_string(), Value::String(domain));
    }

    // OS and kernel version
    if let Some(os_version) = sysinfo::System::os_version() {
        dict.insert("os_version".to_string(), Value::String(os_version));
    }
    if let Some(kernel_version) = sysinfo::System::kernel_version() {
        dict.insert("kernel_version".to_string(), Value::String(kernel_version));
    }

    // CPU information
    let cpus: Vec<Arc<Mutex<Value>>> = system
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| {
            let mut cpu_info = HashMap::new();
            cpu_info.insert("core_id".to_string(), Arc::new(Mutex::new(Value::Number(i as f64))));
            cpu_info.insert(
                "brand".to_string(),
                Arc::new(Mutex::new(Value::String(cpu.brand().to_string())))
            );
            cpu_info.insert(
                "frequency_mhz".to_string(),
                Arc::new(Mutex::new(Value::Number(cpu.frequency() as f64)))
            );
            cpu_info.insert(
                "usage_percent".to_string(),
                Arc::new(Mutex::new(Value::Number(cpu.cpu_usage() as f64)))
            );
            Arc::new(Mutex::new(Value::Dictionary(cpu_info)))
        })
        .collect();

    dict.insert("cpus".to_string(), Value::Array(cpus));
    dict.insert(
        "cpu_count_physical".to_string(),
        Value::Number(sysinfo::System::physical_core_count().unwrap_or(0) as f64)
    );

    // Memory information (in MB)
    let total_memory = system.total_memory() / 1024; // KB to MB
    let used_memory = system.used_memory() / 1024; // KB to MB
    let total_swap = system.total_swap() / 1024; // KB to MB
    let used_swap = system.used_swap() / 1024; // KB to MB
    dict.insert("total_memory_mb".to_string(), Value::Number(total_memory as f64));
    dict.insert("used_memory_mb".to_string(), Value::Number(used_memory as f64));
    dict.insert("total_swap_mb".to_string(), Value::Number(total_swap as f64));
    dict.insert("used_swap_mb".to_string(), Value::Number(used_swap as f64));

    // Disk information
    let disks = sysinfo::Disks::new_with_refreshed_list();

    let disk_infos: Vec<Arc<Mutex<Value>>> = disks
        .list()
        .iter()
        .map(|disk| {
            let mut disk_info = HashMap::new();
            disk_info.insert(
                "mount_point".to_string(),
                Arc::new(Mutex::new(Value::String(disk.mount_point().to_string_lossy().into())))
            );
            disk_info.insert(
                "total_space_mb".to_string(),
                Arc::new(Mutex::new(Value::Number((disk.total_space() / 1024 / 1024) as f64)))
            );
            disk_info.insert(
                "available_space_mb".to_string(),
                Arc::new(Mutex::new(Value::Number((disk.available_space() / 1024 / 1024) as f64)))
            );
            disk_info.insert(
                "is_removable".to_string(),
                Arc::new(Mutex::new(Value::Number(if disk.is_removable() { 1.0 } else { 0.0 })))
            );
            disk_info.insert(
                "disk_type".to_string(),
                Arc::new(Mutex::new(Value::String(format!("{:?}", disk.kind()))))
            );
            disk_info.insert(
                "file_system".to_string(),
                Arc::new(Mutex::new(Value::String(disk.file_system().to_string_lossy().into())))
            );
            Arc::new(Mutex::new(Value::Dictionary(disk_info)))
        })
        .collect();

    dict.insert("disks".to_string(), Value::Array(disk_infos));

    // Refresh and retrieve network data
    let mut networks = Networks::new_with_refreshed_list();

    // Refresh each network interface's data
    networks.refresh(true); // Needed to get up-to-date usage

    let network_infos: Vec<Arc<Mutex<Value>>> = networks
        .iter()
        .map(|(name, data)| {
            let mut net_info: HashMap<String, Arc<Mutex<Value>>> = HashMap::new();

            net_info.insert(
                "interface_name".to_string(),
                Arc::new(Mutex::new(Value::String(name.to_string())))
            );
            net_info.insert(
                "received_mb".to_string(),
                Arc::new(Mutex::new(Value::Number((data.total_received() / 1024 / 1024) as f64)))
            );
            net_info.insert(
                "transmitted_mb".to_string(),
                Arc::new(Mutex::new(Value::Number((data.total_transmitted() / 1024 / 1024) as f64)))
            );

            net_info.insert(
                "mac_address".to_string(),
                Arc::new(Mutex::new(Value::String(data.mac_address().to_string())))
            );

            Arc::new(Mutex::new(Value::Dictionary(net_info)))
        })
        .collect();

    dict.insert("networks".to_string(), Value::Array(network_infos));

    // System load averages (Linux/Unix only)
    let load_avg = sysinfo::System::load_average(); // ✅ correct usage
    let mut load_info = HashMap::new();
    load_info.insert(
        "one_min".to_string(),
        Arc::new(Mutex::new(Value::Number(load_avg.one as f64)))
    );
    load_info.insert(
        "five_min".to_string(),
        Arc::new(Mutex::new(Value::Number(load_avg.five as f64)))
    );
    load_info.insert(
        "fifteen_min".to_string(),
        Arc::new(Mutex::new(Value::Number(load_avg.fifteen as f64)))
    );
    dict.insert("load_average".to_string(), Value::Dictionary(load_info));

    // Process information (limited to top 5 by CPU usage)
    // Refresh all processes
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    #[cfg(unix)]
    let mut processes: Vec<(NixPid, &sysinfo::Process)> = system
        .processes()
        .iter()
        .map(|(sys_pid, process)| {
            // Convert sysinfo::Pid to libc::pid_t then to nix::unistd::Pid
            let raw_pid: libc::pid_t = sys_pid.as_u32() as libc::pid_t;
            let nix_pid = NixPid::from_raw(raw_pid);
            (nix_pid, process)
        })
        .collect();
    
    
    #[cfg(not(unix))]
    let mut processes: Vec<(SysPid, &sysinfo::Process)> = system
        .processes()
        .iter()
        .map(|(sys_pid, process)| (*sys_pid, process))
        .collect();


    // Sort by CPU usage in descending order
    processes.sort_by(|a, b|
        b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
    );

    // Take top 5 processes by CPU usage
    // Map processes to Arc<Mutex<Value>> and handle name conversion
    let top_processes: Vec<Arc<Mutex<Value>>> = processes
        .into_iter()
        .take(5)
        .map(|(pid, process)| {
            let mut proc_info = HashMap::new();

            // Convert process name properly using to_string_lossy()
            let process_name = process.name().to_string_lossy().into_owned(); // Handles non-UTF8 names gracefully

            proc_info.insert(
                "pid".to_string(),
                Arc::new(Mutex::new(Value::Number(pid.as_u32() as f64)))
            );
            proc_info.insert("name".to_string(), Arc::new(Mutex::new(Value::String(process_name))));
            proc_info.insert(
                "cpu_usage_percent".to_string(),
                Arc::new(Mutex::new(Value::Number(process.cpu_usage() as f64)))
            );
            proc_info.insert(
                "memory_mb".to_string(),
                Arc::new(Mutex::new(Value::Number((process.memory() / 1024 / 1024) as f64)))
            );

            Arc::new(Mutex::new(Value::Dictionary(proc_info)))
        })
        .collect();

    // Insert the top processes into the dictionary
    dict.insert("top_processes".to_string(), Value::Array(top_processes));

    // User information
    let users = Users::new_with_refreshed_list();

    let user_info: Vec<Arc<Mutex<Value>>> = users
        .list()
        .iter()
        .map(|user| {
            let mut user_details = HashMap::new();

            // Insert user name as an Arc<Mutex<Value>>
            user_details.insert(
                "name".to_string(),
                Arc::new(Mutex::new(Value::String(user.name().to_string())))
            );

            // Insert user groups as an Arc<Mutex<Value>> inside an array
            user_details.insert(
                "groups".to_string(),
                Arc::new(
                    Mutex::new(
                        Value::Array(
                            user
                                .groups()
                                .iter()
                                .map(|group|
                                    Arc::new(Mutex::new(Value::String(group.name().to_string())))
                                )
                                .collect()
                        )
                    )
                )
            );

            Arc::new(Mutex::new(Value::Dictionary(user_details)))
        })
        .collect();

    dict.insert("users".to_string(), Value::Array(user_info));

    // Timezone and current time
    if let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH) {
        dict.insert("current_time_unix".to_string(), Value::Number(current_time.as_secs() as f64));
    }
    let binding = chrono::Local::now();
    let local_offset = binding.offset();
    let local_time = local_offset.to_string(); // This should give you the offset as a string.

    dict.insert("timezone".to_string(), Value::String(local_time));

    // Uptime and boot time
    dict.insert("uptime_seconds".to_string(), Value::Number(sysinfo::System::uptime() as f64));

    // Use associated function syntax
    let mut system = System::new_all();
    system.refresh_all();

    // Get the system uptime (in seconds)
    let uptime = sysinfo::System::uptime();

    // Get the current time (in seconds)
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Calculate the boot time by subtracting the uptime from the current time
    let boot_time = current_time - uptime;

    dict.insert("boot_time".to_string(), Value::Number(boot_time as f64));

    // Convert to thread-safe dictionary
    let arc_dict = dict
        .into_iter()
        .map(|(k, v)| (k, Arc::new(Mutex::new(v))))
        .collect();
    Ok(Value::Dictionary(arc_dict))
}

pub fn sleep(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("sleep() expects 1 argument (duration in milliseconds)".to_string());
    }
    let duration_ms = match &args[0] {
        Value::Number(n) => *n as u64,
        _ => {
            return Err("sleep() expects a number".to_string());
        }
    };
    thread::sleep(Duration::from_millis(duration_ms));
    Ok(Value::Null)
}

//
// ==================== NETWORK UTILITIES ====================
//

pub fn get_public_ip(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("get_public_ip() expects no arguments".to_string());
    }
    let response = reqwest::blocking
        ::get("https://api.ipify.org")
        .map_err(|e| format!("get_public_ip() error: {}", e))?;
    let ip = response.text().map_err(|e| format!("get_public_ip() error: {}", e))?;
    Ok(Value::String(ip))
}

pub fn get_private_ip(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("get_private_ip() expects no arguments".to_string());
    }
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e|
        format!("get_private_ip() error: {}", e)
    )?;
    socket.connect("8.8.8.8:80").map_err(|e| format!("get_private_ip() error: {}", e))?;
    let local_addr = socket.local_addr().map_err(|e| format!("get_private_ip() error: {}", e))?;
    Ok(Value::String(local_addr.ip().to_string()))
}

/// Returns the MAC address of the primary network interface.
/// Requires the `get_if_addrs` crate.
pub fn get_mac_address(args: Vec<Value>) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("get_mac_address() expects no arguments".to_string());
    }
    match mac_address() {
        Ok(Some(ma)) => Ok(Value::String(ma.to_string())),
        Ok(None) => Err("get_mac_address() error: No MAC address found".to_string()),
        Err(e) => Err(format!("get_mac_address() error: {}", e)),
    }
}

pub fn is_port_open(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("is_port_open() expects 2 arguments".to_string());
    }
    let ip = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("is_port_open() expects the first argument as an IP string".to_string());
        }
    };
    let port = match &args[1] {
        Value::Number(n) => *n as u16,
        _ => {
            return Err("is_port_open() expects the second argument as a port number".to_string());
        }
    };
    let addr = format!("{}:{}", ip, port);
    let parsed_addr = addr.parse().map_err(|e| format!("is_port_open() invalid address: {}", e))?;
    match TcpStream::connect_timeout(&parsed_addr, Duration::from_secs(1)) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

pub fn list_open_ports(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("list_open_ports() expects 1 argument (IP)".to_string());
    }
    let ip = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("list_open_ports() expects a string for the IP".to_string());
        }
    };
    let mut open_ports = Vec::new();
    // For demonstration purposes, scan ports 1 through 1024.
    for port in 1..1025 {
        let addr = format!("{}:{}", ip, port);
        let parsed_addr = addr
            .parse()
            .map_err(|e| format!("list_open_ports() invalid address: {}", e))?;
        if TcpStream::connect_timeout(&parsed_addr, Duration::from_millis(200)).is_ok() {
            open_ports.push(Value::Number(port as f64));
        }
    }
    Ok(
        Value::Array(
            open_ports
                .into_iter()
                .map(|v| Arc::new(Mutex::new(v)))
                .collect()
        )
    )
}

pub fn ping(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("ping() expects 1 argument".to_string());
    }
    let host = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err("ping() expects a string".to_string());
        }
    };
    #[cfg(windows)]
    let output = Command::new("ping").args(&["-n", "1", &host]).output();
    #[cfg(not(windows))]
    let output = Command::new("ping").args(&["-c", "1", &host]).output();
    match output {
        Ok(o) => Ok(Value::Bool(o.status.success())),
        Err(e) => Err(format!("ping() error: {}", e)),
    }
}
