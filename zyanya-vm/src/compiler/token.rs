use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Contract,
    State,
    Fn,
    Let,
    If,
    Else,
    Return,
    Sstore,
    Sload,
    Call,

    // Identifiers & Literals
    Identifier(String),
    Number(u64),
    StringLiteral(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Assign,

    // Delimiters & Punctuation
    Semi,
    Comma,
    Colon,
    Arrow, // ->
    LParen,
    RParen,
    LBrace,
    RBrace,

    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Contract => write!(f, "contract"),
            TokenKind::State => write!(f, "state"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Sstore => write!(f, "sstore"),
            TokenKind::Sload => write!(f, "sload"),
            TokenKind::Call => write!(f, "call"),
            TokenKind::Identifier(id) => write!(f, "{}", id),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Eof => write!(f, "<EOF>"),
        }
    }
}
