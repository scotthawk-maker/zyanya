# Smart Contracts

## VM Overview

The `zyanya-vm` crate implements a stack-based virtual machine with:

- **64-bit unsigned integer arithmetic** (no floats — consensus-safe)
- **Gas metering** — every opcode has a base gas cost; `Pow` charges dynamic gas based on exponent
- **Persistent state** — `SLOAD`/`SSTORE` opcodes read/write contract storage keys
- **Inter-contract calls** — `Call` opcode invokes another contract by 32-byte address
- **Checked arithmetic** — `Add`, `Sub`, `Mul`, `Pow` all use `checked_*` and return `ArithmeticOverflow` on overflow

## Opcode Set

| Category | Opcodes |
|----------|---------|
| Stack | `NOP`, `HALT`, `PUSH`, `POP`, `DUP`, `SWAP` |
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW` |
| Logic | `AND`, `OR`, `XOR`, `NOT` |
| Comparison | `EQ`, `LT`, `GT`, `LTE`, `GTE` |
| Control Flow | `JUMP`, `JUMPIF` |
| Memory | `LOAD`, `STORE` |
| Storage | `SLOAD`, `SSTORE` |
| Inter-Contract | `CALL` |
| Return | `RETURN` |

## ZCL Language

Zyanya Contract Language (ZCL) is a simple assembly-like language compiled to VM opcodes:

```asm
// Token Contract
PUSH 0       // entry point: init
SLOAD        // check if initialized
PUSH 0
EQ
JUMPIF :initialize
// ... dispatch by entry point
```

Features: labels, jumps, comments, multi-function contracts with entry point dispatch.

## Bonding Curve Token

The bonding curve contract (`bonding_curve_token.rs`) implements:

### Storage Keys
| Key | Purpose |
|-----|---------|
| 0 | Total Supply |
| 1 | Slope |
| 2 | Reserve (ZYAN sompi) |
| 3 | Phase (0=bonding, 1=frozen, 2=AMM) |
| 4 | AMM X Reserve (ZYAN side) |
| 5 | AMM Y Supply (token side) |
| 6 | AMM K (constant product) |
| <address> | Holder Balance |

### Entry Points
| EP | Operation |
|----|-----------|
| 0 | Initialize (set slope) |
| 1 | Transfer |
| 2 | Balance of |
| 3 | Total supply |
| 4 | Buy (bonding curve) |
| 5 | Sell (bonding curve) |
| 6 | Price query |
| 7 | AMM swap (both directions) |

### Bonding Curve Math
- **Buy cost**: `slope * supply^2 / 2` (integral of linear price curve)
- **Sell return**: `slope * (supply^2 - new_supply^2) / 2`

### AMM Graduation
When reserve ≥ 1,000,000,000 sompi (10 ZYAN) after a buy:
1. Consensus auto-fires graduation
2. Phase set to 2 (AMM)
3. AMM reserves initialized: `x_reserve = reserve`, `y_supply = supply`, `k = reserve * supply`
4. Future buys/sells rejected; all trading via AMM swap (entry point 7)

### Fee Routing
- **0.3% of buy cost** → staking contract (key 1: `totalRewardsDistributed`)
- **0.3% of AMM trade value** → staking contract
- Fee routed atomically in the same TX

## ZYAN Staking Contract

Staking contract (`staking.zcl`) allows ZYAN holders to:
- **Stake** — lock ZYAN to earn protocol fees
- **Unstake** — withdraw staked ZYAN
- **Claim rewards** — withdraw accumulated fee share

Storage key 1 (`totalRewardsDistributed`) receives the 0.3% protocol fee from all DEX activity.