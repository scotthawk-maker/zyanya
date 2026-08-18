use crate::compiler::token::{Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, col {2}")]
    UnexpectedChar(char, usize, usize),

    #[error("Unterminated string literal at line {0}, col {1}")]
    UnterminatedString(usize, usize),

    #[error("Invalid integer literal '{0}' at line {1}, col {2}")]
    InvalidNumber(String, usize, usize),
}

pub struct Lexer {
    chars: Vec<(usize, char)>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars = input.char_indices().collect();
        Self {
            chars,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
                self.advance();
                continue;
            }

            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // Line comments
            if ch == '/' && self.peek_next_char() == Some('/') {
                while let Some(c) = self.peek_char() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;

            // String literals
            if ch == '"' {
                let s = self.read_string(start_line, start_col)?;
                tokens.push(Token::new(TokenKind::StringLiteral(s), start_line, start_col));
                continue;
            }

            // Hex or Decimal Numbers
            if ch.is_ascii_digit() {
                let num = self.read_number(start_line, start_col)?;
                tokens.push(Token::new(TokenKind::Number(num), start_line, start_col));
                continue;
            }

            // Identifiers & Keywords
            if ch.is_ascii_alphabetic() || ch == '_' {
                let word = self.read_identifier();
                let kind = match word.as_str() {
                    "contract" => TokenKind::Contract,
                    "state" => TokenKind::State,
                    "fn" => TokenKind::Fn,
                    "let" => TokenKind::Let,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "return" => TokenKind::Return,
                    "sstore" => TokenKind::Sstore,
                    "sload" => TokenKind::Sload,
                    "call" => TokenKind::Call,
                    _ => TokenKind::Identifier(word),
                };
                tokens.push(Token::new(kind, start_line, start_col));
                continue;
            }

            // Operators & Delimiters
            let kind = match ch {
                '+' => {
                    self.advance();
                    TokenKind::Plus
                }
                '-' => {
                    self.advance();
                    if self.peek_char() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        TokenKind::Minus
                    }
                }
                '*' => {
                    self.advance();
                    TokenKind::Star
                }
                '/' => {
                    self.advance();
                    TokenKind::Slash
                }
                '%' => {
                    self.advance();
                    TokenKind::Percent
                }
                '=' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::EqEq
                    } else {
                        TokenKind::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::NotEq
                    } else {
                        return Err(LexerError::UnexpectedChar('!', start_line, start_col));
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::GtEq
                    } else {
                        TokenKind::Gt
                    }
                }
                ';' => {
                    self.advance();
                    TokenKind::Semi
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                ':' => {
                    self.advance();
                    TokenKind::Colon
                }
                '(' => {
                    self.advance();
                    TokenKind::LParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RParen
                }
                '{' => {
                    self.advance();
                    TokenKind::LBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RBrace
                }
                _ => return Err(LexerError::UnexpectedChar(ch, start_line, start_col)),
            };

            tokens.push(Token::new(kind, start_line, start_col));
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.col));
        Ok(tokens)
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.chars.get(self.pos + 1).map(|&(_, c)| c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.pos) {
            self.pos += 1;
            self.col += 1;
            Some(c)
        } else {
            None
        }
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<String, LexerError> {
        self.advance(); // consume opening quote
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(s);
            }
            if ch == '\n' {
                return Err(LexerError::UnterminatedString(line, col));
            }
            s.push(ch);
            self.advance();
        }
        Err(LexerError::UnterminatedString(line, col))
    }

    fn read_number(&mut self, line: usize, col: usize) -> Result<u64, LexerError> {
        let mut raw = String::new();

        if self.peek_char() == Some('0') && (self.peek_next_char() == Some('x') || self.peek_next_char() == Some('X')) {
            raw.push(self.advance().unwrap()); // '0'
            raw.push(self.advance().unwrap()); // 'x'
            while let Some(c) = self.peek_char() {
                if c.is_ascii_hexdigit() {
                    raw.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            let stripped = &raw[2..];
            u64::from_str_radix(stripped, 16).map_err(|_| LexerError::InvalidNumber(raw, line, col))
        } else {
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    raw.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            raw.parse::<u64>().map_err(|_| LexerError::InvalidNumber(raw, line, col))
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut id = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                id.push(c);
                self.advance();
            } else {
                break;
            }
        }
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let source = r#"
            // Counter contract
            fn increment(n) {
                let count = sload(0);
                sstore(0, count + n);
                return count + n;
            }
        "#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("Lexing failed");

        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Fn,
                TokenKind::Identifier("increment".into()),
                TokenKind::LParen,
                TokenKind::Identifier("n".into()),
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::Let,
                TokenKind::Identifier("count".into()),
                TokenKind::Assign,
                TokenKind::Sload,
                TokenKind::LParen,
                TokenKind::Number(0),
                TokenKind::RParen,
                TokenKind::Semi,
                TokenKind::Sstore,
                TokenKind::LParen,
                TokenKind::Number(0),
                TokenKind::Comma,
                TokenKind::Identifier("count".into()),
                TokenKind::Plus,
                TokenKind::Identifier("n".into()),
                TokenKind::RParen,
                TokenKind::Semi,
                TokenKind::Return,
                TokenKind::Identifier("count".into()),
                TokenKind::Plus,
                TokenKind::Identifier("n".into()),
                TokenKind::Semi,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }
}
