pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::*;
pub use codegen::{CodeGenerator, CodegenError};
pub use lexer::{Lexer, LexerError};
pub use parser::{Parser, ParserError};
pub use token::{Token, TokenKind};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CompilerError {
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),

    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Codegen error: {0}")]
    Codegen(#[from] CodegenError),
}

/// The Zyanya Contract Language (ZCL) Compiler.
pub struct Compiler;

impl Compiler {
    /// Compile ZCL source code into Assembly text.
    pub fn compile_to_assembly(source: &str) -> Result<String, CompilerError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        let mut codegen = CodeGenerator::new();
        let asm = codegen.generate_assembly(&program)?;
        Ok(asm)
    }

    /// Compile ZCL source code into bytecode.
    pub fn compile(source: &str) -> Result<Vec<u8>, CompilerError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.compile_to_bytecode(&program)?;
        Ok(bytecode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockStateBackend, OpCode, VM};

    #[test]
    fn test_compiler_counter_contract_execution() {
        let zcl_source = r#"
            // Counter Contract in ZCL
            fn init() {
                sstore(0, 0);
                return 0;
            }

            fn increment(n) {
                let count = sload(0);
                sstore(0, count + n);
                return sload(0);
            }

            fn get() {
                return sload(0);
            }
        "#;

        let bytecode = Compiler::compile(zcl_source).expect("ZCL compilation failed");
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");

        let mut state = MockStateBackend::new();
        let addr = [0x55u8; 32];

        // 1. Execute fn init() (entry point 0)
        let mut vm = VM::new(100_000);
        vm.stack.push(0).unwrap(); // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("init failed");
        assert_eq!(res.return_value, Some(0));
        assert_eq!(state.get(&addr, 0), 0);

        // 2. Execute fn increment(15) (entry point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(15).unwrap(); // n = 15
        vm.stack.push(1).unwrap();  // entry_point = 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("increment failed");
        assert_eq!(res.return_value, Some(15));
        assert_eq!(state.get(&addr, 0), 15);

        // 3. Execute fn increment(27) (entry point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(27).unwrap(); // n = 27
        vm.stack.push(1).unwrap();  // entry_point = 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("second increment failed");
        assert_eq!(res.return_value, Some(42));
        assert_eq!(state.get(&addr, 0), 42);

        // 4. Execute fn get() (entry point 2)
        let mut vm = VM::new(100_000);
        vm.stack.push(2).unwrap(); // entry_point = 2
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("get failed");
        assert_eq!(res.return_value, Some(42));
    }

    #[test]
    fn test_compiler_token_contract_execution() {
        let token_zcl = r#"
            // Token Contract in ZCL
            fn transfer(from, to, amount) {
                let from_bal = sload(from);
                if (from_bal < amount) {
                    return 0;
                }
                sstore(from, from_bal - amount);
                let to_bal = sload(to);
                sstore(to, to_bal + amount);
                return 1;
            }

            fn balance_of(holder) {
                return sload(holder);
            }

            fn total_supply() {
                return sload(0);
            }

            fn mint(to, amount) {
                let supply = sload(0);
                sstore(0, supply + amount);
                let to_bal = sload(to);
                sstore(to, to_bal + amount);
                return 1;
            }
        "#;

        let bytecode = Compiler::compile(token_zcl).expect("Token ZCL compile failed");
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Deserialization failed");
        let mut state = MockStateBackend::new();
        let addr = [0xAAu8; 32];
        let owner = 10u64;
        let recipient = 20u64;

        // Initialize owner balance & supply manually
        state.set(&addr, 0, 1_000_000);
        state.set(&addr, owner, 1_000_000);

        // 1. Query total supply (entry point 2)
        let mut vm = VM::new(100_000);
        vm.stack.push(2).unwrap(); // entry_point = 2
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1_000_000));

        // 2. Query owner balance (entry point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(owner).unwrap();
        vm.stack.push(1).unwrap(); // entry_point = 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1_000_000));

        // 3. Transfer 500 tokens from owner to recipient (entry point 0)
        let mut vm = VM::new(100_000);
        vm.stack.push(500).unwrap(); // amount
        vm.stack.push(recipient).unwrap(); // to
        vm.stack.push(owner).unwrap(); // from
        vm.stack.push(0).unwrap(); // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, owner), 999_500);
        assert_eq!(state.get(&addr, recipient), 500);

        // 4. Mint 1,000 tokens to recipient (entry point 3)
        let mut vm = VM::new(100_000);
        vm.stack.push(1_000).unwrap(); // amount
        vm.stack.push(recipient).unwrap(); // to
        vm.stack.push(3).unwrap(); // entry_point = 3
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, 0), 1_001_000);
        assert_eq!(state.get(&addr, recipient), 1_500);
    }
}
