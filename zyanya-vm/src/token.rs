use crate::assembler::{Assembler, AssemblerError};

/// Assembly template for the Zyanya Reference Token (ERC-20 style).
pub const TOKEN_CONTRACT_ASM_TEMPLATE: &str = r#"
// Zyanya Reference Token Contract (ERC-20 Style)
// Key 0: Total Supply
// Key 1: Owner Address
// Key <address>: Holder Balance

// --- Check Initialization ---
PUSH 0
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
JUMPIF :op_transfer

DUP
PUSH 1
EQ
JUMPIF :op_balance_of

DUP
PUSH 2
EQ
JUMPIF :op_total_supply

DUP
PUSH 3
EQ
JUMPIF :op_mint

PUSH 0
RETURN

// --- Initialization ---
:initialize
POP
PUSH {SUPPLY}
DUP
PUSH 0
SWAP
SSTORE
PUSH {OWNER}
SWAP
SSTORE
PUSH {SUPPLY}
RETURN

// --- Entry Point 0: Transfer ---
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

// --- Entry Point 1: BalanceOf ---
:op_balance_of
POP
SLOAD
RETURN

// --- Entry Point 2: TotalSupply ---
:op_total_supply
POP
PUSH 0
SLOAD
RETURN

// --- Entry Point 3: Mint ---
:op_mint
POP
STORE 0
STORE 1

PUSH 0
SLOAD
LOAD 1
ADD
PUSH 0
SWAP
SSTORE

LOAD 0
SLOAD
LOAD 1
ADD
LOAD 0
SWAP
SSTORE

PUSH 1
RETURN
"#;

/// Generate assembly source code for reference token contract with custom supply and owner address.
pub fn token_contract_asm(initial_supply: u64, owner_addr: u64) -> String {
    TOKEN_CONTRACT_ASM_TEMPLATE
        .replace("{SUPPLY}", &initial_supply.to_string())
        .replace("{OWNER}", &owner_addr.to_string())
}

/// Assemble token contract source code into VM bytecode.
pub fn token_contract_bytecode(initial_supply: u64, owner_addr: u64) -> Result<Vec<u8>, AssemblerError> {
    let source = token_contract_asm(initial_supply, owner_addr);
    Assembler::assemble(&source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockStateBackend, OpCode, VM};

    #[test]
    fn test_token_contract_full_lifecycle() {
        let bytecode = token_contract_bytecode(1_000_000, 1).expect("Failed to assemble token contract");
        let opcodes = OpCode::deserialize_slice(&bytecode).expect("Failed to deserialize bytecode");
        let mut state = MockStateBackend::new();
        let addr = [0x99u8; 32];
        let owner = 1u64;
        let recipient = 2u64;

        // 1. Deploy / Initial Execution
        let mut vm = VM::new(100_000);
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Deploy failed");
        assert_eq!(res.return_value, Some(1_000_000));
        assert_eq!(state.get(&addr, 0), 1_000_000, "Total supply stored at Key 0");
        assert_eq!(state.get(&addr, owner), 1_000_000, "Owner balance stored at Key 1");

        // 2. Query Total Supply (Entry Point 2)
        let mut vm = VM::new(100_000);
        vm.stack.push(2).unwrap(); // entry_point = 2
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("TotalSupply query failed");
        assert_eq!(res.return_value, Some(1_000_000));

        // 3. Query Owner Balance (Entry Point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(owner).unwrap(); // holder = 1
        vm.stack.push(1).unwrap(); // entry_point = 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("BalanceOf owner failed");
        assert_eq!(res.return_value, Some(1_000_000));

        // 4. Query Recipient Balance before transfer (Entry Point 1)
        let mut vm = VM::new(100_000);
        vm.stack.push(recipient).unwrap(); // holder = 2
        vm.stack.push(1).unwrap(); // entry_point = 1
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("BalanceOf recipient failed");
        assert_eq!(res.return_value, Some(0));

        // 5. Transfer 100 tokens from owner (1) to recipient (2) (Entry Point 0)
        let mut vm = VM::new(100_000);
        vm.stack.push(100).unwrap(); // amount
        vm.stack.push(recipient).unwrap(); // to
        vm.stack.push(owner).unwrap(); // from
        vm.stack.push(0).unwrap(); // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Transfer failed");
        assert_eq!(res.return_value, Some(1), "Transfer succeeded");
        assert_eq!(state.get(&addr, owner), 999_900, "Owner balance reduced by 100");
        assert_eq!(state.get(&addr, recipient), 100, "Recipient balance increased by 100");

        // 6. Verify balances via BalanceOf queries
        let mut vm = VM::new(100_000);
        vm.stack.push(owner).unwrap();
        vm.stack.push(1).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(999_900));

        let mut vm = VM::new(100_000);
        vm.stack.push(recipient).unwrap();
        vm.stack.push(1).unwrap();
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(100));

        // 7. Mint 50,000 tokens to recipient (2) (Entry Point 3)
        let mut vm = VM::new(100_000);
        vm.stack.push(50_000).unwrap(); // amount
        vm.stack.push(recipient).unwrap(); // to
        vm.stack.push(3).unwrap(); // entry_point = 3
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).expect("Mint failed");
        assert_eq!(res.return_value, Some(1), "Mint succeeded");
        assert_eq!(state.get(&addr, 0), 1_050_000, "Total supply increased to 1,050,000");
        assert_eq!(state.get(&addr, recipient), 50_100, "Recipient balance increased to 50,100");
    }

    #[test]
    fn test_token_transfer_insufficient_balance() {
        let bytecode = token_contract_bytecode(100, 1).unwrap();
        let opcodes = OpCode::deserialize_slice(&bytecode).unwrap();
        let mut state = MockStateBackend::new();
        let addr = [0x88u8; 32];

        // Deploy (owner=1 gets 100 supply)
        let mut vm = VM::new(100_000);
        let _ = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();

        // Attempt to transfer 200 tokens from owner (1) to recipient (2) -> should fail
        let mut vm = VM::new(100_000);
        vm.stack.push(200).unwrap(); // amount
        vm.stack.push(2).unwrap(); // to
        vm.stack.push(1).unwrap(); // from
        vm.stack.push(0).unwrap(); // entry_point = 0
        let res = vm.execute_stateful(&opcodes, &addr, &mut state).unwrap();
        assert_eq!(res.return_value, Some(0), "Transfer failed due to insufficient balance");
        assert_eq!(state.get(&addr, 1), 100, "Owner balance unchanged");
        assert_eq!(state.get(&addr, 2), 0, "Recipient balance unchanged");
    }
}
