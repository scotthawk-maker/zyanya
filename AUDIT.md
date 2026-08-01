# Zyanya Pre-Launch Code Review & Security Audit

**Target System**: Zyanya Blockchain Node & Tooling (`rusty-spectre`)  
**Scope**: `zyanya-wallet`, `zyanya-vm`, `consensus`, `zyanya-explorer`, `database`, `zyanyad`  
**Date**: July 30, 2026  
**Auditor**: Antigravity AI Code Audit Team  

---

## 1. Executive Summary

A comprehensive pre-launch security and correctness review was conducted on the **Zyanya** codebase (a high-performance Spectre/GhostDAG fork featuring an embedded smart-contract VM and custom token ecosystem).

The audit focused on **consensus determinism**, **virtual machine safety**, **cryptographic key management**, **UTXO selection correctness**, **public IPv6 API resilience**, and **cross-platform database safety**.

### Overall Assessment
The Zyanya architecture is well-structured, leveraging Rust's safety guarantees and modular crate design. However, **one CRITICAL flaw** in the VM opcode stack manipulation and several **HIGH severity issues** in state storage non-determinism, integer arithmetic, secret key exposure, and public API query limits were identified. Addressing these findings prior to public testnet launch is mandatory to prevent consensus forks and security vulnerabilities.

---

## 2. Findings Summary by Severity

| Severity | Count | Primary Areas |
| :--- | :---: | :--- |
| 🔴 **CRITICAL** | 1 | Smart Contract VM (`SStore` Operand Stack Order) |
| 🟠 **HIGH** | 4 | State Non-Determinism (`HashMap`), Unchecked VM Math, Wallet Secret Leakage, Explorer DoS |
| 🟡 **MEDIUM** | 4 | VM `Pow` Resource Abuse, Wallet Float Amount Precision, Public RPC Unauthenticated Endpoints, Hex Parsing |
| 🔵 **LOW** | 2 | Keyfile File Permissions, RocksDB FD Limits |
| ℹ️ **INFO** | 1 | Genesis Zero-Premine Verification |

---

## 3. Detailed Findings (by Severity) & Concrete Fixes

### 🔴 CRITICAL SEVERITY

#### [CRIT-01] ~~Stack Operand Transposition in `OpCode::SStore`~~ — **FALSE POSITIVE (verified 2026-08-01)**
- **Location**: `zyanya-vm/src/vm.rs:225-230`
- **Original claim**: `OpCode::SStore` pops `val` first, followed by `key`:
  ```rust
  OpCode::SStore => {
      let val = self.stack.pop()?;
      let key = self.stack.pop()?;
      state.sstore(contract_address, key, val)?;
      self.pc += 1;
  }
  ```
  However, standard VM conventions and compiler codegen push `key` then `val` (so `val` is on top of the stack). When `SStore` pops `val` first, it interprets the top-of-stack as value and second item as key. But in test setup `Push(42); Swap; SStore` with stack `[300, 42]`, `pop()` assigns `val = 42` and `key = 300`, storing key `300` with value `42` instead of key `42` with value `300`.
- **Why it matters**: Severe contract state corruption where key and value are transposed, corrupting DEX liquidity pools, token balances, and contract storage on-chain.
- **Concrete Fix**:
  Standardize opcode stack popping order across compiler and VM. In `zyanya-vm/src/vm.rs`:
  ```rust
  OpCode::SStore => {
      let key = self.stack.pop()?;
      let val = self.stack.pop()?;
      state.sstore(contract_address, key, val)?;
      self.pc += 1;
  }
  ```

> **VERDICT: DO NOT APPLY — false positive.** The compiler codegen (`zyanya-vm/src/compiler/codegen.rs:209-215`) emits `sstore(key, val)` by pushing `args[0]` (key) **first**, then `args[1]` (val) **second**, so **val is on top of the stack**. The VM popping `val` first then `key` is therefore **correct and consistent**. Applying the proposed fix transposes the operands and **breaks 10 unit tests** (`test_stateful_sstore_sload`, `test_inter_contract_call`, `test_token_contract_full_lifecycle`, `test_bonding_curve_*`, etc.). All 28 `zyanya-vm` tests pass with the current code. The auditor's `Push(42); Swap; SStore` example mischaracterized the post-`Swap` stack order. — *Pi, 2026-08-01*

