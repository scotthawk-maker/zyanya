pub mod assembler;
pub mod compiler;
pub mod error;
pub mod gas;
pub mod memory;
pub mod opcode;
pub mod stack;
pub mod state;
pub mod token;
pub mod vm;

pub use assembler::{Assembler, AssemblerError};
pub use compiler::{Compiler, CompilerError};
pub use error::VMError;
pub use gas::GasMeter;
pub use memory::Memory;
pub use opcode::OpCode;
pub use stack::Stack;
pub use state::{MockStateBackend, NoopStateBackend, StateBackend};
pub use token::*;
pub use vm::{VMResult, VM};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut vm = VM::new(1000);
        let program = vec![
            OpCode::Push(15),
            OpCode::Push(27),
            OpCode::Add,
            OpCode::Return,
        ];

        let result = vm.execute(&program).expect("execution failed");
        assert_eq!(result.return_value, Some(42));
        assert!(result.gas_used > 0);
    }

    #[test]
    fn test_extended_arithmetic() {
        // Test DIV, MOD, POW
        let mut vm = VM::new(1000);
        let program = vec![
            OpCode::Push(100),
            OpCode::Push(7),
            OpCode::Div, // 100 / 7 = 14
            OpCode::Push(100),
            OpCode::Push(7),
            OpCode::Mod, // 100 % 7 = 2
            OpCode::Push(2),
            OpCode::Push(10),
            OpCode::Pow, // 2^10 = 1024
            OpCode::Return,
        ];

        let result = vm.execute(&program).expect("execution failed");
        assert_eq!(result.return_value, Some(1024));
        assert_eq!(result.stack_dump, vec![14, 2]);
    }

    #[test]
    fn test_division_by_zero() {
        let mut vm = VM::new(1000);
        let program = vec![OpCode::Push(10), OpCode::Push(0), OpCode::Div];
        assert_eq!(vm.execute(&program), Err(VMError::DivisionByZero));
    }

    #[test]
    fn test_logic_and_comparison_opcodes() {
        let mut vm = VM::new(1000);
        let program = vec![
            OpCode::Push(0b1100),
            OpCode::Push(0b1010),
            OpCode::And, // 0b1000 = 8
            OpCode::Push(10),
            OpCode::Push(20),
            OpCode::Lt, // 10 < 20 = 1
            OpCode::Push(30),
            OpCode::Push(30),
            OpCode::Lte, // 30 <= 30 = 1
            OpCode::Return,
        ];

        let result = vm.execute(&program).expect("execution failed");
        assert_eq!(result.return_value, Some(1));
        assert_eq!(result.stack_dump, vec![8, 1]);
    }

    #[test]
    fn test_state_load_and_store() {
        let mut vm = VM::new(10000);
        let mut state = MockStateBackend::new();
        let addr = [0x42u8; 32];

        let program = vec![
            OpCode::Push(1234), // Key
            OpCode::Push(9999), // Value
            OpCode::SStore,     // Store key 1234 -> 9999
            OpCode::Push(1234), // Key
            OpCode::SLoad,      // Load key 1234
            OpCode::Return,
        ];

        let result = vm
            .execute_stateful(&program, &addr, &mut state)
            .expect("execution failed");
        assert_eq!(result.return_value, Some(9999));
        assert_eq!(state.get(&addr, 1234), 9999);
    }

    #[test]
    fn test_serialization_round_trip() {
        let code = vec![
            OpCode::Push(42),
            OpCode::Push(100),
            OpCode::Add,
            OpCode::Store(3),
            OpCode::Load(3),
            OpCode::Push(1),
            OpCode::SStore,
            OpCode::Push(1),
            OpCode::SLoad,
            OpCode::JumpIf(12),
            OpCode::Halt,
            OpCode::Return,
        ];

        let bytes = OpCode::serialize_slice(&code);
        let deserialized = OpCode::deserialize_slice(&bytes).expect("deserialization failed");

        assert_eq!(code, deserialized);
    }

    #[test]
    fn test_inter_contract_call() {
        let mut vm = VM::new(10000);
        let mut state = MockStateBackend::new();

        let addr_a = [0x01u8; 32];
        let addr_b = [0x02u8; 32];

        // Contract B: takes calldata, adds 100, SSTORE key 1 = result, returns result
        let code_b = vec![
            OpCode::Push(100),
            OpCode::Add,
            OpCode::Dup,
            OpCode::Push(1),
            OpCode::Swap,
            OpCode::SStore,
            OpCode::Push(1),
            OpCode::SLoad,
            OpCode::Return,
        ];
        state.set_code(addr_b, OpCode::serialize_slice(&code_b));

        // Contract A: calls Contract B with forward_gas=5000, calldata=42
        // Gets return value (142), SSTORE key 99 = 142, returns 142
        let code_a = vec![
            OpCode::Push(5000), // forward_gas
            OpCode::Push(42),   // calldata
            OpCode::Call(addr_b),
            OpCode::Dup,
            OpCode::Push(99),
            OpCode::Swap,
            OpCode::SStore,
            OpCode::Return,
        ];

        let result = vm
            .execute_stateful(&code_a, &addr_a, &mut state)
            .expect("Contract A execution failed");

        assert_eq!(result.return_value, Some(142));
        assert_eq!(state.get(&addr_b, 1), 142, "Contract B storage isolated and updated");
        assert_eq!(state.get(&addr_a, 99), 142, "Contract A storage updated with return value");
    }
}
