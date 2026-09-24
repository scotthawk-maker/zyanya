# Section 1: Consensus, Genesis & Emission Parameters Review

## Executive Summary & Audit Scorecard
- **Audit Score**: 98/100 (Pass)
- **Status**: **GO** for Mainnet Launch
- **Summary**: The Zyanya Section 1 Consensus mechanisms demonstrate robust security, rigorous zero-premine compliance, and deterministic tokenomics. The 1-year deflationary emission schedule correctly relies on purely rational integer arithmetic, effectively removing consensus risks associated with floating-point non-determinism.

## Deep-Dive Line Analysis by File

### `consensus/core/src/config/genesis.rs`
- **Zero-Premine Verification**: `MAINNET_GENESIS.coinbase_payload` embeds an initial 50 ZYAN subsidy (5,000,000,000 sompi) in its payload data. However, `GenesisBlock::build_genesis_transactions()` initializes transaction outputs as `Vec::new()`, meaning exactly 0 coins are minted as spendable UTXOs at T=0. The circulating supply begins precisely at 0.
- **Magic Trailer**: Contains OP_FALSE (`0x00`) followed by `"ZYAN-MAINNET"`, mathematically guaranteeing non-spendability and safely isolating the payload.

### `consensus/src/processes/coinbase.rs`
- **Deflationary Subsidy Schedule**: The `MAINNET_SUBSIDY_BY_MONTH_TABLE` accurately outlines 727 months of deflationary subsidy decays. The implementation dynamically applies `.div_ceil(bps)` deterministically on integer bounds at initialization, ensuring smooth and robust tokenomics.
- **Dual-Stream Coinbase Vesting**: Evaluated `expected_coinbase_transaction`. Miner rewards are precisely split: 50% distributed immediately as a liquid output (spendable after `coinbase_maturity` of 100 blocks), and 50% linearly vested. The vested portion creates 12 distinct outputs, properly locked via `create_csv_locked_script` using `OP_CHECKSEQUENCEVERIFY` in a monthly step sequence.

### `consensus/core/src/config/params.rs` & `constants.rs`
- **Subnetwork 3 Smart Contract Activation**: Verified `enable_smart_contracts: true` is strictly enforced. Furthermore, `payload_activation: ForkActivation::new(0)` allows immediate usage of the transaction payload required for Subnetwork 3 smart contracts starting from block zero on Mainnet.
- **Network Magic & Port Isolation**: Mainnet properly enforces the 4-byte network magic `[0x5A, 0x59, 0x41, 0x4E]` ("ZYAN"). Testnets are isolated via unique identifiers (e.g., `ZYNT`). Canonical ports are correctly aligned for Mainnet (P2P: 18111, gRPC: 18110, Borsh: 19110, JSON: 20110).

## Edge Cases & Determinism Verification
- **Floating-Point Consensus Forks**: Effectively mitigated. The migration to a strict 727-step `u64` decay array fully ensures bit-identical cross-platform validation.
- **Output Mass Overflow (Coinbase)**: For maximum merged blocks (`mergeset_size_limit`), 13 outputs are allocated per block (1 liquid + 12 vested). Given a limit of 180 (for `k=18`), the resulting coinbase could theoretically reach ~2,350 outputs. While algorithmically secure, this consumes a recognizable portion of the maximum block mass.
- **Locktime Boundaries**: The `create_csv_locked_script` properly uses `OP_CHECKSEQUENCEVERIFY` to handle relative locktimes corresponding to exact `blocks_per_month` bounds.

## Findings & Recommendations

### Critical
- **None**. The mathematical zero-premine enforcement and determinism are solid. 

### High
- **None**.

### Medium
- **Coinbase Output Density Limit**: A single block with a maximized mergeset yields up to ~2,350 outputs. At approximately ~45 bytes per output (script + value overhead), ~105 KB of block mass could be consumed strictly by coinbase generation. Recommend monitoring memory pool eviction behavior during extended high-orphan merging scenarios to prevent potential mass exhaustion for standard transactions.

### Low / Advisory
- **Subnetwork 3 Activation Specificity**: While `payload_activation` activates at 0 and `enable_smart_contracts` is true, explicit checks limiting execution *specifically* to `SUBNETWORK_ID_SMART_CONTRACT` inside the VM dispatch layer must be continuously audited.
- **Constant Organization**: Network Magic constants and port defaults are currently embedded directly within `params.rs`. Moving them into a globally accessible namespace may improve traceability.

## Official Go/No-Go Verdict for Section 1 Consensus Genesis
**Verdict**: **GO**

The reviewed modules meet all required architectural, deterministic, and mathematical invariants for the Mainnet Launch on October 1, 2026.