use zyanya_vm::{MockStateBackend, OpCode, VM};
use zyanya_vm::bonding_curve_token::bonding_curve_bytecode;

#[test]
fn test_buy_after_graduation_rejected() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x11u8; 32];
    let caller = 100u64;

    // Initialize contract (slope = 1)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap(); // entry point 0
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // Set phase = 2 (graduated to AMM mode)
    state.set(&addr, 3, 2);

    // Attempt buy after graduation (entry point 4)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(500).unwrap();   // tokens_to_mint
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(4).unwrap();      // entry point 4
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Buying after graduation must be rejected (return 0)");
}

#[test]
fn test_sell_after_graduation_rejected() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x22u8; 32];
    let caller = 100u64;

    // Initialize contract (slope = 1)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap(); // entry point 0
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // Give caller 1,000 tokens
    state.set(&addr, caller, 1_000);

    // Set phase = 2 (graduated to AMM mode)
    state.set(&addr, 3, 2);

    // Attempt sell after graduation (entry point 5)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(100).unwrap();    // tokens_in
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(5).unwrap();      // entry point 5
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Selling after graduation must be rejected (return 0)");
}

#[test]
fn test_add_liquidity_after_graduation_rejected() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x33u8; 32];
    let caller = 100u64;

    // Initialize contract (slope = 1)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap(); // entry point 0
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // Set phase = 2 (graduated to AMM mode)
    state.set(&addr, 3, 2);

    // 1. Bonding curve buy / liquidity minting attempt (EP 4) after graduation -> returns 0
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1000).unwrap();
    vm.stack.push(caller).unwrap();
    vm.stack.push(4).unwrap();
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Adding bonding curve liquidity via buy after graduation must return 0");

    // 2. Unsupported / invalid entry point attempt after graduation -> returns 0
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1000).unwrap();
    vm.stack.push(caller).unwrap();
    vm.stack.push(99).unwrap(); // unhandled entry point
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Invalid entry point after graduation must return 0");
}

#[test]
fn test_amm_swap_after_graduation_works() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x44u8; 32];
    let caller = 100u64;

    // Initialize contract (slope = 1)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap(); // entry point 0
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // Set up graduated AMM state (phase 2)
    let reserve = 1_000_000_000u64;
    let supply = 45_000u64;
    let k = reserve.saturating_mul(supply);

    state.set(&addr, 3, 2);        // phase = 2
    state.set(&addr, 4, reserve);  // x_reserve
    state.set(&addr, 5, supply);   // y_supply
    state.set(&addr, 6, k);        // k

    // 1. Swap X to Y (ZYAN in -> Tokens out)
    let zyan_in = 1_000_000u64;
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();        // is_x_to_y = 1
    vm.stack.push(zyan_in).unwrap();   // token_in_amount
    vm.stack.push(caller).unwrap();    // caller
    vm.stack.push(7).unwrap();         // entry point 7 (amm_swap)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    let tokens_out = res.return_value.expect("AMM swap X->Y should return tokens_out");
    assert!(tokens_out > 0, "Tokens out must be > 0");

    // 2. Swap Y to X (Tokens in -> ZYAN out)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(0).unwrap();          // is_x_to_y = 0
    vm.stack.push(tokens_out).unwrap(); // token_in_amount
    vm.stack.push(caller).unwrap();     // caller
    vm.stack.push(7).unwrap();          // entry point 7 (amm_swap)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    let zyan_out = res.return_value.expect("AMM swap Y->X should return zyan_out");
    assert!(zyan_out > 0, "ZYAN out must be > 0");
}

#[test]
fn test_graduation_threshold_configurable() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x55u8; 32];

    // 1. Initialize contract (slope = 1)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // 2. Check default graduation threshold (EP 9) -> 1_000_000_000
    let mut vm = VM::new(10_000_000);
    vm.stack.push(9).unwrap();
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1_000_000_000), "Default threshold must be 1_000_000_000");

    // 3. Set custom graduation threshold to 500_000_000 (EP 8)
    let custom_threshold = 500_000_000u64;
    let mut vm = VM::new(10_000_000);
    vm.stack.push(custom_threshold).unwrap();
    vm.stack.push(8).unwrap();
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1), "setGraduationThreshold must return 1");

    // 4. Query threshold again (EP 9)
    let mut vm = VM::new(10_000_000);
    vm.stack.push(9).unwrap();
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(custom_threshold), "Configured threshold must be 500_000_000");

    // Verify storage key 7 directly
    assert_eq!(state.get(&addr, 7), custom_threshold, "Storage key 7 must match configured threshold");
}
