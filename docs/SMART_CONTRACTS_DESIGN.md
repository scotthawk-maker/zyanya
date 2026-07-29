# Zyanya Smart Contracts Architecture & Design Document

**Status:** Proposed / Phase 1 Scaffold  
**Target Engine:** `zyanya-vm`  
**Consensus Basis:** UTXO-based GhostDAG (Spectre / Kaspa derivative)  
**Rust Toolchain:** 1.89  

---

## 1. Executive Summary & Core Objectives

Zyanya is a high-throughput, UTXO-based BlockDAG cryptocurrency utilizing GhostDAG consensus, time-locked vesting, CPU mining, and native IPv6 peer-to-peer networking. The goal of this initiative is to introduce **general-purpose, stateful smart contract execution** into Zyanya without compromising its core UTXO consensus, parallel block ordering, or transaction processing performance.

This document outlines the architectural blueprint for smart contracts on Zyanya, details the design choices (specifically why a custom, transparent, stack-based VM is preferred over external dependencies), defines the UTXO-to-Account state overlay model, and specifies the consensus integration points within the GhostDAG virtual state resolution pipeline.

---

## 2. Architectural Trade-offs & Strategic Recommendation

When introducing smart contracts to a UTXO-based BlockDAG, three primary architectural patterns exist:

| Strategy | Description | Advantages | Disadvantages | Verdict |
| :--- | :--- | :--- | :--- | :--- |
| **Option A: Dependency on `xelis-vm`** | Import the reference `xelis-vm` directly as a Cargo dependency. | Pre-built parser, lexer, compiler, and opcode suite. | Tightly coupled with Xelis privacy primitives (homomorphic encryption, ElGamal ciphertexts) and Xelis-specific account module formats. Overly complex for Zyanya's transparent model. | **Rejected** |
| **Option B: Pure Extended UTXO (eUTXO)** | Require contract state to be passed through input/output UTXOs (Cardano style). | Purely functional, highly parallelizable per UTXO. | High state contention for global shared states (e.g., automated market makers, global counters). Complex transaction chaining. | **Rejected** |
| **Option C: Hybrid Account-State Overlay + Native VM (`zyanya-vm`)** | Maintain standard UTXO transactions for transfer/vesting, while introducing an account/storage overlay tree managed by node consensus for contracts. Execute bytecode via a custom lightweight VM (`zyanya-vm`). | Clean separation of concerns; fast UTXO transfers; global contract storage access; zero privacy bloat; customized for GhostDAG. | Requires state-sync and reorg-aware storage handling in node consensus. | **RECOMMENDED** |

### Recommendation
We adopt **Option C**: A dedicated **Account-State Overlay** operating alongside the UTXO set, powered by a purpose-built, high-performance, deterministic stack-based execution engine (`zyanya-vm`).

---

## 3. UTXO → Account/State Integration (The Overlay Architecture)

### 3.1 Dual-Layer Ledger Architecture
Zyanya operates a dual-layer ledger:
1. **UTXO Layer**: Handles ZYAN coin minting (coinbase), peer-to-peer transfers, CSV time-locked vesting, and fee payments.
2. **Contract State Layer**: A global state database storing contract account balances, compiled bytecode, and persistent key-value storage.

```
+-------------------------------------------------------------------+
|                        Zyanya BlockDAG                            |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
|                      GhostDAG Consensus                           |
|         (Deterministic Topological Ordering of Blocks)            |
+-------------------------------------------------------------------+
          /                                               \
         v                                                 v
+-----------------------+                       +-----------------------+
|      UTXO Set         |                       | Contract State Overlay|
| (Inputs / Outputs /   |                       | (Contract Addresses,  |
|  Vesting / Coinbase)  |                       |  Storage KV, Balances)|
+-----------------------+                       +-----------------------+
```

### 3.2 Contract Identification & Lifecycle
- **Contract Address**: Derived deterministically when a deploy transaction is accepted into consensus:
  $$\text{ContractAddress} = \text{Blake2b}(\text{DeployTxID} \parallel \text{OutputIndex})$$
- **Value Deposits (UTXO → Contract Balance)**:
  When an `InvokeContract` or `DeployContract` transaction is issued:
  - UTXO inputs are spent to cover:
    1. Base transaction fee (miner reward).
    2. Maximum execution gas deposit (`max_gas * gas_price`).
    3. Direct coin deposits transferred to the contract's balance.
  - Spent UTXOs are removed from the UTXO set. The deposited ZYAN amount is credited to the contract's balance in the Contract State Overlay.
- **Value Withdrawals (Contract Balance → UTXO)**:
  Contracts can initiate syscall transfers during execution. When validated by consensus during virtual parent state resolution, contract balance decreases, and new spendable UTXOs are generated for the target recipient.

