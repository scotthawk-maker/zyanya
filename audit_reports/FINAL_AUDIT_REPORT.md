# Zyanya Blockchain — Consolidated Security Audit Report

**Date:** 2026-08-18  
**Scope:** All five audit phases — `phase1_findings.md` through `phase5_findings.md`  
**Methodology:** Source code was **not modified** at any point. All `file:line` references were re-verified per phase. This report is a synthesis and deduplication of the five phase reports; no phase report or source file was altered.

> **Phase 2 count discrepancy:** The original task brief states Phase 2 has "18 findings (3 CRITICAL, 5 HIGH, 5 MEDIUM, 5 LOW)". The Phase 2 report's own summary table lists **17** findings (3 CRITICAL, **4** HIGH, 5 MEDIUM, 5 LOW) because its `H-05` is marked "Verified safe (no finding)". This report uses the report's actual numbers (4 HIGH) and records `P2 H-05` in Appendix B as verified-safe.

> **Phase 5 informational items:** Phase 5 reports 3 CRITICAL, 4 HIGH, 10 MEDIUM, 20 LOW, plus 14 Informational items (`I-01`…`I-14`). The informational items are listed in Appendix B and are **not** included in the severity counts.

---

## 1. Executive Summary

### Deduplicated totals

| Severity | Count |
|----------|-------|
| CRITICAL | 16 |
| HIGH     | 26 |
| MEDIUM   | 39 |
| LOW      | 47 |
| **Total**| **128** |

### Raw totals (before deduplication)

| Severity | Count |
|----------|-------|
| CRITICAL | 17 |
| HIGH     | 29 |
| MEDIUM   | 40 |
| LOW      | 49 |
| **Total**| **135** |

Seven raw entries across phases collapse into six canonical findings via the deduplication mapping in §2, reducing 135 → 128.

### Top risks (thematic)

1. **Consensus integrity** — RPC fallback bypasses consensus and persists contract state without a mined tx (F-C-01); failed contract txs still commit state (F-C-02); `f64::powf` in the subsidy schedule risks a cross-platform consensus fork (F-C-06); the coinbase output-count limit is too small for the 13× vesting split and can halt the chain (F-C-07); transaction mass calculation overflow enables block-mass bypass (F-C-08).

2. **Smart-contract / VM security** — No `msg.sender` or authentication in token, bonding-curve, or DEX contracts (F-C-04); no reentrancy guard or call-depth limit in `CALL` (F-C-05); bonding-curve custody accounting is broken (buy never deducts, sell destroys the refund) (F-C-03); DEX/bonding-curve math overflows u64 (F-H-02); jump-target byte-offset vs opcode-index confusion (F-H-03).

3. **Network / RPC DoS** — 1 GB message size limits across all transports with gzip compression bombs (F-C-12); no authentication on any RPC method (F-C-13); unbounded bytecode/calldata in contract RPC handlers (F-C-10); no P2P or wRPC connection limits (F-H-18, F-H-23); no ban for spam peers (F-H-20).

4. **Wallet key management** — Private key material leaked via derived `Debug` impls (F-H-11); wallet file written with world-readable permissions (F-H-12); `minimum_signatures == 0` produces an anyone-can-spend multisig (F-C-09); no `Drop`/`Zeroize` on key-bearing types (F-H-14, F-H-15); double addition of change to fees (F-H-13).

5. **Explorer web security** — Stored XSS via token metadata social links and token name/symbol (F-C-14, F-C-15); unsigned tx metadata injection because signatures don't cover metadata (F-C-16); reflected XSS via API error messages (F-H-26); no CORS restrictions or security headers (F-M-31, F-L-33).

### Overall security posture

Zyanya is a fork of Kaspa/rusty-spectre. The upstream consensus, crypto, and wallet code is largely sound — the majority of verified-safe items (Appendix B) trace to upstream Kaspa. However, the **Zyanya-specific additions** — the custom VM, smart-contract layer, bonding-curve/DEX contracts, contract RPC handlers, and the block explorer — introduce the overwhelming majority of CRITICAL and HIGH findings.

**Verdict: NOT production-ready.** All 16 CRITICAL findings must be remediated before any mainnet launch. The consensus-bypass (F-C-01), commit-on-failure (F-C-02), and `f64::powf` subsidy fork (F-C-06) are each individually sufficient to prevent a safe launch.

---

## 2. Deduplication & Cross-Reference Table

### Merged findings (6 canonical findings from 7+ raw entries)

| # | Canonical ID | Title | Phase IDs merged | Canonical severity |
|---|--------------|-------|------------------|--------------------|
| 1 | F-C-01 | RPC fallback bypasses consensus, persisting contract state without a mined tx | P1 C-01 + P4 C-01 | CRITICAL |
| 2 | F-H-02 | DEX / bonding-curve math overflows u64 (on-chain ZCL + off-chain explorer client) | P1 H-02 + P5 H-01 + P5 H-02 | HIGH |
| 3 | F-H-04 | Ungated `/api/compile` endpoint + parser-recursion DoS | P1 H-04 + P5 H-04 | HIGH |
| 4 | F-H-07 | Contract payload validation gaps — no size bounds on bytecode/parameters (borsh OOM) | P1 M-05 + P2 H-01 | HIGH |
| 5 | F-L-01 | `check_transaction_outputs_count` reports input fields in error (copy-paste) | P1 L-01 + P2 L-01 | LOW |
| 6 | F-M-06 | `unsafe` `AsRef<[u8]>` for `#[repr(C)] ContractStorageKey` | P1 L-02 + P2 M-02 | MEDIUM |

For each merged finding, the canonical entry below combines the most complete description/code/fix from the contributing phases and lists all original phase IDs in a `Cross-references:` line.

### Related-but-distinct findings (cross-referenced, NOT merged)

These share a root cause or pattern but are at different files/layers. They are kept as separate findings with `Related:` pointers:

- F-H-06 (wallet_ops.rs UTXO unchecked add) ↔ F-L-15 (utxo/context.rs `sum()` overflow)
- F-C-10 (RPC unbounded bytecode/calldata) ↔ F-H-07 (consensus payload validation)
- F-M-34 (database registry unsafe cast) ↔ F-M-06 (ContractStorageKey unsafe cast)
- F-M-03 (parser recursion depth) ↔ F-H-04 (ungated compile endpoint)
- F-C-13 (no RPC auth) ↔ F-C-01 (RPC consensus bypass)
- F-C-12 (1 GB message size) ↔ F-H-24 (inconsistent wRPC size)
- F-H-18 (no P2P conn limit) ↔ F-H-23 (no wRPC conn limit)
- F-H-20 (tx relay no ban) ↔ F-M-24 (RequestTransactionsFlow no rate limit)
- F-C-14/F-C-15 (stored XSS) ↔ F-H-26 (reflected XSS) ↔ F-M-30/F-M-37 (DAG modal XSS) ↔ F-L-33 (no security headers)
- F-H-11 (Debug key leak) ↔ F-L-46 (Rothschild logs private key) ↔ F-H-14/F-H-15 (no zeroize)
- F-M-11 (gen1 decrypt panics/UB) ↔ F-L-18 (from_utf8_unchecked in gen1)
- F-M-07 / F-M-23 / F-L-35 / F-L-43 (unsafe `from_utf8_unchecked` hex/display family)
- F-M-32 (unbounded notify channels) ↔ F-L-31 (unbounded mining sender)
- F-M-36 (/tmp fallback) ↔ F-H-12 (wallet file perms)
- F-M-38 (DB conn builder panic) ↔ F-L-37 (delete_db panic)
- F-L-39 (network() panic) ↔ F-L-40 (daemon expect)
- F-L-41 (topological sort assert) ↔ F-L-42 (mass assert)
- F-L-44 (hex.rs unwrap) ↔ F-L-45 (networking unwrap)
- F-L-47 (CLI wizard secrets) ↔ F-H-11/F-H-14/F-H-15 (key handling)

---

## 3. CRITICAL Findings (full details)

### F-C-01 — RPC fallback bypasses consensus, persisting contract state without a mined transaction

- **Severity:** CRITICAL
- **File:** `rpc/service/src/service.rs:614-635` (deploy) and `:676-716` (invoke)
- **Cross-references:** P1 C-01, P4 C-01 *(P4 re-verified line numbers; unchanged)*

**Description:** Both `deploy_contract_call` and `invoke_contract_call` first try to submit the transaction to the mempool via `submit_rpc_transaction`. If that call returns **any** error (mempool full, validation failure, network glitch, or simply a tx with no inputs that the mempool rejects), the handler falls back to executing the contract directly with `ContractProcessor::process_contract_tx` against a freshly constructed `ContractStateCache` and then **writes the resulting state to the durable consensus store** via `session.write_contract_code / write_contract_storage / write_contract_balance`. State is mutated and persisted **without** a mined/accepted transaction, without GhostDAG ordering, and without consensus validation. There is **no authentication** on the RPC surface — any RPC client can trigger this path by submitting a tx that the mempool rejects (trivial: deploy/invoke txs built here have `inputs = []` and `outputs = []`, which the mempool routinely rejects, making the fallback the *primary* path in practice). The `invoke` fallback even seeds `cache.fallback_storage` / `fallback_balance` from the live session, so it reads + writes real chain state. This is a direct consensus-bypass / unauthenticated-state-mutation primitive.

**Code** (`service.rs:614-635`):
```rust
let submit_res = self.flow_context.submit_rpc_transaction(
    &session, tx.clone(), Orphan::Forbidden).await;
let (gas_used, success) = match submit_res {
    Ok(_) => (request.max_gas, true),
    Err(_) => {
        // Fallback to direct execution for input-less RPC convenience txs
        let mut cache = ContractStateCache::new();
        let processor = ContractProcessor::new();
        let outcome = processor.process_contract_tx(&tx, &mut cache);
        let (g_used, succ) = outcome.map(|o| (o.gas_used, o.success)).unwrap_or((0, false));
        if succ {
            for (addr, code) in cache.code {
                let hash_addr = Hash::from_bytes(addr);
                let _ = session.write_contract_code(hash_addr, code);
            }
            for ((addr, key), val) in cache.storage {
                let _ = session.write_contract_storage(addr, key, val);
            }
        }
        (g_used, succ)
    }
};
```
The `invoke_contract_call` fallback (`:698-716`) additionally writes `cache.balances`.

**Recommended fix:** Remove the fallback entirely; return the mempool/RPC error to the caller. If a read-only simulation is desired, use a discarded cache as `call_contract_call` (`:749+`) already does — never persist. Any state-changing operation must go through the transaction pool and consensus.

---

### F-C-02 — Failed contract transactions still commit state (commit-on-failure)

- **Severity:** CRITICAL
- **File:** `consensus/src/pipeline/virtual_processor/processor.rs:511-521` and `consensus/src/model/stores/contract.rs:213` (Deploy) / `:281` (Invoke)
- **Cross-references:** P1 C-02

**Description:** In the virtual processor, every contract tx is passed to `ContractProcessor::process_contract_tx`, and the caller commits the cache whenever the return value `.is_some()` — **regardless of `outcome.success`**. `process_contract_tx` returns `Some(outcome)` even when `success == false`, so a **failed** contract tx persists all cache mutations made before/during execution. Compounding this: the Deploy branch (`contract.rs:213`) inserts bytecode *before* any execution attempt — a deploy with invalid bytecode still permanently installs the contract code. The Invoke branch (`:281-283`) credits `deposit_amount` to `cache.balances` *before* execution — a failed invoke still credits the deposit, meaning the caller's ZYAN is created out of thin air. Result: failed deploys install bytecode; failed invokes mint balance to contracts; all committed by the virtual processor.

**Code** (`contract.rs:213-218`):
```rust
cache.code.insert(addr_bytes, deploy.bytecode.clone());
if deploy.deposit_amount > 0 {
    let current_balance = cache.get_balance(&addr_bytes);
    cache.balances.insert(addr_bytes, current_balance.saturating_add(deposit.deposit_amount));
}
```

**Recommended fix:** (1) In `processor.rs`, commit only when `outcome.map(|o| o.success).unwrap_or(false)`. (2) In `process_contract_tx`, perform *all* cache mutations on a `temp_cache` and swap into the real `cache` only on `success == true` — mirror the existing Invoke `temp_cache` pattern in the Deploy branch and move the `deposit_amount` credit to after a successful execution.

---

### F-C-03 — Bonding-curve custody accounting gap: buy does not deduct ZYAN, sell destroys refund

- **Severity:** CRITICAL
- **File:** `consensus/src/model/stores/contract.rs:213-217` (deploy deposit credit), `:281-283` (invoke deposit credit), `:320-333` (buy), `:335-353` (sell)
- **Cross-references:** P1 C-03

**Description:** The consensus layer models contract ZYAN custody via a `cache.balances` map, but the accounting is broken in both directions. **Buy (entry_point 4, `:320-333`):** the contract code computes `cost` and adds it to `reserve` (storage key 2), and the processor checks `deposit_amount >= cost`. However, `deposit_amount` was already credited to `cache.balances` before execution, and **nothing ever deducts `cost` from the contract balance**. The buyer's ZYAN is never escrowed — `deposit_amount` is a free-form payload field, not a UTXO output locked to the contract — so the contract "reserve" grows while no real ZYAN is held. A buyer can set `deposit_amount = 0` and still mint tokens. **Sell (entry_point 5, `:335-353`):** the processor deducts `refund` from `temp_cache.balances`, but **no UTXO output is created paying the seller**. The seller's refund is simply destroyed from the contract balance — the seller receives nothing, and the ZYAN vanishes. There is no reconciliation between `balances` and `reserve`, and no real UTXO value transfer on either side. The entire bonding-curve economy is uncollateralized.

**Code** (`contract.rs:281-283`):
```rust
if invoke.deposit_amount > 0 {
    let current_balance = cache.get_balance(&addr_bytes);
    cache.balances.insert(addr_bytes, current_balance.saturating_add(invoke.deposit_amount));
}
```
(`contract.rs:320-333` — buy checks `deposit_amount >= cost` but never deducts `cost`; `:335-353` — sell deducts `refund` from `balances` but creates no payout UTXO.)

**Recommended fix:** Model custody as real UTXO value transfer: the invoke tx must include an output locking `cost` sompi to the contract address on buy, and the block processor must create an output paying `refund` to the seller on sell. Reconcile `cache.balances` with `reserve` (key 2) on every buy/sell. Reject buy when `deposit_amount < cost` *including* the `deposit_amount == 0` case.

---

### F-C-04 — No `msg.sender` / no authentication in token, bonding-curve, and DEX contracts

- **Severity:** CRITICAL
- **Files:** `bonding_curve.zcl:18-26` (transfer), `:40-52` (buy), `:54-72` (sell); `dex.zcl:17-36` (addLiquidity), `:49-66` (swap), `:74-97` (removeLiquidity); `zyanya-vm/src/token.rs` (transfer `:80-100`, mint `:115-130`); `zyanya-vm/src/bonding_curve_token.rs` (transfer, buy, sell)
- **Cross-references:** P1 C-04

**Description:** The VM has no notion of an authenticated caller (`msg.sender`). Every function that should be sender-gated takes the "caller" / "from" as a **caller-supplied parameter** with no check that the transaction signer owns that identity. `transfer(from, to, amount)` — anyone can pass any `from` and drain that holder's entire balance (direct theft primitive). `buy(caller, tokens_to_mint)` and `sell(caller, tokens_in)` — anyone can mint tokens to / sell tokens from any address. `mint` in `token.rs` is callable by anyone at any time (entry point 3), with no owner check — unlimited inflation. `init` (entry point 0) in the bonding curve is callable by anyone at any time, resetting the slope and zeroing supply/reserve — a griefing/destroy primitive. DEX `addLiquidity(caller, …)`, `removeLiquidity(caller, lpAmount)` — anyone can claim/remove another user's LP tokens. The consensus layer passes only `entry_point` and `parameters` to the VM (`contract.rs:307-309`); the transaction signer is never propagated as an authenticated identity.

**Code** (`bonding_curve.zcl:18-26`):
```js
fn transfer(from, to, amount) {
    let from_bal = sload(from);
    if (from_bal < amount) { return 0; }
    sstore(from, from_bal - amount);
    let to_bal = sload(to);
    sstore(to, to_bal + amount);
    return 1;
}
```

**Recommended fix:** Introduce an authenticated caller identity in the VM (derive `msg.sender` from the transaction signer and expose it via a new opcode or a reserved stack slot). Enforce `from == msg.sender` in `transfer`, `caller == msg.sender` in buy/sell/addLiquidity/removeLiquidity, and gate `mint`/`init` to the contract owner.

---

### F-C-05 — VM `CALL` opcode has no reentrancy guard and no call-depth limit

- **Severity:** CRITICAL
- **File:** `zyanya-vm/src/vm.rs:236-272`
- **Cross-references:** P1 C-05

**Description:** `OpCode::Call` spawns a child `VM` that shares the **same `&mut S StateBackend`** as the caller. Two critical problems: (1) **No call-depth limit**: A contract can `CALL` itself (or a mutually-recursive pair) indefinitely. Each `CALL` recurses into `execute_stateful`, consuming a Rust stack frame. Deep recursion → Rust stack overflow → **node process crash (DoS)**. Gas bounds work, but a single tx with large `max_gas` can recurse thousands of frames before running out of gas. (2) **No reentrancy guard**: The child shares the same `state` backend, so a callee can re-enter the caller mid-execution and mutate shared storage. This is the classic reentrancy pattern exploitable against the bonding-curve and DEX contracts (which read-then-write storage without locks). Combined with F-C-04 (no auth), a malicious contract called during a buy/sell/swap can manipulate reserves.

**Code** (`vm.rs:236-272`):
```rust
OpCode::Call(target_addr) => {
    let calldata = self.stack.pop()?;
    let forward_gas = self.stack.pop()?;
    self.gas_meter.consume(forward_gas)?;
    // ... fetch + deserialize code ...
    let mut child_vm = VM::new(forward_gas);
    if calldata > 0 { let _ = child_vm.stack.push(calldata); }
    match child_vm.execute_stateful(&target_opcodes, target_addr, state) {
        Ok(res) => { /* refund unused */ self.stack.push(res.return_value.unwrap_or(0))?; }
        Err(_) => { self.stack.push(0)?; }
    }
    self.pc += 1;
}
```

**Recommended fix:** Add a call-depth counter (threaded through `execute_stateful` or stored on the VM) and reject `CALL` beyond a limit (e.g. 1024). Add a per-contract reentrancy guard ("entered" flag in state) and reject re-entry. Consider giving the child a *copy* of state for read-only calls, or a journal that commits atomically on success.

---

