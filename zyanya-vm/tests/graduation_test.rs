use zyanya_vm::{MockStateBackend, OpCode, VM};
use zyanya_vm::bonding_curve_token::bonding_curve_bytecode;

#[test]
fn test_graduation_amm_lifecycle() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x33u8; 32];
    let caller = 100u64;

    // --- 1. Deploy / Init (slope = 1) ---
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap(); // slope = 1
    vm.stack.push(0).unwrap(); // entry point 0
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(1), "Init should succeed");
    // Verify phase is explicitly set to 0
    assert_eq!(state.get(&addr, 3), 0, "Phase should be 0 after init");

    // --- 2. Buy 45,000 tokens (cost = 45000^2 / 2 = 1,012,500,000 >= 1B) ---
    let mut vm = VM::new(10_000_000);
    vm.stack.push(45_000).unwrap(); // tokens_to_mint
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(4).unwrap(); // entry point 4 (buy)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    let buy_cost = res.return_value.expect("Buy should return cost");
    assert_eq!(buy_cost, 1_012_500_000, "Buy cost = 45000^2 / 2");

    let supply = state.get(&addr, 0);
    let reserve = state.get(&addr, 2);
    assert_eq!(supply, 45_000, "Total supply 45000");
    assert_eq!(reserve, 1_012_500_000, "Reserve >= 1B sompi");
    assert_eq!(state.get(&addr, caller), 45_000, "Caller balance 45000");
    assert_eq!(state.get(&addr, 3), 0, "Phase still 0 at VM level");

    // --- 3. Simulate graduation (consensus does this; we set it manually at VM level) ---
    state.set(&addr, 3, 2); // phase = 2 (AMM active)
    state.set(&addr, 4, reserve); // x_reserve = reserve (ZYAN side)
    state.set(&addr, 5, supply); // y_supply = supply (token side)
    let k = reserve.saturating_mul(supply);
    state.set(&addr, 6, k); // k = reserve * supply

    // Verify AMM state
    assert_eq!(state.get(&addr, 3), 2, "Phase is 2");
    assert_eq!(state.get(&addr, 4), reserve, "AMM x_reserve = reserve");
    assert_eq!(state.get(&addr, 5), supply, "AMM y_supply = supply");
    assert_eq!(state.get(&addr, 6), k, "AMM k = reserve * supply");

    // --- 4. Attempt another buy — should return 0 (phase != 0) ---
    let mut vm = VM::new(10_000_000);
    vm.stack.push(100).unwrap(); // tokens_to_mint
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(4).unwrap(); // entry point 4 (buy)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Buy should be rejected when phase != 0");

    // --- 5. Attempt a sell — should return 0 (phase != 0) ---
    let mut vm = VM::new(10_000_000);
    vm.stack.push(100).unwrap(); // tokens_in
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(5).unwrap(); // entry point 5 (sell)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "Sell should be rejected when phase != 0");

    // --- 6. AMM swap x_to_y: send ZYAN, receive tokens ---
    let zyan_in = 1_000_000u64;
    let x_reserve_before = state.get(&addr, 4);
    let y_supply_before = state.get(&addr, 5);
    let holder_before = state.get(&addr, caller);

    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap(); // is_x_to_y = 1
    vm.stack.push(zyan_in).unwrap(); // token_in_amount (ZYAN)
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(7).unwrap(); // entry point 7 (amm_swap)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    let tokens_out = res.return_value.expect("AMM swap should return tokens_out");
    assert!(tokens_out > 0, "x_to_y swap should return > 0 tokens");

    // Verify holder balance increased
    assert_eq!(state.get(&addr, caller), holder_before + tokens_out, "Holder balance increased by tokens_out");

    // Verify x_reserve increased, y_supply decreased
    assert_eq!(state.get(&addr, 4), x_reserve_before + zyan_in, "x_reserve increased by zyan_in");
    assert_eq!(state.get(&addr, 5), y_supply_before - tokens_out, "y_supply decreased by tokens_out");

    // Verify k is approximately preserved (x * y ≈ k, with rounding dust).
    // new_y = floor(k / new_x), so new_x * new_y <= k. The dust is at most new_x
    // (the fractional part of k/new_x is < 1, so dust = new_x * frac < new_x).
    let new_x = state.get(&addr, 4);
    let new_y = state.get(&addr, 5);
    let new_k = new_x.saturating_mul(new_y);
    assert!(new_k <= k, "new_k should be <= k (floor division rounds down)");
    assert!(k - new_k < new_x, "dust should be < new_x (fractional rounding)");

    // --- 7. AMM swap y_to_x: send tokens, receive ZYAN ---
    let tokens_in = tokens_out; // send back the tokens we just received
    let x_reserve_before2 = state.get(&addr, 4);
    let y_supply_before2 = state.get(&addr, 5);
    let holder_before2 = state.get(&addr, caller);

    let mut vm = VM::new(10_000_000);
    vm.stack.push(0).unwrap(); // is_x_to_y = 0 (y to x)
    vm.stack.push(tokens_in).unwrap(); // token_in_amount (tokens)
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(7).unwrap(); // entry point 7 (amm_swap)
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    let zyan_out = res.return_value.expect("AMM swap should return zyan_out");
    assert!(zyan_out > 0, "y_to_x swap should return > 0 ZYAN");

    // Verify holder balance decreased
    assert_eq!(state.get(&addr, caller), holder_before2 - tokens_in, "Holder balance decreased by tokens_in");

    // Verify x_reserve decreased, y_supply increased
    assert_eq!(state.get(&addr, 4), x_reserve_before2 - zyan_out, "x_reserve decreased by zyan_out");
    assert_eq!(state.get(&addr, 5), y_supply_before2 + tokens_in, "y_supply increased by tokens_in");
    // Verify k approximately preserved against the stored k (key 6 never updates after graduation,
    // so every swap uses the original k for computation: new_x * new_y <= k_stored).
    let final_x = state.get(&addr, 4);
    let final_y = state.get(&addr, 5);
    let final_k = final_x.saturating_mul(final_y);
    assert!(final_k <= k, "final_k should be <= original stored k");
    assert!(k - final_k < k / 100, "dust should be < 1% of k");

}

#[test]
fn test_amm_swap_rejected_before_graduation() {
    let bytecode = bonding_curve_bytecode();
    let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
    let mut state = MockStateBackend::new();
    let addr = [0x44u8; 32];
    let caller = 100u64;

    // Init with slope = 1
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap();
    vm.stack.push(0).unwrap();
    vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

    // AMM swap when phase == 0 should return 0
    let mut vm = VM::new(10_000_000);
    vm.stack.push(1).unwrap(); // is_x_to_y
    vm.stack.push(1_000).unwrap(); // token_in_amount
    vm.stack.push(caller).unwrap(); // caller
    vm.stack.push(7).unwrap(); // entry point 7
    let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
    assert_eq!(res.return_value, Some(0), "AMM swap should be rejected when phase != 2");
}