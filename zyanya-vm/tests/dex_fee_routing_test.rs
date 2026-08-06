use zyanya_vm::{Compiler, MockStateBackend, OpCode, VM};

#[test]
fn test_dex_fee_routing_to_stakers_full_lifecycle() {
    let staking_zcl = include_str!("../../staking.zcl");
    let dex_zcl = include_str!("../../dex.zcl");

    let staking_bytecode = Compiler::compile(staking_zcl).expect("Staking compilation failed");
    let dex_bytecode = Compiler::compile(dex_zcl).expect("DEX compilation failed");

    let staking_opcodes = OpCode::deserialize_slice(&staking_bytecode).expect("Staking opcodes deserialization failed");
    let dex_opcodes = OpCode::deserialize_slice(&dex_bytecode).expect("DEX opcodes deserialization failed");

    let mut state = MockStateBackend::new();
    let staking_addr = [0x54u8; 32];
    let dex_addr = [0xddu8; 32];

    // Register contract bytecodes in state so inter-contract calls succeed
    state.set_code(staking_addr, staking_bytecode);
    state.set_code(dex_addr, dex_bytecode);

    let alice = 100u64;
    let bob = 200u64;
    let lp_provider = 10u64;
    let trader = 300u64;

    // --- 0. Initialize both contracts ---
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // EP 0: init
    vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).expect("Staking init failed");

    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // EP 0: init
    vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).expect("DEX init failed");

    // Verify initial fee share in DEX is 50%
    let mut vm = VM::new(100_000);
    vm.stack.push(10).unwrap(); // EP 10: getFeeShare
    let res = vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(50), "Default DEX fee share to stakers should be 50%");

    // --- 1. Stakers stake ZYAN in Staking Contract ---
    // Alice stakes 1,000 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap(); // amount = 1000
    vm.stack.push(alice).unwrap(); // caller = Alice
    vm.stack.push(1).unwrap();    // EP 1: stake
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).expect("Alice stake failed");
    assert_eq!(res.return_value, Some(1000), "Alice staked 1000 ZYAN");

    // Bob stakes 3,000 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(3000).unwrap(); // amount = 3000
    vm.stack.push(bob).unwrap();  // caller = Bob
    vm.stack.push(1).unwrap();    // EP 1: stake
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).expect("Bob stake failed");
    assert_eq!(res.return_value, Some(3000), "Bob staked 3000 ZYAN");

    // Verify total staked in pool = 4000 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(6).unwrap(); // EP 6: getTotalStaked
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(4000), "Total staked should be 4000 ZYAN");

    // Verify total rewards distributed initially = 0
    let mut vm = VM::new(100_000);
    vm.stack.push(7).unwrap(); // EP 7: getTotalRewards
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Initial total rewards distributed should be 0");

    // --- 2. Add Liquidity to DEX ---
    // LP Provider adds 10,000 ZYAN + 10,000 GHOST
    let mut vm = VM::new(100_000);
    vm.stack.push(10000).unwrap(); // amountB = 10000
    vm.stack.push(10000).unwrap(); // amountA = 10000
    vm.stack.push(lp_provider).unwrap(); // caller
    vm.stack.push(1).unwrap();     // EP 1: addLiquidity
    let res = vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).expect("addLiquidity failed");
    assert_eq!(res.return_value, Some(20000), "Initial LP minted = 20000");

    // --- 3. Trader performs DEX Swap ---
    // Swap 10,000 ZYAN (Token A) for GHOST (Token B)
    // 0.3% total fee = 30 ZYAN.
    // 50% split to stakers = 15 ZYAN routed to Staking contract depositRewards.
    let mut vm = VM::new(100_000);
    vm.stack.push(10000).unwrap(); // amountIn = 10000
    vm.stack.push(0).unwrap();     // tokenIn = 0 (ZYAN)
    vm.stack.push(2).unwrap();     // EP 2: swap
    vm.set_caller(trader);
    let res = vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).expect("swap failed");
    assert!(res.return_value.unwrap() > 0, "Swap should yield Token B output");

    // --- 4. Verify Fee Routing & Staker Rewards ---
    // Verify DEX total fees routed counter (EP 11) = 15
    let mut vm = VM::new(100_000);
    vm.stack.push(11).unwrap(); // EP 11: getTotalFeesRouted
    let res = vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(15), "DEX should record 15 ZYAN routed to stakers");

    // Verify Staking Contract total rewards distributed (EP 7) = 15
    let mut vm = VM::new(100_000);
    vm.stack.push(7).unwrap(); // EP 7: getTotalRewards
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(15), "Staking contract total rewards distributed should be 15 ZYAN");

    // Verify Alice pending rewards (EP 8): 1000 / 4000 = 25% share of 15 = 3 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap(); // caller = Alice
    vm.stack.push(8).unwrap();     // EP 8: getPendingRewards
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(3), "Alice pending rewards should be 3 ZYAN (25% of 15)");

    // Verify Bob pending rewards (EP 8): 3000 / 4000 = 75% share of 15 = 11 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(bob).unwrap(); // caller = Bob
    vm.stack.push(8).unwrap();   // EP 8: getPendingRewards
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(11), "Bob pending rewards should be 11 ZYAN (75% of 15)");

    // --- 5. Alice Claims Rewards ---
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap(); // caller = Alice
    vm.stack.push(4).unwrap();     // EP 4: claimRewards
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).expect("Alice claim failed");
    assert_eq!(res.return_value, Some(3), "Alice should claim 3 ZYAN rewards");

    // Second claim yields 0
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap();
    vm.stack.push(4).unwrap();
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Alice second claim should yield 0");
}