### F-C-06 — Floating-point (`f64::powf`) in consensus-critical subsidy schedule

- **Severity:** CRITICAL
- **File:** `consensus/src/processes/coinbase.rs:88-95`
- **Cross-references:** P2 C-01

**Description:** For mainnet (`deflationary_phase_daa_score == 31_449_600`), the 727-entry subsidy table is computed at startup using `f64::powf`. `f64::powf` is **not guaranteed bit-identical across platforms, compilers, or libm implementations** (x86 vs ARM, glibc vs musl, compiler flags like `-ffast-math`). A 1-ULP difference in `powf` can flip `.round()` to a different integer for some month index, causing two nodes to compute different subsidy tables. Since the table is built once at startup and used for every `calc_block_subsidy` call, the divergence is silent and persistent per node. If a node's table disagrees with the majority, every coinbase it validates after the deflationary phase begins will be rejected or accepted incorrectly → **consensus fork**. The non-mainnet path correctly uses the pre-computed integer `SUBSIDY_BY_MONTH_TABLE` (line 97), which is deterministic. Only the mainnet path uses floats — this is a Zyanya-specific deviation.

**Code:**
```rust
// coinbase.rs:86-95
let subsidy_by_month_table: SubsidyByMonthTable = if deflationary_phase_daa_score == 31_449_600 {
    core::array::from_fn(|i| {
        let m = i as f64;
        let decay_rate = 0.004847033515f64;
        let initial = 5_000_000_000f64;
        let month_subsidy = (initial * (1.0 - decay_rate).powf(m)).round() as u64;
        month_subsidy.div_ceil(bps)
    })
} else {
    core::array::from_fn(|i| SUBSIDY_BY_MONTH_TABLE[i].div_ceil(bps))
};
```

**Recommended fix:** Replace the float computation with a pre-computed constant integer table (like the existing `SUBSIDY_BY_MONTH_TABLE` for non-mainnet), or use a fixed-point integer recurrence. Add cross-platform golden-vector tests that assert the exact table bytes on at least x86-64 and aarch64.

---

### F-C-07 — Coinbase output-count limit too small for 13× vesting split

- **Severity:** CRITICAL
- **File:** `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:46-48` and `consensus/src/processes/coinbase.rs:125-190`
- **Cross-references:** P2 C-02

**Description:** The coinbase output limit is set to `(ghostdag_k + 2) * 13`. On mainnet with `ghostdag_k = 18`, this is `(18 + 2) * 13 = 260` outputs. `expected_coinbase_transaction` emits **13 outputs per DAA-window blue block** (1 liquid + 12 CSV-locked vested) plus up to 13 for the red reward block. The number of DAA-window blues in a mergeset is bounded by `mergeset_size_limit = ghostdag_k * 10 = 180`, **not** by `ghostdag_k + 2`. A block whose mergeset contains more than 19 DAA-window blues (each with non-zero reward) will produce a coinbase with more than 260 outputs, which will be **rejected by every node** as `CoinbaseTooManyOutputs`. Since the miner cannot control the exact number of DAA-window blues in the mergeset (it depends on network topology), this can happen organically as the DAG grows. If the network reaches a state where no valid coinbase can be constructed, the chain halts → **consensus split / chain halt**. The `* 13` factor is a Zyanya addition (upstream Kaspa has 1 output per blue, with limit `ghostdag_k + 2`). The base `ghostdag_k + 2` was not re-derived for the vesting split.

**Code:**
```rust
// tx_validation_in_isolation.rs:46-48
let outputs_limit = (self.ghostdag_k as u64 + 2) * 13;
if tx.outputs.len() as u64 > outputs_limit {
    return Err(TxRuleError::CoinbaseTooManyOutputs(tx.outputs.len(), outputs_limit));
}
```
```rust
// coinbase.rs:125
let mut outputs = Vec::with_capacity((ghostdag_data.mergeset_blues.len() + 1) * 13);
```

**Recommended fix:** Derive the limit from the actual maximum number of payable blues, e.g. `(mergeset_size_limit + 1) * 13` (where `+1` accounts for the red reward block). On mainnet this would be `(180 + 1) * 13 = 2353`. Alternatively, cap the number of vested outputs per blue. Add a test that builds a coinbase with > 19 DAA-window blues and verifies it passes validation.

---

### F-C-08 — Transaction mass calculation overflow enables block-mass limit bypass

- **Severity:** CRITICAL
- **File:** `consensus/core/src/mass/mod.rs:87-105` (`calc_tx_compute_mass`), and `consensus/core/src/config/params.rs:366-369` (`max_tx_inputs/outputs = 1_000_000_000`)
- **Cross-references:** P2 C-03

**Description:** `calc_tx_compute_mass` uses plain `*` and `+` (wrapping in release) to compute mass. With mainnet `max_tx_inputs = max_tx_outputs = 1_000_000_000` and `max_signature_script_len = 1_000_000_000`, a single transaction's estimated serialized size can reach ~10^18 bytes. If the total overflows and wraps to a small value, `max_block_mass = 500_000` can be satisfied by a transaction that is actually enormous → **block-mass limit bypass**. The params comment acknowledges this: *"These values should be lowered to more reasonable amounts on the next planned HF/SF."*

**Code:**
```rust
// mass/mod.rs:91-104
let size = transaction_estimated_serialized_size(tx);
let mass_for_size = size * self.mass_per_tx_byte;
let total_script_public_key_size: u64 = tx.outputs.iter()
    .map(|output| 2 + output.script_public_key.script().len() as u64).sum();
let total_script_public_key_mass = total_script_public_key_size * self.mass_per_script_pub_key_byte;
let total_sigops: u64 = tx.inputs.iter().map(|input| input.sig_op_count as u64).sum();
let total_sigops_mass = total_sigops * self.mass_per_sig_op;
mass_for_size + total_script_public_key_mass + total_sigops_mass
```

**Recommended fix:** Use `checked_add`/`saturating_add` in `calc_tx_compute_mass` and `transaction_estimated_serialized_size`, returning `u64::MAX` on overflow. Additionally, lower the mainnet `max_tx_inputs`, `max_tx_outputs`, `max_signature_script_len`, and `max_script_public_key_len` to values that cannot overflow (e.g. 10_000 as on testnet11).

---

### F-C-09 — `minimum_signatures == 0` produces an anyone-can-spend multisig address

- **Severity:** CRITICAL
- **File:** `wallet/core/src/derivation.rs:400` (`create_address`), `:428` (`create_multisig_address`); `wallet/core/src/account/variants/multisig.rs:117` (`Payload` deserialization); `crypto/txscript/src/standard/multisig.rs:18`
- **Cross-references:** P3 C-01

**Description:** `create_address` validates only `length < minimum_signatures` — it never rejects `minimum_signatures == 0`. When `minimum_signatures` is 0 and `keys.len() > 1`, the function calls `create_multisig_address(0, keys, ...)`, which calls `multisig_redeem_script(..., 0)`. The txscript `multisig_redeem_script` function does **not** reject `required == 0` (it only rejects `required > count` and `count == 0`). The resulting redeem script is `OP_0 <pubkeys...> OP_N OP_CHECKMULTISIG` — a 0-of-N multisig that is spendable by anyone who supplies an empty signature vector. `Payload::minimum_signatures: u16` is deserialized from wallet storage with no validation, so a crafted or corrupted wallet file can set it to 0. Any funds sent to such an address are immediately stealable.

**Code:**
```rust
// wallet/core/src/derivation.rs:400
pub fn create_address(minimum_signatures: usize, keys: Vec<secp256k1::PublicKey>, ...) -> Result<Address> {
    let length = keys.len();
    if length < minimum_signatures {   // does NOT check minimum_signatures == 0
        return Err(...);
    }
    if length > 1 {
        return create_multisig_address(minimum_signatures, keys, prefix, ecdsa);
    }
    ...
}

// crypto/txscript/src/standard/multisig.rs:18
pub fn multisig_redeem_script(pub_keys: ..., required: usize) -> Result<Vec<u8>, Error> {
    // no check for required == 0
    builder.add_i64(required as i64)?;
    ...
}
```

**Recommended fix:** Reject `minimum_signatures == 0` in `create_address`, `AddressManager::new`, and `Payload` deserialization (add a guard that returns `Err` when `minimum_signatures == 0 || minimum_signatures > xpub_keys.len()`). Also add the guard to `multisig_redeem_script` in `crypto/txscript`.

---

### F-C-10 — Unbounded bytecode/calldata with no size cap — memory DoS and VM gas bypass

- **Severity:** CRITICAL
- **File:** `rpc/service/src/service.rs:589` (`request.bytecode`), `:660` (`request.parameters`), `:770` (`request.calldata`), `:773` (`request.max_gas`)
- **Cross-references:** P4 C-02. **Related:** F-H-07 (consensus payload validation).

**Description:** The `DeployContractRequest.bytecode`, `InvokeContractRequest.parameters`, and `CallContractRequest.calldata` fields are `Vec<u8>` / `Vec<u64>` with **no size limit** enforced before being passed to the VM or contract processor. Additionally, `max_gas` is a client-supplied `u64` with no server-side cap in any of the contract RPC handlers. An attacker can: submit a multi-hundred-MB bytecode (up to the 1 GB gRPC message limit) to exhaust memory; submit `calldata` of arbitrary length to `call_contract_call`, where `calldata.chunks_exact(8)` pushes an unbounded number of values onto the VM stack; set `max_gas = u64::MAX` to force the VM to run for an arbitrary duration (CPU DoS).

**Code** (`service.rs:773-793`):
```rust
let mut vm = zyanya_vm::VM::new(request.max_gas);
// ...
for chunk in request.calldata.chunks_exact(8) {
    if let Ok(arr) = chunk.try_into() {
        let val = u64::from_le_bytes(arr);
        let _ = vm.stack.push(val);
    }
}
```

**Recommended fix:** Enforce maximum bytecode size (e.g. 1 MB), maximum calldata/parameters length, and a server-side `max_gas` cap on all contract RPC handlers. Reject oversized requests before any VM execution.

---

### F-C-11 — Panic vectors in contract RPC handlers — `.try_into().unwrap()` on contract address

- **Severity:** CRITICAL
- **File:** `rpc/service/src/service.rs:691`, `:733`, `:797`
- **Cross-references:** P4 C-03

**Description:** Three contract RPC handlers convert `request.contract_address.as_bytes()` to `[u8; 32]` via `.try_into().unwrap()`. While `RpcHash` is currently a 32-byte hash type, this pattern is a latent panic: if the hash type ever changes size, or if a malformed/differently-sized hash arrives through a future code path, the `unwrap()` will panic the handler task. In a multi-tenant RPC server this constitutes a denial-of-service vector (task panic → connection drop, and potentially cascading if the panic propagates to a shared runtime).

**Code** (`service.rs:691`):
```rust
let addr_bytes: [u8; 32] = request.contract_address.as_bytes().try_into().unwrap();
```
Repeated at `:733` (`call_contract_call`) and `:797`.

**Recommended fix:** Replace `.unwrap()` with proper error handling:
```rust
let addr_bytes: [u8; 32] = request.contract_address.as_bytes()
    .try_into()
    .map_err(|_| RpcError::General("invalid contract address length"))?;
```

---

### F-C-12 — 1 GB message size limits across all transports — memory exhaustion DoS

- **Severity:** CRITICAL
- **File:** `rpc/grpc/core/src/lib.rs:8` (`RPC_MAX_MESSAGE_SIZE = 1 GB`), `protocol/p2p/src/core/connection_handler.rs:45` (`P2P_MAX_MESSAGE_SIZE = 1 GB`), `rpc/wrpc/client/src/client.rs:443-444` (1 GB), `rpc/wrpc/proxy/src/main.rs:99` (1 GB), `rpc/grpc/server/src/connection_handler.rs:129-131`, `rpc/grpc/client/src/lib.rs:570-572`
- **Cross-references:** P4 C-04. **Related:** F-H-24 (inconsistent wRPC size limits).

**Description:** The gRPC server, P2P server, wRPC client, and wRPC proxy all accept messages up to **1 GB**. The wRPC server uses 128 MB. All transports also accept gzip compression, meaning a small compressed payload can decompress to ~1 GB — a classic compression-bomb attack. A single peer or RPC client can force the node to allocate ~1 GB per message, and with multiple concurrent connections this quickly exhausts available memory. Additionally, `max_encoding_message_size` is **never set** on any server, meaning outbound responses are also unbounded.

**Code** (`rpc/grpc/core/src/lib.rs:8`):
```rust
pub const RPC_MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GB
```
(`protocol/p2p/src/core/connection_handler.rs:45`):
```rust
const P2P_MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GB
```
(`rpc/grpc/server/src/connection_handler.rs:129-131`):
```rust
let protowire_server = RpcServer::new(connection_handler)
    .accept_compressed(CompressionEncoding::Gzip)
    .send_compressed(CompressionEncoding::Gzip)
    .max_decoding_message_size(RPC_MAX_MESSAGE_SIZE);
```

**Recommended fix:** Reduce message size limits to the practical maximum needed (e.g. 64 MB for gRPC, 32 MB for P2P). Set `max_encoding_message_size` on all servers. Consider disabling gzip decompression or limiting the decompressed size. Apply per-connection memory quotas.

---

### F-C-13 — No authentication on any RPC method — admin and contract methods remotely accessible

- **Severity:** CRITICAL
- **File:** `rpc/service/src/service.rs` (all `*_call` methods), `rpc/core/src/api/ops.rs:117-164` (all op codes), `rpc/grpc/server/src/connection_handler.rs:125-176`, `rpc/wrpc/server/src/service.rs:70-86` (wRPC handshake commented out)
- **Cross-references:** P4 C-05. **Related:** F-C-01 (RPC consensus bypass).

**Description:** There is **no authentication layer** on any RPC transport. Every method in `RpcApiOps` is reachable by any client that can connect. While `unsafe_rpc` mode gating prevents some admin methods (`Shutdown`, `Ban`, `Unban`, `AddPeer`, `ResolveFinalityConflict`) from executing in safe mode, these methods are still *callable* — they just return an error. More critically: `SubmitBlock`, `SubmitTransaction`, `SubmitTransactionReplacement` have **no safe-mode gating** and are always callable. All 5 contract ops (`DeployContract`, `InvokeContract`, `GetContractState`, `GetContractCode`, `CallContract`) have **no safe-mode gating** and are always callable — including the consensus-bypassing `deploy`/`invoke` (see F-C-01). The wRPC handshake is **commented out**, meaning there is no protocol-level negotiation or authentication — any WebSocket client can immediately issue any RPC. The gRPC server has no TLS configuration in the default setup.

**Code** (`rpc/wrpc/server/src/service.rs:70-80`):
```rust
async fn handshake(
    self: Arc<Self>,
    peer: &SocketAddr,
    _sender: &mut WebSocketSender,
    _receiver: &mut WebSocketReceiver,
    messenger: Arc<Messenger>,
) -> WebSocketResult<Connection> {
    // TODO - discuss and implement handshake
    // handshake::greeting(...)
    let connection = self.server.connect(peer, messenger).await
        .map_err(|err| err.to_string())?;
    Ok(connection)
}
```

**Recommended fix:** Implement authentication (API key, JWT, or mutual TLS) for all state-mutating RPC methods. Require auth for admin methods even in unsafe mode. Gate contract ops behind `unsafe_rpc` or a separate `enable_contracts` flag. Implement the wRPC handshake or enforce TLS.

---

### F-C-14 — Stored XSS via token metadata social links rendered into innerHTML

- **Severity:** CRITICAL
- **File:** `zyanya-explorer/src/web.rs:3334-3339`; source: `zyanya-explorer/src/client.rs:1225-1234`
- **Cross-references:** P5 C-01. **Related:** F-C-15, F-C-16, F-H-26, F-M-30, F-M-37, F-L-33.

**Description:** Token metadata fields (`twitter`, `telegram`, `website`) are rendered directly into `innerHTML` via template-literal interpolation without any HTML escaping or sanitization. An attacker who deploys a token with malicious metadata (e.g. `twitter: '"><img src=x onerror=alert(document.cookie)>'`) causes arbitrary JavaScript execution in the browser of any user visiting that token's page. The metadata is stored server-side in `token-metadata.json` and served to all visitors. The flow: attacker calls `/api/submit-signed-tx` with a `SignableTxData` payload containing the XSS payload → server saves it via `save_token_metadata()` → any user visiting `/token/<address>` triggers `loadTokenPage()` JS which fetches metadata and renders social links into `innerHTML`.

**Code:**
```javascript
// web.rs:3334-3339
const socialsHtml = [];
if (meta.twitter) socialsHtml.push(`<a href="${meta.twitter}" target="_blank">Twitter / X</a>`);
if (meta.telegram) socialsHtml.push(`<a href="${meta.telegram}" target="_blank">Telegram</a>`);
if (meta.website) socialsHtml.push(`<a href="${meta.website}" target="_blank">Website</a>`);
document.getElementById('social-links').innerHTML = socialsHtml.join('');
```

```rust
// client.rs:1225-1234 — metadata saved from client-supplied SignableTxData
let metadata = TokenMetadata {
    name: Some(data.name.clone()),
    symbol: Some(data.symbol.clone()),
    description: data.description.clone(),
    twitter: data.twitter.clone(),    // ← attacker-controlled
    telegram: data.telegram.clone(),  // ← attacker-controlled
    website: data.website.clone(),    // ← attacker-controlled
    icon_uri: data.icon_uri.clone(),
};
self.save_token_metadata(&data.contract_address, metadata).await?;
```

**Recommended fix:** Escape all metadata fields before rendering. Use `textContent` or create elements via `document.createElement('a')` and set `href` via `setAttribute`. Alternatively, sanitize on the server side by stripping `<`, `>`, `"`, `'` characters from metadata fields before storage. Validate that `twitter`/`telegram`/`website` are valid URLs (starting with `https://`).

---

### F-C-15 — Stored XSS via token name/symbol rendered into innerHTML in token list

- **Severity:** CRITICAL
- **File:** `zyanya-explorer/src/web.rs:1382-1383`; source: `zyanya-explorer/src/client.rs:1466-1468`
- **Cross-references:** P5 C-02. **Related:** F-C-14, F-C-16, F-H-26, F-L-33.

**Description:** The token listing page (`/explorer`) renders token `name` and `symbol` into an HTML string that is assigned to `innerHTML`. Since these values come from the metadata store (which is populated by client-supplied data via `submit_signed_tx`), an attacker can inject arbitrary HTML/JS that executes when any user views the token list.