---

### 🟠 HIGH SEVERITY

#### [HIGH-01] ~~Non-Deterministic Storage Iteration Order in Consensus `ContractStateCache`~~ — **FIXED (2026-08-01)**
- **Location**: `consensus/src/model/stores/contract.rs:57-59`, `139-153`
- **What is wrong**: `ContractStateCache` uses standard `std::collections::HashMap` for `code`, `storage`, and `balances`. When `commit_cache_batch` iterates over state updates to write to RocksDB:
  ```rust
  for ((addr, key), val) in &cache.storage {
      let storage_key = ContractStorageKey::new(*addr, *key);
      self.storage_access.write(&mut writer, storage_key, *val)?;
  }
  ```
  `HashMap` iteration order is non-deterministic (SipHash randomized per process execution).
- **Why it matters**: If batch commits, state hashes, or Merkle roots rely on iteration order, different consensus nodes will process or commit state in different orders, leading to immediate consensus network forks.
- **Concrete Fix**:
  In `consensus/src/model/stores/contract.rs`:
  ```rust
  // Change HashMap to BTreeMap for deterministic key ordering:
  use std::collections::BTreeMap;

  #[derive(Clone, Default)]
  pub struct ContractStateCache {
      pub code: BTreeMap<[u8; 32], Vec<u8>>,
      pub storage: BTreeMap<([u8; 32], u64), u64>,
      pub balances: BTreeMap<[u8; 32], u64>,
      ...
  }
  ```

> **VERDICT: APPLIED.** Switched all three cache fields (`code`, `storage`, `balances`) from `HashMap` to `BTreeMap` for deterministic lexicographic iteration order in `commit_cache_batch`. Keys (`[u8;32]`, `([u8;32],u64)`) both implement `Ord`, and all call sites use only `insert`/`get`/`contains_key` (BTreeMap-compatible). Also fixed the stale `test_smart_contract_end_to_end_integration` deploy assertions (deploy no longer executes init since commit a14be79). All 49 consensus + 28 VM tests pass. — *Pi, 2026-08-01*

#### [HIGH-02] Unchecked Integer Overflow / Underflow in VM Arithmetic
- **Location**: `zyanya-vm/src/vm.rs:86-103`
- **What is wrong**: VM math opcodes (`Add`, `Sub`, `Mul`, `Pow`) perform `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_pow`:
  ```rust
  OpCode::Sub => {
      let b = self.stack.pop()?;
      let a = self.stack.pop()?;
      self.stack.push(a.wrapping_sub(b))?;
      self.pc += 1;
  }
  ```
- **Why it matters**: Smart contracts (e.g. DEX token balances or transfer calculations) will silently underflow or overflow instead of reverting. An attacker can underflow a transfer calculation `balance - amount` to obtain `u64::MAX` tokens.
- **Concrete Fix**:
  In `zyanya-vm/src/vm.rs`, use checked arithmetic or return explicit error:
  ```rust
  OpCode::Sub => {
      let b = self.stack.pop()?;
      let a = self.stack.pop()?;
      let res = a.checked_sub(b).ok_or(VMError::ArithmeticOverflow)?;
      self.stack.push(res)?;
      self.pc += 1;
  }
  ```

#### [HIGH-03] Secret Keys Printed to Terminal Output in Wallet CLI
- **Location**: `zyanya-wallet/src/main.rs:131`, `145`, `214`, `363`, `376`
- **What is wrong**: CLI commands (`--generate-key`, `--generate-mnemonic`, `--import-mnemonic`, `--demo`) print unencrypted 64-character secret hex keys to standard output (`SecretKey: {}`).
- **Why it matters**: Secret keys are recorded in terminal scrollback buffers, shell history files (`.bash_history`), process capture logs, or CI/CD logs, exposing private keys to fund theft.
- **Concrete Fix**:
  In `zyanya-wallet/src/main.rs`, suppress secret key printing by default or require `--show-secret`:
  ```rust
  // Print address only by default:
  println!("  Address:   {}", keypair.address);
  // Do NOT print secret_hex() unless explicitly requested via --show-secret flag.
  ```

