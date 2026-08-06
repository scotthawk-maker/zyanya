use crate::{error::VMError, opcode::OpCode};
use std::collections::HashMap;
use thiserror::Error;
use zyanya_utils::hex::FromHex;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum AssemblerError {
    #[error("Unknown opcode: '{0}' on line {1}")]
    UnknownOpcode(String, usize),

    #[error("Missing argument for opcode '{0}' on line {1}")]
    MissingArgument(String, usize),

    #[error("Invalid integer argument '{0}' on line {1}")]
    InvalidInteger(String, usize),

    #[error("Invalid contract address '{0}' on line {1}")]
    InvalidAddress(String, usize),

    #[error("Undefined label '{0}' on line {1}")]
    UndefinedLabel(String, usize),

    #[error("Duplicate label '{0}' on line {1}")]
    DuplicateLabel(String, usize),
}

/// A minimal assembler for `zyanya-vm`.
/// Converts human-readable assembly text into VM bytecode or a list of `OpCode`s.
pub struct Assembler;

impl Assembler {
    /// Assemble source text into serialized VM bytecode.
    pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblerError> {
        let opcodes = Self::assemble_to_opcodes(source)?;
        Ok(OpCode::serialize_slice(&opcodes))
    }

    /// Assemble source text into a vector of `OpCode`s.
    pub fn assemble_to_opcodes(source: &str) -> Result<Vec<OpCode>, AssemblerError> {
        let mut raw_lines = Vec::new();
        let mut labels: HashMap<String, usize> = HashMap::new();
        let mut current_byte_offset = 0usize;

        // Pass 1: Strip comments, trim whitespace, and register labels with byte offsets
        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let mut cleaned = line;
            if let Some(pos) = cleaned.find("//") {
                cleaned = &cleaned[..pos];
            }
            if let Some(pos) = cleaned.find(';') {
                cleaned = &cleaned[..pos];
            }
            let trimmed = cleaned.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if label definition (starts with ':' or ends with ':')
            if trimmed.starts_with(':') || trimmed.ends_with(':') {
                let label_name = trimmed.trim_matches(':').trim().to_string();
                if labels.contains_key(&label_name) {
                    return Err(AssemblerError::DuplicateLabel(label_name, line_num));
                }
                labels.insert(label_name, current_byte_offset);
            } else {
                raw_lines.push((line_num, trimmed));
                let size = opcode_line_byte_size(trimmed, line_num)?;
                current_byte_offset += size;
            }
        }

        // Pass 2: Parse opcodes
        let mut opcodes = Vec::with_capacity(raw_lines.len());
        for (line_num, line) in raw_lines {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let op_str = tokens[0].to_uppercase();
            let opcode = match op_str.as_str() {
                "NOP" => OpCode::Nop,
                "HALT" => OpCode::Halt,
                "PUSH" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("PUSH".into(), line_num))?;
                    let val = parse_u64(arg).map_err(|_| AssemblerError::InvalidInteger((*arg).into(), line_num))?;
                    OpCode::Push(val)
                }
                "POP" => OpCode::Pop,
                "DUP" => OpCode::Dup,
                "SWAP" => OpCode::Swap,
                "ADD" => OpCode::Add,
                "SUB" => OpCode::Sub,
                "MUL" => OpCode::Mul,
                "DIV" => OpCode::Div,
                "MOD" => OpCode::Mod,
                "POW" => OpCode::Pow,
                "AND" => OpCode::And,
                "OR" => OpCode::Or,
                "XOR" => OpCode::Xor,
                "NOT" => OpCode::Not,
                "EQ" => OpCode::Eq,
                "LT" => OpCode::Lt,
                "GT" => OpCode::Gt,
                "LTE" => OpCode::Lte,
                "GTE" => OpCode::Gte,
                "JUMP" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("JUMP".into(), line_num))?;
                    let target = resolve_target(arg, &labels, line_num)?;
                    OpCode::Jump(target)
                }
                "JUMPIF" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("JUMPIF".into(), line_num))?;
                    let target = resolve_target(arg, &labels, line_num)?;
                    OpCode::JumpIf(target)
                }
                "LOAD" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("LOAD".into(), line_num))?;
                    let idx = parse_u64(arg).map_err(|_| AssemblerError::InvalidInteger((*arg).into(), line_num))? as usize;
                    OpCode::Load(idx)
                }
                "STORE" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("STORE".into(), line_num))?;
                    let idx = parse_u64(arg).map_err(|_| AssemblerError::InvalidInteger((*arg).into(), line_num))? as usize;
                    OpCode::Store(idx)
                }
                "SLOAD" => OpCode::SLoad,
                "SSTORE" => OpCode::SStore,
                "CALL" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("CALL".into(), line_num))?;
                    let hex_str = arg.trim_start_matches("0x");
                    let addr = <[u8; 32]>::from_hex(hex_str).map_err(|_| AssemblerError::InvalidAddress((*arg).into(), line_num))?;
                    OpCode::Call(addr)
                }
                "CALLMULTI" => {
                    let arg = tokens.get(1).ok_or_else(|| AssemblerError::MissingArgument("CALLMULTI".into(), line_num))?;
                    let hex_str = arg.trim_start_matches("0x");
                    let addr = <[u8; 32]>::from_hex(hex_str).map_err(|_| AssemblerError::InvalidAddress((*arg).into(), line_num))?;
                    OpCode::CallMulti(addr)
                }
                "RETURN" => OpCode::Return,
                _ => return Err(AssemblerError::UnknownOpcode(tokens[0].to_string(), line_num)),
            };

            opcodes.push(opcode);
        }

        Ok(opcodes)
    }

    /// Disassemble raw bytecode into assembly string representation.
    pub fn disassemble(bytes: &[u8]) -> Result<String, VMError> {
        let opcodes = OpCode::deserialize_slice(bytes)?;
        let mut lines = Vec::with_capacity(opcodes.len());
        for op in opcodes {
            lines.push(op.to_string());
        }
        Ok(lines.join("\n"))
    }
}