**Code:**
```javascript
// web.rs:1382-1383 — built into 'html' string, then set as innerHTML at line 1389
'<td class="mono" style="color:#7EC8D3; font-weight:bold;">' + t.name + ' (' + t.symbol + ')</td>' +
'<td class="mono">' + t.total_supply.toLocaleString() + ' ' + t.symbol + '</td>' +
```
```javascript
// web.rs:1389
document.getElementById('tokens-tbody').innerHTML = html;
```

**Recommended fix:** Escape `t.name` and `t.symbol` before concatenation. Use a helper like `escapeHtml()` that replaces `<`, `>`, `&`, `"`, `'` with HTML entities. Apply the same escaping to all other dynamic values rendered via `innerHTML` throughout `web.rs`.

---

### F-C-16 — Unsigned transaction metadata injection — signatures do not cover metadata

- **Severity:** CRITICAL
- **File:** `zyanya-explorer/src/client.rs:1180-1245`
- **Cross-references:** P5 C-03. **Related:** F-C-14, F-C-15.

**Description:** The `submit_signed_tx` function deserializes a `SignableTxData` struct from the client-supplied `unsigned_tx` hex payload. The Schnorr signatures only cover the transaction data (`data.tx` and `data.entries`), not the metadata fields (`name`, `symbol`, `description`, `twitter`, `telegram`, `website`, `icon_uri`, `contract_address`). An attacker can: (1) request an unsigned deploy-token tx from the server with legitimate metadata, (2) modify the `SignableTxData` JSON to change metadata fields (e.g. inject XSS payloads into `twitter`), (3) re-encode as hex and sign the unchanged sighashes, (4) submit the modified payload — signatures still validate because `data.tx` is unchanged, (5) the server saves the attacker-controlled metadata. This enables both the stored XSS (F-C-14, F-C-15) and arbitrary metadata spoofing for any token.

**Code:**
```rust
// client.rs:1182-1184 — deserialized from client hex, NOT from server's original response
let data: SignableTxData = serde_json::from_slice(&json_bytes)
    .map_err(|e| format!("Failed to parse unsigned transaction payload: {}", e))?;

// client.rs:1207-1209 — signatures verified against data.tx only
if let Err(e) = verify(&signable_tx.as_verifiable()) {
    return Err(format!("Signature verification failed: {:?}", e));
}

// client.rs:1225-1234 — metadata from the (tampered) data is saved directly
let metadata = TokenMetadata {
    name: Some(data.name.clone()),    // ← attacker can modify this
    symbol: Some(data.symbol.clone()), // ← attacker can modify this
    ...
};
self.save_token_metadata(&data.contract_address, metadata).await?;
```

**Recommended fix:** Either sign the entire `SignableTxData` payload (not just the transaction), or store the metadata server-side when building the unsigned tx and look it up by tx-id after submission, ignoring client-supplied metadata fields. At minimum, validate/sanitize all metadata fields before storage.

---

## 4. HIGH Findings (full details)

### F-H-01 — `POW` opcode uses `wrapping_pow` — silent overflow in financial math

- **Severity:** HIGH
- **File:** `zyanya-vm/src/vm.rs:128-131`
- **Cross-references:** P1 H-01

**Description:** `Add`, `Sub`, and `Mul` all use `checked_*` and return `VMError::ArithmeticOverflow` on overflow. `Pow` uses `a.wrapping_pow(b as u32)`, which **silently wraps** modulo `2^64`. A contract can produce a wrapped result that flows into bonding-curve/DEX math without any error, yielding incorrect (and exploitable) financial values. The exponent is also cast `b as u32` (truncating a u64 exponent) and the extra gas `1 + (b as u64 / 32)` is computed on the *truncated* value used for gas, while the *full* `b` is used for the cast — minor inconsistency.

**Code** (`vm.rs:128-131`):
```rust
OpCode::Pow => {
    let b = self.stack.pop()?;
    let a = self.stack.pop()?;
    let extra_gas = 1 + (b as u64 / 32);
    self.gas_meter.consume(extra_gas)?;
    self.stack.push(a.wrapping_pow(b as u32))?;
    self.pc += 1;
}
```

**Recommended fix:** Use `a.checked_pow(b as u32)` and return `VMError::ArithmeticOverflow` on `None`. Reject exponents that exceed `u32::MAX` (or cap and charge gas) instead of silently truncating.

---

### F-H-02 — DEX constant-product math overflows u64 on-chain (griefing/DoS) and off-chain (wrong quotes)

- **Severity:** HIGH
- **Files:** `dex.zcl:55` (`resB * amountIn * 997`), `:62` (`resA * amountIn * 997`); `bonding_curve.zcl:44` (`slope * (2 * supply * k + k * k)`), `:62` (`slope * (2 * supply * k - k * k)`); off-chain mirror in `zyanya-explorer/src/client.rs:897` and `:1045`
- **Cross-references:** P1 H-02, P5 H-01, P5 H-02 *(merged — on-chain ZCL overflow + off-chain explorer client overflow)*

**Description:** **On-chain:** The VM `MUL` is checked, so overflow *reverts* the transaction. But the DEX formula `resB * amountIn * 997` overflows u64 for realistic reserves (e.g. `resB = 10^9`, `amountIn = 10^9`, `997` → `~10^21 ≫ u64::MAX`). Any swap against a moderately-sized pool reverts, making the DEX unusable (griefing/DoS). The bonding-curve `2 * supply * k + k * k` overflows for large supplies/k as well. **Off-chain (`client.rs:897`):** `let cost = slope.saturating_mul(2 * S * k + k * k) / 2;` — here `2 * S * k` and `k * k` are **plain u64 arithmetic** (not `saturating_`/`checked_`), so they silently wrap in release builds. The `saturating_mul` only guards the outer product. A user can be quoted a wildly wrong `cost` (or `refund` at `:1045`), then sign a tx that the node rejects or that charges a different amount — funds mispriced. In release mode, wrapping can produce a small value, potentially allowing an attacker to buy tokens for near-zero cost.

**Code** (`client.rs:897`):
```rust
let cost = slope.saturating_mul(2 * S * k + k * k) / 2;
```
(`client.rs:1045`): `let refund = if S >= k { slope.saturating_mul(2 * S * k - k * k) / 2 } else { 0 };`
(`dex.zcl:55`): `let num = resB * amountIn * 997;`

**Recommended fix:** Use `u128` intermediates (or `checked_*` math) in both the ZCL contracts and the explorer client. In the VM, consider a `MUL128`/wide-mul opcode or reorder the AMM formula to divide before multiplying to keep intermediates within u64. In `client.rs`, compute `2*S*k` and `k*k` as `u128` and reject overflow explicitly:
```rust
let cost = (slope as u128)
    .saturating_mul(2u128 * S as u128 * k as u128 + k as u128 * k as u128)
    / 2;
let cost = cost.min(u64::MAX as u128) as u64;
```

---

### F-H-03 — Jump-target validation: byte-offset vs opcode-index confusion allows mid-opcode targets

- **Severity:** HIGH
- **File:** `zyanya-vm/src/opcode.rs:266-278` (remap loop) and `zyanya-vm/src/vm.rs:146-163`
- **Cross-references:** P1 H-03

**Description:** Jump targets are serialized as **byte offsets**. During deserialization, a `byte_to_opcode` map records the byte offset of each opcode start. The remap loop converts a `Jump`/`JumpIf` target to an opcode index **only if** the byte offset exactly matches a recorded opcode start. If a target points **into the middle of a multi-byte opcode** (e.g. inside a `PUSH` 8-byte payload), it is *left as a raw byte offset* and never rejected. Later, the VM checks `*target >= code.len()` where `code.len()` is the **opcode count**, not the byte length — so a raw byte offset that happens to be ≤ opcode count is treated as a valid opcode index and **jumps to an unrelated opcode**. This is a deserialization-safety / control-flow integrity bug.

**Code** (`opcode.rs:266-278`):
```rust
for op in &mut opcodes {
    match op {
        OpCode::Jump(target) => {
            if let Some(&op_idx) = byte_to_opcode.get(target) {
                *target = op_idx;
            }
            // else: target left as raw byte offset — NOT rejected
        }
        OpCode::JumpIf(target) => { /* same */ }
        _ => {}
    }
}
```

**Recommended fix:** After the remap loop, validate that every `Jump`/`JumpIf` target resolved to a known opcode boundary; return `VMError::InvalidJumpTarget` for any target that did not resolve. Alternatively, serialize jump targets as opcode indices directly.

---

### F-H-04 — Explorer `/api/compile` endpoint is not write-gated and accepts arbitrary ZCL source (parser-recursion DoS)

- **Severity:** HIGH
- **Files:** `zyanya-explorer/src/api.rs:587-595` (`api_compile_contract_handler`), `zyanya-vm/src/compiler/parser.rs` (recursive-descent, no depth limit), `zyanya-vm/src/compiler/codegen.rs:224-241` (`call` builtin)
- **Cross-references:** P1 H-04, P5 H-04 *(merged — unauthenticated endpoint + parser recursion + CPU DoS)*. **Related:** F-M-03 (parser recursion depth).

**Description:** `api_compile_contract_handler` does **not** call `check_write_enabled()`, so it is open on all deployments including public explorers. It accepts arbitrary `source` text and runs the full ZCL pipeline (lexer → parser → codegen → assembler). The parser is recursive-descent with **no recursion depth limit**; deeply nested expressions like `(((((((…)))))))` recurse one frame per nesting level and can overflow the Rust stack → **remote DoS crash**. Separately, the `call` builtin in codegen takes an `Expression::Variable` as the address and emits `CALL <name>` verbatim — no injection path into opcodes was found, but the unbounded recursion is the primary risk. Even without stack overflow, repeated compilation of large/complex source causes CPU exhaustion on public explorers.

**Code** (`api.rs:587-595`):
```rust
pub async fn api_compile_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<CompileContractReq>,
) -> Response {
    match client.compile_contract(&payload.source) {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}
```

**Recommended fix:** Gate the compile endpoint behind `check_write_enabled()` (or a separate compile-enabled flag). Add a recursion-depth counter in the parser and reject beyond a limit (e.g. 256). Bound the input `source` length (e.g. 64 KiB). Consider running compilation in a `std::thread` with a fixed stack size and a timeout.

---

### F-H-05 — Wallet `holder_u64` derives a u64 from the first 8 bytes of the address — balance collision / theft

- **Severity:** HIGH
- **File:** `zyanya-wallet/src/wallet_ops.rs:279-285`
- **Cross-references:** P1 H-05

**Description:** `holder_u64` reduces a 32-byte address to a u64 by reading its first 8 bytes. Two distinct addresses that share their first 8 payload bytes map to the **same storage key** in the token contract. Since the token contract has no authentication (F-C-04) and uses `sload(holder)/sstore(holder, …)`, a collision means two different users share one balance slot — an attacker who finds or crafts a colliding address can spend another user's tokens. The 8-byte (64-bit) space means collisions become likely (birthday bound) at ~2^32 addresses — feasible on a busy chain.

**Code** (`wallet_ops.rs:279-285`):
```rust
pub fn holder_u64(address: &Address) -> u64 {
    if address.payload.len() >= 8 {
        u64::from_le_bytes(address.payload[0..8].try_into().unwrap_or([0u8; 8]))
    } else {
        1
    }
}
```

**Recommended fix:** Use the full address (or a hash of it) as the storage key. If the VM storage key must be u64, derive it as `u64::from_be_bytes(hash(address)[0..8])` and document the collision risk, or switch the storage key type to `[u8; 32]`.

---

### F-H-06 — UTXO selection uses unchecked addition for `total_balance` / `selected_amount`

- **Severity:** HIGH
- **File:** `zyanya-wallet/src/wallet_ops.rs:158` and `:204`
- **Cross-references:** P1 H-06. **Related:** F-L-15 (utxo/context.rs `sum()` overflow).

**Description:** In `get_zyan_balance`, `total_balance += entry.utxo_entry.amount` (`:158`) is plain `u64` addition with no overflow check. In `send_zyan`, `selected_amount += entry.amount` (`:204`) is likewise unchecked. While individual UTXO amounts are bounded by `MAX_SOMPI`, a wallet with many UTXOs whose sum exceeds `u64::MAX` would silently wrap, causing `total_balance < required` to be computed incorrectly — potentially selecting too few UTXOs and producing an invalid tx, or computing a wrong change amount.

**Code** (`wallet_ops.rs:158`):
```rust
total_balance += entry.utxo_entry.amount;
```

**Recommended fix:** Use `total_balance.checked_add(entry.utxo_entry.amount)` and handle overflow (cap at `u64::MAX` or return an error). Apply the same to `selected_amount`.

---

### F-H-07 — Contract payload validation gaps — no size bounds on bytecode/parameters (borsh OOM)

- **Severity:** HIGH
- **File:** `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:157-200` and `consensus/core/src/tx.rs:134-170`
- **Cross-references:** P1 M-05, P2 H-01 *(merged — isolation validation gap + borsh deserialization OOM; canonical severity = HIGH)*. **Related:** F-C-10 (RPC unbounded bytecode/calldata).

**Description:** `check_contract_payload_in_isolation` checks that `bytecode` is non-empty, `max_gas != 0`, `gas_price != 0`, and `tx.gas == max_gas` — but there are **no bounds** on `bytecode.len()`, `parameters.len()`, or `deposit_amount`. A deploy transaction with a multi-GB `bytecode` vector, or an invoke transaction with millions of `parameters`, passes isolation validation and is a memory/CPU DoS vector. Additionally, `ContractPayload::from_slice` (tx.rs:168) calls `borsh::from_slice` on untrusted `tx.payload`. Borsh deserialization of `Vec<u8>` and `Vec<u64>` reads a `u32` length prefix and then allocates that many elements. A malicious `tx.payload` with a length prefix of `0xFFFFFFFF` will attempt to allocate ~4 GB for `bytecode` or ~32 GB for `parameters` before any consensus check runs — an **OOM DoS** at the validation stage.

**Code:**
```rust
// tx_validation_in_isolation.rs:162-180
let payload = zyanya_consensus_core::tx::ContractPayload::from_slice(&tx.payload)
    .map_err(|e| TxRuleError::InvalidContractPayload(e.to_string()))?;
match payload {
    zyanya_consensus_core::tx::ContractPayload::Deploy(deploy) => {
        if deploy.bytecode.is_empty() { return Err(...); }
        if deploy.max_gas == 0 { return Err(...); }
        if deploy.gas_price == 0 { return Err(...); }
        if tx.gas != deploy.max_gas { return Err(...); }
        // No check on deploy.bytecode.len() or deploy.deposit_amount
    }
    zyanya_consensus_core::tx::ContractPayload::Invoke(invoke) => {
        // No check on invoke.parameters.len() or invoke.deposit_amount
    }
}
```

**Recommended fix:** Add explicit `max_bytecode_len`, `max_parameters_len`, and `max_deposit_amount` to consensus params. Bound `tx.payload.len()` before calling `borsh::from_slice`. Consider using `borsh::BorshDeserialize::deserialize_reader` with a `Take`-wrapped reader to limit total bytes read.

---

### F-H-08 — Muhash `U3072::mul` contains non-debug `assert!` invariants (remote panic DoS)

- **Severity:** HIGH
- **File:** `crypto/muhash/src/u3072.rs:123`, `:143`, `:144`
- **Cross-references:** P2 H-02

**Description:** The `mul` function contains three **non-debug** assertions (`assert_eq!(carry_highest, 0)`, `assert_eq!(carry_high, 0)`, `assert!(carry_low == 0 || carry_low == 1)`). These execute in release builds. If any invariant is violated, the node **panics** → remote DoS. The MuHash state is updated from every UTXO-set element during block processing. While these invariants are expected to hold for valid internal state, the `deserialize` function accepts arbitrary 384-byte arrays, and a deserialized value is used in subsequent `mul` operations. The `inverse` function (line 157) also uses `Uint3072::mod_inverse(...).expect(...)`, which panics if the precondition is somehow violated.

**Code:**
```rust
// u3072.rs:123
assert_eq!(carry_highest, 0);
// u3072.rs:143-144
assert_eq!(carry_high, 0);
assert!(carry_low == 0 || carry_low == 1);
```

**Recommended fix:** If these invariants are provably unreachable from any valid input (including deserialized values), document the proof and keep them. If there is any path from untrusted input, convert to `debug_assert!` and return an error instead. At minimum, audit whether a crafted serialized MuHash can trigger a violation through `combine` or `normalize`.

---

### F-H-09 — `debug_assert!` overflow checks in `Uint` arithmetic silently wrap in release

- **Severity:** HIGH
- **File:** `math/src/uint.rs:533-582` (`Add`, `Sub`, `Mul` impls for all `Uint*` types)
- **Cross-references:** P2 H-03

**Description:** The `Add`, `Sub`, and `Mul` trait implementations for `Uint192`, `Uint256`, `Uint320`, and `Uint3072` use `overflowing_*` methods but only check for carry with `debug_assert!`. In release builds, the carry is **silently discarded** and the wrapped value is returned. Any consensus-critical code path that uses these operators without its own overflow checking is vulnerable to silent wraparound. While no currently-reachable consensus path appears to trigger overflow (difficulty uses `Uint320` with bounded window, `calc_work` is safe because `from_compact_target_bits` rejects `MAX`), the pattern is fragile — any future code using `Uint256`/`Uint320` arithmetic without explicit overflow checks will silently wrap in production.

**Code:**
```rust
// math/src/uint.rs:533-542
impl core::ops::Add<$name> for $name {
    type Output = $name;
    fn add(self, other: $name) -> $name {
        let (sum, carry) = self.overflowing_add(other);
        debug_assert!(!carry, "attempt to add with overflow");
        sum
    }
}
```

**Recommended fix:** For consensus-critical paths, prefer explicit `overflowing_*`/`checked_*` handling with error propagation. Consider making the `Add`/`Sub`/`Mul` impls return wrapped values but add a `checked_add`/`checked_mul` API for consensus code. Document which consensus paths are provably within range.

---

### F-H-10 — GhostDAG `assert!` sanity check is non-debug (remote panic DoS)

- **Severity:** HIGH
- **File:** `consensus/src/processes/ghostdag/protocol.rs:220`
- **Cross-references:** P2 H-04

**Description:** In `check_blue_candidate_with_chain_block`, there is a non-debug assertion: `assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k, "found blue anticone larger than K")`. This is checked on every block's GhostDAG processing. If a maliciously constructed DAG or a bug in the reachability store causes this invariant to be violated, the node **panics** in release builds. Additionally, `blue_anticone_size` (line 234) contains a `panic!("block {block} is not in blue set of the given context")` — an internal invariant that should also be unreachable, but if triggered by store corruption or a race condition, it crashes the node.

**Code** (`protocol.rs:220`):
```rust
assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k,
    "found blue anticone larger than K");
```

