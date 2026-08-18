use crate::compiler::ast::*;
use crate::compiler::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ParserError {
    #[error("Unexpected token '{0}' at line {1}, col {2}, expected '{3}'")]
    UnexpectedToken(String, usize, usize, String),

    #[error("Unexpected EOF at line {0}, col {1}")]
    UnexpectedEof(usize, usize),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let mut contract_name = None;
        let mut functions = Vec::new();

        if self.check_kind(&TokenKind::Contract) {
            self.advance(); // consume 'contract'
            let name_tok = self.expect_identifier("contract name")?;
            contract_name = Some(name_tok);
            self.expect_kind(&TokenKind::LBrace, "{ after contract name")?;

            while !self.check_kind(&TokenKind::RBrace) && !self.is_at_end() {
                // Check if 'state' block
                if self.check_kind(&TokenKind::State) {
                    self.advance();
                    self.expect_kind(&TokenKind::LBrace, "{ after state")?;
                    let mut brace_depth = 1;
                    while brace_depth > 0 && !self.is_at_end() {
                        if self.check_kind(&TokenKind::LBrace) {
                            brace_depth += 1;
                        } else if self.check_kind(&TokenKind::RBrace) {
                            brace_depth -= 1;
                        }
                        self.advance();
                    }
                    continue;
                }

                if self.check_kind(&TokenKind::Fn) {
                    functions.push(self.parse_function()?);
                } else {
                    let tok = self.peek();
                    return Err(ParserError::UnexpectedToken(
                        tok.kind.to_string(),
                        tok.line,
                        tok.col,
                        "fn declaration".into(),
                    ));
                }
            }

            self.expect_kind(&TokenKind::RBrace, "} at end of contract")?;
        } else {
            while !self.is_at_end() && !self.check_kind(&TokenKind::Eof) {
                functions.push(self.parse_function()?);
            }
        }

        Ok(Program {
            contract_name,
            functions,
        })
    }

    fn parse_function(&mut self) -> Result<FunctionDef, ParserError> {
        self.expect_kind(&TokenKind::Fn, "fn keyword")?;
        let name = self.expect_identifier("function name")?;

        self.expect_kind(&TokenKind::LParen, "( after function name")?;
        let mut params = Vec::new();

        if !self.check_kind(&TokenKind::RParen) {
            loop {
                let param_name = self.expect_identifier("parameter name")?;
                // Skip optional type annotation ': type'
                if self.check_kind(&TokenKind::Colon) {
                    self.advance();
                    let _ = self.expect_identifier("parameter type")?;
                }
                params.push(param_name);

                if self.check_kind(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_kind(&TokenKind::RParen, ") after parameters")?;

        // Skip optional return type '-> type'
        if self.check_kind(&TokenKind::Arrow) {
            self.advance();
            let _ = self.expect_identifier("return type")?;
        }

        self.expect_kind(&TokenKind::LBrace, "{ before function body")?;
        let mut body = Vec::new();
        while !self.check_kind(&TokenKind::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect_kind(&TokenKind::RBrace, "} after function body")?;

        Ok(FunctionDef { name, params, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        if self.check_kind(&TokenKind::Let) {
            self.advance();
            let name = self.expect_identifier("variable name")?;
            if self.check_kind(&TokenKind::Colon) {
                self.advance();
                let _ = self.expect_identifier("variable type")?;
            }
            self.expect_kind(&TokenKind::Assign, "= in let statement")?;
            let initializer = self.parse_expression()?;
            self.optional_semi();
            Ok(Statement::Let { name, initializer })
        } else if self.check_kind(&TokenKind::If) {
            self.advance();
            let has_paren = self.check_kind(&TokenKind::LParen);
            if has_paren {
                self.advance();
            }
            let condition = self.parse_expression()?;
            if has_paren {
                self.expect_kind(&TokenKind::RParen, ") after condition")?;
            }

            self.expect_kind(&TokenKind::LBrace, "{ after if condition")?;
            let mut then_branch = Vec::new();
            while !self.check_kind(&TokenKind::RBrace) && !self.is_at_end() {
                then_branch.push(self.parse_statement()?);
            }
            self.expect_kind(&TokenKind::RBrace, "} after then block")?;

            let else_branch = if self.check_kind(&TokenKind::Else) {
                self.advance();
                if self.check_kind(&TokenKind::If) {
                    let else_if_stmt = self.parse_statement()?;
                    Some(vec![else_if_stmt])
                } else {
                    self.expect_kind(&TokenKind::LBrace, "{ after else")?;
                    let mut else_stmts = Vec::new();
                    while !self.check_kind(&TokenKind::RBrace) && !self.is_at_end() {
                        else_stmts.push(self.parse_statement()?);
                    }
                    self.expect_kind(&TokenKind::RBrace, "} after else block")?;
                    Some(else_stmts)
                }
            } else {
                None
            };

            Ok(Statement::If {
                condition,
                then_branch,
                else_branch,
            })
        } else if self.check_kind(&TokenKind::Return) {
            self.advance();
            let expr = if self.check_kind(&TokenKind::Semi) || self.check_kind(&TokenKind::RBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            self.optional_semi();
            Ok(Statement::Return(expr))
        } else if let TokenKind::Identifier(ref id) = self.peek().kind {
            if self.peek_next().kind == TokenKind::Assign {
                let name = id.clone();
                self.advance(); // consume id
                self.advance(); // consume =
                let value = self.parse_expression()?;
                self.optional_semi();
                return Ok(Statement::Assign { name, value });
            }
            let expr = self.parse_expression()?;
            self.optional_semi();
            Ok(Statement::Expr(expr))
        } else {
            let expr = self.parse_expression()?;
            self.optional_semi();
            Ok(Statement::Expr(expr))
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_relational()?;

        while self.check_kind(&TokenKind::EqEq) || self.check_kind(&TokenKind::NotEq) {
            let op = match self.peek().kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_relational()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_additive()?;

        while self.check_kind(&TokenKind::Lt)
            || self.check_kind(&TokenKind::Gt)
            || self.check_kind(&TokenKind::LtEq)
            || self.check_kind(&TokenKind::GtEq)
        {
            let op = match self.peek().kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_multiplicative()?;

        while self.check_kind(&TokenKind::Plus) || self.check_kind(&TokenKind::Minus) {
            let op = match self.peek().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_primary()?;

        while self.check_kind(&TokenKind::Star)
            || self.check_kind(&TokenKind::Slash)
            || self.check_kind(&TokenKind::Percent)
        {
            let op = match self.peek().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_primary()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        let tok = self.peek();

        match &tok.kind {
            TokenKind::Number(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Number(val))
            }
            TokenKind::StringLiteral(s) => {
                let str_val = s.clone();
                self.advance();
                if let Ok(val) = str_val.parse::<u64>() {
                    Ok(Expression::Number(val))
                } else if let Some(hex) = str_val.strip_prefix("0x").or_else(|| str_val.strip_prefix("0X")) {
                    if let Ok(val) = u64::from_str_radix(hex, 16) {
                        Ok(Expression::Number(val))
                    } else {
                        // Keep as dummy numeric value 0 or name reference
                        Ok(Expression::Variable(str_val))
                    }
                } else {
                    Ok(Expression::Variable(str_val))
                }
            }
            TokenKind::Identifier(id) => {
                let name = id.clone();
                self.advance();
                if self.check_kind(&TokenKind::LParen) {
                    let args = self.parse_args()?;
                    Ok(Expression::Call { name, args })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            TokenKind::Sstore => {
                self.advance();
                let args = self.parse_args()?;
                Ok(Expression::Call {
                    name: "sstore".into(),
                    args,
                })
            }
            TokenKind::Sload => {
                self.advance();
                let args = self.parse_args()?;
                Ok(Expression::Call {
                    name: "sload".into(),
                    args,
                })
            }
            TokenKind::Call => {
                self.advance();
                let args = self.parse_args()?;
                Ok(Expression::Call {
                    name: "call".into(),
                    args,
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_kind(&TokenKind::RParen, ") after nested expression")?;
                Ok(expr)
            }
            other => Err(ParserError::UnexpectedToken(
                other.to_string(),
                tok.line,
                tok.col,
                "expression".into(),
            )),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, ParserError> {
        self.expect_kind(&TokenKind::LParen, "( for function call")?;
        let mut args = Vec::new();
        if !self.check_kind(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if self.check_kind(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect_kind(&TokenKind::RParen, ") for function call")?;
        Ok(args)
    }

    fn check_kind(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn expect_kind(&mut self, expected: &TokenKind, desc: &str) -> Result<(), ParserError> {
        if self.check_kind(expected) {
            self.advance();
            Ok(())
        } else {
            let tok = self.peek();
            Err(ParserError::UnexpectedToken(
                tok.kind.to_string(),
                tok.line,
                tok.col,
                desc.into(),
            ))
        }
    }

    fn expect_identifier(&mut self, desc: &str) -> Result<String, ParserError> {
        let tok = self.peek();
        if let TokenKind::Identifier(ref id) = tok.kind {
            let name = id.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParserError::UnexpectedToken(
                tok.kind.to_string(),
                tok.line,
                tok.col,
                desc.into(),
            ))
        }
    }

    fn optional_semi(&mut self) {
        if self.check_kind(&TokenKind::Semi) {
            self.advance();
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.pos + 1)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::Lexer;

    #[test]
    fn test_parser_counter_contract() {
        let source = r#"
            contract Counter {
                fn init() {
                    sstore(0, 0);
                }

                fn increment(n: u64) -> u64 {
                    let count = sload(0);
                    sstore(0, count + n);
                    return sload(0);
                }

                fn get() -> u64 {
                    return sload(0);
                }
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("Lexing failed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().expect("Parsing failed");

        assert_eq!(program.contract_name, Some("Counter".to_string()));
        assert_eq!(program.functions.len(), 3);
        assert_eq!(program.functions[0].name, "init");
        assert_eq!(program.functions[1].name, "increment");
        assert_eq!(program.functions[1].params, vec!["n".to_string()]);
        assert_eq!(program.functions[2].name, "get");
    }
}