---

## 4. Contract Transactions (Deploy & Invoke Payloads)

New transaction types are introduced into Zyanya's transaction model:

### 4.1 DeployContract Payload
```rust
pub struct DeployContractPayload {
    /// Compiled bytecode for the contract executable.
    pub bytecode: Vec<u8>,
    /// Maximum gas allocated for executing the constructor.
    pub max_gas: u64,
    /// Gas price in ZYAN sompi per gas unit.
    pub gas_price: u64,
    /// Optional initial constructor parameters.
    pub constructor_args: Vec<Value>,
    /// Amount of ZYAN deposited into the contract account balance.
    pub deposit_amount: u64,
}
```

### 4.2 InvokeContract Payload
```rust
pub struct InvokeContractPayload {
    /// 32-byte hash identifier of the target contract.
    pub contract_address: Hash,
    /// Function/Entry point identifier or chunk index.
    pub entry_point: u16,
    /// Arguments passed to the entry point function.
    pub parameters: Vec<Value>,
    /// Maximum gas limit for this execution call.
    pub max_gas: u64,
    /// Gas price in ZYAN sompi per gas unit.
    pub gas_price: u64,
    /// Amount of ZYAN deposited into the contract during this invocation.
    pub deposit_amount: u64,
}
```

---

## 5. VM Architecture & Execution Engine (`zyanya-vm`)

### 5.1 Design Principles
- **Stack-based execution**: Simple, verifiable, and deterministic execution semantics.
- **Explicit Gas Metering**: Every opcode decrements the gas counter before execution. Execution aborts immediately with `OutOfGas` if gas reaches zero.
- **Bounded Resources**: Strict call stack depth limits, operand stack depth limits, memory caps, and execution timeouts.
- **No Float Non-Determinism**: Only fixed-size integer arithmetic (`u64`, `i64`, `u256`) and byte arrays are supported.

### 5.2 Opcode Specification (Phase 1 Scaffold & Initial Target)

| Opcode | Hex | Description | Stack Before | Stack After | Gas Cost |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `NOP` | `0x00` | No operation | `[]` | `[]` | 1 |
| `HALT` | `0x01` | Stop execution successfully | `[]` | `[]` | 1 |
| `PUSH` | `0x02` | Push 64-bit integer constant | `[]` | `[val]` | 2 |
| `POP` | `0x03` | Remove top stack item | `[val]` | `[]` | 1 |
| `DUP` | `0x04` | Duplicate top stack item | `[val]` | `[val, val]` | 2 |
| `SWAP` | `0x05` | Swap top two stack items | `[a, b]` | `[b, a]` | 2 |
| `ADD` | `0x10` | Integer addition (`a + b`) | `[a, b]` | `[a + b]` | 3 |
| `SUB` | `0x11` | Integer subtraction (`a - b`) | `[a, b]` | `[a - b]` | 3 |
| `MUL` | `0x12` | Integer multiplication (`a * b`) | `[a, b]` | `[a * b]` | 5 |
| `DIV` | `0x13` | Unsigned integer division | `[a, b]` | `[a / b]` | 5 |
| `EQ` | `0x20` | Equality check (`a == b`) | `[a, b]` | `[bool]` | 3 |
| `LT` | `0x21` | Less-than check (`a < b`) | `[a, b]` | `[bool]` | 3 |
| `GT` | `0x22` | Greater-than check (`a > b`) | `[a, b]` | `[bool]` | 3 |
| `JUMP` | `0x30` | Unconditional jump to PC | `[pc]` | `[]` | 4 |
| `JUMPIF`| `0x31` | Jump to PC if condition is true | `[pc, cond]`| `[]` | 4 |
| `LOAD` | `0x40` | Read value from local variable register | `[reg_idx]` | `[val]` | 3 |
| `STORE`| `0x41` | Write value to local variable register | `[reg_idx, val]` | `[]` | 4 |
| `SLOAD`| `0x50` | Read from contract state storage | `[key]` | `[val]` | 100 |
| `SSTORE`| `0x51`| Write to contract state storage | `[key, val]` | `[]` | 500 |
| `RETURN`| `0xF0` | Exit chunk with return value | `[val]` | `[]` | 1 |

---

## 6. Gas & Fee Model

1. **Gas Fee Calculation**:
   $$\text{GasFee} = \text{GasUsed} \times \text{GasPrice}$$
2. **Fee Distribution**:
   - **50% Burned**: Deducted permanently from supply (deflationary pressure for smart contract usage).
   - **50% Miner Compensation**: Included in the accepting block miner's fee reward.
