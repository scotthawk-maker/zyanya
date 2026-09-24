# Security & Architecture Review: Section 2 Subnetwork 3 VM & Contracts
**Date**: September 24, 2026
**Scope**: `zyanya-vm/` (VM Engine) and `.zcl` Smart Contracts
**Status**: ⛔ NO-GO for Mainnet Launch

## Executive Summary & Audit Scorecard
A comprehensive line-by-line review was conducted on the Zyanya Subnetwork 3 Virtual Machine and Core ZCL Contracts ahead of the October 1, 2026 Mainnet Launch. While gas metering and stack depth constraints are correctly implemented to defend against DoS, several critical consensus-breaking vulnerabilities and unimplemented protocol requirements were discovered. 

**Scorecard:**
- Virtual Machine Determinism: **FAIL** (State rollback and BTreeMap violations)
- Gas Metering & DoS Defenses: **PASS** (Strict limits, checked math)
- Arithmetic Solvency & Precision: **PASS** (Overflows guarded, division truncation handled)
- Contract Custody & Economic Mechanics: **FAIL** (Missing graduation threshold, broken fee routing)
- Authentication & Access Control: **FAIL** (Missing caller verification in Staking)

## Deep-Dive Line Analysis

### VM Engine (`zyanya-vm/`)
- **`vm.rs` (Interpreter & Call Depth)**: The VM effectively guards against recursion depth via `MAX_CALL_DEPTH` (1024) and refunds gas on child context failure. However, when a nested `Call` fails (e.g., reverting via `Err(_)`, `OutOfGas`), the VM fails to snapshot and rollback the `StateBackend`. State modifications made by the child contract prior to the error persist.
- **`state.rs` (State Storage Determinism)**: The `MockStateBackend` (and default reference implementation) utilizes Rust's `HashMap`. Rust's `HashMap` uses randomized hashing (SipHash) by default, meaning state iteration or state root derivation will yield non-deterministic results across different architectures. This violates consensus rules.
- **`gas.rs` (Gas Metering)**: Dynamic costs (e.g., `Pow`) properly compute required gas and bounds-check exponents (e.g., capping at `u32::MAX`). `checked_add` and `checked_sub` are strictly applied.
- **`opcode.rs` (Execution Semantics & Jumps)**: Jump offsets accurately resolve to mapped opcode boundaries (preventing mid-opcode arbitrary execution). The `SStore` and `SLoad` stack popping orders align correctly with CRIT-01 expectations (`[key, val]` where `val` is top of stack for `SStore`).

### ZCL Smart Contracts
- **`bonding_curve.zcl`**: The contract successfully implements overflow guards using explicit quintillion range checks. However, the requirement for an autonomous graduation transition to the AMM at 10 ZYAN (1,000,000,000 sompi) is completely missing.
- **`dex.zcl`**: The constant-product `swap` function correctly maintains the 0.3% fee internally within the reserves to prevent precision loss. However, it fails to implement the required protocol fee routing to the staking vault.
- **`staking.zcl`**: Critical failure in `stake(caller, amount)` and `unstake(caller, amount)`. The contract accepts a `caller` parameter but fails to assert `caller == caller()` (msg.sender). This allows any network participant to unilaterally stake or unstake another user's tokens.
- **`token.zcl`**: Properly gates transfers (`from != caller()`) and correctly tracks the global supply invariant.

## Findings & Recommendations

### Critical
1. **CRIT-01: No State Rollback on Contract Failure**
   - **File:** `zyanya-vm/src/vm.rs`
   - **Description:** Child calls that panic or run out of gas do not revert their local state modifications in the `StateBackend`.
   - **Recommendation:** Implement a transactional state snapshot mechanism in `StateBackend` that is rolled back if `execute_stateful` returns an `Err`.
2. **CRIT-02: Non-Deterministic State Mapping**
   - **File:** `zyanya-vm/src/state.rs`
   - **Description:** Usage of `HashMap` breaks global consensus state root determinism.
   - **Recommendation:** Migrate all internal state mappings to `BTreeMap`.
3. **CRIT-03: Missing AMM Graduation Threshold**
   - **File:** `bonding_curve.zcl`
   - **Description:** The contract fails to graduate and lock liquidity upon reaching 10 ZYAN reserves.
   - **Recommendation:** Implement the 10 ZYAN reserve threshold check in `buy()` and trigger a subnetwork `Call` to the DEX factory to initialize the AMM pool.
4. **CRIT-04: Staking Authentication Bypass**
   - **File:** `staking.zcl`
   - **Description:** `stake` and `unstake` are missing `if (caller_param != caller()) { return 0; }`.
   - **Recommendation:** Add explicit caller verification to both functions.

### High
1. **HIGH-01: Broken DEX Protocol Fee Routing**
   - **File:** `dex.zcl`
   - **Description:** The AMM does not route the 0.3% fee to the Staking Vault.
   - **Recommendation:** Modify the `swap` function to calculate the 0.3% split, retain 0.25% for LPs, and issue an external `Call` to the `Staking` contract's `depositRewards` function for the remaining 0.05%.

## Official Go/No-Go Verdict
**NO-GO for Section 2 Subnetwork 3 Execution.**
The identified critical vulnerabilities (state rollback failure, non-deterministic state, broken staking authentication, and missing graduation logic) represent catastrophic risks to the network's economic security and consensus determinism. Mainnet launch for Subnetwork 3 must be delayed until these findings are remediated and the integration test suite (`cargo test -p zyanya-vm`) is expanded to cover state reverts and graduation paths.
