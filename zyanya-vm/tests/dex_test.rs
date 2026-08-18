use zyanya_vm::{Compiler, MockStateBackend, OpCode, VM};

#[test]
fn test_dex_contract_full_lifecycle() {
    let dex_zcl = include_str!("../../dex.zcl");

    let bytecode = Compiler::compile(dex_zcl).expect("DEX compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("DEX opcode deserialization failed");

    let mut state = MockStateBackend::new();
    let dex_addr = [0xddu8; 32];
    let caller = 10u64;

    // 0. Deploy Initialization (Entry Point 0: init)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // entry point 0
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).expect("init failed");
    assert_eq!(res.return_value, Some(0));

    // 1. Initial State Check
    assert_eq!(state.get(&dex_addr, 0), 0, "reserveA should be 0");
    assert_eq!(state.get(&dex_addr, 1), 0, "reserveB should be 0");
    assert_eq!(state.get(&dex_addr, 2), 0, "totalLPSupply should be 0");

    // 2. Add Liquidity: 1000 ZYAN (Token A) + 1000 GHOST (Token B)
    // Entry Point 1: addLiquidity(caller, amountA, amountB)
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap(); // amountB
    vm.stack.push(1000).unwrap(); // amountA
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(1).unwrap();    // entry point 1
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).expect("addLiquidity failed");
    assert_eq!(res.return_value, Some(2000), "Initial LP minted should be 1000+1000 = 2000");

    assert_eq!(state.get(&dex_addr, 0), 1000, "reserveA updated to 1000");
    assert_eq!(state.get(&dex_addr, 1), 1000, "reserveB updated to 1000");
    assert_eq!(state.get(&dex_addr, 2), 2000, "totalLPSupply updated to 2000");
    assert_eq!(state.get(&dex_addr, caller + 10), 2000, "Caller LP balance updated to 2000");

    // 3. Query getReserves (Entry Point 4) & getReserveB (Entry Point 5) & getPrice (Entry Point 6)
    let mut vm = VM::new(100_000);
    vm.stack.push(4).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1000), "reserveA is 1000");

    let mut vm = VM::new(100_000);
    vm.stack.push(5).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1000), "reserveB is 1000");

    let mut vm = VM::new(100_000);
    vm.stack.push(6).unwrap();
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1000), "Price ratio (1000 * 1000 / 1000) = 1000");

    // 4. Swap 100 Token A (ZYAN) for Token B (GHOST)
    // Entry Point 2: swap(tokenIn, amountIn)
    // tokenIn = 0
    let mut vm = VM::new(100_000);
    vm.stack.push(100).unwrap(); // amountIn = 100
    vm.stack.push(0).unwrap();   // tokenIn = 0 (Token A)
    vm.stack.push(2).unwrap();   // entry point 2
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).expect("swap failed");
    assert_eq!(res.return_value, Some(90), "Amount out should be 90 GHOST");

    assert_eq!(state.get(&dex_addr, 0), 1100, "reserveA increased to 1100");
    assert_eq!(state.get(&dex_addr, 1), 910, "reserveB decreased to 910");

    // 5. Remove Liquidity: burn 500 LP tokens
    // Entry Point 3: removeLiquidity(caller, lpAmount)
    let mut vm = VM::new(100_000);
    vm.stack.push(500).unwrap();   // lpAmount = 500
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(3).unwrap();      // entry point 3
    let res = vm.execute_stateful(&opcodes, &dex_addr, &mut state).expect("removeLiquidity failed");
    assert_eq!(res.return_value, Some(275), "Withdrawn amountA should be 500 * 1100 / 2000 = 275");

    assert_eq!(state.get(&dex_addr, 0), 825, "reserveA reduced from 1100 to 825");
    assert_eq!(state.get(&dex_addr, 1), 683, "reserveB reduced from 910 to 683 (910 - 227)");
    assert_eq!(state.get(&dex_addr, 2), 1500, "totalLPSupply reduced from 2000 to 1500");
    assert_eq!(state.get(&dex_addr, caller + 10), 1500, "Caller LP balance reduced to 1500");
}