**Recommended fix:** If this is provably unreachable from valid DAG state, convert to `debug_assert!` and return a `RuleError` instead. If reachable from malformed input, handle gracefully with an error return.

---

### F-H-11 — Private key material leaked via derived `Debug` impls on `PrvKeyDataVariant` and `PrvKeyDataPayload`

- **Severity:** HIGH
- **File:** `wallet/core/src/storage/keydata/data.rs:20` (`PrvKeyDataVariant`), `:117` (`PrvKeyDataPayload`), `:191` (`PrvKeyData`)
- **Cross-references:** P3 H-01. **Related:** F-L-46 (Rothschild logs private key), F-H-14, F-H-15 (no zeroize).

**Description:** `PrvKeyDataVariant` is `#[derive(Clone, Debug, Serialize, Deserialize)]` and its variants hold the mnemonic phrase, BIP39 seed hex, xprv string, or secret key hex as a plain `String`. `{:?}` of `PrvKeyDataVariant` (or of `PrvKeyDataPayload`/`PrvKeyData` which transitively contain it) prints the secret in full. Any `log_*!`/`dbg!`/`println!` of these types — or of `Encryptable::Plain(PrvKeyDataPayload)` before encryption — leaks the key to logs.

**Code:**
```rust
// wallet/core/src/storage/keydata/data.rs:20
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrvKeyDataVariant {
    Mnemonic(String),      // <-- Debug prints the mnemonic phrase
    Bip39Seed(String),     // <-- Debug prints the seed hex
    ExtendedPrivateKey(String),
    SecretKey(String),
}

// data.rs:117
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PrvKeyDataPayload { prv_key_variant: PrvKeyDataVariant }
```

**Recommended fix:** Replace the derived `Debug` with a manual redacting impl that prints `"********"` instead of the secret. Do the same for `PrvKeyDataPayload` and `PrvKeyData`. Consider removing `Debug` entirely. Add a CI lint (`#![deny(clippy::dbg_macro)]`) and grep for `{:?}` on these types.

---

### F-H-12 — Wallet file written with default (world-readable) permissions

- **Severity:** HIGH
- **File:** `wallet/core/src/storage/local/wallet.rs:69` (`WalletStorage::try_store`)
- **Cross-references:** P3 H-02. **Related:** F-M-36 (/tmp fallback for metadata).

**Description:** The native (non-WASM) storage path uses `std::fs::File::create(store.filename())` with no explicit file mode. `File::create` creates the file with mode `0o666 & !umask` → `0o644` on the default umask 022, meaning the wallet file — which contains the encrypted mnemonic/private keys — is world-readable. An attacker with local filesystem access can copy the encrypted blob and brute-force the payment secret offline.

**Code:**
```rust
// wallet/core/src/storage/local/wallet.rs:69
let mut file = std::fs::File::create(store.filename(), )?;
BorshSerialize::serialize(self, &mut file)?;
```

**Recommended fix:** On Unix, create the file with restrictive permissions:
```rust
use std::os::unix::fs::OpenOptionsExt;
std::fs::OpenOptions::new()
    .write(true).create(true).truncate(true)
    .mode(0o600)
    .open(store.filename())?;
```
Or call `file.set_permissions(Permissions::from_mode(0o600))` immediately after create. Apply the same to transaction-record writes in `fsio.rs`.

---

### F-H-13 — Double addition of `change_output_value` to `transaction_fees`

- **Severity:** HIGH
- **File:** `wallet/core/src/tx/generator/generator.rs:797` and `:809`
- **Cross-references:** P3 H-03

**Description:** In `try_finish_standard_stage_processing`, the `absorb_change_to_fees || change_output_value == 0` branch executes `transaction_fees += change_output_value;` **twice** — once at line 797 and again at line 809 after the mass recompute. For `Fees::SenderPays`, the change is absorbed into the fee, so the double-count over-reports `transaction_fees` by `change_output_value`, producing a wrong fee in the `PendingTransaction`. For `Fees::ReceiverPays`, `generate_transaction` reduces the receiver's output by `transaction_fees`, so the receiver is **underpaid by `change_output_value`** — a fund-loss bug.

**Code:**
```rust
// wallet/core/src/tx/generator/generator.rs:795-810
if absorb_change_to_fees || change_output_value == 0 {
    transaction_fees += change_output_value;       // line 797 — first add

    let compute_mass = ...;
    let storage_mass = self.calc_storage_mass(...);
    data.aggregate_mass = calc.combine_mass(compute_mass, storage_mass);

    transaction_fees += change_output_value;       // line 809 — DUPLICATE
    data.transaction_fees = transaction_fees;
    ...
}
```

**Recommended fix:** Remove the duplicate `transaction_fees += change_output_value;` at line 809 (keep exactly one). Add a unit test asserting `transaction_fees == inputs - outputs` for the absorb-change case.

---

### F-H-14 — `ExtendedPrivateKey` derives `Clone` but has no `Drop`/`Zeroize`

- **Severity:** HIGH
- **File:** `wallet/bip32/src/xprivate_key.rs:19`
- **Cross-references:** P3 H-04. **Related:** F-H-11, F-H-15, F-L-47.

**Description:** `ExtendedPrivateKey<K>` is `#[derive(Clone)]` and wraps a `secp256k1::SecretKey` (which does **not** zeroize on drop by default). There is no `Drop` or `Zeroize` impl on `ExtendedPrivateKey`, so when the struct is dropped the private key bytes remain in memory. The `Clone` derive also creates transient copies that are never wiped. While the `Debug` impl is manual and redacts, and `to_string()` returns a `Zeroizing<String>`, the core key material in the struct itself is not zeroized on drop.

**Code:**
```rust
// wallet/bip32/src/xprivate_key.rs:19
#[derive(Clone)]
pub struct ExtendedPrivateKey<K: PrivateKey> {
    private_key: K,   // secp256k1::SecretKey — not zeroized on drop
    attrs: ExtendedKeyAttrs,
}
// no impl Drop, no impl Zeroize
```

**Recommended fix:** Implement `Drop` (or `Zeroize`) on `ExtendedPrivateKey` that calls `self.private_key.to_bytes()` and zeroizes the result. Consider wrapping `private_key` in `Zeroizing<K>`. Remove `Clone` where possible or implement a manual `Clone` that is explicit about the copy.

---

### F-H-15 — `Decrypted<T>` derives `Debug` and `Clone` with no `Drop`/`Zeroize`

- **Severity:** HIGH
- **File:** `wallet/core/src/encryption.rs:98`
- **Cross-references:** P3 H-05. **Related:** F-H-11, F-H-14.

**Description:** `Decrypted<T>` is `#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]` and holds the plaintext `T`. When `T` is `PrvKeyDataPayload` (which itself leaks via `Debug` — see F-H-11), `{:?}` on `Decrypted<PrvKeyDataPayload>` prints the full mnemonic/seed/xprv. `Clone` duplicates the plaintext without zeroizing the transient copy. There is no `Drop`/`Zeroize`, so decrypted plaintext persists in memory after the `Decrypted` guard is dropped.

**Code:**
```rust
// wallet/core/src/encryption.rs:98
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct Decrypted<T>(pub(crate) T)
where T: BorshSerialize + BorshDeserialize;
```

**Recommended fix:** Remove `Debug` or replace with a redacting impl. Add `Drop`/`Zeroize` bound (`impl<T: Zeroize> Drop for Decrypted<T>`). Remove `Clone` or make it explicit.

---

### F-H-16 — P2P `user_agent` not bounded on receive — memory DoS and log flooding

- **Severity:** HIGH
- **File:** `protocol/p2p/src/convert/messages.rs:50-53` (receive path), `protocol/p2p/src/convert/model/version.rs:9` (constant), `protocol/p2p/src/handshake.rs:28` (debug log)
- **Cross-references:** P4 H-01

**Description:** `MAX_USER_AGENT_LEN` (256 bytes) is only enforced in `Version::add_user_agent` (the **send** path). On the **receive** path, `Version::try_from(protowire::VersionMessage)` clones `msg.user_agent` verbatim with no length check. Since the P2P message size limit is 1 GB, a peer can send a multi-MB or multi-GB `user_agent` string. This string is then stored in `PeerProperties.user_agent` and logged via `debug!("accepted version message: {version_message:?}")`, causing both memory consumption and log flooding.

**Code:**
```rust
impl TryFrom<protowire::VersionMessage> for Version {
    fn try_from(msg: protowire::VersionMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            // ...
            user_agent: msg.user_agent.clone(),  // No length check!
            // ...
        })
    }
}
```

**Recommended fix:** Enforce `MAX_USER_AGENT_LEN` on the receive path: reject or truncate `msg.user_agent` if `len() > MAX_USER_AGENT_LEN`.

---

### F-H-17 — P2P timestamp cast `i64` → `u64` and time_offset arithmetic — integer wraparound

- **Severity:** HIGH
- **File:** `protocol/p2p/src/convert/messages.rs:50`, `protocol/flows/src/flow_context.rs:714`
- **Cross-references:** P4 H-02

**Description:** In `Version::try_from`, the peer's `timestamp` (an `i64` in protobuf) is cast to `u64` via `msg.timestamp as u64`. A negative timestamp wraps to a huge `u64` value. Then in `FlowContext::initialize_connection`, `time_offset` is computed as `unix_now() as i64 - peer_version_message.timestamp` where `peer_version_message.timestamp` is the already-wrapped `u64` cast back to `i64` context. This can cause `time_offset` to overflow/underflow `i64`, producing nonsensical time-offset values that feed into `PeerProperties.time_offset` and may affect consensus timing logic.

**Code:**
```rust
// protocol/p2p/src/convert/messages.rs:50
timestamp: msg.timestamp as u64,  // i64 → u64, negative wraps
```
```rust
// protocol/flows/src/flow_context.rs:714
let time_offset = unix_now() as i64 - peer_version_message.timestamp;
```

**Recommended fix:** Validate `msg.timestamp` is non-negative and within a reasonable range of `unix_now()` (e.g. ±24 hours) before casting. Use `checked_sub` for the `time_offset` computation.

---

### F-H-18 — No P2P connection limit or per-IP rate limiting — resource exhaustion

- **Severity:** HIGH
- **File:** `protocol/p2p/src/core/connection_handler.rs:202-223`
- **Cross-references:** P4 H-03. **Related:** F-H-23 (no wRPC conn limit).

**Description:** The P2P `message_stream` handler accepts **every** inbound connection without checking any maximum connection count or per-IP limit at the P2P layer. Unlike the gRPC server (which has a `Manager` with capacity checks), the P2P server spawns a full `Router` + flow set for each connection unconditionally. An attacker can open thousands of connections from different IPs to exhaust file descriptors, memory, and CPU. The `ConnectionManager` does disconnect random inbound peers above `inbound_limit`, but this happens on a 30-second timer — rapid connection floods can overwhelm the node before eviction runs.

**Code:**
```rust
async fn message_stream(
    &self,
    request: Request<Streaming<ZyanyadMessage>>,
) -> Result<Response<Self::MessageStreamStream>, TonicStatus> {
    let Some(remote_address) = request.remote_addr() else {
        return Err(TonicStatus::new(...));
    };
    // No connection limit check, no per-IP check
    let (outgoing_route, outgoing_receiver) = mpsc_channel(Self::outgoing_network_channel_size());
    let router = Router::new(remote_address, false, self.hub_sender.clone(), incoming_stream, outgoing_route).await;
    self.hub_sender.send(HubEvent::NewPeer(router)).await.expect(...);
    Ok(Response::new(...))
}
```

**Recommended fix:** Add a max-connections check at the P2P listener level. Track per-IP connection counts and reject new connections from IPs that exceed a threshold. Consider a connection rate limiter.

---

### F-H-19 — Peer identity spoofing — self-declared `PeerId` trusted; duplicate eviction

- **Severity:** HIGH
- **File:** `protocol/flows/src/flow_context.rs:717-725`, `protocol/p2p/src/core/hub.rs:79-84`, `protocol/p2p/src/core/peer.rs:70-73`
- **Cross-references:** P4 H-04

**Description:** The P2P handshake trusts the peer's self-declared `PeerId`. The `Hub` keys peers on `PeerKey { identity: PeerId, ip: IpAddress }`. While duplicate connections are rejected, the `insert_new_router` function will **close the previous router** if a duplicate key is inserted (e.g. due to a race condition). An attacker that knows (or guesses) a legitimate peer's `PeerId` can connect from the same IP with that ID and potentially evict the legitimate connection during a race window. The `PeerId` is a `uuid::Uuid` (random, 128-bit), making guessing impractical in most cases, but the pattern is still unsafe — there is no cryptographic binding of `PeerId` to the connection.

**Code** (`protocol/p2p/src/core/hub.rs:79-84`):
```rust
async fn insert_new_router(&self, new_router: Arc<Router>) {
    let prev = self.peers.write().insert(new_router.key(), new_router);
    if let Some(previous_router) = prev {
        previous_router.close().await;
        warn!("P2P, Hub event loop, removing peer with duplicate key: {}", previous_router.key());
    }
}
```

**Recommended fix:** When a duplicate `PeerKey` is detected, reject the *new* connection rather than closing the existing one. Consider cryptographically authenticating peer identities.

---

### F-H-20 — Transaction relay: no ban/disconnect for spam peers — amplification

- **Severity:** HIGH
- **File:** `protocol/flows/src/v5/txrelay/flow.rs:229-237` (spam counter), `:271-292` (RequestTransactionsFlow responder)
- **Cross-references:** P4 H-05. **Related:** F-M-24 (RequestTransactionsFlow no rate limit).

**Description:** In `RelayTransactionsFlow::receive_transactions`, when a peer sends spam or non-standard transactions, the code only increments a `spam_counter` and logs a warning every 100 spam txs. There is **no ban, no disconnect, and no rate limiting** applied to spam peers. A malicious peer can continuously flood the node with invalid transactions, consuming validation CPU and bandwidth indefinitely. Additionally, `RequestTransactionsFlow` responds to **any** `RequestTransactionsMessage` with the full transaction data from the mempool — there is no rate limit on the responder side, enabling amplification (small request → large response).

**Code:**
```rust
// protocol/flows/src/v5/txrelay/flow.rs:229-237
// TODO: discuss a banning process
Err(MiningManagerError::MempoolError(RuleError::RejectSpamTransaction(_)))
| Err(MiningManagerError::MempoolError(RuleError::RejectNonStandard(..))) => {
    self.spam_counter += 1;
    if self.spam_counter % 100 == 0 {
        zyanya_core::warn!("Peer {} has shared {} spam/non-standard txs ({:?})", self.router, self.spam_counter, res);
    }
}
```

**Recommended fix:** Implement a ban threshold — after N spam transactions, disconnect and ban the peer. Add rate limiting to `RequestTransactionsFlow` responses (e.g. track per-peer request rates and throttle).

---

### F-H-21 — Address manager poisoning — no source validation, eviction can displace good peers

- **Severity:** HIGH
- **File:** `components/addressmanager/src/lib.rs:254-264` (`add_address`), `:386-392` (`keep_limit`)
- **Cross-references:** P4 H-06

**Description:** `add_address` accepts any `NetAddress` that is not loopback or unspecified, with **no source validation, no port validation, and no rate limiting**. The address store is bounded to `MAX_ADDRESSES` (4096) via `keep_limit`, which evicts the address with the **highest** `connection_failed_count`. Newly added addresses start with `connection_failed_count = 1`, while established addresses may have `connection_failed_count = 0`. An attacker can flood 4096 junk addresses; if a good address has had even 1 failure (count = 1, same as new junk), it can be randomly evicted in favor of junk. Addresses received from untrusted peers via the P2P `AddressesMessage` flow go directly through `add_address` with no additional validation.

**Code:**
```rust
pub fn add_address(&mut self, address: NetAddress) {
    if address.ip.is_loopback() || address.ip.is_unspecified() {
        return;
    }
    if self.address_store.has(address) { return; }
    self.address_store.set(address, 1);  // connection_failed_count = 1
}
```

**Recommended fix:** Rate-limit `add_address` calls per peer. Validate ports (reject port 0, reject privileged ports). Consider a separate "untrusted" address pool for peer-advertised addresses. Give established addresses with successful connections a priority bonus in eviction.

---

### F-H-22 — Address manager `is_banned` — u64 underflow on future timestamp (ban bypass)

- **Severity:** HIGH
- **File:** `components/addressmanager/src/lib.rs:306-316`
- **Cross-references:** P4 H-07

**Description:** In `is_banned`, the ban expiry check computes `unix_now() - timestamp.0` as a `u64` subtraction. If `timestamp.0` (the ban timestamp stored in the DB) is in the **future** relative to `unix_now()` (due to clock skew, NTP jump, or DB manipulation), this subtraction **underflows** to a huge `u64` value, which will be greater than `MAX_BANNED_TIME`, causing the ban to expire immediately — a **ban bypass**.

**Code:**
```rust
pub fn is_banned(&mut self, ip: IpAddress) -> bool {
    const MAX_BANNED_TIME: u64 = 24 * 60 * 60 * 1000;
    match self.banned_address_store.get(ip.into()).unwrap_option() {
        Some(timestamp) => {
            if unix_now() - timestamp.0 > MAX_BANNED_TIME {  // underflow if timestamp.0 > unix_now()
                self.unban(ip);
                false
            } else {
                true
            }
        }
        None => false,
    }
}
```

**Recommended fix:** Use `checked_sub` or `saturating_sub`:
```rust
let elapsed = unix_now().saturating_sub(timestamp.0);
if elapsed > MAX_BANNED_TIME { ... }
```

---

### F-H-23 — wRPC server has no connection limit — unbounded WebSocket connections

- **Severity:** HIGH
- **File:** `rpc/wrpc/server/src/server.rs:112-145` (`connect`), `:147-165` (`disconnect`)
- **Cross-references:** P4 H-08. **Related:** F-H-18 (no P2P conn limit).

**Description:** The wRPC `Server::connect` method registers every incoming WebSocket connection in a `Mutex<HashMap<u64, Connection>>` with **no maximum connection limit**. The `next_connection_id` is an `AtomicU64` that increments without bound. Unlike the gRPC server which has a `Manager` with capacity checks, the wRPC server has no such mechanism. An attacker can open thousands of WebSocket connections to exhaust memory and file descriptors.

**Code:**
```rust
pub async fn connect(&self, peer: &SocketAddr, messenger: Arc<Messenger>) -> Result<Connection> {
    let id = self.inner.next_connection_id.fetch_add(1, Ordering::SeqCst);
    // ... no connection limit check ...
    self.inner.sockets.lock()?.insert(id, connection.clone());
    Ok(connection)
}
```

