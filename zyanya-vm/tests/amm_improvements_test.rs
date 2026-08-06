use zyanya_vm::{Compiler, MockStateBackend, OpCode, VM};

#[test]
fn test_lp_token_transfer() {
    let dex_zcl = include_str!("../../dex.zcl");
    let bytecode = Compiler::compile(dex_zcl).expect("DEX compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("DEX opcode deserialization failed");

    let mut state = MockStateBackend::new();
    let dex_addr = [0xddu8; 32];
    let user1 = 10u64;
    let user2 = 20u64;

    // Init contract (EP 0)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0));

    // Add liquidity for user1: 1000 ZYAN + 1000 GHOST -> 2000 LP tokens minted
    // EP 1: addLiquidity(caller, amountA, amountB)
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap(); // amountB
    vm.stack.push(1000).unwrap(); // amountA
    vm.stack.push(user1).unwrap(); // caller
    vm.stack.push(1).unwrap();     // EP 1
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(2000));
    assert_eq!(state.get(&dex_addr, user1 + 10), 2000, "user1 should have 2000 LP tokens");
    assert_eq!(state.get(&dex_addr, user2 + 10), 0, "user2 should have 0 LP tokens");

    // EP 13: transferLP(caller, to, amount)
    // Transfer 500 LP tokens from user1 (10) to user2 (20)
    // VM LIFO stack push order: amount, to, caller, EP
    let mut vm = VM::new(100_000);
    vm.stack.push(500).unwrap();   // amount = 500
    vm.stack.push(user2).unwrap(); // to = 20
    vm.stack.push(user1).unwrap(); // caller = 10
    vm.stack.push(13).unwrap();    // EP 13
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1), "transferLP should return 1 on success");

    assert_eq!(state.get(&dex_addr, user1 + 10), 1500, "user1 LP balance should be 1500 after transfer");
    assert_eq!(state.get(&dex_addr, user2 + 10), 500, "user2 LP balance should be 500 after transfer");

    // Attempt to transfer more than balance (e.g. user1 transfers 2000 LP when balance is 1500)
    let mut vm = VM::new(100_000);
    vm.stack.push(2000).unwrap();  // amount = 2000
    vm.stack.push(user2).unwrap(); // to = 20
    vm.stack.push(user1).unwrap(); // caller = 10
    vm.stack.push(13).unwrap();    // EP 13
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "transferLP should return 0 on insufficient balance");

    // Balances remain unchanged
    assert_eq!(state.get(&dex_addr, user1 + 10), 1500);
    assert_eq!(state.get(&dex_addr, user2 + 10), 500);

    // user2 transfers 200 LP tokens back to user1
    let mut vm = VM::new(100_000);
    vm.stack.push(200).unwrap();   // amount = 200
    vm.stack.push(user1).unwrap(); // to = 10
    vm.stack.push(user2).unwrap(); // caller = 20
    vm.stack.push(13).unwrap();    // EP 13
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1));

    assert_eq!(state.get(&dex_addr, user1 + 10), 1700);
    assert_eq!(state.get(&dex_addr, user2 + 10), 300);

    // Query EP 7 getLiquidity for user1 & user2
    let mut vm = VM::new(100_000);
    vm.stack.push(user1).unwrap();
    vm.stack.push(7).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1700));

    let mut vm = VM::new(100_000);
    vm.stack.push(user2).unwrap();
    vm.stack.push(7).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(300));
}

