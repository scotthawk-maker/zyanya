use crate::assembler::Assembler;

pub const BONDING_CURVE_ASM: &str = r#"
// Bonding Curve Token Contract
// Key 0: Total Supply
// Key 1: Slope
// Key 2: Reserve
// Key <address>: Holder Balance

// --- Check Initialization ---
PUSH 1
SLOAD
PUSH 0
EQ
JUMPIF :initialize

// --- Dispatch Invoke by Entry Point ---
DUP
PUSH 0
EQ
JUMPIF :op_init

DUP
PUSH 1
EQ
JUMPIF :op_transfer

DUP
PUSH 2
EQ
JUMPIF :op_balance_of

DUP
PUSH 3
EQ
JUMPIF :op_total_supply

DUP
PUSH 4
EQ
JUMPIF :op_buy

DUP
PUSH 5
EQ
JUMPIF :op_sell

DUP
PUSH 6
EQ
JUMPIF :op_price

POP
PUSH 0
RETURN

// --- Initialization (on Deploy) ---
:initialize
JUMP :op_init

// --- Entry Point 0: Init(slope) ---
:op_init
POP
STORE 0
LOAD 0
PUSH 1
SWAP
SSTORE
PUSH 0
PUSH 0
SWAP
SSTORE
PUSH 0
PUSH 2
SWAP
SSTORE
PUSH 1
RETURN

// --- Entry Point 1: Transfer(from, to, amount) ---
:op_transfer
POP
STORE 0
STORE 1
STORE 2

LOAD 0
SLOAD
DUP
LOAD 2
GTE
JUMPIF :do_transfer

POP
PUSH 0
RETURN

:do_transfer
LOAD 2
SUB
LOAD 0
SWAP
SSTORE

LOAD 1
SLOAD
LOAD 2
ADD
LOAD 1
SWAP
SSTORE

PUSH 1
RETURN

// --- Entry Point 2: BalanceOf(holder) ---
:op_balance_of
POP
SLOAD
RETURN

// --- Entry Point 3: TotalSupply ---
:op_total_supply
POP
PUSH 0
SLOAD
RETURN

// --- Entry Point 4: Buy(caller, tokens_to_mint) ---
:op_buy
POP
STORE 0
STORE 1

PUSH 2
PUSH 0
SLOAD
MUL
LOAD 1
MUL

LOAD 1
LOAD 1
MUL

ADD

PUSH 1
SLOAD
MUL

PUSH 2
DIV

STORE 2

PUSH 0
SLOAD
LOAD 1
ADD
PUSH 0
SWAP
SSTORE

PUSH 2
SLOAD
LOAD 2
ADD
PUSH 2
SWAP
SSTORE

LOAD 0
SLOAD
LOAD 1
ADD
LOAD 0
SWAP
SSTORE

LOAD 2
RETURN

// --- Entry Point 5: Sell(caller, tokens_in) ---
:op_sell
POP
STORE 0
STORE 1

LOAD 0
SLOAD
STORE 3

LOAD 3
LOAD 1
GTE
JUMPIF :calc_refund

PUSH 0
RETURN

:calc_refund
PUSH 2
PUSH 0
SLOAD
MUL
LOAD 1
MUL

LOAD 1
LOAD 1
MUL

SUB

PUSH 1
SLOAD
MUL

PUSH 2
DIV

STORE 2

PUSH 2
SLOAD
STORE 5

LOAD 5
LOAD 2
GTE
JUMPIF :do_sell

PUSH 0
RETURN

:do_sell
LOAD 3
LOAD 1
SUB
LOAD 0
SWAP
SSTORE

PUSH 0
SLOAD
LOAD 1
SUB
PUSH 0
SWAP
SSTORE

LOAD 5
LOAD 2
SUB
PUSH 2
SWAP
SSTORE

LOAD 2
RETURN

// --- Entry Point 6: Price ---
:op_price
POP
PUSH 1
SLOAD
PUSH 0
SLOAD
MUL
RETURN
"#;