#### [HIGH-04] Unbounded Database Collection Query Responses in Explorer API
- **Location**: `zyanya-explorer/src/api.rs:180-210`, `client.rs:600-650`
- **What is wrong**: Public API endpoints `/api/contracts`, `/api/tokens`, and `/api/dag` fetch and return all records from the node without server-side pagination limits.
- **Why it matters**: As the blockchain state grows, a single HTTP request to `/api/contracts` will consume large amounts of node memory and bandwidth, creating a trivial Remote Denial-of-Service vector against public nodes.
- **Concrete Fix**:
  In `zyanya-explorer/src/api.rs`, enforce strict limit and offset parameters:
  ```rust
  let limit = query.limit.unwrap_or(20).min(100);
  ```

---

### 🟡 MEDIUM SEVERITY

#### [MED-01] Resource Exhaustion Vector via Unbounded `Pow` Opcode Exponent
- **Location**: `zyanya-vm/src/vm.rs:122-127`
- **What is wrong**: `OpCode::Pow` consumes 1 base gas unit but calculates `a.wrapping_pow(b as u32)`. `b` can be set to `u32::MAX` (4,294,967,295).
- **Why it matters**: A contract can trigger expensive BigInt/exponent computations for 1 gas, slowing down block processing across validators.
- **Concrete Fix**:
  In `zyanya-vm/src/vm.rs`, charge dynamic gas based on the exponent:
  ```rust
  OpCode::Pow => {
      let b = self.stack.pop()?;
      let a = self.stack.pop()?;
      let gas_cost = 1 + (b as u64 / 32);
      self.gas_meter.consume(gas_cost)?;
      self.stack.push(a.wrapping_pow(b as u32))?;
      self.pc += 1;
  }
  ```

#### [MED-02] Loss of Precision from Floating-Point Currency Conversions
- **Location**: `zyanya-wallet/src/main.rs:265`, `zyanya-wallet/src/tui.rs:158`
- **What is wrong**: Currency amounts are parsed as IEEE-754 floats and multiplied by `SOMPI_PER_ZYANYA`:
  ```rust
  let amount_sompi = (zyan_val * SOMPI_PER_ZYANYA as f64) as u64;
  ```
- **Why it matters**: Floating-point precision issues (e.g. `0.1 + 0.2 = 0.30000000000000004`) lead to off-by-one sompi discrepancies or transaction construction failures.
- **Concrete Fix**:
  Parse fixed-point decimal strings directly into integer sompi values without `f64`.

#### [MED-03] Hex Parameter Parsing Logic Discrepancy in State Handler
- **Location**: `zyanya-explorer/src/api.rs:86-88`
- **What is wrong**: `api_contract_state_handler` strips `"0x"` prefix, but calls standard decimal `.parse::<u64>()`:
  ```rust
  let key_val = query.key.as_deref()
      .map(|k| k.trim_start_matches("0x").parse::<u64>().unwrap_or(0))
      .unwrap_or(0);
  ```
- **Why it matters**: Querying `?key=0x10` parses to `10` decimal instead of key `16` (`0x10` hex).
- **Concrete Fix**:
  Use `u64::from_str_radix(clean_str, 16)` when the string starts with `"0x"`.

#### [MED-04] Unauthenticated Public Contract Invocation Endpoints
- **Location**: `zyanya-explorer/src/api.rs:218-285`
- **What is wrong**: Endpoints `/api/deploy-contract`, `/api/invoke-contract`, `/api/swap-on-dex` permit unauthenticated external users to trigger node contract operations.
- **Why it matters**: Attackers can flood nodes with invalid contract invocations or CPU-heavy executions.
- **Concrete Fix**:
  Add rate-limiting middleware (`tower_governor`) and disable state-changing RPC endpoints on public read-only block explorer deployments.

---

