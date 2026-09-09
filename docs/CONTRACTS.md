# Subnetwork 3 ZCL Virtual Machine Specification

> **Architecture**: Deterministic 64-bit Stack Machine  
> **Execution Subnetwork**: Subnetwork 3  
> **Arithmetic Precision**: Fixed-Point 64-bit ($10^8$ decimal units)

---

## 1. Execution Model

Zyanya separates general UTXO value transfers (Subnetwork 0) from smart contract execution (Subnetwork 3). This architecture preserves ultra-fast 1 BPS GhostDAG DAG propagation while allowing stateful decentralized finance (DeFi) execution.

### Stack Machine Constraints
- **Word Size**: 64 bits (signed and unsigned integer operations).
- **Stack Depth**: Hard limit of 1,024 elements per execution frame.
- **Gas Metering**: Every opcode has a strictly metered gas cost to prevent infinite loops and denial-of-service vectors.
- **State Storage**: Key-value slot storage indexed by 32-byte keys.

---

## 2. Core Opcode Reference

| Category | Opcodes | Description |
| :--- | :--- | :--- |
| **Stack Control** | `DUP`, `SWAP`, `POP`, `PUSH`, `ROT` | Stack element manipulation and reordering |
| **Arithmetic** | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `ABS` | 64-bit checked arithmetic with overflow protection |
| **Logic & Bitwise**| `AND`, `OR`, `XOR`, `NOT`, `SHL`, `SHR` | Bitwise logical operations |
| **Flow Control** | `JMP`, `JMPIF`, `CALL`, `RET`, `HALT` | Conditional branching and function invocation |
| **Context** | `CALLER`, `VALUE`, `GASLIMIT`, `BLOCKTIME`, `DAASCORE` | Execution environment and blockchain state access |
| **Storage** | `SLOAD`, `SSTORE` | Persistent 32-byte slot state read/write |
| **Crypto** | `SHA256`, `BLAKE2B`, `ASTROBWT`, `VERIFY_SIG` | In-VM hashing and signature verification |

---

## 3. Bundled Reference Contracts

### 1. `bonding_curve.zcl`
Implements an automated pricing curve for continuous token issuance and redemption.
- **Mechanism**: Calculates token price as a deterministic polynomial function of total supply.
- **Liquidity**: Guaranteed instant buy and sell liquidity via locked collateral reserve.

### 2. `dex.zcl`
Constant-product Automated Market Maker ($x \cdot y = k$) for ZRC-20 token pairs.
- **Swaps**: Minimum output slippage protections.
- **Liquidity Pools**: LP token minting and proportional burning upon withdrawal.

### 3. `staking.zcl`
Time-locked UTXO staking vault with epoch-based reward distribution.
- **Lock Periods**: Configurable time locks enforced by blockchain DAA score and timestamps.
- **Slashing / Unbonding**: Strict unbonding cooldowns to safeguard consensus security.

### 4. `router.zcl`
Multi-hop swap router that identifies optimal routing paths across disparate DEX liquidity pools.
