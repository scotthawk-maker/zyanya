use zyanya_vm::{compiler::Compiler, opcode::OpCode, MockStateBackend, VM};

#[test]
fn test_multi_hop_router_2_hop_swap() {
    let staking_zcl = include_str!("../../staking.zcl");
    let dex_zcl = include_str!("../../dex.zcl");
    let router_zcl = include_str!("../../router.zcl");

    let staking_bytecode = Compiler::compile(staking_zcl).unwrap();
    let dex_bytecode = Compiler::compile(dex_zcl).unwrap();
    let router_bytecode = Compiler::compile(router_zcl).unwrap();

    let staking_opcodes = OpCode::deserialize_slice(&staking_bytecode).unwrap();
    let dex_opcodes = OpCode::deserialize_slice(&dex_bytecode).unwrap();
    let router_opcodes = OpCode::deserialize_slice(&router_bytecode).unwrap();

    let mut state = MockStateBackend::new();

    let staking_addr = [0x54u8; 32];
    let pool1_addr = [0x11u8; 32]; // Pool 1: Token A (ZYAN) <-> Token B (GHOST)
    let pool2_addr = [0x22u8; 32]; // Pool 2: Token B (GHOST) <-> Token C (SHADOW)
    let router_addr = [0x33u8; 32];

    state.set_code(staking_addr, staking_bytecode);
    state.set_code(pool1_addr, dex_bytecode.clone());
    state.set_code(pool2_addr, dex_bytecode);
    state.set_code(router_addr, router_bytecode.clone());

    let lp_provider = 100u64;

    // --- 1. Init Contracts ---
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();

    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&dex_opcodes, &pool1_addr, &mut state).unwrap();

    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&dex_opcodes, &pool2_addr, &mut state).unwrap();

    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&router_opcodes, &router_addr, &mut state).unwrap();

    // --- 2. Add Liquidity to Pool 1 (50,000 ZYAN + 50,000 GHOST) ---
    let mut vm = VM::new(100_000);
    vm.stack.push(50000).unwrap(); // amountB
    vm.stack.push(50000).unwrap(); // amountA
    vm.stack.push(lp_provider).unwrap(); // caller
    vm.stack.push(1).unwrap();     // EP 1: addLiquidity
    vm.execute_stateful(&dex_opcodes, &pool1_addr, &mut state).expect("Pool 1 liquidity failed");

    // --- 3. Add Liquidity to Pool 2 (50,000 GHOST + 50,000 SHADOW) ---
    let mut vm = VM::new(100_000);
    vm.stack.push(50000).unwrap(); // amountB
    vm.stack.push(50000).unwrap(); // amountA
    vm.stack.push(lp_provider).unwrap(); // caller
    vm.stack.push(1).unwrap();     // EP 1: addLiquidity
    vm.execute_stateful(&dex_opcodes, &pool2_addr, &mut state).expect("Pool 2 liquidity failed");

    // --- 4. Perform 2-Hop Swap via Router: 10,000 ZYAN -> GHOST (Pool 1) -> SHADOW (Pool 2) ---
    // Parameter order: (pool1, pool2, token1, token2, amountIn, minAmountOut)
    // VM LIFO stack push order (reverse): minAmountOut, amountIn, token2, token1, pool2, pool1, EP
    let pool1_u64 = pool1_addr[0] as u64;
    let pool2_u64 = pool2_addr[0] as u64;

    let mut vm = VM::new(500_000);
    vm.stack.push(1000).unwrap();      // minAmountOut = 1000
    vm.stack.push(10000).unwrap();     // amountIn = 10000
    vm.stack.push(1).unwrap();         // token2 = 1 (GHOST tokenIn on Pool 2)
    vm.stack.push(0).unwrap();         // token1 = 0 (ZYAN tokenIn on Pool 1)
    vm.stack.push(pool2_u64).unwrap();  // pool2
    vm.stack.push(pool1_u64).unwrap();  // pool1
    vm.stack.push(1).unwrap();         // EP 1: swap2Hop

    let router_asm = zyanya_vm::assembler::Assembler::disassemble(&router_bytecode).unwrap();
    println!("--- ROUTER ASM ---\n{}", router_asm);

    let res = vm.execute_stateful(&router_opcodes, &router_addr, &mut state).expect("2-hop swap failed");
    let final_out = res.return_value.expect("Should return swap output");
    println!("--- RETURN VALUE ---: {:?}", final_out);

    assert!(final_out > 5000, "2-hop swap should yield significant SHADOW output, got {}", final_out);

    // --- 5. Verify Slippage Protection ---
    // Asking for impossibly high output (999,999 SHADOW) should revert (return 0)
    let mut vm = VM::new(500_000);
    vm.stack.push(999999).unwrap();   // minAmountOut = 999999 (impossible)
    vm.stack.push(10000).unwrap();
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap();
    vm.stack.push(pool2_u64).unwrap();
    vm.stack.push(pool1_u64).unwrap();
    vm.stack.push(1).unwrap();

    let res = vm.execute_stateful(&router_opcodes, &router_addr, &mut state).expect("Execution ok");
    assert_eq!(res.return_value, Some(0), "Slippage check failure should return 0");
}

