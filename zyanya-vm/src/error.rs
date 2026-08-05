use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum VMError {
    #[error("Out of gas: limit {limit}, requested {requested}")]
    OutOfGas { limit: u64, requested: u64 },

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    #[error("Stack overflow (max depth: {0})")]
    StackOverflow(usize),

    #[error("Invalid memory index: {0}")]
    InvalidMemoryIndex(usize),

    #[error("Invalid jump target: PC {pc}, code length {code_len}")]
    InvalidJumpTarget { pc: usize, code_len: usize },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Execution halted by HALT instruction")]
    Halted,

    #[error("Unexpected end of code at PC {0}")]
    UnexpectedEndOfCode(usize),

    #[error("Insufficient contract balance: have {have}, need {need}")]
    InsufficientBalance { have: u64, need: u64 },
}
