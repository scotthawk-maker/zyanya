use crate::error::VMError;
use std::fmt;
use zyanya_utils::hex::ToHex;

/// Opcodes supported by the `zyanya-vm` execution engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCode {
    /// No operation.
    Nop,
    /// Stop execution successfully.
    Halt,
    /// Push a 64-bit unsigned integer onto the operand stack.
    Push(u64),
    /// Pop the top item from the operand stack.
    Pop,
    /// Duplicate the top item on the operand stack.
    Dup,
    /// Swap the top two items on the operand stack.
    Swap,

    // --- Arithmetic ---
    /// Add top two items on stack (`b + a`).
    Add,
    /// Subtract top item from second top item (`b - a`).
    Sub,
    /// Multiply top two items on stack (`b * a`).
    Mul,
    /// Unsigned integer division (`b / a`).
    Div,
    /// Unsigned modulo (`b % a`).
    Mod,
    /// Exponentiation (`b ^ a`).
    Pow,

    // --- Logic ---
    /// Bitwise AND (`b & a`).
    And,
    /// Bitwise OR (`b | a`).
    Or,
    /// Bitwise XOR (`b ^ a`).
    Xor,
    /// Bitwise NOT (`!a`).
    Not,

    // --- Comparisons ---
    /// Compare top two items for equality (`b == a`), pushes 1 for true, 0 for false.
    Eq,
    /// Less-than check (`b < a`).
    Lt,
    /// Greater-than check (`b > a`).
    Gt,
    /// Less-than-or-equal check (`b <= a`).
    Lte,
    /// Greater-than-or-equal check (`b >= a`).
    Gte,

    // --- Control Flow ---
    /// Unconditional jump to program instruction index.
    Jump(usize),
    /// Conditional jump: pops condition; if non-zero, jumps to instruction index.
    JumpIf(usize),

    // --- Local Memory ---
    /// Load value from memory register index onto the stack.
    Load(usize),
    /// Store top stack value into memory register index.
    Store(usize),

    // --- Contract Storage ---
    /// Load value from persistent contract state key.
    SLoad,
    /// Store value into persistent contract state key.
    SStore,

    // --- Inter-Contract Call ---
    /// Call another contract at specified 32-byte address.
    Call([u8; 32]),

    // --- Contract Context ---
    /// Push the verified caller's address (u64) onto the stack. Set by the consensus layer.
    Caller,
    /// Push the contract's own ZYAN balance onto the stack.
    Balance,
    /// Withdraw ZYAN from the contract to a recipient. Pops `amount` then `recipient`,
    /// pushes 1 on success or 0 on failure.
    Withdraw,

    /// Return from execution with top stack value as result.
    Return,
}

impl OpCode {
    /// Base gas cost for executing this opcode.
    pub fn base_gas_cost(&self) -> u64 {
        match self {
            OpCode::Nop => 1,
            OpCode::Halt => 1,
            OpCode::Push(_) => 2,
            OpCode::Pop => 1,
            OpCode::Dup => 2,
            OpCode::Swap => 2,
            OpCode::Add => 3,
            OpCode::Sub => 3,
            OpCode::Mul => 5,
            OpCode::Div => 5,
            OpCode::Mod => 5,
            OpCode::Pow => 8,
            OpCode::And => 3,
            OpCode::Or => 3,
            OpCode::Xor => 3,
            OpCode::Not => 2,
            OpCode::Eq => 3,
            OpCode::Lt => 3,
            OpCode::Gt => 3,
            OpCode::Lte => 3,
            OpCode::Gte => 3,
            OpCode::Jump(_) => 4,
            OpCode::JumpIf(_) => 4,
            OpCode::Load(_) => 3,
            OpCode::Store(_) => 4,
            OpCode::SLoad => 100,
            OpCode::SStore => 500,
            OpCode::Call(_) => 200,
            OpCode::Caller => 1,
            OpCode::Balance => 3,
            OpCode::Withdraw => 10,
            OpCode::Return => 1,
        }
    }

