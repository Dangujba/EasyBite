/****************************************************************************************
 * File: parser.rs
 * Author: Muhammad Baba Goni
 * Email: <muhammadgoni51@gmail.com>
 * Date:  02/03/2025
 *
 * Description:
 * ------------
 * This file implements the Parser for the scripting language.
 *
 * The Parser receives a stream of tokens from the Lexer and constructs an
 * Abstract Syntax Tree (AST) representing the program's syntactic structure.
 *
 * Responsibilities:
 * -----------------
 * - Parse expressions, statements, blocks, functions, and control flow constructs.
 * - Validate syntactical correctness according to the language grammar.
 * - Produce a navigable AST for the Interpreter to execute.
 * - Handle parsing errors gracefully with meaningful messages.
 *
 * Usage:
 * ------
 * The `parser` transforms a list of `Token`s into an AST composed of `ASTNode`s.
 *
 * License:
 * --------
 * MIT License or similar permissive licensing.
 ****************************************************************************************/

use crate::lexer::Lexer;
use crate::token::{ Token, TokenType };
use crate::astnode::ASTNode;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(source);
        let first_token = lexer.next_token()?;
        Ok(Parser {
            lexer,
            current_token: first_token,
        })
    }

    fn advance(&mut self) -> Result<(), String> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: TokenType) -> Result<(), String> {
        if self.current_token.token_type == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(
                format!(
                    "Expected {:?}, found {:?} at line {}, column {}",
                    expected,
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            )
        }
    }

    // Peek at the current token's type
    fn peek(&self) -> &TokenType {
        &self.current_token.token_type
    }

    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        match self.current_token.token_type {
            TokenType::Declare => self.parse_declaration(),
            TokenType::Set => self.parse_set_statement(),
            TokenType::Identifier(_) => {
                let mut lexer_clone = self.lexer.clone();
                let mut next_token = lexer_clone.next_token()?;
                let mut is_assignment = false;
                if next_token.token_type == TokenType::To {
                    is_assignment = true;
                } else if next_token.token_type == TokenType::LeftBracket {
                    while next_token.token_type == TokenType::LeftBracket {
                        next_token = lexer_clone.next_token()?;
                        while next_token.token_type != TokenType::RightBracket {
                            next_token = lexer_clone.next_token()?;
                        }
                        next_token = lexer_clone.next_token()?;
                    }
                    if next_token.token_type == TokenType::To {
                        is_assignment = true;
                    }
                }
                if is_assignment {
                    self.parse_assign_statement()
                } else {
                    self.parse_expression()
                }
            }
            TokenType::If => self.parse_if_statement(),
            TokenType::Input => self.parse_input_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Foreach => self.parse_foreach_statement(),
            TokenType::Generate => self.parse_generate_statement(),
            TokenType::Show => self.parse_show_statement(),
            TokenType::Repeat => {
                if self.lexer.peek_token()?.map(|t| t.token_type) == Some(TokenType::While) {
                    self.parse_repeat_statement()
                } else {
                    self.parse_repeat_time_statement()
                }
            }
            TokenType::ShowLine => self.parse_showln_statement(),
            TokenType::Iterate => self.parse_iterate_statement(),
            TokenType::Choose => self.parse_choose_statement(),
            TokenType::Function => self.parse_function_decl(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Class => self.parse_class_decl(),
            TokenType::Import => self.parse_import_statement(),
            TokenType::From => self.parse_from_import_statement(),
            TokenType::Raise => self.parse_raise_exception(),
            TokenType::Try => self.parse_try_capture(),
            TokenType::Skip => self.parse_skip_statement(),
            TokenType::Exit => self.parse_exit_statement(),
            TokenType::Await => self.parse_await_statement(),
            TokenType::This => {
                let field_access = self.parse_field_access()?;
                if self.current_token.token_type == TokenType::To {
                    self.advance()?; // Consume 'to'
                    let value = Box::new(self.parse_expression()?);
                    Ok(ASTNode::AssignStatement {
                        target: Box::new(field_access),
                        value,
                        line: 0,
                        column: 0,
                    })
                } else {
                    Ok(field_access) // Treat as an expression statement if not followed by 'to'
                }
            }
            TokenType::Parent => {
                if self.lexer.peek_token()?.map(|t| t.token_type) == Some(TokenType::Dot) {
                    self.advance()?; // Consume 'parent'
                    self.advance()?; // Consume '.'
                    match &self.current_token.token_type {
                        TokenType::Identifier(method_name) if
                            method_name == "init" &&
                            self.lexer.peek_token()?.map(|t| t.token_type) ==
                                Some(TokenType::LeftParen)
                        => {
                            let parent_token = self.current_token.clone();
                            self.advance()?;
                            self.expect(TokenType::LeftParen)?;
                            let args = self.parse_arg_list()?;
                            self.expect(TokenType::RightParen)?;
                            Ok(
                                ASTNode::from_token(&parent_token, ASTNode::ParentConstructorCall {
                                    args,
                                    line: parent_token.line,
                                    column: parent_token.column,
                                })
                            )
                        }
                        TokenType::Constructor if
                            self.lexer.peek_token()?.map(|t| t.token_type) ==
                            Some(TokenType::LeftParen)
                        => {
                            let parent_token = self.current_token.clone();
                            self.advance()?;
                            self.expect(TokenType::LeftParen)?;
                            let args = self.parse_arg_list()?;
                            self.expect(TokenType::RightParen)?;
                            Ok(
                                ASTNode::from_token(&parent_token, ASTNode::ParentConstructorCall {
                                    args,
                                    line: parent_token.line,
                                    column: parent_token.column,
                                })
                            )
                        }
                        TokenType::Identifier(_) if
                            self.lexer.peek_token()?.map(|t| t.token_type) ==
                            Some(TokenType::LeftParen)
                        => {
                            self.parse_parent_method_access()
                        }
                        _ => self.parse_parent_access(),
                    }
                } else {
                    Err(
                        format!(
                            "Expected '.' after 'parent' at line {}, column {}",
                            self.current_token.line,
                            self.current_token.column
                        )
                    )
                }
            }
            TokenType::Callback => self.parse_callback(),
            TokenType::EOF => Ok(ASTNode::EOF),
            _ =>
                Err(
                    format!(
                        "Unexpected token {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                ),
        }
    }

    fn parse_declaration(&mut self) -> Result<ASTNode, String> {
        let declare_token = self.current_token.clone();
        self.advance()?;
        let mut names = Vec::new();

        loop {
            names.push(self.parse_target()?);
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            self.advance()?;
        }

        Ok(ASTNode::from_token(&declare_token, ASTNode::Declaration { names, line: 0, column: 0 }))
    }

    fn parse_set_statement(&mut self) -> Result<ASTNode, String> {
        let set_token = self.current_token.clone();
        self.advance()?;
        let target = self.parse_target()?;
        self.expect(TokenType::To)?;
        let value = Box::new(self.parse_expression()?);
        Ok(
            ASTNode::from_token(&set_token, ASTNode::SetStatement {
                target,
                value,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_assign_statement(&mut self) -> Result<ASTNode, String> {
        let assign_token = self.current_token.clone();
        let target = self.parse_target()?;
        self.expect(TokenType::To)?;
        let value = Box::new(self.parse_expression()?);
        Ok(
            ASTNode::from_token(&assign_token, ASTNode::AssignStatement {
                target,
                value,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_if_statement(&mut self) -> Result<ASTNode, String> {
        let if_token = self.current_token.clone();
        self.advance()?;
        let condition = Box::new(self.parse_expression()?);
        self.expect(TokenType::Then)?;
        let then_body = self.parse_statement_list(TokenType::EndIf)?;
        let mut elseif_clauses = Vec::new();

        while self.current_token.token_type == TokenType::ElseIf {
            self.advance()?;
            let elseif_condition = Box::new(self.parse_expression()?);
            self.expect(TokenType::Then)?;
            let elseif_body = self.parse_statement_list(TokenType::EndIf)?;
            elseif_clauses.push((elseif_condition, elseif_body));
        }

        let else_body = if self.current_token.token_type == TokenType::Else {
            self.advance()?;
            Some(self.parse_statement_list(TokenType::EndIf)?)
        } else {
            None
        };

        self.expect(TokenType::EndIf)?;
        Ok(
            ASTNode::from_token(&if_token, ASTNode::IfStatement {
                condition,
                then_body,
                elseif_clauses,
                else_body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_input_statement(&mut self) -> Result<ASTNode, String> {
        let input_token = self.current_token.clone();
        self.advance()?;
        let target = self.parse_target()?;
        self.expect(TokenType::To)?;
        self.expect(TokenType::Input)?;
        self.expect(TokenType::LeftParen)?;
        let prompt = Box::new(self.parse_expression()?);
        self.expect(TokenType::RightParen)?;
        Ok(
            ASTNode::from_token(&input_token, ASTNode::InputStatement {
                target,
                prompt,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_for_statement(&mut self) -> Result<ASTNode, String> {
        let for_token = self.current_token.clone();
        self.advance()?;
        let variable = if let TokenType::Identifier(name) = &self.current_token.token_type {
            let name = name.clone();
            self.advance()?;
            name
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::From)?;
        let start = Box::new(self.parse_expression()?);
        self.expect(TokenType::To)?;
        let end = Box::new(self.parse_expression()?);
        let step = if self.current_token.token_type == TokenType::Step {
            self.advance()?;
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        let body = self.parse_statement_list(TokenType::EndFor)?;
        self.expect(TokenType::EndFor)?;
        Ok(
            ASTNode::from_token(&for_token, ASTNode::ForStatement {
                variable,
                start,
                end,
                step,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_foreach_statement(&mut self) -> Result<ASTNode, String> {
        let foreach_token = self.current_token.clone();
        self.advance()?; // Consume 'foreach'

        // Check if variables are enclosed in parentheses
        let mut variables = Vec::new();
        let has_parens = self.current_token.token_type == TokenType::LeftParen;
        if has_parens {
            self.advance()?; // Consume '('
        }

        // Parse variable list
        loop {
            if let TokenType::Identifier(name) = &self.current_token.token_type {
                variables.push(name.clone());
                self.advance()?;
            } else {
                return Err(
                    format!(
                        "Expected identifier, found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }

            // Check if we're done with variables
            if has_parens && self.current_token.token_type == TokenType::RightParen {
                self.advance()?;
                break;
            }
            if !has_parens && self.current_token.token_type == TokenType::In {
                break;
            }
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            self.advance()?; // Consume ','
        }

        // Expect 'in'
        if self.current_token.token_type != TokenType::In {
            return Err(
                format!(
                    "Expected 'in', found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        }
        self.advance()?; // Consume 'in'

        // Parse iterable expression
        let iterable = Box::new(self.parse_expression()?);

        // Parse body until 'end foreach'
        let body = self.parse_statement_list(TokenType::EndForeach)?;
        self.expect(TokenType::EndForeach)?;

        Ok(
            ASTNode::from_token(&foreach_token, ASTNode::ForeachStatement {
                variables,
                iterable,
                body,
                line: foreach_token.line,
                column: foreach_token.column,
            })
        )
    }

    fn parse_generate_statement(&mut self) -> Result<ASTNode, String> {
        let gen_token = self.current_token.clone();
        self.advance()?;
        let variable = if let TokenType::Identifier(name) = &self.current_token.token_type {
            let name = name.clone();
            self.advance()?;
            name
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::From)?;
        let start = Box::new(self.parse_expression()?);
        self.expect(TokenType::To)?;
        let end = Box::new(self.parse_expression()?);
        let by = if self.current_token.token_type == TokenType::By {
            self.advance()?;
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        let body = self.parse_statement_list(TokenType::Stop)?;
        self.expect(TokenType::Stop)?;
        Ok(
            ASTNode::from_token(&gen_token, ASTNode::GenerateStatement {
                variable,
                start,
                end,
                by,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_show_statement(&mut self) -> Result<ASTNode, String> {
        let show_token = self.current_token.clone();
        self.advance()?;
        let expr = Box::new(self.parse_expression()?);
        Ok(ASTNode::from_token(&show_token, ASTNode::ShowStatement { expr, line: 0, column: 0 }))
    }

    fn parse_repeat_statement(&mut self) -> Result<ASTNode, String> {
        let repeat_token = self.current_token.clone();
        self.advance()?;
        self.expect(TokenType::While)?;
        self.expect(TokenType::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect(TokenType::RightParen)?;
        let body = self.parse_statement_list(TokenType::EndRepeat)?;
        self.expect(TokenType::EndRepeat)?;
        Ok(
            ASTNode::from_token(&repeat_token, ASTNode::RepeatStatement {
                condition,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_repeat_time_statement(&mut self) -> Result<ASTNode, String> {
        let repeat_token = self.current_token.clone();
        self.advance()?;
        let times = Box::new(self.parse_expression()?);
        self.expect(TokenType::Times)?;
        let body = self.parse_statement_list(TokenType::EndRepeat)?;
        self.expect(TokenType::EndRepeat)?;
        Ok(
            ASTNode::from_token(&repeat_token, ASTNode::RepeatTimeStatement {
                times,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_showln_statement(&mut self) -> Result<ASTNode, String> {
        let showln_token = self.current_token.clone();
        self.advance()?;
        self.expect(TokenType::LeftParen)?;
        self.expect(TokenType::RightParen)?;
        Ok(ASTNode::from_token(&showln_token, ASTNode::ShowLnStatement { line: 0, column: 0 }))
    }

    fn parse_iterate_statement(&mut self) -> Result<ASTNode, String> {
        let iterate_token = self.current_token.clone();
        self.advance()?;
        let variable = if let TokenType::Identifier(name) = &self.current_token.token_type {
            let name = name.clone();
            self.advance()?;
            name
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::Over)?;
        self.expect(TokenType::LeftParen)?;
        let iterable = Box::new(self.parse_expression()?);
        self.expect(TokenType::RightParen)?;
        let body = self.parse_statement_list(TokenType::EndIterate)?;
        self.expect(TokenType::EndIterate)?;
        Ok(
            ASTNode::from_token(&iterate_token, ASTNode::IterateStatement {
                variable,
                iterable,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_choose_statement(&mut self) -> Result<ASTNode, String> {
        let choose_token = self.current_token.clone();
        self.advance()?;
        let expr = Box::new(self.parse_expression()?);
        let mut when_clauses = Vec::new();

        while self.current_token.token_type == TokenType::When {
            self.advance()?;
            let condition = Box::new(self.parse_expression()?);
            self.expect(TokenType::Colon)?;
            let statements = self.parse_statement_list(TokenType::EndChoose)?;
            when_clauses.push((condition, statements));
        }

        let default = if self.current_token.token_type == TokenType::Otherwise {
            self.advance()?;
            self.expect(TokenType::Colon)?; // Consume the colon after 'otherwise'
            Some(self.parse_statement_list(TokenType::EndChoose)?)
        } else {
            None
        };

        self.expect(TokenType::EndChoose)?;
        Ok(
            ASTNode::from_token(&choose_token, ASTNode::ChooseStatement {
                expr,
                when_clauses,
                default,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_function_decl(&mut self) -> Result<ASTNode, String> {
        let func_token = self.current_token.clone();
        self.advance()?;
        let name = if let TokenType::Identifier(n) = &self.current_token.token_type {
            let n = n.clone();
            self.advance()?;
            n
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::LeftParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenType::RightParen)?;
        let body = self.parse_statement_list(TokenType::EndFunction)?;
        self.expect(TokenType::EndFunction)?;
        Ok(
            ASTNode::from_token(&func_token, ASTNode::FunctionDecl {
                name,
                params,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_function_call(&mut self) -> Result<ASTNode, String> {
        let call_token = self.current_token.clone();
        let name = if let TokenType::Identifier(n) = &call_token.token_type {
            n.clone()
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    call_token.token_type,
                    call_token.line,
                    call_token.column
                )
            );
        };
        self.advance()?;
        self.expect(TokenType::LeftParen)?;
        let args = self.parse_arg_list()?;
        self.expect(TokenType::RightParen)?;
        Ok(ASTNode::FunctionCall { name, args, line: call_token.line, column: call_token.column })
    }

    fn parse_return_statement(&mut self) -> Result<ASTNode, String> {
        let return_token = self.current_token.clone();
        self.advance()?;
        let value = if
            matches!(self.current_token.token_type, TokenType::Newline | TokenType::Semicolon)
        {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        Ok(
            ASTNode::from_token(&return_token, ASTNode::ReturnStatement {
                value,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_class_decl(&mut self) -> Result<ASTNode, String> {
        let class_token = self.current_token.clone();
        self.advance()?;
        let name = if let TokenType::Identifier(n) = &self.current_token.token_type {
            let n = n.clone();
            self.advance()?;
            n
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        let inherit = if self.current_token.token_type == TokenType::Inherit {
            self.advance()?;
            if let TokenType::Identifier(n) = &self.current_token.token_type {
                let n = n.clone();
                self.advance()?;
                Some(n)
            } else {
                return Err(
                    format!(
                        "Expected identifier after 'inherit', found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
        } else {
            None
        };
        let body = self.parse_class_body()?;
        self.expect(TokenType::EndClass)?;
        Ok(
            ASTNode::from_token(&class_token, ASTNode::ClassDecl {
                name,
                inherit,
                body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_import_statement(&mut self) -> Result<ASTNode, String> {
        let import_token = self.current_token.clone();
        self.advance()?;
        let mut modules = Vec::new();

        loop {
            match &self.current_token.token_type {
                TokenType::StringLiteral(s) | TokenType::Identifier(s) => {
                    modules.push(s.clone()); // Single module path with dots
                    self.advance()?;
                }
                _ => {
                    return Err(
                        format!(
                            "Expected string or identifier, found {:?} at line {}, column {}",
                            self.current_token.token_type,
                            self.current_token.line,
                            self.current_token.column
                        )
                    );
                }
            }
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            self.advance()?;
        }

        Ok(
            ASTNode::from_token(&import_token, ASTNode::ImportStatement {
                modules,
                line: import_token.line,
                column: import_token.column,
            })
        )
    }

    fn parse_from_import_statement(&mut self) -> Result<ASTNode, String> {
        let from_token = self.current_token.clone();
        self.advance()?;

        // Parse the module path (e.g., FirstFolder.subfolder.mainfile)
        let module = match &self.current_token.token_type {
            TokenType::Identifier(m) | TokenType::StringLiteral(m) => {
                let module_path = m.clone();
                self.advance()?;
                module_path
            }
            _ => {
                return Err(
                    format!(
                        "Expected identifier or string, found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
        };

        self.expect(TokenType::Import)?;

        let mut imported = Vec::new();
        loop {
            if let TokenType::Identifier(i) = &self.current_token.token_type {
                imported.push(i.clone());
                self.advance()?;
            } else {
                return Err(
                    format!(
                        "Expected identifier, found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            self.advance()?;
        }

        Ok(
            ASTNode::from_token(&from_token, ASTNode::FromImportStatement {
                module,
                imported,
                line: from_token.line,
                column: from_token.column,
            })
        )
    }

    fn parse_raise_exception(&mut self) -> Result<ASTNode, String> {
        let raise_token = self.current_token.clone();
        self.advance()?;
        self.expect(TokenType::Error)?;
        self.expect(TokenType::LeftParen)?;
        let error = if self.current_token.token_type != TokenType::RightParen {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect(TokenType::RightParen)?;
        Ok(ASTNode::from_token(&raise_token, ASTNode::RaiseException { error, line: 0, column: 0 }))
    }

    fn parse_try_capture(&mut self) -> Result<ASTNode, String> {
        let try_token = self.current_token.clone();
        self.advance()?;
        let try_body = self.parse_statement_list(TokenType::Capture)?;
        self.expect(TokenType::Capture)?;
        self.expect(TokenType::LeftParen)?;
        let capture_var = if let TokenType::Identifier(v) = &self.current_token.token_type {
            let v = v.clone();
            self.advance()?;
            v
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::RightParen)?;
        let capture_body = self.parse_statement_list(TokenType::Stop)?;
        self.expect(TokenType::Stop)?;
        Ok(
            ASTNode::from_token(&try_token, ASTNode::TryCapture {
                try_body,
                capture_var,
                capture_body,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_skip_statement(&mut self) -> Result<ASTNode, String> {
        let skip_token = self.current_token.clone();
        self.advance()?;
        Ok(ASTNode::from_token(&skip_token, ASTNode::SkipStatement { line: 0, column: 0 }))
    }

    fn parse_exit_statement(&mut self) -> Result<ASTNode, String> {
        let exit_token = self.current_token.clone();
        self.advance()?;
        Ok(ASTNode::from_token(&exit_token, ASTNode::ExitStatement { line: 0, column: 0 }))
    }

    fn parse_await_statement(&mut self) -> Result<ASTNode, String> {
        let await_token = self.current_token.clone();
        self.advance()?;
        let expr = Box::new(self.parse_expression()?);
        Ok(ASTNode::from_token(&await_token, ASTNode::AwaitStatement { expr, line: 0, column: 0 }))
    }

    fn parse_field_access(&mut self) -> Result<ASTNode, String> {
        let field_token = self.current_token.clone();
        let object = if self.current_token.token_type == TokenType::This {
            self.advance()?;
            Box::new(
                ASTNode::from_token(&field_token, ASTNode::Identifier {
                    name: "this".to_string(),
                    line: field_token.line,
                    column: field_token.column,
                })
            )
        } else {
            self.parse_target()?
        };
        self.expect(TokenType::Dot)?;
        let field = self.parse_target()?;

        // Check for method call after field access
        if self.current_token.token_type == TokenType::LeftParen {
            if let ASTNode::Identifier { name, .. } = &*field {
                self.advance()?; // Consume '('
                let args = self.parse_arg_list()?;
                self.expect(TokenType::RightParen)?;
                return Ok(
                    ASTNode::from_token(&field_token, ASTNode::MethodCall {
                        object,
                        method: name.clone(),
                        args,
                        line: field_token.line,
                        column: field_token.column,
                    })
                );
            } else {
                return Err(
                    format!(
                        "Expected identifier for method name after '.' at line {}, column {}",
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
        }

        // If no parentheses, it’s a plain field access
        Ok(
            ASTNode::from_token(&field_token, ASTNode::FieldAccess {
                object,
                field,
                line: field_token.line,
                column: field_token.column,
            })
        )
    }
    fn parse_parent_method_access(&mut self) -> Result<ASTNode, String> {
        let parent_token = self.current_token.clone();
        let method = if let TokenType::Identifier(m) = &self.current_token.token_type {
            let m = m.clone();
            self.advance()?;
            m
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::LeftParen)?;
        let args = self.parse_arg_list()?;
        self.expect(TokenType::RightParen)?;
        Ok(
            ASTNode::from_token(&parent_token, ASTNode::ParentMethodAccess {
                method,
                args,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_parent_access(&mut self) -> Result<ASTNode, String> {
        let parent_token = self.current_token.clone();
        let field = self.parse_target()?;
        Ok(ASTNode::from_token(&parent_token, ASTNode::ParentAccess { field, line: 0, column: 0 }))
    }

    fn parse_callback(&mut self) -> Result<ASTNode, String> {
        let callback_token = self.current_token.clone();
        self.advance()?;
        let name = if let TokenType::Identifier(n) = &self.current_token.token_type {
            let n = n.clone();
            self.advance()?;
            n
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };
        self.expect(TokenType::LeftParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenType::RightParen)?;
        Ok(
            ASTNode::from_token(&callback_token, ASTNode::Callback {
                name,
                params,
                line: 0,
                column: 0,
            })
        )
    }

    fn parse_target(&mut self) -> Result<Box<ASTNode>, String> {
        if let TokenType::Identifier(name) = &self.current_token.token_type {
            let name = name.clone();
            let target_token = self.current_token.clone();
            self.advance()?;

            if self.current_token.token_type == TokenType::LeftBracket {
                let mut array_indices: Vec<Box<ASTNode>> = Vec::new();
                let mut dict_keys: Vec<Box<ASTNode>> = Vec::new();

                while self.current_token.token_type == TokenType::LeftBracket {
                    self.advance()?; // consume '['
                    let expr = Box::new(self.parse_expression()?);
                    self.expect(TokenType::RightBracket)?; // consume ']'

                    // Determine the kind of index based on the AST variant.
                    // Here we assume that a string literal indicates dictionary access.
                    match *expr {
                        ASTNode::StringLiteral { .. } => dict_keys.push(expr),
                        _ => array_indices.push(expr),
                    }
                }

                // If only dictionary keys are found, it's dictionary access.
                if !dict_keys.is_empty() && array_indices.is_empty() {
                    Ok(
                        Box::new(
                            ASTNode::from_token(&target_token, ASTNode::DictionaryAccess {
                                name,
                                keys: dict_keys,
                                line: target_token.line,
                                column: target_token.column,
                            })
                        )
                    )
                } else if
                    // If only array indices are found, it's array access.
                    !array_indices.is_empty() &&
                    dict_keys.is_empty()
                {
                    Ok(
                        Box::new(
                            ASTNode::from_token(&target_token, ASTNode::ArrayAccess {
                                name,
                                indices: array_indices,
                                line: target_token.line,
                                column: target_token.column,
                            })
                        )
                    )
                } else if
                    // If a mix is found, you can choose to signal an error or decide on a specific behavior.
                    !array_indices.is_empty() ||
                    !dict_keys.is_empty()
                {
                    Err("Mixed dictionary and array access is not supported".to_string())
                } else {
                    // No indices; return simple identifier.
                    Ok(
                        Box::new(
                            ASTNode::from_token(&target_token, ASTNode::Identifier {
                                name,
                                line: target_token.line,
                                column: target_token.column,
                            })
                        )
                    )
                }
            } else {
                Ok(
                    Box::new(
                        ASTNode::from_token(&target_token, ASTNode::Identifier {
                            name,
                            line: target_token.line,
                            column: target_token.column,
                        })
                    )
                )
            }
        } else {
            Err(
                format!(
                    "Expected identifier or index access, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            )
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<Box<ASTNode>>, String> {
        let mut params = Vec::new();
        if self.current_token.token_type != TokenType::RightParen {
            loop {
                if let TokenType::Identifier(name) = &self.current_token.token_type {
                    let param_token = self.current_token.clone();
                    let name = name.clone();
                    self.advance()?;
                    if self.current_token.token_type == TokenType::To {
                        self.advance()?;
                        let value = Box::new(self.parse_expression()?);
                        params.push(
                            Box::new(
                                ASTNode::from_token(&param_token, ASTNode::AssignStatement {
                                    target: Box::new(ASTNode::Identifier {
                                        name,
                                        line: param_token.line,
                                        column: param_token.column,
                                    }),
                                    value,
                                    line: 0,
                                    column: 0,
                                })
                            )
                        );
                    } else {
                        params.push(
                            Box::new(
                                ASTNode::from_token(&param_token, ASTNode::Identifier {
                                    name,
                                    line: 0,
                                    column: 0,
                                })
                            )
                        );
                    }
                } else {
                    return Err(
                        format!(
                            "Expected identifier in parameter list, found {:?} at line {}, column {}",
                            self.current_token.token_type,
                            self.current_token.line,
                            self.current_token.column
                        )
                    );
                }
                if self.current_token.token_type != TokenType::Comma {
                    break;
                }
                self.advance()?;
            }
        }
        Ok(params)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Box<ASTNode>>, String> {
        let mut args = Vec::new();
        if self.current_token.token_type != TokenType::RightParen {
            loop {
                args.push(Box::new(self.parse_expression()?));
                if self.current_token.token_type != TokenType::Comma {
                    break;
                }
                self.advance()?;
            }
        }
        Ok(args)
    }

    fn parse_class_body(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut members = Vec::new();
        while !matches!(self.current_token.token_type, TokenType::EndClass | TokenType::EOF) {
            let modifier = if
                matches!(self.current_token.token_type, TokenType::Public | TokenType::Private)
            {
                let m = self.current_token.token_type.clone();
                self.advance()?;
                Some(m)
            } else {
                None
            };

            match self.current_token.token_type {
                TokenType::Declare => {
                    let decl = self.parse_declaration()?;
                    members.push(
                        ASTNode::from_token(&self.current_token, ASTNode::FieldDecl {
                            modifier,
                            decl: Box::new(decl),
                            line: 0,
                            column: 0,
                        })
                    );
                }
                TokenType::Set => {
                    let assign = self.parse_set_statement()?;
                    members.push(
                        ASTNode::from_token(&self.current_token, ASTNode::FieldDecl {
                            modifier,
                            decl: Box::new(assign),
                            line: 0,
                            column: 0,
                        })
                    );
                }
                TokenType::Identifier(_) => {
                    if self.lexer.peek_token()?.map(|t| t.token_type) == Some(TokenType::To) {
                        let assign = self.parse_assign_statement()?;
                        members.push(
                            ASTNode::from_token(&self.current_token, ASTNode::FieldDecl {
                                modifier,
                                decl: Box::new(assign),
                                line: 0,
                                column: 0,
                            })
                        );
                    } else {
                        return Err(
                            format!(
                                "Unexpected identifier in class body: {:?} at line {}, column {}",
                                self.current_token.token_type,
                                self.current_token.line,
                                self.current_token.column
                            )
                        );
                    }
                }
                TokenType::Method => {
                    self.advance()?;
                    let name = if let TokenType::Identifier(n) = &self.current_token.token_type {
                        let n = n.clone();
                        self.advance()?;
                        n
                    } else {
                        return Err(
                            format!(
                                "Expected identifier, found {:?} at line {}, column {}",
                                self.current_token.token_type,
                                self.current_token.line,
                                self.current_token.column
                            )
                        );
                    };
                    self.expect(TokenType::LeftParen)?;
                    let params = self.parse_param_list()?;
                    self.expect(TokenType::RightParen)?;
                    let body = self.parse_statement_list(TokenType::EndMethod)?;
                    self.expect(TokenType::EndMethod)?;
                    members.push(
                        ASTNode::from_token(&self.current_token, ASTNode::MethodDecl {
                            modifier,
                            name,
                            params,
                            body,
                            line: 0,
                            column: 0,
                        })
                    );
                }
                TokenType::Constructor => {
                    self.advance()?;
                    self.expect(TokenType::LeftParen)?;
                    let params = self.parse_param_list()?;
                    self.expect(TokenType::RightParen)?;
                    let body = self.parse_statement_list(TokenType::EndInit)?;
                    self.expect(TokenType::EndInit)?;
                    members.push(
                        ASTNode::from_token(&self.current_token, ASTNode::ConstructorDecl {
                            modifier,
                            params,
                            body,
                            line: 0,
                            column: 0,
                        })
                    );
                }
                _ => {
                    return Err(
                        format!(
                            "Unexpected token in class body: {:?} at line {}, column {}",
                            self.current_token.token_type,
                            self.current_token.line,
                            self.current_token.column
                        )
                    );
                }
            }
        }
        Ok(members)
    }

    fn parse_statement_list(&mut self, terminator: TokenType) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();
        while
            !matches!(&self.current_token.token_type, t if t == &terminator 
            || matches!(t, TokenType::When | TokenType::Otherwise | TokenType::Else | TokenType::ElseIf | TokenType::EOF))
        {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        self.parse_binary_operation(0)
    }

    fn parse_binary_operation(&mut self, min_precedence: u8) -> Result<ASTNode, String> {
        let mut left = self.parse_primary()?;

        while let Some(op_precedence) = self.get_precedence(&self.current_token.token_type) {
            if op_precedence < min_precedence {
                break;
            }
            let operator = self.current_token.token_type.clone();
            self.advance()?;
            let right = self.parse_binary_operation(op_precedence + 1)?;
            left = ASTNode::from_token(&self.current_token, ASTNode::BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line: 0,
                column: 0,
            });
        }
        Ok(left)
    }

    fn get_precedence(&self, token_type: &TokenType) -> Option<u8> {
        match token_type {
            TokenType::Or => Some(1),
            TokenType::And => Some(2),
            | TokenType::Is
            | TokenType::IsNot
            | TokenType::In
            | TokenType::IsIn
            | TokenType::IsEqual
            | TokenType::NotEqual => Some(3),
            | TokenType::LessThan
            | TokenType::LessThanEqual
            | TokenType::GreaterThan
            | TokenType::GreaterThanEqual => Some(4),
            TokenType::Plus | TokenType::Minus => Some(5),
            TokenType::Multiply | TokenType::Divide | TokenType::Modulus => Some(6),
            TokenType::Power => Some(7),
            _ => None,
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode, String> {
        let token = self.current_token.clone();
        let mut expr = (match &token.token_type {
            // Parenthesized expression
            TokenType::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(expr)
            }
            // Ternary expression (if-then-else)
            TokenType::If => {
                self.advance()?;
                let condition = Box::new(self.parse_expression()?);
                self.expect(TokenType::Then)?;
                let then_expr = Box::new(self.parse_expression()?);
                self.expect(TokenType::Else)?;
                let else_expr = Box::new(self.parse_expression()?);
                Ok(
                    ASTNode::from_token(&token, ASTNode::Ternary {
                        condition,
                        then_expr,
                        else_expr,
                        line: token.line,
                        column: token.column,
                    })
                )
            }
            // Unary 'not' operation
            TokenType::Not => {
                self.advance()?;
                let expr = Box::new(self.parse_expression()?);
                Ok(
                    ASTNode::from_token(&token, ASTNode::UnaryOperation {
                        operator: TokenType::Not,
                        expr,
                        line: token.line,
                        column: token.column,
                    })
                )
            }
            // Identifier (variable, function call, method call, field access, array/dictionary access)
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                if self.current_token.token_type == TokenType::LeftBracket {
                    // Array or dictionary access
                    let mut array_indices: Vec<Box<ASTNode>> = Vec::new();
                    let mut dict_keys: Vec<Box<ASTNode>> = Vec::new();
                    while self.current_token.token_type == TokenType::LeftBracket {
                        self.advance()?; // consume '['
                        let index_expr = self.parse_expression()?;
                        self.expect(TokenType::RightBracket)?; // consume ']'
                        match &index_expr {
                            ASTNode::StringLiteral { .. } => dict_keys.push(Box::new(index_expr)),
                            _ => array_indices.push(Box::new(index_expr)),
                        }
                    }
                    if !dict_keys.is_empty() && array_indices.is_empty() {
                        Ok(ASTNode::DictionaryAccess {
                            name,
                            keys: dict_keys,
                            line: token.line,
                            column: token.column,
                        })
                    } else if !array_indices.is_empty() && dict_keys.is_empty() {
                        Ok(ASTNode::ArrayAccess {
                            name,
                            indices: array_indices,
                            line: token.line,
                            column: token.column,
                        })
                    } else {
                        Err("Mixed array and dictionary indexing is not supported".to_string())
                    }
                } else if self.current_token.token_type == TokenType::LeftParen {
                    // Function call
                    self.expect(TokenType::LeftParen)?;
                    let args = self.parse_arg_list()?;
                    self.expect(TokenType::RightParen)?;
                    Ok(ASTNode::FunctionCall {
                        name,
                        args,
                        line: token.line,
                        column: token.column,
                    })
                } else if self.current_token.token_type == TokenType::Dot {
                    // Field access or method call
                    self.advance()?; // consume '.'
                    if let TokenType::Identifier(field) = &self.current_token.token_type {
                        let field_token = self.current_token.clone();
                        let field_name = field.clone();
                        self.advance()?; // consume field name
                        if self.current_token.token_type == TokenType::LeftParen {
                            // Method call
                            self.expect(TokenType::LeftParen)?;
                            let args = self.parse_arg_list()?;
                            self.expect(TokenType::RightParen)?;
                            Ok(ASTNode::MethodCall {
                                object: Box::new(ASTNode::Identifier {
                                    name,
                                    line: token.line,
                                    column: token.column,
                                }),
                                method: field_name,
                                args,
                                line: token.line,
                                column: token.column,
                            })
                        } else {
                            // Field access
                            Ok(ASTNode::FieldAccess {
                                object: Box::new(ASTNode::Identifier {
                                    name,
                                    line: token.line,
                                    column: token.column,
                                }),
                                field: Box::new(ASTNode::Identifier {
                                    name: field_name,
                                    line: field_token.line,
                                    column: field_token.column,
                                }),
                                line: token.line,
                                column: token.column,
                            })
                        }
                    } else {
                        Err(
                            format!(
                                "Expected identifier after '.', found {:?} at line {}, column {}",
                                self.current_token.token_type,
                                self.current_token.line,
                                self.current_token.column
                            )
                        )
                    }
                } else {
                    // Simple identifier
                    Ok(ASTNode::Identifier {
                        name,
                        line: token.line,
                        column: token.column,
                    })
                }
            }
            // Array literal
            TokenType::LeftBracket => {
                self.advance()?;
                let elements = if self.current_token.token_type != TokenType::RightBracket {
                    self.parse_arg_list()?
                } else {
                    Vec::new()
                };
                self.expect(TokenType::RightBracket)?;
                Ok(ASTNode::ArrayElement {
                    elements,
                    line: token.line,
                    column: token.column,
                })
            }
            // Dictionary literal
            TokenType::LeftBrace => {
                self.advance()?;
                let mut pairs = Vec::new();
                if self.current_token.token_type != TokenType::RightBrace {
                    loop {
                        let key = if
                            let TokenType::StringLiteral(k) = &self.current_token.token_type
                        {
                            let key_node = Box::new(ASTNode::StringLiteral {
                                value: k.clone(),
                                line: self.current_token.line,
                                column: self.current_token.column,
                            });
                            self.advance()?;
                            key_node
                        } else {
                            return Err(
                                format!(
                                    "Expected string key, found {:?} at line {}, column {}",
                                    self.current_token.token_type,
                                    self.current_token.line,
                                    self.current_token.column
                                )
                            );
                        };
                        self.expect(TokenType::Colon)?;
                        let value = Box::new(self.parse_expression()?);
                        pairs.push((key, value));
                        if self.current_token.token_type != TokenType::Comma {
                            break;
                        }
                        self.advance()?;
                    }
                }
                self.expect(TokenType::RightBrace)?;
                Ok(ASTNode::Dictionary {
                    pairs,
                    line: token.line,
                    column: token.column,
                })
            }
            // Object instantiation with 'new'
            TokenType::New => {
                self.advance()?;
                let class_expr = Box::new(self.parse_path()?);
                self.expect(TokenType::LeftParen)?;
                let args = self.parse_arg_list()?;
                self.expect(TokenType::RightParen)?;
                Ok(ASTNode::ClassInstantiation {
                    class_expr,
                    args,
                    line: token.line,
                    column: token.column,
                })
            }
            // Byte array
            TokenType::ByteArray => {
                self.advance()?;
                self.expect(TokenType::LeftParen)?;
                let mut args = Vec::new();
                if self.current_token.token_type != TokenType::RightParen {
                    for _ in 0..3 {
                        if self.current_token.token_type == TokenType::RightParen {
                            break;
                        }
                        args.push(Some(Box::new(self.parse_expression()?)));
                        if self.current_token.token_type == TokenType::Comma {
                            self.advance()?;
                        }
                    }
                }
                self.expect(TokenType::RightParen)?;
                while args.len() < 3 {
                    args.push(None);
                }
                Ok(ASTNode::ByteArray {
                    args,
                    line: token.line,
                    column: token.column,
                })
            }
            // Input function
            TokenType::Input => {
                self.advance()?; // Move past 'input'
                self.expect(TokenType::LeftParen)?;
                let args = self.parse_arg_list()?;
                self.expect(TokenType::RightParen)?;
                Ok(ASTNode::FunctionCall {
                    name: "input".to_string(),
                    args,
                    line: token.line,
                    column: token.column,
                })
            }
            // Number literal
            TokenType::Number(value) => {
                self.advance()?;
                Ok(ASTNode::NumberLiteral {
                    value: *value,
                    line: token.line,
                    column: token.column,
                })
            }
            // String literal
            TokenType::StringLiteral(value) => {
                self.advance()?;
                Ok(ASTNode::StringLiteral {
                    value: value.clone(),
                    line: token.line,
                    column: token.column,
                })
            }
            // Hex literal
            TokenType::Hex(value) => {
                self.advance()?;
                Ok(ASTNode::HexLiteral {
                    value: value.clone(),
                    line: token.line,
                    column: token.column,
                })
            }
            // Bytes literal
            TokenType::Bytes(value) => {
                self.advance()?;
                let string_value = String::from_utf8_lossy(&value).to_string();
                Ok(ASTNode::BytesLiteral {
                    value: string_value,
                    line: token.line,
                    column: token.column,
                })
            }
            // Scientific notation literal
            TokenType::Scientific(value) => {
                self.advance()?;
                Ok(ASTNode::ScientificLiteral {
                    value: *value,
                    line: token.line,
                    column: token.column,
                })
            }
            // Boolean true
            TokenType::True => {
                self.advance()?;
                Ok(ASTNode::True {
                    line: token.line,
                    column: token.column,
                })
            }
            // Boolean false
            TokenType::False => {
                self.advance()?;
                Ok(ASTNode::False {
                    line: token.line,
                    column: token.column,
                })
            }
            // Null
            TokenType::Null => {
                self.advance()?;
                Ok(ASTNode::Null {
                    line: token.line,
                    column: token.column,
                })
            }
            // 'this' keyword for field access
            TokenType::This => self.parse_field_access(),
            // Unexpected token
            _ =>
                Err(
                    format!(
                        "Expected primary expression, found {:?} at line {}, column {}",
                        token.token_type,
                        token.line,
                        token.column
                    )
                ),
        })?;

        // Handle postfix '.' for method calls or field access on any primary expression
        while self.current_token.token_type == TokenType::Dot {
            self.advance()?; // consume '.'
            if let TokenType::Identifier(field) = &self.current_token.token_type {
                let field_token = self.current_token.clone();
                let field_name = field.clone();
                self.advance()?; // consume field name
                if self.current_token.token_type == TokenType::LeftParen {
                    // Method call
                    self.expect(TokenType::LeftParen)?;
                    let args = self.parse_arg_list()?;
                    self.expect(TokenType::RightParen)?;
                    expr = ASTNode::MethodCall {
                        object: Box::new(expr),
                        method: field_name,
                        args,
                        line: field_token.line,
                        column: field_token.column,
                    };
                } else {
                    // Field access
                    expr = ASTNode::FieldAccess {
                        object: Box::new(expr),
                        field: Box::new(ASTNode::Identifier {
                            name: field_name,
                            line: field_token.line,
                            column: field_token.column,
                        }),
                        line: field_token.line,
                        column: field_token.column,
                    };
                }
            } else {
                return Err(
                    format!(
                        "Expected identifier after '.', found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
        }

        Ok(expr)
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        let start_token = self.current_token.clone();
        let statements = self.parse_statement_list(TokenType::EOF)?;
        Ok(ASTNode::from_token(&start_token, ASTNode::Block { statements, line: 0, column: 0 }))
    }

    fn parse_path(&mut self) -> Result<ASTNode, String> {
        let mut base = if let TokenType::Identifier(name) = &self.current_token.token_type {
            let token = self.current_token.clone();
            let name = name.clone();
            self.advance()?;
            ASTNode::Identifier { name, line: token.line, column: token.column }
        } else {
            return Err(
                format!(
                    "Expected identifier, found {:?} at line {}, column {}",
                    self.current_token.token_type,
                    self.current_token.line,
                    self.current_token.column
                )
            );
        };

        while self.current_token.token_type == TokenType::Dot {
            self.advance()?;
            if let TokenType::Identifier(field) = &self.current_token.token_type {
                let field_token = self.current_token.clone();
                let field = field.clone();
                self.advance()?;
                base = ASTNode::FieldAccess {
                    object: Box::new(base),
                    field: Box::new(ASTNode::Identifier {
                        name: field,
                        line: field_token.line,
                        column: field_token.column,
                    }),
                    line: field_token.line,
                    column: field_token.column,
                };
            } else {
                return Err(
                    format!(
                        "Expected identifier after '.', found {:?} at line {}, column {}",
                        self.current_token.token_type,
                        self.current_token.line,
                        self.current_token.column
                    )
                );
            }
        }
        Ok(base)
    }
}
