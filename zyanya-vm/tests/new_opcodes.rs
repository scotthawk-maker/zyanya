use zyanya_vm::{MockStateBackend, OpCode, VM, VMError};

/// The CALLER opcode pushes the verified caller address (u64) set via `VM::set_caller`.
#[test]
fn test_caller_opcode() {
    let mut vm = VM::new(1000);
    vm.set_caller(0xDEAD_BEEF);

    let program = vec![OpCode::Caller, OpCode::Halt];
    let res = vm.execute(&program).expect("execution failed");

    assert_eq!(res.stack_dump, vec![0xDEAD_BEEF], "CALLER should push the injected caller u64");
}

/// CALLER defaults to 0 when no caller was injected.
#[test]
fn test_caller_opcode_default_zero() {
    let mut vm = VM::new(1000);
    let program = vec![OpCode::Caller, OpCode::Halt];
    let res = vm.execute(&program).expect("execution failed");
    assert_eq!(res.stack_dump, vec![0], "CALLER should default to 0");
}

/// BALANCE pushes the contract's own ZYAN balance from the state backend.
#[test]
fn test_balance_opcode() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];
    state.set_balance(&contract_addr, 5_000_000);

    let mut vm = VM::new(1000);
    let program = vec![OpCode::Balance, OpCode::Halt];
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    assert_eq!(res.stack_dump, vec![5_000_000], "BALANCE should push the contract's balance");
}

/// BALANCE returns 0 when the contract has no balance recorded.
#[test]
fn test_balance_opcode_zero() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];

    let mut vm = VM::new(1000);
    let program = vec![OpCode::Balance, OpCode::Halt];
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    assert_eq!(res.stack_dump, vec![0], "BALANCE should push 0 for unbalanced contract");
}

/// WITHDRAW decrements the contract balance and records a withdrawal in VMResult.
#[test]
fn test_withdraw_opcode_success() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];
    state.set_balance(&contract_addr, 10_000);

    // Stack: [recipient, amount] -> WITHDRAW pops amount then recipient
    let program = vec![
        OpCode::Push(100),   // recipient
        OpCode::Push(3_000), // amount
        OpCode::Withdraw,
        OpCode::Halt,
    ];

    let mut vm = VM::new(10_000);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    // Success pushes 1
    assert_eq!(res.stack_dump, vec![1], "WITHDRAW should push 1 on success");
    // Balance decremented
    assert_eq!(state.balance(&contract_addr), 7_000, "contract balance should be decremented");
    // Withdrawal recorded
    assert_eq!(res.withdrawals, vec![(100, 3_000)], "withdrawal should be recorded in VMResult");
}

/// WITHDRAW pushes 0 and records nothing when the contract has insufficient balance.
#[test]
fn test_withdraw_opcode_insufficient_balance() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];
    state.set_balance(&contract_addr, 100);

    let program = vec![
        OpCode::Push(200), // recipient
        OpCode::Push(500), // amount (more than balance)
        OpCode::Withdraw,
        OpCode::Halt,
    ];

    let mut vm = VM::new(10_000);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    assert_eq!(res.stack_dump, vec![0], "WITHDRAW should push 0 on insufficient balance");
    assert_eq!(state.balance(&contract_addr), 100, "balance should be unchanged on failed withdraw");
    assert!(res.withdrawals.is_empty(), "no withdrawal should be recorded on failure");
}

/// WITHDRAW with zero balance is a failure (push 0), not a panic.
#[test]
fn test_withdraw_opcode_no_balance() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];

    let program = vec![
        OpCode::Push(1),
        OpCode::Push(1),
        OpCode::Withdraw,
        OpCode::Halt,
    ];

    let mut vm = VM::new(10_000);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    assert_eq!(res.stack_dump, vec![0], "WITHDRAW should push 0 when contract has no balance");
    assert!(res.withdrawals.is_empty());
}

/// The new opcodes serialize and deserialize correctly (round-trip).
#[test]
fn test_new_opcodes_serialization_roundtrip() {
    let code = vec![
        OpCode::Caller,
        OpCode::Balance,
        OpCode::Push(42),
        OpCode::Withdraw,
        OpCode::Halt,
    ];

    let bytes = OpCode::serialize_slice(&code);
    let decoded = OpCode::deserialize_slice(&bytes).expect("deserialization failed");

    assert_eq!(code, decoded, "round-trip should preserve the new opcodes");
}

/// CALLER + WITHDRAW combined: a contract reads its caller and withdraws to it.
#[test]
fn test_caller_and_withdraw_combined() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x99u8; 32];
    state.set_balance(&contract_addr, 50_000);

    // Stack: [recipient (caller), amount (50_000)] -> Withdraw pops amount then recipient.
    let program = vec![
        OpCode::Caller,       // stack: [caller]
        OpCode::Push(50_000), // stack: [caller, 50_000]
        OpCode::Withdraw,     // pop amount=50_000, pop recipient=caller, withdraw
        OpCode::Halt,
    ];

    let mut vm = VM::new(10_000);
    vm.set_caller(777);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");

    assert_eq!(res.stack_dump, vec![1], "withdraw should succeed");
    assert_eq!(state.balance(&contract_addr), 0, "contract balance should be zeroed");
    assert_eq!(res.withdrawals, vec![(777, 50_000)], "withdrawal to caller recorded");
}

/// WITHDRAW gas cost is 10; ensure gas is deducted.
#[test]
fn test_withdraw_gas_cost() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];
    state.set_balance(&contract_addr, 10_000);

    let program = vec![
        OpCode::Push(1),
        OpCode::Push(1),
        OpCode::Withdraw,
    ];

    // Exact gas: Push(2) + Push(2) + Withdraw(10) = 14
    let mut vm = VM::new(14);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");
    assert_eq!(res.gas_used, 14);

    // One unit short -> out of gas
    let mut vm2 = VM::new(13);
    let res2 = vm2.execute_stateful(&program, &contract_addr, &mut state);
    assert_eq!(res2, Err(VMError::OutOfGas { limit: 13, requested: 14 }));
}

/// BALANCE gas cost is 3; CALLER gas cost is 1.
#[test]
fn test_balance_and_caller_gas_cost() {
    let mut state = MockStateBackend::new();
    let contract_addr = [0x42u8; 32];
    state.set_balance(&contract_addr, 1_000);

    let program = vec![OpCode::Caller, OpCode::Balance];
    // Gas: Caller(1) + Balance(3) = 4
    let mut vm = VM::new(4);
    let res = vm.execute_stateful(&program, &contract_addr, &mut state).expect("execution failed");
    assert_eq!(res.gas_used, 4);

    // One unit short -> out of gas
    let mut vm2 = VM::new(3);
    let res2 = vm2.execute_stateful(&program, &contract_addr, &mut state);
    assert_eq!(res2, Err(VMError::OutOfGas { limit: 3, requested: 4 }));
}