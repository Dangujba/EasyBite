use crate::evaluation::Value;
use std::fs;
use std::io::{ BufRead, BufReader, Write };
use std::path::Path;
use std::sync::{ Arc, Mutex };
use std::time::UNIX_EPOCH;

/// append(filename, content)
/// Appends the content (a string) to the specified file.
pub fn append(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("append() expects 2 arguments, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("append() expects first argument to be a string".to_string());
        }
    };
    let content = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("append() expects second argument to be a string".to_string());
        }
    };

    let mut file = fs::OpenOptions
        ::new()
        .append(true)
        .create(true)
        .open(filename)
        .map_err(|e| format!("append() error: {}", e))?;
    writeln!(file, "{}", content).map_err(|e| format!("append() error: {}", e))?;
    Ok(Value::Null)
}

/// copy(source, destination)
/// Copies a file from source to destination.
pub fn copy(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("copy() expects 2 arguments, got {}", args.len()));
    }
    let source = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("copy() expects first argument to be a string".to_string());
        }
    };
    let destination = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("copy() expects second argument to be a string".to_string());
        }
    };
    fs::copy(source, destination).map_err(|e| format!("copy() error: {}", e))?;
    Ok(Value::Null)
}

/// create(filename)
/// Creates a new file with the specified filename.
pub fn create(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("create() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("create() expects a string".to_string());
        }
    };
    fs::File::create(filename).map_err(|e| format!("create() error: {}", e))?;
    Ok(Value::Null)
}

/// delete(filename)
/// Deletes the file with the specified filename.
pub fn delete(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("delete() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("delete() expects a string".to_string());
        }
    };
    fs::remove_file(filename).map_err(|e| format!("delete() error: {}", e))?;
    Ok(Value::Null)
}

/// exists(filename)
/// Checks if the file with the specified filename exists.
pub fn exists(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("exists() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("exists() expects a string".to_string());
        }
    };
    let exists = Path::new(filename).exists();
    Ok(Value::Bool(exists))
}

/// move(source, destination)
/// Moves (or renames) a file from source to destination.
pub fn remove(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("move() expects 2 arguments, got {}", args.len()));
    }
    let source = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("move() expects first argument to be a string".to_string());
        }
    };
    let destination = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("move() expects second argument to be a string".to_string());
        }
    };
    fs::rename(source, destination).map_err(|e| format!("move() error: {}", e))?;
    Ok(Value::Null)
}

/// read(filename)
/// Reads the entire content of the file and returns it as a string.
pub fn read(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("read() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("read() expects a string".to_string());
        }
    };
    let content = fs::read_to_string(filename).map_err(|e| format!("read() error: {}", e))?;
    Ok(Value::String(content))
}

/// write(filename, content)
/// Writes the content (a string) to the specified filename.
pub fn write(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("write() expects 2 arguments, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("write() expects first argument to be a string".to_string());
        }
    };
    let content = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("write() expects second argument to be a string".to_string());
        }
    };
    fs::write(filename, content).map_err(|e| format!("write() error: {}", e))?;
    Ok(Value::Null)
}

/// filename(filepath)
/// Returns the name of the file from the given filepath.
pub fn filename(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("filename() expects 1 argument, got {}", args.len()));
    }
    let filepath = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("filename() expects a string".to_string());
        }
    };
    let path = Path::new(filepath);
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            return Ok(Value::String(name_str.to_string()));
        }
    }
    Err("filename() could not extract filename".to_string())
}

/// filepath(filename)
/// Returns the path (parent directory) of the given file.
pub fn filepath(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("filepath() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("filepath() expects a string".to_string());
        }
    };
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        if let Some(parent_str) = parent.to_str() {
            return Ok(Value::String(parent_str.to_string()));
        }
    }
    Err("filepath() could not extract file path".to_string())
}

/// folderexist(foldername)
/// Checks if the folder with the specified foldername exists.
pub fn folderexist(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("folderexist() expects 1 argument, got {}", args.len()));
    }
    let foldername = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("folderexist() expects a string".to_string());
        }
    };
    let exists = Path::new(foldername).is_dir();
    Ok(Value::Bool(exists))
}