#[test]
fn test_dex_custom_fee_share_routing() {
    let staking_zcl = include_str!("../../staking.zcl");
    let dex_zcl = include_str!("../../dex.zcl");

    let staking_bytecode = Compiler::compile(staking_zcl).unwrap();
    let dex_bytecode = Compiler::compile(dex_zcl).unwrap();

    let staking_opcodes = OpCode::deserialize_slice(&staking_bytecode).unwrap();
    let dex_opcodes = OpCode::deserialize_slice(&dex_bytecode).unwrap();

    let mut state = MockStateBackend::new();
    let staking_addr = [0x54u8; 32];
    let dex_addr = [0xddu8; 32];

    state.set_code(staking_addr, staking_bytecode);
    state.set_code(dex_addr, dex_bytecode);

    let alice = 100u64;

    // Initialize
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();

    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();

    // Alice stakes 10,000 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(10000).unwrap();
    vm.stack.push(alice).unwrap();
    vm.stack.push(1).unwrap();
    vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();

    // Set fee share to 100% (EP 9: setFeeShare)
    let mut vm = VM::new(100_000);
    vm.stack.push(100).unwrap(); // share = 100%
    vm.stack.push(9).unwrap();   // EP 9
    let res = vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(100), "Fee share set to 100%");

    // Add Liquidity
    let mut vm = VM::new(100_000);
    vm.stack.push(100000).unwrap();
    vm.stack.push(100000).unwrap();
    vm.stack.push(10).unwrap();
    vm.stack.push(1).unwrap();
    vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();

    // Swap 10,000 ZYAN -> 0.3% fee = 30 ZYAN. 100% split = 30 ZYAN to stakers
    let mut vm = VM::new(100_000);
    vm.stack.push(10000).unwrap();
    vm.stack.push(0).unwrap();
    vm.stack.push(2).unwrap();
    vm.execute_stateful(&dex_opcodes, &dex_addr, &mut state).unwrap();

    // Verify 30 ZYAN routed to totalRewards
    let mut vm = VM::new(100_000);
    vm.stack.push(7).unwrap();
    let res = vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(30), "100% fee split should route 30 ZYAN rewards to stakers");
}
