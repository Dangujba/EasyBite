/****************************************************************************************
 * File: lexer.rs
 * Author: Muhammad Baba Goni
 * Email: <muhammadgoni51@gmail.com>
 * Date:  02/03/2025
 *
 * Description:
 * ------------
 * This file implements the Lexer (also known as the tokenizer) for the scripting language.
 *
 * The Lexer reads the raw source code character-by-character and breaks it into
 * meaningful tokens (identifiers, keywords, numbers, strings, operators, etc.)
 * according to the syntax rules.
 *
 * Responsibilities:
 * -----------------
 * - Scan the input source code.
 * - Recognize and categorize sequences of characters into `Token`s.
 * - Handle whitespace, comments, string literals, escape sequences, and error recovery.
 *
 * Usage:
 * ------
 * The `lexer` feeds a stream of `Token`s into the `parser` for syntax analysis.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use std::iter::Peekable;
use std::str::Chars;
use crate::token::{ Token, TokenType };

#[derive(Clone)]
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    current_line: usize,
    current_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source: source.chars().peekable(),
            current_line: 1,
            current_column: 0,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let next_char = self.source.next();
        if next_char.is_some() {
            self.current_column += 1;
        }
        next_char
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }

    pub fn peek_token(&mut self) -> Result<Option<Token>, String> {
        let mut cloned = Lexer {
            source: self.source.clone(),
            current_line: self.current_line,
            current_column: self.current_column,
        };
        if cloned.peek().is_some() {
            Ok(Some(cloned.next_token()?))
        } else {
            Ok(None)
        }
    }

    fn peek_next(&mut self) -> Option<char> {
        let mut iter = self.source.clone();
        iter.next(); // Skip current character
        iter.next()
    }

    fn skip_whitespace_and_newlines(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() && c != '\t' {
                self.advance();
                if c == '\n' {
                    self.current_line += 1;
                    self.current_column = 0;
                }
            } else {
                break;
            }
        }
    }

    fn skip_single_line_comment(&mut self) {
        while let Some(&c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_multi_line_comment(&mut self) -> Result<(), String> {
        self.advance(); // Consume '*'
        while let Some(c) = self.advance() {
            if c == '*' && self.peek() == Some(&'/') {
                self.advance(); // Consume '/'
                return Ok(());
            }
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 0;
            }
        }
        Err(format!("Unterminated multi-line comment at line {}", self.current_line))
    }

    fn read_identifier(&mut self, first_char: char) -> String {
        let mut identifier = String::new();
        identifier.push(first_char);

        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        identifier
    }

    fn read_number(&mut self, first_char: char) -> String {
        let mut number = String::new();
        number.push(first_char);

        let is_negative = first_char == '-';
        if is_negative && self.peek().map_or(false, |&c| c.is_numeric()) {
            number.push(self.advance().unwrap());
        }

        if number == "0" {
            if let Some(&'x') = self.peek() {
                number.push(self.advance().unwrap());
                while let Some(&c) = self.peek() {
                    if c.is_digit(16) {
                        number.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                return number;
            } else if let Some(&'b') = self.peek() {
                number.push(self.advance().unwrap());
                while let Some(&c) = self.peek() {
                    if c == '0' || c == '1' {
                        number.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                return number;
            }
        }

        let mut has_decimal = false;
        while let Some(&c) = self.peek() {
            if c.is_numeric() {
                number.push(self.advance().unwrap());
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                number.push(self.advance().unwrap());
            } else if (c == 'e' || c == 'E') && !number.ends_with('e') {
                number.push(self.advance().unwrap());
                if let Some(&'-') = self.peek() {
                    number.push(self.advance().unwrap());
                }
                while let Some(&c) = self.peek() {
                    if c.is_numeric() {
                        number.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                break;
            } else {
                break;
            }
        }
        number
    }

    fn read_string(&mut self, quote: char) -> Result<String, String> {
        let mut string_value = String::new();
        // Remove self.advance(); // Quote already consumed by next_token

        while let Some(&c) = self.peek() {
            if c == quote {
                self.advance(); // Consume closing quote

                return Ok(string_value);
            }
            match c {
                '\\' => {
                    self.advance(); // Skip backslash
                    if let Some(next_char) = self.advance() {
                        string_value.push(match next_char {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            _ => next_char,
                        });
                    } else {
                        return Err(format!("Unterminated string at line {}", self.current_line));
                    }
                }
                _ => {
                    let ch = self.advance().unwrap();

                    string_value.push(ch);
                }
            }
        }
        Err(format!("Unterminated string at line {}", self.current_line))
    }

    fn peek_word(&mut self) -> String {
        let mut iter = self.source.clone();
        let mut word = String::new();
        while let Some(&c) = iter.peek() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                iter.next();
            } else {
                break;
            }
        }
        word
    }

    fn read_multi_word_token(&mut self, first_word: &str) -> String {
        let mut token_string = first_word.to_string();
        self.skip_whitespace_and_newlines();

        if let Some(&c) = self.peek() {
            if c.is_alphabetic() {
                let second_word = self.peek_word();
                let combined = format!("{} {}", first_word, second_word);
                match combined.as_str() {
                    | "end if"
                    | "end repeat"
                    | "end choose"
                    | "end function"
                    | "end class"
                    | "end method"
                    | "end init"
                    | "end for"
                    | "end iterate"
                    | "else if"
                    | "end foreach"
                    | "is not"
                    | "is in" => {
                        self.advance(); // Move past the space
                        for _ in second_word.chars() {
                            self.advance();
                        }
                        return combined;
                    }
                    _ => {}
                }
            }
        }
        token_string
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_newlines();
        let start_column = self.current_column;

        if self.peek().is_none() {
            return Ok(
                Token::new(TokenType::EOF, "EOF".to_string(), self.current_line, start_column)
            );
        }

        if let Some(&'\t') = self.peek() {
            let mut indent = String::new();
            while let Some(&'\t') = self.peek() {
                indent.push(self.advance().unwrap());
            }
            return Ok(Token::new(TokenType::Indent, indent, self.current_line, start_column));
        }

        let current_char = self.advance().unwrap(); // Safe due to peek check
        match current_char {
            '/' =>
                match self.peek() {
                    Some(&'/') => {
                        self.advance(); // Consume second '/'
                        self.skip_single_line_comment();
                        self.next_token() // Skip comment and get next token
                    }
                    Some(&'*') => {
                        self.skip_multi_line_comment()?;
                        self.next_token() // Skip comment and get next token
                    }
                    _ =>
                        Ok(
                            Token::new(
                                TokenType::Divide,
                                "/".to_string(),
                                self.current_line,
                                start_column
                            )
                        ),
                }
            '+' =>
                Ok(Token::new(TokenType::Plus, "+".to_string(), self.current_line, start_column)),
            '-' => {
                if let Some(&c) = self.peek() {
                    if c.is_numeric() {
                        let number = self.read_number(current_char);
                        return Ok(
                            Token::new(
                                TokenType::Number(number.parse().unwrap_or(0.0)),
                                number,
                                self.current_line,
                                start_column
                            )
                        );
                    }
                }
                Ok(Token::new(TokenType::Minus, "-".to_string(), self.current_line, start_column))
            }
            '*' =>
                Ok(
                    Token::new(
                        TokenType::Multiply,
                        "*".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            '^' =>
                Ok(Token::new(TokenType::Power, "^".to_string(), self.current_line, start_column)),
            '(' =>
                Ok(
                    Token::new(
                        TokenType::LeftParen,
                        "(".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            ')' =>
                Ok(
                    Token::new(
                        TokenType::RightParen,
                        ")".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            '{' =>
                Ok(
                    Token::new(
                        TokenType::LeftBrace,
                        "{".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            '}' =>
                Ok(
                    Token::new(
                        TokenType::RightBrace,
                        "}".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            '[' =>
                Ok(
                    Token::new(
                        TokenType::LeftBracket,
                        "[".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            ']' =>
                Ok(
                    Token::new(
                        TokenType::RightBracket,
                        "]".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            ',' =>
                Ok(Token::new(TokenType::Comma, ",".to_string(), self.current_line, start_column)),
            ';' =>
                Ok(
                    Token::new(
                        TokenType::Semicolon,
                        ";".to_string(),
                        self.current_line,
                        start_column
                    )
                ),
            '.' => Ok(Token::new(TokenType::Dot, ".".to_string(), self.current_line, start_column)),
            ':' =>
                Ok(Token::new(TokenType::Colon, ":".to_string(), self.current_line, start_column)),
            '<' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(
                        Token::new(
                            TokenType::LessThanEqual,
                            "<=".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                } else {
                    Ok(
                        Token::new(
                            TokenType::LessThan,
                            "<".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                }
            }
            '>' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(
                        Token::new(
                            TokenType::GreaterThanEqual,
                            ">=".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                } else {
                    Ok(
                        Token::new(
                            TokenType::GreaterThan,
                            ">".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                }
            }
            '=' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(
                        Token::new(
                            TokenType::IsEqual,
                            "==".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                } else {
                    Err(
                        format!(
                            "Invalid standalone '=' at line {}, column {}",
                            self.current_line,
                            start_column
                        )
                    )
                }
            }
            '!' => {
                if let Some(&'=') = self.peek() {
                    self.advance();
                    Ok(
                        Token::new(
                            TokenType::NotEqual,
                            "!=".to_string(),
                            self.current_line,
                            start_column
                        )
                    )
                } else {
                    Ok(Token::new(TokenType::Not, "!".to_string(), self.current_line, start_column))
                }
            }
            '"' | '\'' => {
                let string_value = self.read_string(current_char)?;
                Ok(
                    Token::new(
                        TokenType::StringLiteral(string_value.clone()),
                        string_value, // Reflect the actual string value
                        self.current_line,
                        start_column
                    )
                )
            }
            _ if current_char.is_alphabetic() || current_char == '_' => {
                let identifier = self.read_identifier(current_char);
                // Added multi-word logic here:
                let full_identifier = self.read_multi_word_token(&identifier);
                let token_type = match full_identifier.as_str() {
                    "declare" => TokenType::Declare,
                    "set" => TokenType::Set,
                    "to" => TokenType::To,
                    "show" => TokenType::Show,
                    "showline" => TokenType::ShowLine,
                    "null" => TokenType::Null,
                    "input" => TokenType::Input,
                    "generate" => TokenType::Generate,
                    "stop" => TokenType::Stop,
                    "exit" => TokenType::Exit,
                    "skip" => TokenType::Skip,
                    "import" => TokenType::Import,
                    "is" => TokenType::Is,
                    "as" => TokenType::As,
                    "inc" => TokenType::Inc,
                    "dec" => TokenType::Dec,
                    "new" => TokenType::New,
                    "parent" => TokenType::Parent,
                    "await" => TokenType::Await,
                    "async" => TokenType::Async,
                    "error" => TokenType::Error,
                    "raise" => TokenType::Raise,
                    "bytearray" => TokenType::ByteArray,
                    "callback" => TokenType::Callback,
                    "if" => TokenType::If,
                    "then" => TokenType::Then,
                    "else" => TokenType::Else,
                    "else if" => TokenType::ElseIf,
                    "end if" => TokenType::EndIf,
                    "choose" => TokenType::Choose,
                    "when" => TokenType::When,
                    "otherwise" => TokenType::Otherwise,
                    "end choose" => TokenType::EndChoose,
                    "try" => TokenType::Try,
                    "capture" => TokenType::Capture,
                    "while" => TokenType::While,
                    "repeat" => TokenType::Repeat,
                    "times" => TokenType::Times,
                    "end repeat" => TokenType::EndRepeat,
                    "for" => TokenType::For,
                    "foreach" => TokenType::Foreach,
                    "end foreach" => TokenType::EndForeach,
                    "from" => TokenType::From,
                    "step" => TokenType::Step,
                    "by" => TokenType::By,
                    "end for" => TokenType::EndFor,
                    "iterate" => TokenType::Iterate,
                    "in" => TokenType::In,
                    "over" => TokenType::Over,
                    "end iterate" => TokenType::EndIterate,
                    "function" => TokenType::Function,
                    "return" => TokenType::Return,
                    "end function" => TokenType::EndFunction,
                    "class" => TokenType::Class,
                    "inherit" => TokenType::Inherit,
                    "end class" => TokenType::EndClass,
                    "method" => TokenType::Method,
                    "end method" => TokenType::EndMethod,
                    "init" => TokenType::Constructor,
                    "end init" => TokenType::EndInit,
                    "this" => TokenType::This,
                    "secret" => TokenType::Private,
                    "public" => TokenType::Public,
                    "and" => TokenType::And,
                    "or" => TokenType::Or,
                    "not" => TokenType::Not,
                    "true" => TokenType::True,
                    "false" => TokenType::False,
                    "remind" => TokenType::Modulus,
                    "is in" => TokenType::IsIn,
                    "is not" => TokenType::IsNot,
                    _ => TokenType::Identifier(full_identifier.clone()),
                };
                Ok(Token::new(token_type, full_identifier, self.current_line, start_column))
            }
            _ if current_char.is_numeric() => {
                let number = self.read_number(current_char);
                Ok(
                    Token::new(
                        TokenType::Number(number.parse().unwrap_or(0.0)),
                        number,
                        self.current_line,
                        start_column
                    )
                )
            }
            _ =>
                Err(
                    format!(
                        "Unknown character '{}' at line {}, column {}",
                        current_char,
                        self.current_line,
                        start_column
                    )
                ),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => {
                    tokens.push(token.clone());
                    if token.token_type == TokenType::EOF {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(tokens)
    }
}