pub fn bonding_curve_bytecode() -> Vec<u8> {
    Assembler::assemble(BONDING_CURVE_ASM).expect("Failed to assemble bonding curve contract")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockStateBackend, OpCode, VM};

    #[test]
    fn test_bonding_curve_contract_lifecycle() {
        let bytecode = bonding_curve_bytecode();
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
        let mut state = MockStateBackend::new();
        let addr = [0x55u8; 32];
        let caller1 = 100u64;
        let caller2 = 200u64;

        // 0. Deploy Initialization (as node consensus does)
        let mut vm = VM::new(100_000);
        vm.stack.push(1).unwrap(); // slope = 1
        vm.stack.push(0).unwrap(); // entry point 0 on deploy
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Deploy failed");
        assert_eq!(res.return_value, Some(1), "Deploy should succeed");

        // 1. Init: slope = 2 (entry point 0)
        let mut vm = VM::new(100_000);
        vm.stack.push(2).unwrap(); // slope = 2
        vm.stack.push(0).unwrap(); // entry point 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Init failed");
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, 1), 2, "Slope is 2");

        // 2. Buy caller 1, k=10 (entry point 4)
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap(); // k = 10
        vm.stack.push(caller1).unwrap(); // caller = 1
        vm.stack.push(4).unwrap(); // entry point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Buy failed");
        assert_eq!(res.return_value, Some(100), "Cost is 100");
        assert_eq!(state.get(&addr, 0), 10, "Total supply 10");
        assert_eq!(state.get(&addr, 2), 100, "Reserve 100");
        assert_eq!(state.get(&addr, caller1), 10, "Caller 1 balance 10");

        // 3. Buy caller 2, k=5 (entry point 4)
        let mut vm = VM::new(100_000);
        vm.stack.push(5).unwrap(); // k = 5
        vm.stack.push(caller2).unwrap(); // caller = 2
        vm.stack.push(4).unwrap(); // entry point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Buy failed");
        assert_eq!(res.return_value, Some(125), "Cost is 125");
        assert_eq!(state.get(&addr, 0), 15, "Total supply 15");
        assert_eq!(state.get(&addr, 2), 225, "Reserve 225");
        assert_eq!(state.get(&addr, caller2), 5, "Caller 2 balance 5");

        // 4. Price query (entry point 6)
        let mut vm = VM::new(100_000);
        vm.stack.push(6).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Price query failed");
        assert_eq!(res.return_value, Some(30), "Price is 30 (2 * 15)");

        // 5. Sell caller 1, k=10 (entry point 5)
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap(); // k = 10
        vm.stack.push(caller1).unwrap(); // caller = 1
        vm.stack.push(5).unwrap(); // entry point 5
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Sell failed");
        assert_eq!(res.return_value, Some(200), "Refund is 200");
        assert_eq!(state.get(&addr, 0), 5, "Total supply 5");
        assert_eq!(state.get(&addr, 2), 25, "Reserve 25");
        assert_eq!(state.get(&addr, caller1), 0, "Caller 1 balance 0");

        // 6. Price query after sell
        let mut vm = VM::new(100_000);
        vm.stack.push(6).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Price query failed");
        assert_eq!(res.return_value, Some(10), "Price after sell is 10 (2 * 5)");

        // 7. Total supply after sell (entry point 3)
        let mut vm = VM::new(100_000);
        vm.stack.push(3).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Total supply query failed");
        assert_eq!(res.return_value, Some(5), "Total supply after sell is 5");

        // 8. Sell caller 1, k=10 should FAIL (return 0) because balance[1]=0
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap(); // k = 10
        vm.stack.push(caller1).unwrap(); // caller = 1
        vm.stack.push(5).unwrap(); // entry point 5
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Sell failed");
        assert_eq!(res.return_value, Some(0), "Sell should fail and return 0");
    }

    #[test]
    fn test_bonding_curve_deploy_and_all_operations() {
        let bytecode = bonding_curve_bytecode();
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
        let mut state = MockStateBackend::new();
        let addr = [0x77u8; 32];
        let alice = 1000u64;
        let bob = 2000u64;

        // Step 1: Deploy contract
        let mut vm = VM::new(100_000);
        vm.stack.push(1).unwrap(); // slope = 1
        vm.stack.push(0).unwrap(); // entry_point 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1), "Deploy returns 1");

        // Step 2: Init slope = 10
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap(); // slope
        vm.stack.push(0).unwrap();  // entry_point 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1));

        // Step 3: Total supply (entry point 3)
        let mut vm = VM::new(100_000);
        vm.stack.push(3).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(0));

        // Step 4: Price (entry point 6)
        let mut vm = VM::new(100_000);
        vm.stack.push(6).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(0));

        // Step 5: Buy 50 tokens for Alice (entry point 4)
        let mut vm = VM::new(100_000);
        vm.stack.push(50).unwrap();    // tokens_to_mint
        vm.stack.push(alice).unwrap(); // caller
        vm.stack.push(4).unwrap();     // entry_point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(12500));

        // Check Alice balance & total supply & reserve
        assert_eq!(state.get(&addr, alice), 50);
        assert_eq!(state.get(&addr, 0), 50);
        assert_eq!(state.get(&addr, 2), 12500);

        // Step 6: Transfer 15 tokens from Alice to Bob (entry point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(15).unwrap();   // amount
        vm.stack.push(bob).unwrap();  // to
        vm.stack.push(alice).unwrap();// from
        vm.stack.push(1).unwrap();    // entry_point 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, alice), 35);
        assert_eq!(state.get(&addr, bob), 15);

        // Step 7: Sell 15 tokens for Bob (entry point 5)
        let mut vm = VM::new(100_000);
        vm.stack.push(15).unwrap();  // tokens_in
        vm.stack.push(bob).unwrap(); // caller
        vm.stack.push(5).unwrap();   // entry_point 5
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(6375));
        assert_eq!(state.get(&addr, bob), 0);
        assert_eq!(state.get(&addr, 0), 35);
        assert_eq!(state.get(&addr, 2), 6125);
    }

    #[test]
    fn test_bonding_curve_init_slope_parameter_and_dispatch() {
        let bytecode = bonding_curve_bytecode();
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
        let mut state = MockStateBackend::new();
        let addr = [0x88u8; 32];
        let user = 999u64;

        // 1. Fresh contract init with slope = 11 (entry point 0, params = [11])
        let mut vm = VM::new(100_000);
        vm.stack.push(11).unwrap(); // slope = 11
        vm.stack.push(0).unwrap();  // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Init failed");
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, 1), 11, "Slope must be read as 11 from stack parameter");

        // 2. Buy 10 tokens for user (entry point 4)
        // Cost formula: slope * (2 * supply * k + k^2) / 2 = 11 * (0 + 100) / 2 = 550
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap();   // k = 10
        vm.stack.push(user).unwrap(); // caller
        vm.stack.push(4).unwrap();    // entry_point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Buy failed");
        assert_eq!(res.return_value, Some(550), "Buy cost should be 550 for slope 11");
        assert_eq!(state.get(&addr, 2), 550, "Reserve should be 550");

        // 3. Sell 10 tokens for user (entry point 5)
        // Refund formula: slope * (2 * 10 * 10 - 100) / 2 = 11 * 100 / 2 = 550
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap();   // k = 10
        vm.stack.push(user).unwrap(); // caller
        vm.stack.push(5).unwrap();    // entry_point 5
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Sell failed");
        assert_eq!(res.return_value, Some(550), "Sell refund should be 550");
        assert_eq!(state.get(&addr, 2), 0, "Reserve should be 0 after full sell");

        // 4. Already-initialized contract calling init(slope = 5) (entry point 0)
        // Should dispatch correctly to :op_init and set slope = 5
        let mut vm = VM::new(100_000);
        vm.stack.push(5).unwrap(); // new slope = 5
        vm.stack.push(0).unwrap(); // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Re-init failed");
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, 1), 5, "Slope updated to 5");
    }
}

