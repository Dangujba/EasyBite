/****************************************************************************************
 * File: astnode.rs
 * Author: Muhammad Baba Goni
 * Email: <muhammadgoni51@gmail.com>
 * Date:  02/03/2025
 * 
 * Description:
 * ------------
 * This file defines the Abstract Syntax Tree (AST) Node structures for the scripting language.
 *
 * The AST is a tree representation of the abstract syntactic structure of the source code.
 * Each node in the tree denotes a construct occurring in the source (like expressions, 
 * statements, loops, etc.).
 *
 * Responsibilities:
 * -----------------
 * - Define various types of AST nodes (literals, binary operations, function calls, etc.).
 * - Represent parent-child relationships between operations and their operands.
 * - Serve as the intermediate data structure between parsing and execution phases.
 *
 * Usage:
 * ------
 * After parsing, the AST is passed to the Interpreter for execution.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::token::{Token, TokenType};

// Represents a node in the Abstract Syntax Tree
#[derive(Debug, Clone)]
pub enum ASTNode {
    
    // Statements
    Declaration {
        names: Vec<Box<ASTNode>>, // IDs or ArrayAccess nodes
        line: usize,
        column: usize,
    },
    SetStatement {
        target: Box<ASTNode>,     // ID or ArrayAccess
        value: Box<ASTNode>,      // Expression after "to"
        line: usize,
        column: usize,
    },
    AssignStatement {
        target: Box<ASTNode>,     // ID or ArrayAccess
        value: Box<ASTNode>,      // Expression after "to"
        line: usize,
        column: usize,
    },
    IfStatement {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        elseif_clauses: Vec<(Box<ASTNode>, Vec<ASTNode>)>, // (condition, body)
        else_body: Option<Vec<ASTNode>>,
        line: usize,
        column: usize,
    },
    InputStatement {
        target: Box<ASTNode>,     // ID
        prompt: Box<ASTNode>,     // Expression inside input()
        line: usize,
        column: usize,
    },
    ForStatement {
        variable: String,
        start: Box<ASTNode>,
        end: Box<ASTNode>,
        step: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ForeachStatement {
        variables: Vec<String>,  // Multiple variable names for iteration
        iterable: Box<ASTNode>,  // The expression being iterated over
        body: Vec<ASTNode>,      // Statements inside the loop
        line: usize,
        column: usize,
    },
    GenerateStatement {
        variable: String,
        start: Box<ASTNode>,
        end: Box<ASTNode>,
        by: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ShowStatement {
        expr: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    RepeatStatement {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    RepeatTimeStatement {
        times: Box<ASTNode>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ShowLnStatement {
        line: usize,
        column: usize,
    },
    IterateStatement {
        variable: String,
        iterable: Box<ASTNode>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ChooseStatement {
        expr: Box<ASTNode>,
        when_clauses: Vec<(Box<ASTNode>, Vec<ASTNode>)>, // (condition, statements)
        default: Option<Vec<ASTNode>>,
        line: usize,
        column: usize,
    },
    FunctionDecl {
        name: String,
        params: Vec<Box<ASTNode>>, // Parameter nodes (ID or Assignment)
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    FunctionCall {
        name: String,
        args: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ReturnStatement {
        value: Option<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ClassDecl {
        name: String,
        inherit: Option<String>,
        body: Vec<ASTNode>,       // Class members (fields, methods, constructor)
        line: usize,
        column: usize,
    },
    MethodCall {
        object: Box<ASTNode>,     // ID or THIS
        method: String,
        args: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ImportStatement {
        modules: Vec<String>,     // STRING or ID
        line: usize,
        column: usize,
    },
    FromImportStatement {
        module: String,
        imported: Vec<String>,
        line: usize,
        column: usize,
    },
    RaiseException {
        error: Option<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    TryCapture {
        try_body: Vec<ASTNode>,
        capture_var: String,
        capture_body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    SkipStatement {
        line: usize,
        column: usize,
    },
    ExitStatement {
        line: usize,
        column: usize,
    },
    AwaitStatement {
        expr: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    FieldAccess {
        object: Box<ASTNode>,     // THIS or ID
        field: Box<ASTNode>,      // ID or ArrayAccess
        line: usize,
        column: usize,
    },
    ParentMethodAccess {
        method: String,
        args: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ParentAccess {
        field: Box<ASTNode>,      // ID or Assignment
        line: usize,
        column: usize,
    },
    Callback {
        name: String,
        params: Vec<Box<ASTNode>>, // Parameter nodes
        line: usize,
        column: usize,
    },

    // Expressions
    Ternary {
        condition: Box<ASTNode>,
        then_expr: Box<ASTNode>,
        else_expr: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    BinaryOperation {
        left: Box<ASTNode>,
        operator: TokenType,
        right: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    UnaryOperation {
        operator: TokenType,      // NOT
        expr: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    ArrayElement {
        elements: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ArrayAccess {
        name: String,
        indices: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    Dictionary {
        pairs: Vec<(Box<ASTNode>, Box<ASTNode>)>, // (key, value)
        line: usize,
        column: usize,
    },
    DictionaryAccess {
        name: String,
        keys: Vec<Box<ASTNode>>,  // STRING keys
        line: usize,
        column: usize,
    },
    ClassInstantiation {
        class_expr: Box<ASTNode>, // Changed from class_name: String
        args: Vec<Box<ASTNode>>,
        line: usize,
        column: usize,
    },
    ByteArray {
        args: Vec<Option<Box<ASTNode>>>, // Up to 3 optional expressions
        line: usize,
        column: usize,
    },
    NumberLiteral {
        value: f64,
        line: usize,
        column: usize,
    },
    StringLiteral {
        value: String,
        line: usize,
        column: usize,
    },
    HexLiteral {
        value: String,            // Raw hex string (e.g., "0x1A")
        line: usize,
        column: usize,
    },
    BytesLiteral {
        value: String,            // Raw binary string (e.g., "0b1010")
        line: usize,
        column: usize,
    },
    ScientificLiteral {
        value: f64,
        line: usize,
        column: usize,
    },
    Identifier {
        name: String,
        line: usize,
        column: usize,
    },
    True {
        line: usize,
        column: usize,
    },
    False {
        line: usize,
        column: usize,
    },
    Null {
        line: usize,
        column: usize,
    },

    // Class Members
    FieldDecl {
        modifier: Option<TokenType>, // PUBLIC or PRIVATE
        decl: Box<ASTNode>,          // Declaration or Assignment
        line: usize,
        column: usize,
    },
    MethodDecl {
        modifier: Option<TokenType>, // PUBLIC or PRIVATE
        name: String,
        params: Vec<Box<ASTNode>>,   // Parameter nodes
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ConstructorDecl {
        modifier: Option<TokenType>, // PUBLIC or PRIVATE
        params: Vec<Box<ASTNode>>,   // Parameter nodes
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    ParentConstructorCall { 
        args: Vec<Box<ASTNode>>, 
        line: usize, 
        column: usize 
    },
    This {
        line: usize,
        column: usize,
    },
    // Block
    Block {
        statements: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    EOF,
}

impl ASTNode {
    pub fn from_token(token: &Token, node: ASTNode) -> Self {
        match node {
            ASTNode::Declaration { names, .. } => ASTNode::Declaration {
                        names,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::SetStatement { target, value, .. } => ASTNode::SetStatement {
                        target,
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ParentConstructorCall { args, line, column } => ASTNode::ParentConstructorCall {
                        args,
                        line: line + token.line,
                        column: column + token.column,
                    },
            ASTNode::AssignStatement { target, value, .. } => ASTNode::AssignStatement {
                        target,
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::IfStatement { condition, then_body, elseif_clauses, else_body, .. } => {
                        ASTNode::IfStatement {
                            condition,
                            then_body,
                            elseif_clauses,
                            else_body,
                            line: token.line,
                            column: token.column,
                        }
                    }
            ASTNode::InputStatement { target, prompt, .. } => ASTNode::InputStatement {
                        target,
                        prompt,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ForStatement { variable, start, end, step, body, .. } => ASTNode::ForStatement {
                        variable,
                        start,
                        end,
                        step,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ForeachStatement { variables, iterable, body, .. } => ASTNode::ForeachStatement {
                        variables,
                        iterable,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::GenerateStatement { variable, start, end, by, body, .. } => {
                        ASTNode::GenerateStatement {
                            variable,
                            start,
                            end,
                            by,
                            body,
                            line: token.line,
                            column: token.column,
                        }
                    }
            ASTNode::ShowStatement { expr, .. } => ASTNode::ShowStatement {
                        expr,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::RepeatStatement { condition, body, .. } => ASTNode::RepeatStatement {
                        condition,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::RepeatTimeStatement { times, body, .. } => ASTNode::RepeatTimeStatement {
                        times,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ShowLnStatement { .. } => ASTNode::ShowLnStatement {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::IterateStatement { variable, iterable, body, .. } => ASTNode::IterateStatement {
                        variable,
                        iterable,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ChooseStatement { expr, when_clauses, default, .. } => ASTNode::ChooseStatement {
                        expr,
                        when_clauses,
                        default,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::FunctionDecl { name, params, body, .. } => ASTNode::FunctionDecl {
                        name,
                        params,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::FunctionCall { name, args, .. } => ASTNode::FunctionCall {
                        name,
                        args,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ReturnStatement { value, .. } => ASTNode::ReturnStatement {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ClassDecl { name, inherit, body, .. } => ASTNode::ClassDecl {
                        name,
                        inherit,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::MethodCall { object, method, args, .. } => ASTNode::MethodCall {
                        object,
                        method,
                        args,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ImportStatement { modules, .. } => ASTNode::ImportStatement {
                        modules,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::FromImportStatement { module, imported, .. } => ASTNode::FromImportStatement {
                        module,
                        imported,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::RaiseException { error, .. } => ASTNode::RaiseException {
                        error,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::TryCapture { try_body, capture_var, capture_body, .. } => ASTNode::TryCapture {
                        try_body,
                        capture_var,
                        capture_body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::SkipStatement { .. } => ASTNode::SkipStatement {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ExitStatement { .. } => ASTNode::ExitStatement {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::AwaitStatement { expr, .. } => ASTNode::AwaitStatement {
                        expr,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::FieldAccess { object, field, .. } => ASTNode::FieldAccess {
                        object,
                        field,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ParentMethodAccess { method, args, .. } => ASTNode::ParentMethodAccess {
                        method,
                        args,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ParentAccess { field, .. } => ASTNode::ParentAccess {
                        field,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Callback { name, params, .. } => ASTNode::Callback {
                        name,
                        params,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Ternary { condition, then_expr, else_expr, .. } => ASTNode::Ternary {
                        condition,
                        then_expr,
                        else_expr,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::BinaryOperation { left, operator, right, .. } => ASTNode::BinaryOperation {
                        left,
                        operator,
                        right,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::UnaryOperation { operator, expr, .. } => ASTNode::UnaryOperation {
                        operator,
                        expr,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ArrayElement { elements, .. } => ASTNode::ArrayElement {
                        elements,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ArrayAccess { name, indices, .. } => ASTNode::ArrayAccess {
                        name,
                        indices,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Dictionary { pairs, .. } => ASTNode::Dictionary {
                        pairs,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::DictionaryAccess { name, keys, .. } => ASTNode::DictionaryAccess {
                        name,
                        keys,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ClassInstantiation { class_expr, args, .. } => ASTNode::ClassInstantiation {
                        class_expr,
                        args,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ByteArray { args, .. } => ASTNode::ByteArray {
                        args,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::NumberLiteral { value, .. } => ASTNode::NumberLiteral {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::StringLiteral { value, .. } => ASTNode::StringLiteral {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::HexLiteral { value, .. } => ASTNode::HexLiteral {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::BytesLiteral { value, .. } => ASTNode::BytesLiteral {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ScientificLiteral { value, .. } => ASTNode::ScientificLiteral {
                        value,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Identifier { name, .. } => ASTNode::Identifier {
                        name,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::True { .. } => ASTNode::True {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::False { .. } => ASTNode::False {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Null { .. } => ASTNode::Null {
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::FieldDecl { modifier, decl, .. } => ASTNode::FieldDecl {
                        modifier,
                        decl,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::MethodDecl { modifier, name, params, body, .. } => ASTNode::MethodDecl {
                        modifier,
                        name,
                        params,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::ConstructorDecl { modifier, params, body, .. } => ASTNode::ConstructorDecl {
                        modifier,
                        params,
                        body,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::Block { statements, .. } => ASTNode::Block {
                        statements,
                        line: token.line,
                        column: token.column,
                    },
            ASTNode::EOF => ASTNode::EOF,
        ASTNode::This { line, column } => ASTNode::This { line: token.line, column: token.column },
        }
    }
}