#[test]
fn test_reentrancy_guard() {
    let dex_zcl = include_str!("../../dex.zcl");
    let bytecode = Compiler::compile(dex_zcl).expect("DEX compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("DEX opcode deserialization failed");

    let mut state = MockStateBackend::new();
    let dex_addr = [0xddu8; 32];
    let caller = 10u64;

    // Init contract (EP 0)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(state.get(&dex_addr, 99), 0, "Reentrancy lock should be 0 after init");

    // Add initial liquidity when lock is 0
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap(); // amountB
    vm.stack.push(1000).unwrap(); // amountA
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(1).unwrap();     // EP 1
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(2000));
    assert_eq!(state.get(&dex_addr, 99), 0, "Reentrancy lock should be reset to 0 on exit");

    // Manually set lock flag (key 99) to 1 to simulate mid-execution reentrant state
    state.set(&dex_addr, 99, 1);

    // 1. addLiquidity should be blocked and return 0
    let mut vm = VM::new(100_000);
    vm.stack.push(500).unwrap();
    vm.stack.push(500).unwrap();
    vm.stack.push(caller).unwrap();
    vm.stack.push(1).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "addLiquidity should return 0 when reentrancy lock is 1");
    assert_eq!(state.get(&dex_addr, 0), 1000, "Reserves should remain unchanged");

    // 2. swap should be blocked and return 0
    let mut vm = VM::new(100_000);
    vm.stack.push(100).unwrap(); // amountIn
    vm.stack.push(0).unwrap();   // tokenIn
    vm.stack.push(2).unwrap();   // EP 2
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "swap should return 0 when reentrancy lock is 1");

    // 3. removeLiquidity should be blocked and return 0
    let mut vm = VM::new(100_000);
    vm.stack.push(500).unwrap();   // lpAmount
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(3).unwrap();      // EP 3
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "removeLiquidity should return 0 when reentrancy lock is 1");

    // Unlock key 99 and verify normal operations succeed again
    state.set(&dex_addr, 99, 0);

    let mut vm = VM::new(100_000);
    vm.stack.push(100).unwrap(); // amountIn
    vm.stack.push(0).unwrap();   // tokenIn
    vm.stack.push(2).unwrap();   // EP 2
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(90), "swap should succeed when lock is reset to 0");
    assert_eq!(state.get(&dex_addr, 99), 0, "Lock is 0 after successful swap");
}

#[test]
fn test_price_oracle() {
    let dex_zcl = include_str!("../../dex.zcl");
    let bytecode = Compiler::compile(dex_zcl).expect("DEX compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("DEX opcode deserialization failed");

    let mut state = MockStateBackend::new();
    let dex_addr = [0xddu8; 32];
    let caller = 10u64;

    // Init contract (EP 0)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();

    // Verify initial price oracle is 0 (key 5)
    let mut vm = VM::new(100_000);
    vm.stack.push(12).unwrap(); // EP 12: getPriceOracle
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Initial price oracle should be 0");
    assert_eq!(state.get(&dex_addr, 5), 0);

    // Add asymmetric liquidity: 100,000 Token A + 1,000 Token B
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap();   // amountB
    vm.stack.push(100000).unwrap(); // amountA
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(1).unwrap();      // EP 1
    vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();

    // Swap 10 Token B for Token A (tokenIn = 1, amountIn = 10)
    // reserveA = 100,000, reserveB = 1,000
    // num = 100,000 * 10 * 997 = 997,000,000
    // den = 1,000 * 1000 + 10 * 997 = 1,009,970
    // amountOut = 997,000,000 / 1,009,970 = 987
    // Price oracle = amountOut / amountIn = 987 / 10 = 98
    let mut vm = VM::new(100_000);
    vm.stack.push(10).unwrap(); // amountIn = 10
    vm.stack.push(1).unwrap();  // tokenIn = 1 (Token B)
    vm.stack.push(2).unwrap();  // EP 2: swap
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    let amount_out = res.return_value.expect("Swap should return amountOut");
    assert_eq!(amount_out, 987);

    // Read price oracle slot key 5 directly from state and via EP 12 getPriceOracle
    assert_eq!(state.get(&dex_addr, 5), 98, "Price oracle storage slot key 5 should be 98 (987 / 10)");

    let mut vm = VM::new(100_000);
    vm.stack.push(12).unwrap(); // EP 12: getPriceOracle
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(98), "getPriceOracle EP 12 should return 98");
}
