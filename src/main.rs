/*************************************************************************************
 * Project: EasyBite Programming Language
 * File: main.rs
 * Author: Muhammad Baba Goni
 * Email:  <muhammadgoni51@gmail.com>
 *
 * Description:
 * ------------
 * This file serves as the entry point for EasyBite Compiler.
 * It handles command-line input, file reading,   parsing the source code
 * into an Abstract Syntax Tree (AST), interpreting and executing the AST, and
 * optionally measuring and displaying execution time.
 *
 * Modules:
 * --------
 * - lexer: Tokenizes input scripts into recognizable components.
 * - token: Defines token structures used by the lexer.
 * - parser: Parses tokens into an AST representation.
 * - astnode: Structures representing nodes in the AST.
 * - evaluation: Interprets and executes AST nodes.
 * - string, array, dictionary: Built-in data types and utilities.
 * - math, datetime, conversion: Standard library functions (math operations, date-time handling, type conversion).
 * - filesystem, sqlite, mysqli: Database and file system interaction.
 * - socket, fetcher, listener: Networking capabilities.
 * - system, thread: System-level operations and threading support.
 * - easyui, easyplot: User Interface and plotting utilities for the scripting environment.
 *
 * License:
 * --------
 * This project is open source and available for use, modification, and distribution
 * under the MIT License or similar permissive licenses.
 *
 ************************************************************************************/

// Import project modules
mod lexer;
mod token;
mod parser;
mod astnode;
mod evaluation;
mod string;
mod array;
mod dictionary;
mod math;
mod datetime;
mod conversion;
mod filesystem;
mod sqlite;
mod mysqli;
mod socket;
mod fetcher;
mod listener;
mod system;
mod thread;
mod easyui;
mod easyplot;

// Import necessary types from modules
use astnode::ASTNode;
use parser::Parser;
use evaluation::Interpreter;
use std::env;
use std::fs;
use std::time::Instant;
use std::process;

/// Entry point of the application
fn main() {
    // Execute the program and handle any errors by printing and exiting
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Runs the interpreter with command-line arguments
fn run() -> Result<(), String> {
    // Collect all command-line arguments
    let args: Vec<String> = env::args().collect();

    // Ensure at least one argument (the filename) is provided
    if args.len() < 2 {
        return Err(format!("Usage: {} <filename> [--time] [--help]", args[0]));
    }

    // Handle help option
    if
        args
            .iter()
            .skip(1)
            .any(|arg| arg == "--help")
    {
        println!("Usage: {} <filename> [--time] [--help]", args[0]);
        println!("  <filename>    The input file to process");
        println!("  --time        Display total execution time");
        println!("  --help        Show this help message");
        return Ok(());
    }

    // Retrieve the filename from arguments
    let filename = &args[1];
    // Check if execution time measurement is requested
    let enable_timing = args
        .iter()
        .skip(2)
        .any(|arg| arg == "--time");

    // Start the timer if timing is enabled
    let start_time = if enable_timing { Some(Instant::now()) } else { None };

    // Read the input file content into a string
    let input = fs
        ::read_to_string(filename)
        .map_err(|e| format!("Error reading file '{}': {}", filename, e))?;

    // Initialize the parser with the input content
    let mut parser = Parser::new(&input).map_err(|e| format!("Error creating parser: {}", e))?;

    // Parse the input into an Abstract Syntax Tree (AST)
    let ast: ASTNode = parser.parse().map_err(|e| format!("Error parsing input: {}", e))?;

    // Initialize the interpreter
    let mut interpreter = Interpreter::new();

    // Interpret (execute) the AST
    interpreter.interpret(&ast).map_err(|e| format!("Error interpreting: {}", e))?;

    // If timing was enabled, calculate and print the duration
    if let Some(start) = start_time {
        let duration = start.elapsed();
        println!("Total execution time: {:?}", duration);
    }

    // Indicate successful execution
    Ok(())
}
