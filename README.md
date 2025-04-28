# EasyBite Programming Language Summary

| Download Setup | Download Setup with Example | Download Latest Version |
| -------------- | --------------------------- | ----------------------- |
| [![Download](https://img.shields.io/badge/Download-Repository-brightgreen)](https://github.com/Dangujba/EasyBite/raw/main/bin/EasyBite.exe) | [![Download](https://img.shields.io/badge/Download-Repository-brightgreen)](https://github.com/Dangujba/EasyBite/raw/main/bin/EasyBiteExample.zip) | [![Download](https://img.shields.io/badge/Download-Repository-brightgreen)](https://github.com/Dangujba/EasyBite/releases/download/v0.3.0/EasyBite-0.3.0-x86_64.msi) |

## Overview
EasyBite is a beginner-friendly programming language designed to be accessible to learners of all ages and backgrounds. It emphasizes simplicity while offering essential programming functionalities, making development both approachable and powerful.

## Installation

**Requirements**  
- EasyBite interpreter v0.3.0 or later  
- (Versions 0.1.0 and 0.2.0 required .NET Framework; starting with 0.3.0, .NET is no longer required)

**Steps**  
1. Download the interpreter from [Bin Folder](https://github.com/Dangujba/EasyBite/bin) or the official GitHub page: [github.com/Dangujba/EasyBite](https://github.com/Dangujba/EasyBite).  
2. Run the installer for your platform (Windows `.msi`, macOS `.pkg`, or Linux `.tar.gz`).  
3. (Optional) Add `easybite` to your PATH so you can run it from any shell:  
   ```bash
   # Example on Linux/macOS
   export PATH="$PATH:/usr/local/bin"
   ```  
4. Verify installation:  
   ```bash
   easybite --version   # should report 0.3.0
   ```

## Key Features

- **Input / Output**  
  - `input()` to read from the user  
  - `print()` to display output  

- **Control Structures**  
  - `if` / `else`  
  - `for`, `while`, and new `foreach` loops  

- **Operators**  
  - Standard arithmetic/comparison  
  - New: `is`, `is in`, `is not`, `not in`  

- **Functions**  
  - Define with `fn`  
  - Full recursion support  

- **Method Access on Literals**  
  ```bite
  "Hello".count()
  [1,2,3].append(4)
  ```  
  (No imports required for these methods)

- **Callback Functions**  
  - Specify callbacks as identifiers (not strings) for type safety

- **Import**  
  - `import socket, requester, listener, plotter, thread, system, mysql, ...`

## In-Built Libraries

### Core Data Types
- **Math**: `abs()`, `pow()`, `sqrt()`, `rand()`  
- **String**: `count()`, `find()`, `replace()`  
- **Array**: `length`, `append()`, `sort()`, `index()`  
- **Dictionary**: `keys()`, `values()`, `get()`, `set()`  
- **DateTime**: `now()`, `diff()`, formatting  

### System & I/O
- **Files**: `open()`, `read()`, `write()`, `exists()`  
- **System**: environment variables, process control  
- **SQLite**: embedded database operations  

### Networking & Concurrency
- **socket**: TCP/UDP client & server  
- **requester**: HTTP client for REST APIs  
- **listener**: lightweight HTTP server  
- **thread**: thread creation & synchronization  

### Visualization & GUI
- **plotter**: generate charts at runtime  
- **GUI Controls**: `DatePicker`, `TimePicker`, `Pages`, `FlowLayout`, `Slider`, `ProgressBar`, `Accordion`, `DropdownMenu`, `OpenFileDialog`, `SaveFileDialog`, `Alert`, `TreeView`, and more  

### Package Management
- **Bitely** (default with Windows `.msi`)  
  ```bash
  bitely install <package>
  bitely publish <package>
  ```  
- Registry: default `https://github.com/Dangujba/bite-registry` (override with `--registry` or `BITE_REGISTRY`)  
- Custom modules: set `BITE_MODULES` to your modules directory  

## Performance & Open Source
- Compiler rewritten in **Rust**, ~10× faster than the prior C# version  
- Now fully **open source** under the MIT License: [github.com/Dangujba/EasyBite](https://github.com/Dangujba/EasyBite)

## Cross-Platform Compatibility
Runs natively on **Windows**, **macOS**, and **Linux** with identical behavior across platforms.

## Documentation
Comprehensive guides, API reference, and examples are available at [easybitedocs.github.io](https://easybitedocs.github.io).

## Contact & Feedback
- **Email:** muhammadgoni51@gmail.com  
- **GitHub Issues:** [Report bugs or suggest features](https://github.com/Dangujba/EasyBite/issues)  
- **Community Forum:** [EasyBite Community](https://community.easybite-lang.com)

## Additional Notes
- **OOP Support:** Full encapsulation, inheritance, polymorphism, composition  
- **Recursion:** Functions may call themselves for complex algorithms  
- **Environment:** No external runtime required (v0.3.0+)