**Recommended fix:** Add a max-connections check before accepting a new WebSocket connection. Track active connections and reject new ones when the limit is reached.

---

### F-H-24 — Inconsistent wRPC message size limits — proxy allows 1 GB while server allows 128 MB

- **Severity:** HIGH
- **File:** `rpc/wrpc/server/src/service.rs:18` (`MAX_WRPC_MESSAGE_SIZE = 128 MB`), `rpc/wrpc/proxy/src/main.rs:99` (1 GB), `rpc/wrpc/client/src/client.rs:443-444` (1 GB)
- **Cross-references:** P4 H-09. **Related:** F-C-12 (1 GB message size limits).

**Description:** The wRPC server limits messages to 128 MB, but the wRPC proxy and wRPC client both allow 1 GB. When the proxy is deployed (the common production setup for browser/wallet access), it accepts 1 GB WebSocket messages and forwards them to the gRPC backend (which also accepts 1 GB). This means the effective limit through the proxy path is 1 GB, not 128 MB — the server's lower limit is bypassed for proxied connections. An attacker can send a 1 GB message through the proxy, bypassing the 128 MB server-side protection.

**Code:**
```rust
// rpc/wrpc/proxy/src/main.rs:99
let config = WebSocketConfig { max_message_size: Some(1024 * 1024 * 1024), ..Default::default() };
```
```rust
// rpc/wrpc/server/src/service.rs:16
static MAX_WRPC_MESSAGE_SIZE: usize = 1024 * 1024 * 128; // 128MB
```

**Recommended fix:** Standardize all transports to a single, lower message size limit (e.g. 32-64 MB). The proxy should enforce the same limit as the server.

---

### F-H-25 — gRPC conversion `expect()` / `unwrap()` on untrusted input — panic vectors

- **Severity:** HIGH
- **File:** `rpc/grpc/core/src/convert/header.rs:18`, `rpc/grpc/core/src/convert/message.rs:159`, `protocol/p2p/src/convert/net_address.rs:47-50`
- **Cross-references:** P4 H-10

**Description:** Several protowire-to-core conversion functions use `.expect()` or `.unwrap()` on data derived from untrusted network input. While the specific instances appear safe due to upstream length checks, the pattern of using `expect`/`unwrap` on conversion paths that handle network data is dangerous — any future refactor that removes the upstream check creates a panic DoS. `header.rs:18`: `item.timestamp.try_into().expect(...)` (send path). `message.rs:159`: `String::from_utf8(item.extra_data.clone()).expect(...)` (send path). `net_address.rs:48-49`: `chunk.try_into().expect(...)` (receive path, relies on length check via `match`).

**Code:**
```rust
// protocol/p2p/src/convert/net_address.rs:47-50
let octets = addr.ip.chunks(size_of::<u16>())
    .map(|chunk| u16::from_be_bytes(chunk.try_into().expect("We already checked the number of bytes")))
    .collect_vec();
let ipv6 = Ipv6Addr::from(<[u16; 8]>::try_from(octets).unwrap());
```

**Recommended fix:** Replace all `expect`/`unwrap` on conversion paths with proper `?` error propagation, even where a preceding check makes them "safe." Use `try_into().map_err(|_| ConversionError::...)?` instead.

---

### F-H-26 — Reflected XSS via API error messages rendered into innerHTML

- **Severity:** HIGH
- **File:** `zyanya-explorer/src/web.rs:3030, 3082, 3086, 3399, 3427, 3430`
- **Cross-references:** P5 H-03. **Related:** F-C-14, F-C-15 (stored XSS), F-L-33 (no security headers).

**Description:** Multiple locations in the explorer's embedded JavaScript insert API error messages (which may contain user-supplied input) directly into `innerHTML` via template literals. The server constructs error strings that include user input (e.g. `format!("Invalid Zyanya address or public key format: {}", address_str)`), and the client renders these into `innerHTML` without escaping. An attacker can craft input containing HTML/JS (e.g. address = `<img src=x onerror=alert(1)>`) which gets reflected through the error message into the victim's DOM.

**Code:**
```javascript
// web.rs:3030
statusEl.innerHTML = `<span style="color: var(--burn-red)">Unsigned tx build error: ${unsignedData.error || 'Failed'}</span>`;

// web.rs:3082
statusEl.innerHTML = `<span style="color: var(--burn-red)">Submission Error: ${submitData.error || 'Submit failed'}</span>`;

// web.rs:3399
statusEl.innerHTML = `<span style="color: var(--burn-red)">Buy failed: ${data.error || 'Unknown error'}</span>`;
```

**Recommended fix:** Escape all error messages before inserting into `innerHTML`, or use `innerText`/`textContent` for error display. Server-side: sanitize user input before including it in error strings.

---

## 5. MEDIUM Findings (summarized)

### F-M-01 — Explorer silently coerces invalid storage keys / addresses to 0 or 1
- **File:** `zyanya-explorer/src/api.rs:155-159`, `client.rs:960`, `:1110`, `:1321`
- **Source:** P1 M-01
- **Summary:** Invalid hex/decimal `key` query params silently become `0` via `unwrap_or(0)`, so a typo'd key mutates storage slot 0 (total supply / reserveA). Buy/sell builders coerce malformed addresses to holder `1` (often the owner). The DEX `swap_on_dex` coerces an unknown `token_in` to `0`, silently swapping the wrong side. All mask errors instead of returning 400.
- **Fix:** Propagate parse errors — return `400 Bad Request` instead of `unwrap_or(0)`/`unwrap_or(1)`; require a valid address and fail closed.

### F-M-02 — Gas metering: `consume` saturates (can mask overflow), and failing child `CALL` burns all forwarded gas
- **File:** `zyanya-vm/src/gas.rs:28-36`, `vm.rs:240`, `:267-270`
- **Source:** P1 M-02
- **Summary:** `consume` uses `saturating_add`; `refund` uses `saturating_sub` which can mask a logic error where a contract nets negative gas. In the `CALL` path, `forward_gas` is consumed up front; on child **failure** no refund is given — the entire `forward_gas` is burned (griefing).
- **Fix:** Consider refunding unused gas on failure too; replace `saturating_add` in `consume` with `checked_add` and return `OutOfGas` on overflow for a clearer error.

### F-M-03 — ZCL parser is recursive-descent with no depth limit (DoS via deep nesting)
- **File:** `zyanya-vm/src/compiler/parser.rs:143-262`
- **Source:** P1 M-03. **Related:** F-H-04 (ungated compile endpoint).
- **Summary:** `parse_expression → … → parse_primary` recurses one Rust frame per precedence level; `parse_primary` handles `LParen → parse_expression`, so each parenthesized sub-expression adds 6 frames. Deeply nested input overflows the Rust stack and crashes the process. When reached via the unauthenticated `/api/compile` endpoint (F-H-04), this is a remote DoS.
- **Fix:** Thread a `depth` counter through parse functions and return `ParserError` beyond a limit (e.g. 256); alternatively run the parser in a thread with a fixed 8 MiB stack.

### F-M-04 — Wallet `from_secret_hex` uses `trim_start_matches("0x")` which strips repeated prefixes
- **File:** `zyanya-wallet/src/key_management.rs:96`
- **Source:** P1 M-04
- **Summary:** `trim_start_matches("0x")` strips **all** leading occurrences, not just one. An input like `"0x0x<64hex>"` becomes `"<64hex>"` silently; a user could restore the wrong key and lose access to funds. The intent was `strip_prefix("0x")`.
- **Fix:** Use `hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim())` to strip at most one prefix; reject input whose length after stripping is not 64 hex chars.

### F-M-05 — Unsafe `MaybeUninit` + `transmute` in PoW matrix heavy_hash
- **File:** `consensus/pow/src/matrix.rs:97-103`
- **Source:** P2 M-01
- **Summary:** `heavy_hash` uses `MaybeUninit::uninit().assume_init()` to create an uninitialized array, writes into it, then `transmute`s to `[u8; 64]`. All 64 entries are written before the `transmute`, so this is sound in practice, but fragile if the `Hash` type ever changes. This is upstream Kaspa code.
- **Fix:** Replace with safe `array_from_fn` code that is equivalent and likely the same performance after optimization.

### F-M-06 — `unsafe` `AsRef<[u8]>` for `#[repr(C)] ContractStorageKey` is fragile UB
- **File:** `consensus/src/model/stores/contract.rs:22-40` (P1) / `:37-42` (P2)
- **Source:** P1 L-02, P2 M-02 *(merged — canonical severity = MEDIUM)*. **Related:** F-M-34 (database registry unsafe cast).
- **Summary:** `ContractStorageKey` is `#[repr(C)]` with `[u8; 32]` + `u64` = 40 bytes. The `AsRef` impl casts `&self` to `&[u8]` via `from_raw_parts`. The layout is currently correct (no padding), but this is **undefined behavior** if a field is added/reordered such that padding is inserted — the raw byte slice would include padding bytes and the RocksDB key would change semantics silently. P2 re-verified the layout and noted the same fragility.
- **Fix:** Add a compile-time assertion (`assert!(size_of::<ContractStorageKey>() == 40)`) and a comment documenting the invariant, or replace with safe serialization (`contract_address.iter().chain(key.to_le_bytes()).copied().collect()`).

### F-M-07 — Unsafe `str::from_utf8_unchecked` in hex display impls
- **File:** `crypto/hashes/src/lib.rs:117`, `consensus/core/src/tx/script_public_key.rs:90`, `math/src/uint.rs:757`, `:816`, `:834`, `:848`
- **Source:** P2 M-03. **Related:** F-M-23, F-L-35, F-L-43 (unsafe `from_utf8_unchecked` family).
- **Summary:** Multiple hex-encoding code paths use `unsafe { str::from_utf8_unchecked(&hex) }` after hex-encoding bytes. Hex encoding only produces ASCII, so this is sound, but if the encoding function ever produces non-ASCII output it would be UB. Upstream Kaspa code.
- **Fix:** Consider `str::from_utf8(&hex).expect("hex is valid utf-8")` for safety at negligible cost, or add a comment documenting why the unchecked conversion is safe.

### F-M-08 — Bech32 `conv5to8` may silently truncate on non-byte-aligned payloads — VERIFIED SAFE
- **File:** `crypto/addresses/src/bech32.rs:77-91`
- **Source:** P2 M-04
- **Summary:** `conv5to8` computes `payload.len() * 5 / 8` output bytes; trailing bits are silently discarded if the input is not byte-aligned. For address decoding this is safe because the encoding side pads to alignment and the checksum ensures integrity. The `REV_CHARSET` lookup uses `.get().unwrap_or(&100)`, safely rejecting out-of-charset characters. No OOB is possible.
- **Status:** **VERIFIED SAFE — no action.** The `conv5to8` truncation is correct for well-formed Bech32 addresses, and the checksum catches corruption.

### F-M-09 — PoW `compute_rank` uses `f64` Gaussian elimination (consensus-critical) — VERIFIED SAFE
- **File:** `consensus/pow/src/matrix.rs:65-90`
- **Source:** P2 M-05
- **Summary:** `compute_rank` performs Gaussian elimination using `f64` with `EPS = 1e-9`. Matrix entries are small integers (0..15), so the float arithmetic is exact for these magnitudes. The theoretical concern is cross-platform FPU rounding disagreement, but the risk is extremely low (entries 0..15, simple operations, >30 orders of magnitude of margin). Upstream Kaspa code.
- **Status:** **VERIFIED SAFE — no action.** Document that the float rank computation is exact for matrix entries in 0..15 and that the `EPS` threshold has >30 orders of magnitude of margin.

### F-M-10 — `decrypt_xchacha20poly1305` panics on ciphertext shorter than 24 bytes
- **File:** `wallet/core/src/encryption.rs:248`
- **Source:** P3 M-01
- **Summary:** `let nonce = &data[0..24];` panics with an index-out-of-bounds if the stored encrypted payload is truncated or corrupted to fewer than 24 bytes. Reachable from any stored `Encrypted` payload (wallet file, transaction records, keydata). Instead of returning a clean error, the wallet crashes (DoS).
- **Fix:** Check `data.len() >= 24 + 16` (nonce + minimum AEAD tag) and return `Err(...)` before slicing.

### F-M-11 — `decrypt_mnemonic`: panics and UB on malformed Go-wallet import files
- **File:** `wallet/core/src/compat/gen1.rs:9`, `:15`, `:18`
- **Source:** P3 M-02. **Related:** F-L-18 (from_utf8_unchecked in gen1).
- **Summary:** Three issues in the Go-wallet import path: (1) `cipher.as_ref().split_at(24)` panics if `cipher.len() < 24`; (2) `p_cost(num_threads).build().unwrap()` panics if `num_threads == 0` or too large; (3) `String::from_utf8_unchecked(decrypted)` is UB if the AEAD-decrypted plaintext is not valid UTF-8. Additionally `num_threads.expect(...)` panics on a v0 file missing the field.
- **Fix:** Validate `cipher.len() >= 24`; clamp `num_threads` to `1..=64`; use `String::from_utf8(decrypted)?`; replace `.unwrap()`/`.expect()` with `?`.

### F-M-12 — Lock-order inversion deadlock in `PubkeyDerivationManagerV0`
- **File:** `wallet/keys/src/derivation/gen0/hd.rs:124` and `:162`
- **Source:** P3 M-03
- **Summary:** `derive_pubkey_range` locks `cache` then `inner`; `derive_pubkey` locks `inner` then `cache`. Concurrent calls from two threads create a classic ABBA deadlock.
- **Fix:** Establish a single lock order (always `inner` before `cache`); in `derive_pubkey_range`, drop the `cache` guard before calling `opt_inner()`, or use a single combined mutex.

### F-M-13 — Signer key caches never zeroized on drop
- **File:** `wallet/core/src/tx/generator/signer.rs:17`, `:62`, `wallet/core/src/account/pssb.rs:31`
- **Source:** P3 M-04. **Related:** F-H-14, F-H-15.
- **Summary:** All three signer caches are `Mutex<AHashMap<Address, [u8;32]>>` holding raw 32-byte private keys. After signing, the temporary `Vec` is zeroized, but cached map entries persist in memory until the signer is dropped — and even then they are not wiped (no `Drop`/`Zeroize`).
- **Fix:** Implement `Drop` on `Inner`/`KeydataSignerInner`/`PSSBSignerInner` that iterates and zeroizes each `[u8;32]` value, or use `Zeroizing<[u8; 32]>` as the map value type.

### F-M-14 — Division-by-zero panics in mass calculation
- **File:** `wallet/core/src/tx/mass.rs:346` and `:350`
- **Source:** P3 M-05
- **Summary:** `calc_storage_mass_output_harmonic_single(0)` panics (integer division by zero); reachable when aggregated input value ≤ edge compute fees. `calc_storage_mass_input_mean_arithmetic` divides by `number_of_inputs`; panics if called with 0 inputs.
- **Fix:** Use `checked_div` and return `0` or propagate an `Error::MassCalculationError` on zero; guard `number_of_inputs == 0` at all call sites.

### F-M-15 — `Mnemonic` WASM surface: panics on invalid input and inconsistent state
- **File:** `wallet/bip32/src/mnemonic/phrase.rs:79`, `:99`, `:91`
- **Source:** P3 M-06
- **Summary:** `set_entropy` panics on invalid hex or wrong length (`panic!("invalid entropy ...")`) — a JS caller can trigger a WASM panic (DoS). `set_phrase` sets `self.phrase` without re-validating or recomputing entropy, leaving the `Mnemonic` in an inconsistent state. `phrase_string()` returns a plain `String` that is not zeroized.
- **Fix:** Return `Result` from setters instead of panicking; re-validate on `set_phrase`; return `Zeroizing<String>` from `phrase_string()` where possible.

### F-M-16 — PSBT/PSSB deserialization: no size limits and silent partial-sig overwrite
- **File:** `wallet/psst/src/psst.rs:151`, `wallet/psst/src/input.rs:108`
- **Source:** P3 M-07
- **Summary:** `PSST::from_hex` calls `serde_json::from_slice` on untrusted hex with no bounds — a multi-GB JSON payload causes memory exhaustion. `Input::add` does `self.partial_sigs.extend(rhs.partial_sigs)` — a conflicting signature for the same pubkey silently overwrites (BTreeMap `insert`) instead of erroring, violating BIP-174.
- **Fix:** Add explicit limits (max inputs/outputs, max JSON byte size) before deserialization; detect conflicting partial sigs before `extend` and return a `CombineError`.

### F-M-17 — `pssb_signer_for_address` panics on malformed input
- **File:** `wallet/core/src/account/pssb.rs:168`, `:170`, `:171`, `:163`
- **Source:** P3 M-08
- **Summary:** The signing closure uses `.expect()` and `.unwrap()` on operations that can fail with malformed/mismatched input (more inputs than addresses, missing key coverage, invalid digest). These panics trigger on a crafted PSST bundle, causing a wallet crash (DoS).
- **Fix:** Propagate errors instead of `.unwrap()`/`.expect()`; validate `addresses.len() == inputs.len()` up front; change the closure return type to `Result`.

### F-M-18 — `psst_to_pending_transaction` uses hardcoded mass and fee, panics on empty outputs
- **File:** `wallet/core/src/account/pssb.rs:291`, `:324`, `:323`/`:329`, `:339`
- **Source:** P3 M-09
- **Summary:** `mass = 10` and `fee_u = 0` are hardcoded placeholders — the reconstructed `PendingTransaction` has wrong mass/fee, breaking fee estimation and relay acceptance. `output[0]` indexing panics on an empty-output PSST. `extract_script_pub_key_address(...).unwrap()` panics on an unparseable script public key.
- **Fix:** Compute mass via `MassCalculator` and fee from the extracted transaction; return errors instead of indexing/`unwrap()`.

### F-M-19 — Multisig signature ordering may not match redeem script pubkey order
- **File:** `wallet/core/src/account/pssb.rs:237-245`
- **Source:** P3 M-10
- **Summary:** Signatures are emitted in `BTreeMap` (pubkey-sorted) order, which may not match the pubkey order in the multisig redeem script. `OP_CHECKMULTISIG` evaluates signatures in stack order, which must correspond to the redeem script pubkey order. Mismatched ordering produces an invalid scriptSig.
- **Fix:** Order signatures to match the redeem script's pubkey order (parse the redeem script, extract pubkeys, then sort `partial_sigs` by their position in the redeem script). Add a multisig finalize test.