#[test]
fn test_multi_hop_router_3_hop_swap() {
    let staking_zcl = include_str!("../../staking.zcl");
    let dex_zcl = include_str!("../../dex.zcl");
    let router_zcl = include_str!("../../router.zcl");

    let staking_bytecode = Compiler::compile(staking_zcl).unwrap();
    let dex_bytecode = Compiler::compile(dex_zcl).unwrap();
    let router_bytecode = Compiler::compile(router_zcl).unwrap();

    let staking_opcodes = OpCode::deserialize_slice(&staking_bytecode).unwrap();
    let dex_opcodes = OpCode::deserialize_slice(&dex_bytecode).unwrap();
    let router_opcodes = OpCode::deserialize_slice(&router_bytecode).unwrap();

    let mut state = MockStateBackend::new();

    let staking_addr = [0x54u8; 32];
    let pool1_addr = [0x11u8; 32]; // ZYAN <-> GHOST
    let pool2_addr = [0x22u8; 32]; // GHOST <-> SHADOW
    let pool3_addr = [0x44u8; 32]; // SHADOW <-> ECLIPSE
    let router_addr = [0x33u8; 32];

    state.set_code(staking_addr, staking_bytecode);
    state.set_code(pool1_addr, dex_bytecode.clone());
    state.set_code(pool2_addr, dex_bytecode.clone());
    state.set_code(pool3_addr, dex_bytecode);
    state.set_code(router_addr, router_bytecode);

    let lp_provider = 100u64;

    // Init Staking, Pools & Router
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&staking_opcodes, &staking_addr, &mut state).unwrap();

    for addr in &[pool1_addr, pool2_addr, pool3_addr, router_addr] {
        let mut vm = VM::new(100_000);
        vm.stack.push(0).unwrap();
        let ops = if *addr == router_addr { &router_opcodes } else { &dex_opcodes };
        vm.execute_stateful(ops, addr, &mut state).unwrap();
    }

    // Add Liquidity 100,000 to all 3 pools
    for addr in &[pool1_addr, pool2_addr, pool3_addr] {
        let mut vm = VM::new(100_000);
        vm.stack.push(100000).unwrap();
        vm.stack.push(100000).unwrap();
        vm.stack.push(lp_provider).unwrap();
        vm.stack.push(1).unwrap();
        vm.execute_stateful(&dex_opcodes, addr, &mut state).unwrap();
    }

    let pool1_u64 = pool1_addr[0] as u64;
    let pool2_u64 = pool2_addr[0] as u64;
    let pool3_u64 = pool3_addr[0] as u64;

    // 3-hop swap: 10,000 ZYAN -> GHOST -> SHADOW -> ECLIPSE
    // Parameter order: (pool1, pool2, pool3, token1, token2, token3, amountIn, minAmountOut)
    // VM LIFO stack push order (reverse): minAmountOut, amountIn, token3, token2, token1, pool3, pool2, pool1, EP
    let mut vm = VM::new(1_000_000);
    vm.stack.push(1000).unwrap();      // minAmountOut = 1000
    vm.stack.push(10000).unwrap();     // amountIn = 10000
    vm.stack.push(1).unwrap();         // token3 = 1
    vm.stack.push(1).unwrap();         // token2 = 1
    vm.stack.push(0).unwrap();         // token1 = 0
    vm.stack.push(pool3_u64).unwrap();  // pool3
    vm.stack.push(pool2_u64).unwrap();  // pool2
    vm.stack.push(pool1_u64).unwrap();  // pool1
    vm.stack.push(2).unwrap();         // EP 2: swap3Hop

    let res = vm.execute_stateful(&router_opcodes, &router_addr, &mut state).expect("3-hop swap execution failed");
    let final_out = res.return_value.expect("Should return swap output");

    assert!(final_out > 4000, "3-hop swap should yield ECLIPSE output, got {}", final_out);
}