    /// Serialize a slice of `OpCode`s into raw byte stream.
    pub fn serialize_slice(code: &[OpCode]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for op in code {
            match op {
                OpCode::Nop => bytes.push(0x00),
                OpCode::Halt => bytes.push(0x01),
                OpCode::Push(val) => {
                    bytes.push(0x02);
                    bytes.extend_from_slice(&val.to_le_bytes());
                }
                OpCode::Pop => bytes.push(0x03),
                OpCode::Dup => bytes.push(0x04),
                OpCode::Swap => bytes.push(0x05),
                OpCode::Add => bytes.push(0x10),
                OpCode::Sub => bytes.push(0x11),
                OpCode::Mul => bytes.push(0x12),
                OpCode::Div => bytes.push(0x13),
                OpCode::Mod => bytes.push(0x14),
                OpCode::Pow => bytes.push(0x15),
                OpCode::Eq => bytes.push(0x20),
                OpCode::Lt => bytes.push(0x21),
                OpCode::Gt => bytes.push(0x22),
                OpCode::Lte => bytes.push(0x23),
                OpCode::Gte => bytes.push(0x24),
                OpCode::And => bytes.push(0x25),
                OpCode::Or => bytes.push(0x26),
                OpCode::Xor => bytes.push(0x27),
                OpCode::Not => bytes.push(0x28),
                OpCode::Jump(target) => {
                    bytes.push(0x30);
                    bytes.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                OpCode::JumpIf(target) => {
                    bytes.push(0x31);
                    bytes.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                OpCode::Load(idx) => {
                    bytes.push(0x40);
                    bytes.extend_from_slice(&(*idx as u64).to_le_bytes());
                }
                OpCode::Store(idx) => {
                    bytes.push(0x41);
                    bytes.extend_from_slice(&(*idx as u64).to_le_bytes());
                }
                OpCode::SLoad => bytes.push(0x50),
                OpCode::SStore => bytes.push(0x51),
                OpCode::Call(addr) => {
                    bytes.push(0x60);
                    bytes.extend_from_slice(addr);
                }
                OpCode::Caller => bytes.push(0x70),
                OpCode::Balance => bytes.push(0x71),
                OpCode::Withdraw => bytes.push(0x72),
                OpCode::Return => bytes.push(0xF0),
            }
        }
        bytes
    }

    /// Deserialize a raw byte slice into a vector of `OpCode`s.
    pub fn deserialize_slice(bytes: &[u8]) -> Result<Vec<OpCode>, VMError> {
        let mut opcodes = Vec::new();
        let mut cursor = 0;
        let mut byte_to_opcode = std::collections::HashMap::new();

        while cursor < bytes.len() {
            let start_byte = cursor;
            let tag = bytes[cursor];
            cursor += 1;

            byte_to_opcode.insert(start_byte, opcodes.len());

            match tag {
                0x00 => opcodes.push(OpCode::Nop),
                0x01 => opcodes.push(OpCode::Halt),
                0x02 => {
                    if cursor + 8 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let val = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
                    cursor += 8;
                    opcodes.push(OpCode::Push(val));
                }
                0x03 => opcodes.push(OpCode::Pop),
                0x04 => opcodes.push(OpCode::Dup),
                0x05 => opcodes.push(OpCode::Swap),
                0x10 => opcodes.push(OpCode::Add),
                0x11 => opcodes.push(OpCode::Sub),
                0x12 => opcodes.push(OpCode::Mul),
                0x13 => opcodes.push(OpCode::Div),
                0x14 => opcodes.push(OpCode::Mod),
                0x15 => opcodes.push(OpCode::Pow),
                0x20 => opcodes.push(OpCode::Eq),
                0x21 => opcodes.push(OpCode::Lt),
                0x22 => opcodes.push(OpCode::Gt),
                0x23 => opcodes.push(OpCode::Lte),
                0x24 => opcodes.push(OpCode::Gte),
                0x25 => opcodes.push(OpCode::And),
                0x26 => opcodes.push(OpCode::Or),
                0x27 => opcodes.push(OpCode::Xor),
                0x28 => opcodes.push(OpCode::Not),
                0x30 => {
                    if cursor + 8 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let target = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
                    cursor += 8;
                    opcodes.push(OpCode::Jump(target));
                }
                0x31 => {
                    if cursor + 8 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let target = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
                    cursor += 8;
                    opcodes.push(OpCode::JumpIf(target));
                }
                0x40 => {
                    if cursor + 8 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let idx = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
                    cursor += 8;
                    opcodes.push(OpCode::Load(idx));
                }
                0x41 => {
                    if cursor + 8 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let idx = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap()) as usize;
                    cursor += 8;
                    opcodes.push(OpCode::Store(idx));
                }
                0x50 => opcodes.push(OpCode::SLoad),
                0x51 => opcodes.push(OpCode::SStore),
                0x60 => {
                    if cursor + 32 > bytes.len() {
                        return Err(VMError::UnexpectedEndOfCode(cursor));
                    }
                    let mut addr = [0u8; 32];
                    addr.copy_from_slice(&bytes[cursor..cursor + 32]);
                    cursor += 32;
                    opcodes.push(OpCode::Call(addr));
                }
                0x70 => opcodes.push(OpCode::Caller),
                0x71 => opcodes.push(OpCode::Balance),
                0x72 => opcodes.push(OpCode::Withdraw),
                0xF0 => opcodes.push(OpCode::Return),
                unknown => return Err(VMError::InvalidOpcode(unknown)),
            }
        }

        for op in &mut opcodes {
            match op {
                OpCode::Jump(target) => {
                    if let Some(&op_idx) = byte_to_opcode.get(target) {
                        *target = op_idx;
                    }
                }
                OpCode::JumpIf(target) => {
                    if let Some(&op_idx) = byte_to_opcode.get(target) {
                        *target = op_idx;
                    }
                }
                _ => {}
            }
        }

        Ok(opcodes)
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::Nop => write!(f, "NOP"),
            OpCode::Halt => write!(f, "HALT"),
            OpCode::Push(v) => write!(f, "PUSH {}", v),
            OpCode::Pop => write!(f, "POP"),
            OpCode::Dup => write!(f, "DUP"),
            OpCode::Swap => write!(f, "SWAP"),
            OpCode::Add => write!(f, "ADD"),
            OpCode::Sub => write!(f, "SUB"),
            OpCode::Mul => write!(f, "MUL"),
            OpCode::Div => write!(f, "DIV"),
            OpCode::Mod => write!(f, "MOD"),
            OpCode::Pow => write!(f, "POW"),
            OpCode::And => write!(f, "AND"),
            OpCode::Or => write!(f, "OR"),
            OpCode::Xor => write!(f, "XOR"),
            OpCode::Not => write!(f, "NOT"),
            OpCode::Eq => write!(f, "EQ"),
            OpCode::Lt => write!(f, "LT"),
            OpCode::Gt => write!(f, "GT"),
            OpCode::Lte => write!(f, "LTE"),
            OpCode::Gte => write!(f, "GTE"),
            OpCode::Jump(target) => write!(f, "JUMP {}", target),
            OpCode::JumpIf(target) => write!(f, "JUMPIF {}", target),
            OpCode::Load(idx) => write!(f, "LOAD {}", idx),
            OpCode::Store(idx) => write!(f, "STORE {}", idx),
            OpCode::SLoad => write!(f, "SLOAD"),
            OpCode::SStore => write!(f, "SSTORE"),
            OpCode::Call(addr) => write!(f, "CALL 0x{}", addr.as_slice().to_hex()),
            OpCode::Caller => write!(f, "CALLER"),
            OpCode::Balance => write!(f, "BALANCE"),
            OpCode::Withdraw => write!(f, "WITHDRAW"),
            OpCode::Return => write!(f, "RETURN"),
        }
    }
}