### F-M-20 — `sign_with_multiple_v2` panics on invalid key or missing UTXO entry
- **File:** `consensus/core/src/sign.rs:129`, `:138`
- **Source:** P3 M-11
- **Summary:** `from_seckey_slice(...).unwrap()` panics on an invalid or zero private key. `mutable_tx.entries[i].as_ref().unwrap()` panics on a missing UTXO entry. The v1 `sign_with_multiple` keys the map by SEC1 pubkey but looks up by full script — it never matches (dead/broken code).
- **Fix:** Return errors instead of `.unwrap()`; document/remove v1 `sign_with_multiple`; route multisig inputs through the PSST path exclusively.

### F-M-21 — `Secret` derives `Clone` and serializable traits — secret duplication
- **File:** `wallet/keys/src/secret.rs:8`
- **Source:** P3 M-12. **Related:** F-H-14, F-H-15, F-L-47.
- **Summary:** `Secret` is `#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]`. Every `clone()` duplicates the secret bytes; the transient copy during `clone()` is not wiped. `Serialize`/`Deserialize`/`Borsh*` allow the secret to be serialized to plaintext without any guard. The `Debug` impl is correctly redacting.
- **Fix:** Remove `Clone` from `Secret` where possible, or implement a manual `Clone`; remove `Serialize`/`Deserialize`/`Borsh*` or gate them behind a feature flag.

### F-M-22 — `argon2_sha256iv_hash` uses `sha256(data)` as salt — deterministic KDF
- **File:** `wallet/core/src/encryption.rs:224`
- **Source:** P3 M-13
- **Summary:** `argon2_sha256iv_hash` computes `salt = sha256_hash(data)` where `data` is the password/secret itself. This makes the Argon2 salt deterministic — the same password always produces the same Argon2 salt and therefore the same encryption key across all wallets. An attacker who knows the password can precompute the key. There is no per-wallet random salt.
- **Fix:** Generate a random per-wallet salt (16+ bytes) and store it alongside the ciphertext; use the password + salt as Argon2 inputs.

### F-M-23 — `unsafe { str::from_utf8_unchecked }` in hex conversion — sound but fragile
- **File:** `rpc/core/src/model/hex_cnv.rs:26`
- **Source:** P4 M-01. **Related:** F-M-07, F-L-35, F-L-43.
- **Summary:** The `ToRpcHex` impl for `&[u8]` uses `unsafe { str::from_utf8_unchecked(&hex) }` after `faster_hex::hex_encode`. Sound because hex output is ASCII, but the safety invariant depends entirely on `faster_hex::hex_encode` — if a future version produces non-ASCII bytes, this would be UB.
- **Fix:** Replace with `String::from_utf8(hex).unwrap()` (safe for the same reason, but avoids `unsafe`), or add a comment documenting the safety invariant.

### F-M-24 — `RequestTransactionsFlow` responds to unlimited tx requests — no per-peer rate limit
- **File:** `protocol/flows/src/v5/txrelay/flow.rs:282-300`
- **Source:** P4 M-02. **Related:** F-H-20 (tx relay no ban).
- **Summary:** The `RequestTransactionsFlow` loops over all requested transaction IDs and responds with full transaction data for each one found in the mempool. There is no limit on the number of transactions requested per message. A peer can repeatedly send `RequestTransactionsMessage` with many IDs, forcing the node to serialize and send large transactions — a bandwidth amplification attack.
- **Fix:** Add a per-peer rate limit on `RequestTransactionsFlow` responses; add a maximum number of tx IDs per `RequestTransactionsMessage`.

### F-M-25 — `RandomWeightedIterator::new` panics on `WeightedError` — not `NoItem`
- **File:** `components/addressmanager/src/lib.rs:470-478`
- **Source:** P4 M-03
- **Summary:** The constructor panics on any `WeightedError` other than `NoItem`: `Err(e) => panic!("{e}")`. While current weight computation always produces positive finite values, a future change or corrupted address store entry could produce NaN/infinity/negative weights, triggering `WeightedError::InvalidWeight` and panicking the connection manager event loop.
- **Fix:** Replace `panic!("{e}")` with a log + `None` return, or return an error.

### F-M-26 — wRPC `disconnect` uses `.lock().unwrap()` — panic on poisoned mutex
- **File:** `rpc/wrpc/server/src/server.rs:160`
- **Source:** P4 M-04
- **Summary:** `Server::disconnect` calls `self.inner.sockets.lock().unwrap()`. If any other thread panicked while holding this `Mutex`, the lock is poisoned and `.unwrap()` will panic again, cascading the failure. `connect` at `:118` uses `lock()?` (propagates the error), creating an inconsistency.
- **Fix:** Use `lock().unwrap_or_else(|e| e.into_inner())` to recover from poison, or use `parking_lot::Mutex` which does not poison.

### F-M-27 — wRPC notification serialization `.unwrap()` — panic on serialization failure
- **File:** `rpc/wrpc/server/src/connection.rs:159-161`
- **Source:** P4 M-05
- **Summary:** The `Connection::into_message` implementation calls `Self::create_serialized_notification_message(...).unwrap()`. If serialization fails (e.g. due to a malformed notification or encoding error), this panics the notification sender task.
- **Fix:** Handle the error by logging and skipping the notification, or return a `Result`.

### F-M-28 — RPC error responses may leak internal state information
- **File:** `rpc/core/src/error.rs`, `rpc/service/src/service.rs`
- **Source:** P4 M-06
- **Summary:** The `RpcError` enum includes variants that expose internal details: `RejectedTransaction(RpcTransactionId, String)` forwards raw mempool/validation errors; `ConsensusError(ConsensusError)` transparently forwards consensus error messages; `General(String)` is a catch-all. These errors are propagated to the RPC client in full, potentially leaking internal state to an attacker probing for weaknesses.
- **Fix:** For unauthenticated RPC clients, sanitize error messages to reveal only what is necessary; log full errors server-side.

### F-M-29 — `MAX_INV_PER_TX_INV_MSG = 131,072` — very large inv messages allowed
- **File:** `protocol/flows/src/flowcontext/transactions.rs:16`
- **Source:** P4 M-07
- **Summary:** `MAX_INV_PER_TX_INV_MSG` is set to 131,072 (128K transaction IDs per inv message). Each transaction ID is 32 bytes, so a single `InvTransactionsMessage` can carry up to ~4 MB of transaction IDs, triggering up to 128K transaction requests and downloads — a large burst of processing and memory usage per message.
- **Fix:** Consider reducing to a more practical limit (e.g. 4096 or 8192); alternatively, add per-peer rate limiting on inv message processing.

### F-M-30 — Reflected XSS via block data in DAG visualization modal
- **File:** `zyanya-explorer/src/web.rs:5712-5724`
- **Source:** P5 M-01. **Related:** F-C-14, F-C-15, F-H-26, F-M-37.
- **Summary:** The DAG visualization page's block detail modal renders block data (hash, parents, selected_parent) from the API into `innerHTML` via template literals. The `parents` array is joined with `<br>` tags and inserted raw. If any field contains unexpected content (e.g. via a malicious node feeding crafted data), it could lead to HTML injection.
- **Fix:** Escape all block data fields before interpolation into HTML template literals; use `textContent` where possible.

### F-M-31 — No CORS restrictions on explorer API
- **File:** `zyanya-explorer/src/main.rs:67-101`
- **Source:** P5 M-02
- **Summary:** The Axum router does not include any CORS middleware. When `ZYANYA_EXPLORER_ENABLE_WRITE=1` is set, any website can make cross-origin POST requests to state-changing endpoints (deploy, invoke, buy, sell, submit-signed-tx, transfer, swap), enabling CSRF attacks.
- **Fix:** Add `tower-http::cors` middleware with restrictive origins, or implement CSRF token validation for state-changing endpoints; at minimum, set `Access-Control-Allow-Origin` to same-origin only.

### F-M-32 — Unbounded notification channels can cause memory exhaustion DoS
- **File:** `notify/src/subscriber.rs:75`, `notify/src/notifier.rs:293`
- **Source:** P5 M-03. **Related:** F-L-31 (unbounded mining sender).
- **Summary:** Both the `Subscriber` and `Notifier` use `Channel::unbounded()` for their incoming notification channels. Under high notification volume, notifications queue up without any backpressure. A sustained burst can cause unbounded memory growth, leading to OOM and process termination.
- **Fix:** Use bounded channels with a reasonable capacity (e.g. 10,000); when full, either drop oldest notifications or apply backpressure; log warnings when the queue approaches capacity.

### F-M-33 — Token icon upload has no content-type or size validation
- **File:** `zyanya-explorer/src/client.rs:274-289`
- **Source:** P5 M-04
- **Summary:** `save_token_icon` accepts arbitrary base64-encoded data, decodes it, and writes it to disk as a `.png` file with no validation that the data is a valid PNG, no size limit, and no check for malicious payloads (e.g. polyglot files). An attacker could upload a large file (disk exhaustion) or a non-image file served with `Content-Type: image/png`.
- **Fix:** Validate that decoded data starts with PNG magic bytes; enforce a maximum size (e.g. 256 KB); consider re-encoding the image server-side to strip embedded payloads.

### F-M-34 — Unsafe pointer cast in database registry relies on repr(u8) invariant
- **File:** `database/src/registry.rs:87`
- **Source:** P5 M-05. **Related:** F-M-06 (ContractStorageKey unsafe cast).
- **Summary:** The `AsRef<[u8]>` implementation for `DatabaseStorePrefixes` uses an unsafe pointer cast that transmutes `&Self` to `&u8`. While the enum is `#[repr(u8)]` which makes this technically sound, the safety invariant is fragile — if the repr attribute is ever removed or changed, this becomes undefined behavior.
- **Fix:** Replace with a safe alternative: `std::slice::from_ref(&(*self as u8))`.

### F-M-35 — Fallback to world-readable /tmp directory for sensitive metadata
- **File:** `zyanya-explorer/src/client.rs:227-230`, `:269`, `:283-288`
- **Source:** P5 M-06. **Related:** F-H-12 (wallet file perms).
- **Summary:** When writing token metadata or icons fails, the code falls back to writing to `/tmp/zyanya-token-metadata.json` and `/tmp/zyanya-token-icons/`. The `/tmp` directory is typically world-readable on multi-user systems. The metadata file is overwritten on every save (no file locking), risking corruption, and the fallback path is predictable, enabling symlink attacks.
- **Fix:** Fail gracefully instead of falling back to `/tmp`; if fallback is necessary, use a process-private directory with restricted permissions (0700); use atomic writes (write to temp file + rename).

### F-M-36 — Reflected XSS via error messages in DAG block modal
- **File:** `zyanya-explorer/src/web.rs:5724`
- **Source:** P5 M-07. **Related:** F-C-14, F-H-26, F-M-30.
- **Summary:** The block detail modal's error handler inserts `err.message` directly into `innerHTML`. If the fetch error message contains HTML (possible from crafted responses), it could lead to HTML injection.
- **Fix:** Use `textContent` for error display, or escape `err.message` before insertion.

### F-M-37 — Integer overflow in DAG handler pagination (limit + offset)
- **File:** `zyanya-explorer/src/api.rs:181-183`
- **Source:** P5 M-08
- **Summary:** The `api_dag_handler` computes `limit + offset` before passing it to `client.get_dag_graph()`. `offset` has no upper bound. If a client sends `limit=100&offset=usize::MAX`, the addition silently wraps in release mode (causing a small DAG graph fetch + `.skip(offset)` with the original huge offset → empty result — a DoS vector) or panics in debug mode.
- **Fix:** Use checked addition and return a 400 Bad Request on overflow, or cap `offset` to a reasonable maximum (e.g. 10,000).

### F-M-38 — Database connection builder panics on DB open failure
- **File:** `database/src/db/conn_builder.rs:117`, `:126`, `:137`
- **Source:** P5 M-09. **Related:** F-L-37 (delete_db panic).
- **Summary:** All three `build()` methods in `ConnBuilder` call `.unwrap()` on `DBWithThreadMode::open()`. If RocksDB fails to open (disk corruption, permissions, locked database), the process panics. If the database path is attacker-controllable (e.g. via `--appdir`), a crafted path could trigger the panic. `self.db_path.to_str().unwrap()` can also panic if the path contains non-UTF8 characters.
- **Fix:** Replace `.unwrap()` with proper error propagation using `?`; the `delete_db` function should also return a `Result` instead of using `.expect()`.

### F-M-39 — Unbounded `ram_scale` f64 can cause resource exhaustion or division issues
- **File:** `zyanyad/src/args.rs:80`, `mining/src/mempool/config.rs:apply_ram_scale`
- **Source:** P5 M-10
- **Summary:** The `--ram-scale` CLI argument accepts any `f64` value with no bounds validation. If set to `0.0`, `maximum_transaction_count` becomes 0, disabling the mempool. If set to `NaN`, comparisons with `.min(1.0)` behave unexpectedly (NaN propagates), potentially causing panics or logic errors in memory allocation.
- **Fix:** Validate `ram_scale` at parse time: reject non-finite values, values ≤ 0.0, and values > 100.0.

---

## 6. LOW Findings (listed)

### F-L-01 — `check_transaction_outputs_count` reports input fields in error (copy-paste)
- **File:** `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:59-64`
- **Source:** P1 L-01, P2 L-01 *(merged)*
- **Summary:** When the output count exceeds the limit, the error uses `tx.inputs.len()` and `self.max_tx_inputs` instead of `tx.outputs.len()` and `self.max_tx_outputs`. The check itself is correct; only the error payload is wrong.
- **Fix:** `Err(TxRuleError::TooManyOutputs(tx.outputs.len(), self.max_tx_outputs))`.

### F-L-02 — Assembler: label-on-same-line-as-opcode is mis-parsed; numeric `JUMP` targets are raw byte offsets
- **File:** `zyanya-vm/src/assembler.rs:87-96`, `:232-238`
- **Source:** P1 L-03
- **Summary:** A line like `:loop PUSH 1` is detected as a label and the opcode is swallowed into the label name. Numeric `JUMP` targets are treated as byte offsets, inconsistent with the VM's opcode-index interpretation.
- **Fix:** Reject label lines that contain additional tokens; document or switch numeric `JUMP` targets to opcode indices consistently.

### F-L-03 — Stack bounds checking is correct, but cross-call stack depth is unbounded (see F-C-05)
- **File:** `zyanya-vm/src/stack.rs:5`, `:37-42`, `:48-50`
- **Source:** P1 L-04
- **Summary:** The operand stack is correctly bounds-checked; the only gap is that each `CALL` spawns a fresh 1024-deep stack, but the call-chain depth is not bounded — covered by F-C-05.
- **Fix:** No change needed to `stack.rs`; resolve via the call-depth limit in F-C-05.

### F-L-04 — Merkle root: `next_power_of_two` could overflow for attacker-controlled leaf counts — VERIFIED SAFE
- **File:** `crypto/merkle/src/lib.rs:3-20`
- **Source:** P2 L-02
- **Summary:** `next_power_of_two()` overflows for `usize::MAX` leaves, but in practice the number of transactions in a block is bounded by block mass. The empty-input case correctly returns `ZERO_HASH`. Upstream Kaspa code.
- **Fix:** No change needed (verified safe for all practical leaf counts).

### F-L-05 — Subnet classification: `is_builtin` does not include `SUBNETWORK_ID_SMART_CONTRACT`
- **File:** `consensus/core/src/subnets.rs:58-62`, `:72-75`
- **Source:** P2 L-03
- **Summary:** `is_builtin()` returns true only for `COINBASE` and `REGISTRY`; the new `SUBNETWORK_ID_SMART_CONTRACT` is not included, meaning partial nodes are not required to validate smart contract transactions. This is by design (smart contract validation requires the VM).
- **Fix:** No change needed (verified correct by design); documented for awareness.

### F-L-06 — Coinbase vesting: `red_reward` accumulation is unchecked
- **File:** `consensus/src/processes/coinbase.rs:162`
- **Source:** P2 L-04
- **Summary:** `red_reward += reward_data.subsidy + reward_data.total_fees` uses plain `+=`. Safe in practice (bounded by `mergeset_size_limit × MAX_SOMPI ≈ 3.6 × 10^14`), but using `checked_add` would be more defensive.
- **Fix:** Use `checked_add` for both the inner and outer additions, returning an error on overflow.

### F-L-07 — `Vec::with_capacity((mergeset_blues.len() + 1) * 13)` — capacity hint only
- **File:** `consensus/src/processes/coinbase.rs:125`
- **Source:** P2 L-05. **Related:** F-C-07.
- **Summary:** `with_capacity` is a hint only and cannot overflow. Confirms that the code expects up to 2353 outputs, but validation (F-C-07) rejects above 260.
- **Fix:** Not a standalone finding; supports F-C-07.

### F-L-08 — `Fees::try_from(&str)` silently maps invalid fee string to 0
- **File:** `wallet/core/src/tx/fees.rs:85`
- **Source:** P3 L-01
- **Summary:** `try_zyanya_str_to_sompi_i64(fee)?.unwrap_or(0)` means a whitespace-only non-empty string silently becomes `Fees::SenderPays(0)`, disabling fees.
- **Fix:** Propagate the `Option<i64>` explicitly — if `None` is returned for a non-empty string, return an error.

### F-L-09 — `DerivationPath::from_str` has no length limit
- **File:** `wallet/bip32/src/derivation_path.rs:117`
- **Source:** P3 L-02
- **Summary:** Parses an unbounded number of path components into a `Vec<ChildNumber>`; a huge path string allocates unbounded memory (DoS).
- **Fix:** Cap path length (e.g. ≤ 255 components) at parse time.

### F-L-10 — `ChildNumber::from_bytes` and `From<u32>` skip validation
- **File:** `wallet/bip32/src/child_number.rs:40`, `:55`
- **Source:** P3 L-03
- **Summary:** `ChildNumber::new` validates `index < HARDENED_FLAG`, but `from_bytes` and `From<u32>` do not. A serialized key with `index >= 2^31` and no hardened flag is accepted, violating the invariant.
- **Fix:** Validate in `from_bytes` or document the invariant that callers must ensure the hardened bit is set correctly.

### F-L-11 — `ExtendedKey::from_str` does not cross-validate prefix string vs version bytes
- **File:** `wallet/bip32/src/xkey.rs:74`
- **Source:** P3 L-04
- **Summary:** `from_str` validates the 4-char prefix string is alphabetic but constructs `Prefix` via `from_parts_unchecked(chars, version)` without checking they agree. A key with prefix `xpub` but a private-key version would be treated as public.
- **Fix:** Cross-check that `Prefix::from_version(version).as_str() == chars` in `from_str`, or derive the prefix from the version bytes.