/// foldername(folderpath)
/// Returns the name of the folder from the given folder path.
pub fn foldername(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("foldername() expects 1 argument, got {}", args.len()));
    }
    let folderpath = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("foldername() expects a string".to_string());
        }
    };
    let path = Path::new(folderpath);
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            return Ok(Value::String(name_str.to_string()));
        }
    }
    Err("foldername() could not extract folder name".to_string())
}

/// folderpath(foldername)
/// Returns the path (parent directory) of the given folder name.
pub fn folderpath(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("folderpath() expects 1 argument, got {}", args.len()));
    }
    let foldername = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("folderpath() expects a string".to_string());
        }
    };
    let path = Path::new(foldername);
    if let Some(parent) = path.parent() {
        if let Some(parent_str) = parent.to_str() {
            return Ok(Value::String(parent_str.to_string()));
        }
    }
    Err("folderpath() could not extract folder path".to_string())
}

/// getfileextension(filename)
/// Retrieves the file extension from the given filename.
pub fn getfileextension(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getfileextension() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getfileextension() expects a string".to_string());
        }
    };
    let path = Path::new(filename);
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return Ok(Value::String(ext_str.to_string()));
        }
    }
    Err("getfileextension() could not extract extension".to_string())
}

/// getfiles(foldername)
/// Returns a list of files in the specified folder.
pub fn getfiles(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getfiles() expects 1 argument, got {}", args.len()));
    }
    let folder = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getfiles() expects a string".to_string());
        }
    };
    let files: Vec<Value> = fs
        ::read_dir(folder)
        .map_err(|e| format!("getfiles() error: {}", e))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() {
                    path.to_str().map(|s| Value::String(s.to_string()))
                } else {
                    None
                }
            })
        })
        .collect();
    let wrapped = files
        .into_iter()
        .map(|v| Arc::new(Mutex::new(v)))
        .collect();
    Ok(Value::Array(wrapped))
}

/// getfolders(foldername)
/// Returns a list of folders in the specified folder.
pub fn getfolders(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getfolders() expects 1 argument, got {}", args.len()));
    }
    let folder = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getfolders() expects a string".to_string());
        }
    };
    let folders: Vec<Value> = fs
        ::read_dir(folder)
        .map_err(|e| format!("getfolders() error: {}", e))?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_dir() {
                    path.to_str().map(|s| Value::String(s.to_string()))
                } else {
                    None
                }
            })
        })
        .collect();
    let wrapped = folders
        .into_iter()
        .map(|v| Arc::new(Mutex::new(v)))
        .collect();
    Ok(Value::Array(wrapped))
}

/// getlastmodifiedtime(filename)
/// Retrieves the last modified timestamp (in seconds since the Unix epoch) of the file.
pub fn getlastmodifiedtime(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getlastmodifiedtime() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getlastmodifiedtime() expects a string".to_string());
        }
    };
    let metadata = fs
        ::metadata(filename)
        .map_err(|e| format!("getlastmodifiedtime() error: {}", e))?;
    let modified = metadata.modified().map_err(|e| format!("getlastmodifiedtime() error: {}", e))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("getlastmodifiedtime() error: {}", e))?;
    Ok(Value::Number(duration.as_secs() as f64))
}

/// getparentdirectory(path)
/// Retrieves the parent directory of the given path.
pub fn getparentdirectory(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getparentdirectory() expects 1 argument, got {}", args.len()));
    }
    let path_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getparentdirectory() expects a string".to_string());
        }
    };
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        if let Some(p) = parent.to_str() {
            return Ok(Value::String(p.to_string()));
        }
    }
    Err("getparentdirectory() could not determine parent directory".to_string())
}

/// getfilesize(filename)
/// Returns the size of the file in bytes.
pub fn getfilesize(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("getfilesize() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("getfilesize() expects a string".to_string());
        }
    };
    let metadata = fs::metadata(filename).map_err(|e| format!("getfilesize() error: {}", e))?;
    Ok(Value::Number(metadata.len() as f64))
}

