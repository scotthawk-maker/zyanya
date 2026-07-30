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
DUP
PUSH 0
EQ
JUMPIF :initialize

// --- Dispatch Invoke by Entry Point ---
POP

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

PUSH 0
RETURN

// --- Initialization (on Deploy) ---
:initialize
POP
PUSH 1
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

PUSH 1
SLOAD
LOAD 1
MUL
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

LOAD 1
RETURN

// --- Entry Point 5: Sell(caller, tokens_in) ---
:op_sell
POP
STORE 0
STORE 1

LOAD 0
SLOAD
DUP
LOAD 1
GTE
JUMPIF :do_sell

POP
PUSH 0
RETURN

:do_sell
LOAD 1
SUB
LOAD 0
SWAP
SSTORE

PUSH 1
SLOAD
LOAD 1
MUL
STORE 2

PUSH 0
SLOAD
LOAD 1
SUB
PUSH 0
SWAP
SSTORE

PUSH 2
SLOAD
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
        let caller = 100u64;

        // 0. Deploy Initialization (as node consensus does)
        let mut vm = VM::new(100_000);
        vm.stack.push(0).unwrap(); // entry point 0 on deploy
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Deploy failed");
        assert_eq!(res.return_value, Some(1), "Deploy should succeed");

        // 1. Init: slope = 5 (entry point 0)
        let mut vm = VM::new(100_000);
        vm.stack.push(5).unwrap(); // slope
        vm.stack.push(0).unwrap(); // entry point 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Init failed");
        assert_eq!(res.return_value, Some(1));
        assert_eq!(state.get(&addr, 1), 5, "Slope is 5");

        // 2. Query Total Supply (entry point 3)
        let mut vm = VM::new(100_000);
        vm.stack.push(3).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Total supply query failed");
        assert_eq!(res.return_value, Some(0), "Initial total supply should be 0");

        // 3. Price query (entry point 6)
        let mut vm = VM::new(100_000);
        vm.stack.push(6).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Price query failed");
        assert_eq!(res.return_value, Some(5));

        // 4. Buy 20 tokens (entry point 4)
        let mut vm = VM::new(100_000);
        vm.stack.push(20).unwrap(); // tokens_to_mint
        vm.stack.push(caller).unwrap(); // caller
        vm.stack.push(4).unwrap(); // entry point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Buy failed");
        assert_eq!(res.return_value, Some(20));

        assert_eq!(state.get(&addr, 0), 20, "Total supply 20");
        assert_eq!(state.get(&addr, 2), 100, "Reserve 100 (20 * 5)");
        assert_eq!(state.get(&addr, caller), 20, "Caller balance 20");

        // 5. Total supply check after buy
        let mut vm = VM::new(100_000);
        vm.stack.push(3).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Total supply query failed");
        assert_eq!(res.return_value, Some(20));

        // 6. BalanceOf (entry point 2)
        let mut vm = VM::new(100_000);
        vm.stack.push(caller).unwrap();
        vm.stack.push(2).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("BalanceOf failed");
        assert_eq!(res.return_value, Some(20));

        // 7. Sell 10 tokens (entry point 5)
        let mut vm = VM::new(100_000);
        vm.stack.push(10).unwrap(); // tokens_in
        vm.stack.push(caller).unwrap(); // caller
        vm.stack.push(5).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Sell failed");
        assert_eq!(res.return_value, Some(50), "Refund is 50 (10 * 5)");

        assert_eq!(state.get(&addr, 0), 10, "Total supply 10");
        assert_eq!(state.get(&addr, 2), 50, "Reserve 50");
        assert_eq!(state.get(&addr, caller), 10, "Caller balance 10");
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
        vm.stack.push(0).unwrap();
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
        assert_eq!(res.return_value, Some(10));

        // Step 5: Buy 50 tokens for Alice (entry point 4)
        let mut vm = VM::new(100_000);
        vm.stack.push(50).unwrap();    // tokens_to_mint
        vm.stack.push(alice).unwrap(); // caller
        vm.stack.push(4).unwrap();     // entry_point 4
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(50));

        // Check Alice balance & total supply & reserve
        assert_eq!(state.get(&addr, alice), 50);
        assert_eq!(state.get(&addr, 0), 50);
        assert_eq!(state.get(&addr, 2), 500); // 50 * 10

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
        assert_eq!(res.return_value, Some(150)); // 15 * 10 = 150 refund
        assert_eq!(state.get(&addr, bob), 0);
        assert_eq!(state.get(&addr, 0), 35);
        assert_eq!(state.get(&addr, 2), 350);
    }
}