### F-L-12 — `static mut` storage globals are not thread-safe
- **File:** `wallet/core/src/storage/local/mod.rs:32-34`
- **Source:** P3 L-05
- **Summary:** `DEFAULT_STORAGE_FOLDER`, `DEFAULT_WALLET_FILE`, and `DEFAULT_SETTINGS_FILE` are `static mut Option<String>`. The `unsafe` accessors are not thread-safe; concurrent calls can cause a data race.
- **Fix:** Replace `static mut` with `OnceLock<String>` (for one-time init) or `Mutex<String>` for mutable access.

### F-L-13 — `utxo/context.rs` uses `panic!`/`unreachable!` on invariant violations
- **File:** `wallet/core/src/utxo/context.rs:382`, `:403`
- **Source:** P3 L-06
- **Summary:** Non-debug panics on invariant violations; a reorg or event sequence that violates the invariant crashes the wallet (DoS). The `unreachable!` at line 382 is reached when `outgoing.get(&txid).is_some()` during promotion — a reorg can cause this.
- **Fix:** Return `Result` and log the error instead of `panic!`/`unreachable!`.

### F-L-14 — `create_xpub_from_xprv`/`build_derivate_path` `panic!` on unsupported account kind
- **File:** `wallet/core/src/derivation.rs:540`, `:561`
- **Source:** P3 L-08
- **Summary:** These functions have `_ => panic!(...)` arms for unsupported account kinds. If `account_kind` is attacker-influenced (e.g. from a crafted wallet file), this is a DoS.
- **Fix:** Return `Error::AccountKindFeature` instead of panicking.

### F-L-15 — `calculate_balance` uses plain `sum()` — potential u64 overflow
- **File:** `wallet/core/src/utxo/context.rs:480-481`
- **Source:** P3 L-07. **Related:** F-H-06.
- **Summary:** `context.mature.iter().map(|e| e.as_ref().amount).sum()` and `context.pending.values().map(|e| e.as_ref().amount).sum()` use plain `sum()` on `u64`. Bounded by `MAX_SOMPI` in practice, but a crafted UTXO set could wrap.
- **Fix:** Use `checked_sum()` or `try_fold` with `checked_add` and propagate an overflow error.

### F-L-16 — `sig_op_count` panics on >255 xpub keys
- **File:** `wallet/core/src/account/variants/multisig.rs:195`
- **Source:** P3 L-09
- **Summary:** `u8::try_from(self.xpub_keys.len()).unwrap()` panics if the multisig has more than 255 xpub keys. A crafted wallet file with >255 keys causes a crash.
- **Fix:** Validate `xpub_keys.len() <= 255` at construction/deserialization; return an error instead of `unwrap()`.

### F-L-17 — `unsafe { from_utf8_unchecked }` in gen1 import and test code
- **File:** `wallet/core/src/compat/gen1.rs:18`, `wallet/core/src/account/mod.rs:856`
- **Source:** P3 L-10. **Related:** F-M-11.
- **Summary:** `gen1.rs:18` uses `String::from_utf8_unchecked(decrypted)` on AEAD-decrypted plaintext — UB if not valid UTF-8. `account/mod.rs:856` uses `from_utf8_unchecked` on a hex-encoded buffer in test code (safe but fragile).
- **Fix:** Use `String::from_utf8(decrypted)?` in `gen1.rs`; replace test-code `from_utf8_unchecked` with `from_utf8`.

### F-L-18 — `Encrypted` `Debug` prints ciphertext hex
- **File:** `wallet/core/src/encryption.rs:152`
- **Source:** P3 L-11
- **Summary:** `Encrypted` has a manual `Debug` impl that prints `self.payload.to_hex()` — the ciphertext. Lower risk than plaintext leakage, but could aid offline brute-force attempts.
- **Fix:** Replace `&self.payload.to_hex()` with `&"********"` or a length-only field.

### F-L-19 — `From<NetworkId> for Prefix` maps all non-mainnet to `KTUB`
- **File:** `wallet/bip32/src/prefix.rs:215`
- **Source:** P3 L-12
- **Summary:** `From<NetworkId>` maps `Mainnet → KPUB`, and all other network types (`Testnet`, `Devnet`, `Simnet`) to `KTUB`, making them indistinguishable in serialized form. A devnet xpub could be accidentally imported as a testnet xpub.
- **Fix:** Consider separate prefixes for devnet/simnet if network isolation is required, or document as intentional.

### F-L-20 — `DerivationPath` deserialize visitor has copy-paste error in `expecting` message
- **File:** `wallet/bip32/src/derivation_path.rs:35`
- **Source:** P3 L-13
- **Summary:** The `expecting` message says `"a string containing list of permissions separated by a '+'"` — a copy-paste error from another module.
- **Fix:** Fix the message to `"a BIP32 derivation path string (e.g. m/44'/123456'/0')"`.

### F-L-21 — TPS throttle math can overflow on extreme counts
- **File:** `protocol/flows/src/v5/txrelay/flow.rs:143`, `:313`
- **Source:** P4 L-01
- **Summary:** `1000 * snapshot_delta.low_priority_tx_counts` can overflow `u64` if `low_priority_tx_counts` exceeds `u64::MAX / 1000`. Practically impossible with real transaction counts.
- **Fix:** Use `checked_mul` or `saturating_mul` for the multiplication.

### F-L-22 — `Hub::broadcast_to_some_peers` asserts `num_peers > 0` — panic on misuse
- **File:** `protocol/p2p/src/core/hub.rs:136`
- **Source:** P4 L-02
- **Summary:** `assert!(num_peers > 0)` panics if any internal caller passes 0, which could be reached through a logic error in the throttling code path.
- **Fix:** Replace `assert!` with an early `return` for `num_peers == 0`, or use `debug_assert!`.

### F-L-23 — `Router::enqueue` uses `assert!` for payload presence — panic on internal bug
- **File:** `protocol/p2p/src/core/router.rs:405-406`
- **Source:** P4 L-03
- **Summary:** `assert!(msg.payload.is_some(), ...)` means any internal bug that constructs a message without a payload will panic the send path.
- **Fix:** Return an `Err(ProtocolError::Other(...))` instead of panicking.

### F-L-24 — `connection_failed_count + 1` can theoretically overflow u64
- **File:** `components/addressmanager/src/lib.rs:271-277`
- **Source:** P4 L-04
- **Summary:** `mark_connection_failure` increments `connection_failed_count` with `+ 1` without checked arithmetic. Overflow requires 2^64 connection failures — practically impossible.
- **Fix:** Use `saturating_add(1)` for defensive programming.

### F-L-25 — `hub_sender.send().expect()` in multiple P2P paths — panic if hub receiver drops
- **File:** `protocol/p2p/src/core/connection_handler.rs:129`, `:218`, `protocol/p2p/src/core/router.rs:451`
- **Source:** P4 L-05
- **Summary:** Several P2P code paths use `.expect("hub receiver should never drop before senders")`. If the hub receiver drops due to a shutdown race or bug, these will panic.
- **Fix:** Handle the `Err` case gracefully by logging and closing the connection, rather than panicking.

### F-L-26 — `P2P Server` and `gRPC Server` panic on serve error — no graceful degradation
- **File:** `protocol/p2p/src/core/connection_handler.rs:79`, `rpc/grpc/server/src/connection_handler.rs:160`, `rpc/wrpc/server/src/service.rs:155`
- **Source:** P4 L-06
- **Summary:** All three server implementations panic if the listener loop encounters an error, preventing graceful shutdown and cleanup. In a production node, this could cause data corruption if consensus state is not properly flushed.
- **Fix:** Signal an error state to the node's shutdown handler instead of panicking, allowing for graceful cleanup.

### F-L-27 — `DaaScoreTimestampEstimate` — division by zero if headers have equal DAA scores
- **File:** `rpc/service/src/service.rs:967-972`
- **Source:** P4 L-07
- **Summary:** If `next_header.daa_score == header.daa_score`, `score_between_headers` is 0.0, causing a division by zero in floating point (produces `inf` or `NaN`), which when cast to `u64` becomes 0 or causes undefined behavior.
- **Fix:** Guard against `score_between_headers == 0.0` and return the header's timestamp directly in that case.

### F-L-28 — Unsafe impl Send for WASM Sink type
- **File:** `wasm/core/src/events.rs:38`
- **Source:** P5 L-01
- **Summary:** `Sink` contains `js_sys::Function` and `js_sys::Object` (JavaScript values) which are not truly `Send`. The `unsafe impl Send` is safe only in single-threaded WASM contexts; if compiled for non-WASM or multi-threaded WASM, it could cause data races.
- **Fix:** Gate the `Send` impl behind `#[cfg(target_arch = "wasm32")]`; add a safety comment; consider using `Sendable` wrapper.

### F-L-29 — No rate limiting on explorer API endpoints
- **File:** `zyanya-explorer/src/main.rs:67-101`
- **Source:** P5 L-02
- **Summary:** None of the explorer's API endpoints implement rate limiting. A malicious client can flood the server with requests, causing excessive gRPC calls to the backend node.
- **Fix:** Add rate-limiting middleware (e.g. `tower-governor`) with per-IP limits on resource-intensive endpoints.

### F-L-30 — Non-atomic metadata file writes risk corruption
- **File:** `zyanya-explorer/src/client.rs:267`
- **Source:** P5 L-03
- **Summary:** `std::fs::write` is not atomic — if the process crashes mid-write, the metadata file can be left truncated or corrupt. The entire metadata store is serialized to a single JSON file on every token deployment.
- **Fix:** Use atomic write: write to a temporary file in the same directory, then rename to the target path.

### F-L-31 — UnboundedSender for transaction IDs in mining manager
- **File:** `mining/src/manager.rs:39`, `:632`, `:932`
- **Source:** P5 L-04. **Related:** F-M-32.
- **Summary:** The mining manager uses `tokio::sync::mpsc::UnboundedSender` for broadcasting transaction IDs. If the receiver falls behind, messages queue indefinitely, causing memory growth under high throughput.
- **Fix:** Use bounded channels, or handle `SendError` gracefully by logging and dropping stale messages; consider `try_send` with capacity checks.

### F-L-32 — No input validation on CLI `--uacomment` values
- **File:** `zyanyad/src/args.rs:228-233`
- **Source:** P5 L-05
- **Summary:** The `--uacomment` CLI argument accepts arbitrary strings. Extremely long values or values containing control characters could cause issues in peer-to-peer handshake messages or log output.
- **Fix:** Validate that user agent comments are under 256 characters, contain only printable ASCII, and don't contain the `/` delimiter.

### F-L-33 — Explorer does not set security headers
- **File:** `zyanya-explorer/src/main.rs:67-101`
- **Source:** P5 L-06. **Related:** F-C-14, F-C-15, F-H-26.
- **Summary:** The explorer's HTTP responses do not include `X-Content-Type-Options`, `X-Frame-Options`, `Content-Security-Policy`, or `Strict-Transport-Security`, making the XSS vulnerabilities more impactful and enabling clickjacking.
- **Fix:** Add `tower-http::set_header` middleware to set `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Content-Security-Policy: default-src 'self'; script-src 'self'`, and `Strict-Transport-Security` (when TLS is used).

### F-L-34 — Unsafe `from_utf8_unchecked` in WASM hex encoding
- **File:** `wasm/core/src/types.rs:52`
- **Source:** P5 L-07. **Related:** F-M-07, F-M-23, F-L-43.
- **Summary:** `From<&[u8]>` for `HexString` uses `unsafe { str::from_utf8_unchecked(&hex) }` after `faster_hex::hex_encode`. Sound but bypasses validation; if `faster_hex` behavior changes, this could cause UB.
- **Fix:** Use the safe `str::from_utf8(&hex).expect("hex output is always valid UTF-8")` instead.

### F-L-35 — DbKey Display indexes into path array without bounds checking
- **File:** `database/src/key.rs:97-119`
- **Source:** P5 L-08
- **Summary:** The `Display` impl for `DbKey` indexes `self.path[0]` and `self.path[1]` based on `self.prefix_len` checks without verifying `path.len()`. Corrupted DB keys could cause panics in logging code.
- **Fix:** Add bounds checks: use `self.path.get(0)` and `self.path.get(1)` which return `Option`.

### F-L-36 — `delete_db` panics on RocksDB destroy failure
- **File:** `database/src/db.rs:42-43`
- **Source:** P5 L-09. **Related:** F-M-38.
- **Summary:** `delete_db` calls `.expect("DB is expected to be deletable")` on the RocksDB `destroy` operation. If the database directory is locked, has permission issues, or the path is invalid, this causes a process panic.
- **Fix:** Return a `Result` and propagate the error to the caller.

### F-L-37 — `network()` panics on conflicting network flags
- **File:** `zyanyad/src/args.rs:170-180`
- **Source:** P5 L-10. **Related:** F-L-40.
- **Summary:** The `network()` method panics with `"only a single net should be activated"` if more than one of `mainnet`, `testnet`, `devnet`, `simnet` is set. Reachable from user input (passing both `--mainnet` and `--testnet`).
- **Fix:** Return a `Result<NetworkId, String>` and handle the error gracefully (print a helpful error message and exit with code 1).

### F-L-38 — Daemon `.expect()` calls can crash on missing daemons
- **File:** `daemon/src/lib.rs:99`, `:105`
- **Source:** P5 L-11. **Related:** F-L-37.
- **Summary:** `Daemons::zyanyad()` and `Daemons::cpu_miner()` call `.expect()` when the daemon is `None`, causing a panic with an unhelpful message when a user hasn't configured the daemon.
- **Fix:** Return `Option<Arc<...>>` or `Result` and let the caller decide how to handle the missing daemon.

### F-L-39 — Topological sort asserts no cycles with panic
- **File:** `mining/src/model/topological_sort.rs:52`
- **Source:** P5 L-12. **Related:** F-L-42.
- **Summary:** `assert_eq!(sorted.len(), self.len(), ...)` runs in release builds. If a bug or consensus failure introduces a cycle, the node crashes instead of handling the error gracefully.
- **Fix:** Replace with a soft check that returns an error or logs a warning, or use `debug_assert_eq!`.

### F-L-40 — Orphan pool eviction is not truly random
- **File:** `mining/src/mempool/model/orphan_pool.rs:210-212`
- **Source:** P5 L-13
- **Summary:** `get_random_low_priority_orphan` uses `find()` which returns the **first** low-priority orphan, not a random one. Under orphan pool pressure, the same orphan is always evicted first, creating a predictable eviction pattern.
- **Fix:** Use a proper random selection (e.g. `choose()` from the `rand` crate) or iterate from a random starting point.

### F-L-41 — `check_transaction_standard_in_context` uses `assert!` for mass invariant
- **File:** `mining/src/mempool/check_transaction_standard.rs:131`
- **Source:** P5 L-14. **Related:** F-L-39.
- **Summary:** `assert!(contextual_mass > 0, "expected to be set by consensus")` runs in release builds. If consensus fails to set the mass, the node crashes instead of rejecting the transaction.
- **Fix:** Replace with an error return: `if contextual_mass == 0 { return Err(NonStandardError::...); }` or use `debug_assert!`.

### F-L-42 — Unsafe `from_utf8_unchecked` in math uint Display and LowerHex impls
- **File:** `math/src/uint.rs:757`, `:816`
- **Source:** P5 L-15. **Related:** F-M-07, F-M-23, F-L-34.
- **Summary:** `Display` and `LowerHex` impls for `U192`/`U256` use `unsafe { str::from_utf8_unchecked(...) }` on byte buffers filled with decimal digit or hex characters. Sound by construction, but bypasses validation.
- **Fix:** Use safe `str::from_utf8(&buf).expect("buffer contains only ASCII digits/hex")` — the performance difference is negligible for formatting operations.

### F-L-43 — Display LUT indexing could overflow on very large integer types
- **File:** `math/src/uint.rs:783-804`
- **Source:** P5 L-16
- **Summary:** The `Display` impl uses a 200-byte `DEC_DIGITS_LUT`. Indices are computed from `rem < 10_000` (guaranteed by `div_rem_u64`), so they fit within the LUT. The code is correct as written but has implicit safety dependencies.
- **Fix:** Add a `debug_assert!(d1 < 200 && d2 < 200)` to catch any regression in `div_rem_u64`.

### F-L-44 — `hex.rs` deserialize uses `unwrap()` on UTF-8 conversion
- **File:** `utils/src/hex.rs:30`
- **Source:** P5 L-17. **Related:** F-L-45.
- **Summary:** `str::from_utf8(buff).unwrap()` on a byte slice from `Deserialize::deserialize` panics if the deserializer produces non-UTF8 bytes (possible for binary formats like borsh or postcard).
- **Fix:** Replace `unwrap()` with `str::from_utf8(buff).map_err(D::Error::custom)?`.

### F-L-45 — `networking.rs` uses `unwrap()` on IP parsing in unroutable networks list
- **File:** `utils/src/networking.rs:120`
- **Source:** P5 L-18. **Related:** F-L-44.
- **Summary:** `IpNet::from_str(curr_net).unwrap()` on each hardcoded CIDR. Safe for compile-time constants, but if the constant list is ever modified with a typo, the unwrap would panic at runtime.
- **Fix:** Parse the list once at compile time using `const` or `LazyLock`, or use `expect("unroutable_nets contains invalid CIDR")`.

### F-L-46 — Rothschild logs private key in plaintext
- **File:** `rothschild/src/main.rs:195-199`
- **Source:** P5 L-19. **Related:** F-H-11, F-H-14, F-H-15.
- **Summary:** When a user provides `--private-key`, rothschild logs the private key via `info!` with `schnorr_key.display_secret()`. If logs are written to disk (default for zyanyad), the private key is persisted in a potentially world-readable log file. When no key is provided, the auto-generated key is also logged.
- **Fix:** Never log private keys; display them to stdout only (not through the logging framework); ensure log files have restrictive permissions (0600).

### F-L-47 — CLI wizard secrets handled via `Secret::new()` but trimmed with `as_bytes().to_vec()`
- **File:** `cli/src/wizards/wallet.rs:68`, `:108`, `:112`
- **Source:** P5 L-20. **Related:** F-H-11, F-H-14, F-H-15, F-M-21.
- **Summary:** The CLI wallet creation wizard collects passwords/mnemonic passphrases via `term.ask()` and wraps them in `Secret::new(... .trim().as_bytes().to_vec())`. The intermediate `String` returned by `term.ask()` is not zeroized — it lives in the allocator until GC'd. On systems with swap, the password string may be written to disk.
- **Fix:** Use a zeroizing string type for the intermediate value; clear the original `String` memory after extracting the trimmed bytes; consider `secrecy::SecretString`.

---

## 7. Remediation Roadmap

### P0 — Block mainnet (CRITICAL, 16 findings)

All 16 CRITICAL findings must be remediated before any mainnet launch. They are grouped into 5 workstreams:

#### Workstream P0-A: Consensus integrity (F-C-01, F-C-02, F-C-06, F-C-07, F-C-08)

| ID | Action | Definition of done |
|----|--------|--------------------|
| F-C-01 | Remove the RPC fallback that persists contract state without consensus; return mempool errors to the caller | No code path writes contract state outside of consensus-accepted transactions; integration test verifies that a rejected mempool submission does not persist state |
| F-C-02 | Commit contract cache only on `success == true`; move all pre-execution mutations to a `temp_cache` | Unit test: failed deploy does not install bytecode; failed invoke does not credit balance; golden vector test with known-failing contracts |
| F-C-06 | Replace `f64::powf` subsidy table with pre-computed integer table | Cross-platform golden-vector test asserting exact table bytes on x86-64 and aarch64; test with `-ffast-math` and musl |
| F-C-07 | Derive coinbase output limit from `(mergeset_size_limit + 1) * 13` | Test building a coinbase with > 19 DAA-window blues and verifying it passes validation; test with `mergeset_size_limit` blues |
| F-C-08 | Use `checked_add`/`saturating_add` in mass calculation; lower `max_tx_*` params to 10_000 | Unit test: a tx at the param limits does not overflow mass; test that `max_block_mass` cannot be bypassed |

#### Workstream P0-B: VM / contract (F-C-03, F-C-04, F-C-05)

| ID | Action | Definition of done |
|----|--------|--------------------|
| F-C-03 | Model custody as real UTXO value transfer; reconcile balances with reserve; reject buy when `deposit_amount < cost` including 0 | Unit test: buy deducts cost from contract balance and creates UTXO output; sell creates payout UTXO; `deposit_amount == 0` rejected |
| F-C-04 | Introduce `msg.sender` in the VM; enforce `from == msg.sender` in transfer, `caller == msg.sender` in buy/sell/DEX; gate `mint`/`init` to owner | Integration test: unauthorized transfer/buy/sell reverts; only owner can mint/init |
| F-C-05 | Add call-depth limit (e.g. 1024); add per-contract reentrancy guard | Unit test: deep recursion hits limit and reverts cleanly (no crash); reentrant call to same contract reverts |

#### Workstream P0-C: RPC / network (F-C-10, F-C-11, F-C-12, F-C-13)

| ID | Action | Definition of done |
|----|--------|--------------------|
| F-C-10 | Enforce max bytecode size (1 MB), max calldata/parameters length, server-side `max_gas` cap | Unit test: oversized requests rejected before VM execution |
| F-C-11 | Replace `.try_into().unwrap()` with `map_err(...)?` on all contract address conversions | Fuzz test: malformed hash lengths return errors, not panics |
| F-C-12 | Reduce message size limits to 64 MB (gRPC) / 32 MB (P2P); set `max_encoding_message_size`; limit gzip decompression | Load test: 1000 concurrent connections with large messages do not exhaust memory; compression bomb rejected |
| F-C-13 | Implement authentication for all state-mutating RPC methods; gate contract ops; implement wRPC handshake or enforce TLS | Integration test: unauthenticated calls to state-mutating methods are rejected; admin methods require auth even in unsafe mode |

#### Workstream P0-D: Wallet (F-C-09)

| ID | Action | Definition of done |
|----|--------|--------------------|
| F-C-09 | Reject `minimum_signatures == 0` in `create_address`, `AddressManager::new`, `Payload` deserialization, and `multisig_redeem_script` | Unit test: `minimum_signatures == 0` returns an error at all entry points; existing 1-of-N and 2-of-N wallets still work |

#### Workstream P0-E: Explorer web security (F-C-14, F-C-15, F-C-16)

| ID | Action | Definition of done |
|----|--------|--------------------|
| F-C-14 | Escape all metadata fields before rendering; validate URLs start with `https://`; sanitize on server side | XSS test: token with `<img src=x onerror=alert(1)>` in metadata does not execute JS |
| F-C-15 | Escape `t.name` and `t.symbol` before `innerHTML` concatenation throughout `web.rs` | XSS test: token with script tag in name does not execute in token list |
| F-C-16 | Sign the entire `SignableTxData` payload, or store metadata server-side and ignore client-supplied fields | Integration test: tampered metadata with valid tx signatures is rejected; metadata matches server-side record |

### P1 — High priority (HIGH, 26 findings)

Grouped by theme:

- **VM math (F-H-01, F-H-02, F-H-03):** Replace `wrapping_pow` with `checked_pow`; use `u128` intermediates in DEX/bonding-curve math (on-chain + off-chain); validate jump targets after remap. Definition of done: unit tests with overflow-inducing inputs revert cleanly; no incorrect financial values.
- **Consensus hardening (F-H-07, F-H-08, F-H-09, F-H-10):** Add payload size bounds and bound borsh deserialization; audit/convert non-debug `assert!` in muhash and GhostDAG to error returns; add `checked_*` APIs for `Uint` arithmetic. Definition of done: fuzz tests with crafted payloads do not panic; consensus paths use checked arithmetic.
- **Wallet key management (F-H-05, F-H-06, F-H-11, F-H-12, F-H-13, F-H-14, F-H-15):** Use full address hash as storage key; use `checked_add` for UTXO sums; redact `Debug` impls on key types; set wallet file to `0o600`; remove duplicate fee addition; implement `Drop`/`Zeroize` on `ExtendedPrivateKey` and `Decrypted<T>`. Definition of done: no key material in logs; wallet file mode is `0o600`; `transaction_fees == inputs - outputs` test passes; memory zeroized after drop.
- **Network (F-H-16, F-H-17, F-H-18, F-H-19, F-H-20, F-H-21, F-H-22, F-H-23, F-H-24, F-H-25):** Bound `user_agent` on receive; validate timestamps; add P2P and wRPC connection limits; reject duplicate `PeerKey` (keep existing); ban spam peers; rate-limit address manager; fix `is_banned` underflow with `saturating_sub`; standardize wRPC size limits; replace `expect`/`unwrap` with `?` in conversions. Definition of done: flood tests with 10K connections are rejected; spam peers are banned; ban expiry is clock-skew-safe.
- **Explorer (F-H-04, F-H-26):** Gate `/api/compile` behind `check_write_enabled()`; add parser depth limit; escape all error messages before `innerHTML`. Definition of done: compile endpoint returns 403 on public explorer; deeply nested input returns a parser error, not a crash; error messages use `textContent`.

### P2 — Medium (39 findings)

Hardening backlog. Prioritize:
- **Safety-critical MEDIUMs:** F-M-03 (parser DoS), F-M-10 (decrypt panic), F-M-11 (gen1 import UB), F-M-12 (deadlock), F-M-14 (div-by-zero), F-M-16 (PSBT OOM), F-M-38 (DB panic).
- **Unsafe code cleanup:** F-M-05, F-M-06, F-M-07, F-M-23, F-M-34 — replace `unsafe` blocks with safe alternatives where possible.
- **Explorer hardening:** F-M-30, F-M-31, F-M-32, F-M-33, F-M-35, F-M-36, F-M-37.
- **Wallet hardening:** F-M-13, F-M-15, F-M-17, F-M-18, F-M-19, F-M-20, F-M-21, F-M-22.
- **VERIFIED SAFE (no action):** F-M-08, F-M-09.

### P3 — Low (47 findings)

Defense-in-depth and code-quality backlog. Address opportunistically during regular development. Key clusters:
- **Panic-to-error conversions:** F-L-13, F-L-14, F-L-16, F-L-22, F-L-23, F-L-25, F-L-26, F-L-36, F-L-37, F-L-38, F-L-39, F-L-41.
- **Unsafe `from_utf8_unchecked` family:** F-L-17, F-L-34, F-L-42, F-L-44, F-L-45 — replace with safe `from_utf8` where feasible.
- **Key handling hygiene:** F-L-46, F-L-47 — never log private keys; zeroize intermediate secrets.
- **Overflow defensive checks:** F-L-06, F-L-15, F-L-21, F-L-24, F-L-43.
- **Explorer hardening:** F-L-29, F-L-30, F-L-33 — rate limiting, atomic writes, security headers.
- **VERIFIED SAFE (no action):** F-L-04, F-L-05, F-L-07.

---

## 8. Appendix A: Files Audited (by phase)

### Phase 1 — Custom Zyanya Code
- `rpc/service/src/service.rs` (contract RPC handlers)
- `consensus/src/pipeline/virtual_processor/processor.rs` (contract tx processing)
- `consensus/src/model/stores/contract.rs` (contract state cache/storage)
- `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`
- `zyanya-vm/src/vm.rs`, `zyanya-vm/src/opcode.rs`, `zyanya-vm/src/stack.rs`, `zyanya-vm/src/gas.rs`
- `zyanya-vm/src/compiler/parser.rs`, `zyanya-vm/src/compiler/codegen.rs`
- `zyanya-vm/src/assembler.rs`
- `zyanya-vm/src/token.rs`, `zyanya-vm/src/bonding_curve_token.rs`
- `bonding_curve.zcl`, `dex.zcl`
- `zyanya-explorer/src/api.rs`, `zyanya-explorer/src/client.rs`
- `zyanya-wallet/src/wallet_ops.rs`, `zyanya-wallet/src/key_management.rs`

### Phase 2 — Consensus + Crypto
- `consensus/src/` (coinbase.rs, ghostdag/protocol.rs, difficulty.rs, header_processor/)
- `consensus/core/src/` (mass/mod.rs, config/params.rs, tx.rs, subnets.rs, tx/script_public_key.rs, hashing/sighash.rs)
- `consensus/client/src/`, `consensus/notify/src/`
- `consensus/pow/src/` (matrix.rs, lib.rs)
- `consensus/wasm/src/`
- `crypto/hashes/src/`, `crypto/muhash/src/u3072.rs`, `crypto/addresses/src/bech32.rs`
- `crypto/merkle/src/lib.rs`, `crypto/txscript/src/`
- `math/src/uint.rs`, `math/src/lib.rs`

### Phase 3 — Wallet Core
- `wallet/core/src/` (derivation.rs, encryption.rs, storage/keydata/data.rs, storage/local/wallet.rs, tx/generator/generator.rs, tx/generator/signer.rs, tx/mass.rs, tx/fees.rs, utxo/context.rs, account/variants/multisig.rs, account/pssb.rs, account/mod.rs, compat/gen1.rs, wallet/mod.rs)
- `wallet/keys/src/` (secret.rs, derivation/gen0/hd.rs)
- `wallet/bip32/src/` (xprivate_key.rs, xkey.rs, child_number.rs, derivation_path.rs, prefix.rs, mnemonic/phrase.rs)
- `wallet/psst/src/` (psst.rs, input.rs)
- `wallet/macros/src/`, `wallet/wasm/src/`, `wallet/native/src/`
- `consensus/core/src/sign.rs`

### Phase 4 — RPC + Protocol + Network
- `rpc/service/src/service.rs`
- `rpc/core/src/` (api/ops.rs, error.rs, model/hex_cnv.rs)
- `rpc/grpc/core/src/` (lib.rs, convert/header.rs, convert/message.rs)
- `rpc/grpc/server/src/connection_handler.rs`, `rpc/grpc/client/src/lib.rs`
- `rpc/wrpc/server/src/` (server.rs, service.rs, connection.rs)
- `rpc/wrpc/proxy/src/main.rs`, `rpc/wrpc/client/src/client.rs`
- `protocol/p2p/src/` (core/connection_handler.rs, core/hub.rs, core/router.rs, core/peer.rs, convert/messages.rs, convert/model/version.rs, convert/net_address.rs, handshake.rs)
- `protocol/flows/src/` (flow_context.rs, v5/txrelay/flow.rs, flowcontext/transactions.rs)
- `components/addressmanager/src/lib.rs`

### Phase 5 — Explorer + Utils + Remaining Modules
- `zyanya-explorer/src/` (web.rs, api.rs, client.rs, main.rs)
- `database/src/` (registry.rs, key.rs, db.rs, db/conn_builder.rs)
- `notify/src/` (subscriber.rs, notifier.rs)
- `mining/src/` (manager.rs, mempool/config.rs, mempool/model/orphan_pool.rs, mempool/check_transaction_standard.rs, mempool/replace_by_fee.rs, model/topological_sort.rs, block_template/)
- `math/src/uint.rs` (Display/LowerHex impls)
- `utils/src/` (hex.rs, networking.rs, serde_bytes/, channel.rs, sync/, fd_budget.rs)
- `wasm/core/src/` (events.rs, types.rs)
- `cli/src/wizards/wallet.rs`, `cli/src/matchers.rs`
- `zyanyad/src/args.rs`
- `daemon/src/lib.rs`
- `rothschild/src/main.rs`
- `simpa/src/`
- `metrics/core/src/data.rs`

---

## 9. Appendix B: Verified-Safe / Informational items

### Phase 2 — Verified Safe (13 items)

1. **PoW target comparison** (`consensus/pow/src/lib.rs:65`): Uses `pow <= target` (inclusive), matching upstream Kaspa/Bitcoin convention. Correct.
2. **`from_compact_target_bits`** (`math/src/lib.rs:64-82`): Correctly rejects negative mantissa; round-trips with `compact_target_bits`. No difficulty manipulation possible.
3. **Difficulty calculation** (`consensus/src/processes/difficulty.rs:150-160`): Uses `Uint320` for target sum; `max(max_ts - min_ts, 1)` avoids div-by-zero; `min(new_target, max_difficulty_target)` enforces floor. Correct.
4. **`calc_work`** (`difficulty.rs:270-275`): `(!target / (target + 1)) + 1` matches Bitcoin; `target + 1` cannot overflow. Correct.
5. **Script VM opcode safety** (`crypto/txscript/src/`): All limits enforced (`MAX_STACK_SIZE = 244`, `MAX_SCRIPT_ELEMENT_SIZE = 520`, etc.); arithmetic opcodes use `checked_*`; `OpMul`/`OpDiv`/etc. disabled; multisig bounds checked. Correct (upstream Kaspa).
6. **Bech32 address decoding** (`crypto/addresses/src/bech32.rs`): `REV_CHARSET` lookup uses `.get().unwrap_or(&100)` — no OOB; checksum verified; version byte validated. Correct (upstream Kaspa).
7. **Muhash data_to_element** (`crypto/muhash/src/lib.rs:190-198`): Uses `MuHashElementHash` (Blake2b) → ChaCha20 → 384 bytes. Matches upstream Kaspa. Correct.
8. **Merkle root** (`crypto/merkle/src/lib.rs`): Empty → `ZERO_HASH`; odd pads with `ZERO_HASH`; domain-separated. Correct for all practical leaf counts.
9. **Signature hash** (`consensus/core/src/hashing/sighash.rs`): Domain-separated hashers for Schnorr vs ECDSA; `SigHashType::from_u8` rejects unknown bits. Correct (upstream Kaspa).
10. **Header validation ordering** (`processor.rs:277-285`): PoW checked in `validate_header_in_isolation` (step 1) before GhostDAG (step 3) and difficulty (step 4). No bypass on mainnet/testnet. Correct.
11. **Coinbase vesting value preservation** (`coinbase.rs:135-155`): `liquid = total / 2`, `vested = total - liquid`, `monthly = vested / 12`, `remainder = vested - monthly * 11`. 12 vested outputs sum to `vested`. Total = `liquid + vested = total`. No value creation/destruction. Correct.
12. **CSV lock semantics** (`coinbase.rs:60-70`): `create_csv_locked_script` prepends `<lock_blocks> OP_CHECKSEQUENCEVERIFY`. Test verifies valid/invalid/disabled sequences. Correct.
13. **Keccak F1600** (`crypto/hashes/src/pow_hashers.rs:75`): Inline asm on x86-64 (non-Windows), pure Rust elsewhere. Both use same Keccak-f[1600] permutation. Sound. Correct.

### Phase 2 — H-05 (Verified safe, no finding)
- **Header validation does not check PoW before GhostDAG on trusted blocks** (`consensus/src/pipeline/header_processor/processor.rs:287-292`): For ordinary blocks, PoW is checked in step 1 before GhostDAG. For trusted blocks, `skip_proof_of_work` bypasses PoW on simnet only (by design). On mainnet/testnet, `skip_proof_of_work` is `false`, so PoW is always checked. **Verified safe — no finding.** This explains the Phase 2 count discrepancy (4 HIGH, not 5).

### Phase 2 — M-04 and M-05 (VERIFIED SAFE)
- **F-M-08 (P2 M-04):** Bech32 `conv5to8` truncation is correct for well-formed Bech32 addresses; checksum catches corruption. No action.
- **F-M-09 (P2 M-05):** PoW `compute_rank` f64 is exact for matrix entries in 0..15; `EPS` threshold has >30 orders of magnitude of margin. No action.

### Phase 5 — Informational (14 items)

1. **I-01:** No Stratum server in `mining/src/` — the "Stratum protocol" focus area is N/A. The mining module contains block-template building and mempool management only.
2. **I-02:** Token icon handler has proper path traversal protection (`Path::file_name()` strips path components).
3. **I-03:** Brand asset handler uses static match (no path traversal risk).
4. **I-04:** CLI link matchers use `open_external` with validated URLs (restrictive regex patterns).
5. **I-05:** Mining block template builder delegates to consensus — no direct coinbase/mass arithmetic in the mining crate.
6. **I-06:** Math `overflowing_*` implementations reviewed — all correctly propagate carry/borrow/overflow. No correctness issues.
7. **I-07:** Utils `serde_bytes` deserializers reviewed — use `FromHexVisitor` pattern with proper `str::from_utf8` validation. No unsafe. Safe.
8. **I-08:** Utils `channel.rs`, `sync/rwlock.rs`, `sync/semaphore.rs`, `fd_budget.rs` reviewed — correct implementations, no data races.
9. **I-09:** Metrics `data.rs` counter overflow reviewed — `per_sec` uses `checked_sub().unwrap_or_default()`; `u64` counters overflow at ~584 years of nanoseconds. Handled.
10. **I-10:** Simpa reviewed — test/simulation tool, not production. Many `.unwrap()` calls acceptable in simulation code.
11. **I-11:** Rothschild reviewed — test/load tool. L-19 (F-L-46) private key logging is the main concern. No other security-relevant findings.
12. **I-12:** Integration tests reviewed — test-only code, `.unwrap()`/`.expect()` standard. Not compiled into production.
13. **I-13:** RBF implementation reviewed — correctly handles all three policies; `Allowed` validates all double-spends before removing any. Correct.
14. **I-14:** Orphan pool size limiting reviewed — bounded by `maximum_orphan_transaction_count` (default 500) and `maximum_orphan_transaction_mass` (default 100,000). Bounded.

---

*End of report.*