/// getsub(foldername)
/// Returns a list of sub-folders in the specified folder.
pub fn getsub(args: Vec<Value>) -> Result<Value, String> {
    getfolders(args)
}

/// makefolder(foldername)
/// Creates a new folder with the specified foldername.
pub fn makefolder(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("makefolder() expects 1 argument, got {}", args.len()));
    }
    let foldername = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("makefolder() expects a string".to_string());
        }
    };
    fs::create_dir_all(foldername).map_err(|e| format!("makefolder() error: {}", e))?;
    Ok(Value::Null)
}

/// movefolder(source, destination)
/// Moves a folder from source to destination.
pub fn movefolder(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!("movefolder() expects 2 arguments, got {}", args.len()));
    }
    let source = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("movefolder() expects first argument to be a string".to_string());
        }
    };
    let destination = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err("movefolder() expects second argument to be a string".to_string());
        }
    };
    fs::rename(source, destination).map_err(|e| format!("movefolder() error: {}", e))?;
    Ok(Value::Null)
}

/// readcontent(filename)
/// Reads the content of the file and returns all lines as a list.
pub fn readcontent(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("readcontent() expects 1 argument, got {}", args.len()));
    }
    let filename = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err("readcontent() expects a string".to_string());
        }
    };
    let file = fs::File::open(filename).map_err(|e| format!("readcontent() error: {}", e))?;
    let reader = BufReader::new(file);
    let lines: Vec<Value> = reader
        .lines()
        .map(|line| line.map_err(|e| format!("readcontent() error: {}", e)))
        .collect::<Result<Vec<String>, String>>()?
        .into_iter()
        .map(Value::String)
        .collect();
    let wrapped = lines
        .into_iter()
        .map(|v| Arc::new(Mutex::new(v)))
        .collect();
    Ok(Value::Array(wrapped))
}

/// readline(...)
/// Overload 1: readline(filename, lineNumber)
/// Returns the specified line (0-indexed) from the file.
/// Overload 2: readline(filename, start, end)
/// Returns a list of lines from start to end (0-indexed).
pub fn readline(args: Vec<Value>) -> Result<Value, String> {
    if args.len() == 2 {
        let filename = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err("readline() expects first argument to be a string".to_string());
            }
        };
        let line_number = match &args[1] {
            Value::Number(n) => *n as usize,
            _ => {
                return Err("readline() expects second argument to be a number".to_string());
            }
        };
        let file = fs::File::open(filename).map_err(|e| format!("readline() error: {}", e))?;
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("readline() error: {}", e))?;
            if i == line_number {
                return Ok(Value::String(line));
            }
        }
        Err("readline(): line number out of range".to_string())
    } else if args.len() == 3 {
        let filename = match &args[0] {
            Value::String(s) => s,
            _ => {
                return Err("readline() expects first argument to be a string".to_string());
            }
        };
        let start = match &args[1] {
            Value::Number(n) => *n as usize,
            _ => {
                return Err("readline() expects second argument to be a number".to_string());
            }
        };
        let end = match &args[2] {
            Value::Number(n) => *n as usize,
            _ => {
                return Err("readline() expects third argument to be a number".to_string());
            }
        };
        let file = fs::File::open(filename).map_err(|e| format!("readline() error: {}", e))?;
        let reader = BufReader::new(file);
        let selected: Vec<Value> = reader
            .lines()
            .skip(start)
            .take(end - start)
            .map(|line| line.map_err(|e| format!("readline() error: {}", e)))
            .collect::<Result<Vec<String>, String>>()?
            .into_iter()
            .map(Value::String)
            .collect();
        let wrapped = selected
            .into_iter()
            .map(|v| Arc::new(Mutex::new(v)))
            .collect();
        Ok(Value::Array(wrapped))
    } else {
        Err(format!("readline() expects 2 or 3 arguments, got {}", args.len()))
    }
}
