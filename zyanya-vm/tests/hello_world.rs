use zyanya_vm::{OpCode, VM};

#[test]
fn test_hello_world_contract() {
    // Hello World Smart Contract:
    // Computes: (10 + 20) * 3 = 90
    // Stores result in memory register 5
    // Checks equality with 90
    // Returns 1 (true)
    let contract_code = vec![
        OpCode::Push(10),
        OpCode::Push(20),
        OpCode::Add,        // Stack: [30]
        OpCode::Push(3),
        OpCode::Mul,        // Stack: [90]
        OpCode::Dup,        // Stack: [90, 90]
        OpCode::Store(5),   // Reg 5 = 90, Stack: [90]
        OpCode::Push(90),   // Stack: [90, 90]
        OpCode::Eq,         // Stack: [1]
        OpCode::Return,     // Returns 1
    ];

    let mut vm = VM::new(10000);
    let result = vm.execute(&contract_code).expect("Smart contract execution failed");

    assert_eq!(result.return_value, Some(1), "Expected contract to return 1 (true)");
    assert_eq!(vm.memory.load(5).unwrap(), 90, "Expected memory register 5 to hold 90");
    assert_eq!(result.gas_used, 26, "Expected gas cost of 26 gas units");
}

#[test]
fn test_conditional_jump_contract() {
    // Contract that loops or jumps based on condition
    // Reg 0 = 0
    // If Push(1) != 0, Jump to PC 6 (Load and Return)
    let contract_code = vec![
        OpCode::Push(99),   // PC 0: Stack [99]
        OpCode::Store(0),   // PC 1: Reg 0 = 99, Stack []
        OpCode::Push(1),    // PC 2: Condition = 1 (true), Stack [1]
        OpCode::Push(6),    // PC 3: Target PC = 6, Stack [1, 6]
        OpCode::Swap,       // PC 4: Stack [6, 1]
        OpCode::JumpIf(6),  // PC 5: Jump to PC 6, Stack []
        OpCode::Push(0),    // PC 6 (skipped if jump works)
        OpCode::Load(0),    // PC 6 (jump target): Load Reg 0 (99)
        OpCode::Return,     // PC 7: Return 99
    ];

    let mut vm = VM::new(10000);
    let result = vm.execute(&contract_code).expect("Jump contract execution failed");
    assert_eq!(result.return_value, Some(99));
}