3. **Unused Gas Refund**:
   - $(\text{MaxGas} - \text{GasUsed}) \times \text{GasPrice}$ is refunded back to the transaction origin address via a change UTXO created during state application.
4. **Execution Failure (OutOfGas or Revert)**:
   - All state changes made during the contract call are rolled back.
   - All `max_gas` funds are forfeited and credited to the block miner / burned (prevents spam attacks).

---

## 7. GhostDAG Consensus Integration

### 7.1 BlockDAG Ordering & Execution Determinism
In a GhostDAG blockDAG, multiple blocks may exist at the same blockDAG height (parallel blocks). Direct state execution inside individual parallel block validation would lead to non-deterministic race conditions across nodes.

**Resolution Engine Strategy**:
1. **Parallel Ingestion**: Blocks are accepted into the DAG structure independently, validating PoW, headers, and individual UTXO format.
2. **Virtual Parent Resolution**: GhostDAG computes a deterministic, globally agreed-upon linear ordering (the "GhostDAG order") of all blocks in the selected parent chain and DAG past.
3. **Sequential State Resolution**: Smart contract transactions (`DeployContract` and `InvokeContract`) are executed **strictly in GhostDAG topological order** during Virtual Parent state updates.

```
       [Block A] (Deploy Contract C)
      /         \
[Genesis]       [Block C (Virtual Parent Consensus)] --> Executes A then B
      \         /
       [Block B] (Invoke Contract C)
```

### 7.2 Handling Conflicting State Transactions
If two parallel blocks (e.g. Block A and Block B) invoke the same contract with conflicting state updates:
- GhostDAG topological sorting establishes whether Block A or Block B comes first in virtual ordering.
- The first transaction succeeds and mutates contract state.
- The second transaction executes against the updated state; if preconditions fail, its state changes revert, but gas fees are still collected.

---

## 8. Node State Storage Architecture

State storage is integrated into `zyanya-database` / RocksDB under distinct Column Families (CFs):

| Column Family | Key | Value | Purpose |
| :--- | :--- | :--- | :--- |
| `cf_contract_code` | `ContractAddress` (32 bytes) | `Bytecode` (bytes) | Stores compiled VM code |
| `cf_contract_storage` | `ContractAddress` + `StorageKey` | `StorageValue` | Persistent key-value storage |
| `cf_contract_balance` | `ContractAddress` | `u64` (ZYAN sompi) | Native coin balance held by contract |
| `cf_contract_meta` | `ContractAddress` | `ContractMetadata` | Owner address, block height, revision |

State changes are maintained in an in-memory `StateCache` during block execution and committed atomically to RocksDB upon Virtual Block acceptance.

---

## 9. Phase 1 Implementation Scope & Roadmap

### Phase 1a: VM Seed Engine (`zyanya-vm`) — *Deliverable Tonight*
- [x] Create standalone `zyanya-vm` crate in workspace.
- [x] Implement stack-based VM core (`VM`, `Stack`, `Memory`, `OpCode`, `GasMeter`).
- [x] Basic opcode suite: `PUSH`, `ADD`, `SUB`, `MUL`, `STORE`, `LOAD`, `JUMP`, `EQ`, `RETURN`, `HALT`.
- [x] Comprehensive unit tests and "Hello World" execution verification test.

### Phase 1b: VM Expansion & Serialization
- [ ] Add extended arithmetic (`DIV`, `MOD`, `POW`), logic (`AND`, `OR`, `XOR`, `NOT`), comparison (`LT`, `GT`), and storage (`SLOAD`, `SSTORE`).
- [ ] Implement bytecode binary serializer and deserializer.
- [ ] Implement memory limit enforcement and safe stack overflow guards.

### Phase 1c: Consensus Transaction Payload Integration
- [ ] Define `DeployContractPayload` and `InvokeContractPayload` in `consensus/core`.
- [ ] Update transaction verification logic in `consensus` to parse contract payloads.

### Phase 1d: Contract State DB & Virtual Processor Integration
- [ ] Add contract storage column families to `database`.
- [ ] Implement state transition processor in `consensus` virtual state loop.
- [ ] Reorg/rollback checkpointing for contract state.

---

## 10. Key Risks, Edge Cases & Technical Considerations

1. **GhostDAG Reorg Overhead**: Deep reorgs require rolling back contract state changes. *Mitigation*: Store incremental state diffs per block height.
2. **Reentrancy Attacks**: Inter-contract calls can trigger reentrancy. *Mitigation*: Enforce state lock patterns and frame depth restrictions in `zyanya-vm`.
3. **Gas Calibration**: Improperly cheap opcodes could allow DoS loops. *Mitigation*: Benchmark every opcode against execution cost and set Conservative base costs.