### 🔵 LOW SEVERITY

#### [LOW-01] Missing Permission Mode Check on Saved Wallet Key Files
- **Location**: `zyanya-wallet/src/key_management.rs:120-127`
- **What is wrong**: `save_to_file` creates `~/.zyanya/wallet.key` with default OS permissions (`0644` on Unix).
- **Why it matters**: Other unprivileged local users on the server can read the secret key file.
- **Concrete Fix**:
  Use `std::os::unix::fs::PermissionsExt` to set file permissions to `0600` on Unix platforms.

#### [LOW-02] RocksDB Connection Builder Safe Mode Analysis
- **Location**: `database/src/db/conn_builder.rs:99-120`
- **What is wrong / Assessment**:
  - The RocksDB connection builder handles file limits via `zyanya_utils::fd_budget::acquire_guard`.
  - Default parallelism is set to `1`, preventing thread over-allocation.
- **Why it matters**: Safe cross-platform fallback.
- **Verdict**: Fully safe and effective on Linux while protecting Windows host systems from FD crashes. WAL durability is preserved.

---

### ℹ️ INFORMATIONAL

#### [INFO-01] Genesis Block Zero-Premine Verification
- **Location**: `consensus/core/src/config/genesis.rs:67-121`
- **Assessment**:
  - `MAINNET_GENESIS` and `TESTNET_GENESIS` coinbase payloads contain 50 ZYAN subsidy sent to an `OP_FALSE` unspendable script (`ZYAN-MAINNET` and `ZYNT-TESTNET`).
  - No pre-allocated balances, team wallets, or initial supply allocations exist in genesis headers.
- **Verdict**: Zero premine setup is **VERIFIED & CORRECT**.

---

## 4. Section-by-Section Review Findings

### 4.1 Consensus Determinism (VM Review)
- **Determinism Check**: Float opcodes are non-existent (VM operates exclusively on `u64`). Host I/O and non-deterministic randomness are not present inside VM execution loops.
- **HashMap Risk**: `ContractStateCache` (`contract.rs`) uses `std::collections::HashMap` for storage maps. Changing to `BTreeMap` is required to eliminate non-deterministic iteration in consensus state commits.
- **OpCode Stack Alignment**: `SStore` operand ordering must be corrected to match compiler codegen.

### 4.2 Wallet Robustness (`zyanya-wallet`)
- **Key Generation & RNG**: `OsRng` from `rand` and BIP-39 mnemonic generation (`zyanya-bip32`) are cryptographically sound.
- **Overwrite Protection**: `--force` flag in `main.rs:108-119` correctly prevents accidental key file clobbering.
- **UTXO Selection**: `wallet_ops.rs:201-203` correctly enforces the 100-block maturity requirement for coinbase UTXOs (`current_daa - entry.block_daa_score >= 100`).

### 4.3 Explorer / Public API Robustness (`zyanya-explorer`)
- **IPv6 Binding**: `main.rs:31-47` correctly sets `IPV6_V6ONLY=true` via `socket2`, successfully enforcing IPv6-only listener semantics.
- **Input Validation**: Needs pagination limits on collection endpoints and hex-radix fix on state queries.

### 4.4 Windows Port Assessment (`database`)
- Connection builder safe mode (`conn_builder.rs`) operates cleanly across Linux and Windows without degrading durability or WAL syncs.

---

## 5. Overall Code Quality & Final Verdict

| Metric | Rating | Notes |
| :--- | :--- | :--- |
| **Architecture** | Excellent | Clean crate breakdown, modular design |
| **Consensus Safety** | Needs Attention | `CRIT-01` is a **false positive** (verified — see note above); `HIGH-01` (`HashMap`) still needs fix |
| **Wallet Security** | Good | Robust BIP-39 seed support; mask secret keys in stdout |
| **API Security** | Moderate | Add pagination limits and API rate-limiting |

**Final Verdict**: **PASS WITH REQUIRED REMEDIATIONS**  
Remediate `HIGH-01` (`HashMap` determinism) prior to public testnet release. (`CRIT-01` was investigated and is a false positive — no change needed.)
