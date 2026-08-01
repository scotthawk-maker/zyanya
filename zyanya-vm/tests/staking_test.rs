use zyanya_vm::{Compiler, MockStateBackend, OpCode, VM};

#[test]
fn test_staking_contract_full_lifecycle() {
    let staking_zcl = include_str!("../../staking.zcl");

    let bytecode = Compiler::compile(staking_zcl).expect("Staking compilation failed");
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Staking opcode deserialization failed");

    let mut state = MockStateBackend::new();
    let staking_addr = [0x54u8; 32];
    let alice = 100u64;
    let bob = 200u64;

    // 0. Init contract (Entry Point 0)
    let mut vm = VM::new(100_000);
    vm.stack.push(0).unwrap(); // entry point 0
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("init failed");
    assert_eq!(res.return_value, Some(0));

    // Initial state checks (Entry Point 6 & 7)
    let mut vm = VM::new(100_000);
    vm.stack.push(6).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "totalStaked initially 0");

    // 1. Alice stakes 1,000 ZYAN (Entry Point 1)
    let mut vm = VM::new(100_000);
    vm.stack.push(1000).unwrap(); // amount = 1000
    vm.stack.push(alice).unwrap(); // caller = Alice
    vm.stack.push(1).unwrap();    // entry point 1
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("Alice stake failed");
    assert_eq!(res.return_value, Some(1000), "Alice staked balance should be 1000");

    // 2. Bob stakes 3,000 ZYAN (Entry Point 1)
    let mut vm = VM::new(100_000);
    vm.stack.push(3000).unwrap(); // amount = 3000
    vm.stack.push(bob).unwrap();  // caller = Bob
    vm.stack.push(1).unwrap();    // entry point 1
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("Bob stake failed");
    assert_eq!(res.return_value, Some(3000), "Bob staked balance should be 3000");

    // Check Total Staked (Entry Point 6) -> 4000
    let mut vm = VM::new(100_000);
    vm.stack.push(6).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(4000), "Total staked should be 4000");

    // 3. Deposit 400 ZYAN Protocol Rewards (Entry Point 3)
    let mut vm = VM::new(100_000);
    vm.stack.push(400).unwrap(); // amount = 400
    vm.stack.push(3).unwrap();   // entry point 3
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("depositRewards failed");
    assert_eq!(res.return_value, Some(400), "Total rewards distributed should be 400");

    // 4. Check Pending Rewards (Entry Point 8)
    // Alice has 1000 / 4000 = 25% share -> 100 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap();
    vm.stack.push(8).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(100), "Alice pending rewards should be 100");

    // Bob has 3000 / 4000 = 75% share -> 300 ZYAN
    let mut vm = VM::new(100_000);
    vm.stack.push(bob).unwrap();
    vm.stack.push(8).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(300), "Bob pending rewards should be 300");

    // 5. Alice claims rewards (Entry Point 4)
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap();
    vm.stack.push(4).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("Alice claim failed");
    assert_eq!(res.return_value, Some(100), "Alice claimed 100 ZYAN rewards");

    // Second claim by Alice should yield 0
    let mut vm = VM::new(100_000);
    vm.stack.push(alice).unwrap();
    vm.stack.push(4).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Second claim should yield 0");

    // 6. Alice unstakes 500 ZYAN (Entry Point 2)
    let mut vm = VM::new(100_000);
    vm.stack.push(500).unwrap();   // amount = 500
    vm.stack.push(alice).unwrap(); // caller = Alice
    vm.stack.push(2).unwrap();     // entry point 2
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).expect("Alice unstake failed");
    assert_eq!(res.return_value, Some(500), "Alice remaining stake should be 500");

    // Total staked is now 3500
    let mut vm = VM::new(100_000);
    vm.stack.push(6).unwrap();
    let res = vm.execute_stateful(&opcodes, &staking_addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(3500), "Total staked updated to 3500");
}