fn parse_u64(s: &str) -> Result<u64, std::num::ParseIntError> {
    if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(stripped, 16)
    } else {
        s.parse::<u64>()
    }
}

fn opcode_line_byte_size(line: &str, line_num: usize) -> Result<usize, AssemblerError> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(0);
    }
    let op_str = tokens[0].to_uppercase();
    match op_str.as_str() {
        "NOP" | "HALT" | "POP" | "DUP" | "SWAP" | "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "POW"
        | "AND" | "OR" | "XOR" | "NOT" | "EQ" | "LT" | "GT" | "LTE" | "GTE" | "SLOAD" | "SSTORE"
        | "RETURN" => Ok(1),
        "PUSH" | "JUMP" | "JUMPIF" | "LOAD" | "STORE" => Ok(9),
        "CALL" | "CALLMULTI" => Ok(33),
        _ => Err(AssemblerError::UnknownOpcode(tokens[0].to_string(), line_num)),
    }
}

fn resolve_target(arg: &str, labels: &HashMap<String, usize>, line_num: usize) -> Result<usize, AssemblerError> {
    let clean_arg = arg.trim_start_matches(':');
    if let Some(&target) = labels.get(clean_arg) {
        Ok(target)
    } else if let Ok(val) = parse_u64(arg) {
        Ok(val as usize)
    } else {
        Err(AssemblerError::UndefinedLabel(arg.to_string(), line_num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockStateBackend, VM};

    #[test]
    fn test_assembler_basic() {
        let asm = r#"
            // Contract assembly example
            PUSH 100
            PUSH 200
            ADD ; 300
            DUP
            PUSH 42
            SWAP
            SSTORE
            PUSH 42
            SLOAD
            RETURN
        "#;

        let bytecode = Assembler::assemble(asm).expect("Assembly failed");
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");

        assert_eq!(opcodes.len(), 10);
        let mut vm = VM::new(10000);
        let mut state = MockStateBackend::new();
        let addr = [0x77u8; 32];

        let result = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Execution failed");
        assert_eq!(result.return_value, Some(300));
        assert_eq!(state.get(&addr, 42), 300);
    }

    #[test]
    fn test_assembler_with_labels_and_loop() {
        let asm = r#"
            PUSH 0
            STORE 0
            :loop
            LOAD 0
            PUSH 1
            ADD
            STORE 0
            LOAD 0
            PUSH 5
            LT
            JUMPIF :loop
            LOAD 0
            RETURN
        "#;

        let bytecode = Assembler::assemble(asm).expect("Assembly failed");
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");

        let mut vm = VM::new(10000);
        let result = vm.execute(&opcodes).expect("Execution failed");
        assert_eq!(result.return_value, Some(5));
    }
}
