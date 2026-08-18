# Document — Group A: Contract/VM CRITICAL Fixes (F-C-01 through F-C-05)

**Session:** `a3535b7c`
**Date:** 2026-08-18
**Scope:** Remediate 5 CRITICAL security findings from `audit_reports/FINAL_AUDIT_REPORT.md` (F-C-01 through F-C-05).
**Verification command:** `source /root/.cargo/env && cargo check -p zyanya-vm -p consensus -p rpc/service 2>&1` — passes clean (warnings only, no errors).

---

## What changed and why it matters

The Zyanya blockchain's smart-contract layer had five CRITICAL vulnerabilities that allowed unauthenticated state mutation, commit-on-failure semantics, custody-accounting gaps, no `msg.sender` authentication, and unbounded reentrancy/stack-depth in the VM. This change remediates all five.

### F-C-01 — RPC fallback consensus bypass (removed)

**Problem:** `deploy_contract_call` and `invoke_contract_call` in `rpc/service/src/service.rs` had a fallback path: when mempool submission failed, they directly invoked `ContractProcessor::process_contract_tx` against a fresh `ContractStateCache` and persisted the results to the durable store via `session.write_contract_*`. This persisted contract state with no mined transaction, no GhostDAG ordering, and no consensus validation — an unauthenticated state-mutation primitive.

**Fix:** The entire `Err(_) => { ... }` fallback arms were removed from both handlers. Each now calls `submit_rpc_transaction` and returns `RpcError::RejectedTransaction(tx.id(), err.to_string())` on failure via `?`. The dead `let bytecode = session.get_contract_code(...)` fetch in `invoke_contract_call` was also removed. The read-only `call_contract_call` (which uses a discarded cache) was left untouched.

**Why it matters:** No contract state can now be written outside of consensus. Any mempool rejection is surfaced to the caller instead of silently persisting.

### F-C-02 — Commit-on-failure (fixed)

**Problem:** In `consensus/src/pipeline/virtual_processor/processor.rs`, `has_contract_changes` was set to `true` whenever `process_contract_tx` returned `Some(_)`, regardless of `outcome.success`. In `consensus/src/model/stores/contract.rs`, the Deploy branch inserted bytecode and credited `deposit_amount` *before* validating the bytecode with `OpCode::deserialize_slice`, and the Invoke branch credited `deposit_amount` to `cache.balances` *before* VM execution — so failed transactions still committed state.

**Fix:**
- **`processor.rs`:** Changed to `if let Some(outcome) = processor.process_contract_tx(...)` and only sets `has_contract_changes = true` when `outcome.success` is true.
- **`contract.rs` Deploy:** Reordered so `OpCode::deserialize_slice` validation runs *before* `cache.code.insert` and *before* the deposit credit. Invalid bytecode returns `success: false` without touching the cache.
- **`contract.rs` Invoke:** Removed the pre-execution `cache.balances.insert(...)`. The deposit credit is now applied to `temp_cache` only after `vm.execute_stateful(...)` returns `Ok(res)`. The `*cache = temp_cache` swap happens only on the success path.

**Why it matters:** Failed contract transactions no longer mint balance or install invalid bytecode. State is committed only on successful execution.

### F-C-03 — Bonding-curve custody accounting (fixed, sell fails closed)

**Problem:** The buy path (entry point 4) checked `deposit_amount > 0 && deposit_amount < cost`, so `deposit_amount == 0` passed and minted tokens for free. Nothing deducted `cost` from the contract balance. The sell path (entry point 5) deducted `refund` from the contract balance but created no UTXO output — the refund was silently destroyed.

**Fix:**
- **Buy:** Changed the guard to `invoke.deposit_amount < cost`, which rejects the `deposit_amount == 0` case. After crediting the deposit to `temp_cache`, `cost` is deducted from the contract balance: `temp_cache.balances.insert(addr_bytes, contract_bal.saturating_sub(cost))`.
- **Sell:** Fails closed — returns `success: false` because the consensus virtual processor does not have UTXO-output-creation plumbing for contract payouts. The previous "deduct and destroy" behavior is gone; no refund is silently lost.

**Why it matters:** Buyers can no longer mint tokens without paying the full cost. The contract balance tracks escrowed ZYAN correctly on buy. Sell is disabled until proper UTXO payout plumbing is implemented, preventing silent fund destruction.

### F-C-04 — No `msg.sender` / no authentication (added CALLER opcode)

**Problem:** The VM had no mechanism to expose the transaction submitter to contract code. Contract functions like `transfer`, `buy`, `sell`, `addLiquidity`, and `removeLiquidity` accepted a `caller` parameter that was entirely spoofable — anyone could call `transfer` on behalf of any holder.

**Fix:**
- **New `CALLER` opcode** (`zyanya-vm/src/opcode.rs`): tag `0x61`, gas cost 2, serializes/deserializes, and has a `Display` impl. Added to assembler (`zyanya-vm/src/assembler.rs`) and compiler `caller()` builtin (`zyanya-vm/src/compiler/codegen.rs`).
- **VM `caller` field** (`zyanya-vm/src/vm.rs`): `pub caller: u64` on the `VM` struct, initialized to 0, settable via `with_caller`. The `OpCode::Caller` handler pushes `self.caller` onto the stack. Caller is propagated to child VMs in `Call`.
- **Consensus caller derivation** (`consensus/src/pipeline/virtual_processor/processor.rs`): `commit_utxo_state` now builds a `tx_callers: HashMap<TransactionId, u64>` by iterating `acceptance_data`, finding smart-contract transactions, looking up the first input's spent UTXO entry in `mergeset_diff.remove`, and calling `derive_caller_from_script_pub_key`. That function uses `zyanya_txscript::extract_script_pub_key_address(script_public_key, Prefix::Mainnet)` and `u64::from_le_bytes(addr.payload[0..8])`, mirroring `wallet_ops::holder_u64`. Returns 0 on failure (fail-closed).
- **`process_contract_tx` signature** (`consensus/src/model/stores/contract.rs`): added `caller: u64` parameter, sets `vm.caller = caller` before execution.
- **Contract updates** (`bonding_curve.zcl`, `dex.zcl`): `init` gated to owner (caller == 1); `transfer`/`buy`/`sell` verify `caller_param == caller()`; `addLiquidity`/`removeLiquidity` verify `caller_param == caller()`. The `caller` parameter was renamed to `caller_param` in buy/sell/addLiquidity/removeLiquidity to avoid shadowing the `caller()` builtin.
- **ASM templates** (`zyanya-vm/src/token.rs`, `zyanya-vm/src/bonding_curve_token.rs`): added `CALLER` checks to `transfer` (from == caller), `mint` (caller == owner from sload(1)), `init` (caller == 1), `buy` (caller_param == caller), `sell` (caller_param == caller).
- **Tests:** All VM unit tests updated to set `vm.caller` matching the expected caller. All `process_contract_tx` test call sites updated to pass `caller = 1`.

**Why it matters:** Contracts can now authenticate the transaction signer. Unauthorized transfers, buys, sells, and liquidity operations are rejected. The caller identity is derived from the on-chain UTXO script, not from a user-supplied parameter.

### F-C-05 — CALL reentrancy guard + call-depth limit (added)

**Problem:** The VM `CALL` opcode had no reentrancy guard and no call-depth limit. A contract could `CALL` itself or a mutually-recursive pair indefinitely, causing stack overflow or reentrancy attacks.

**Fix:**
- **New error variants** (`zyanya-vm/src/error.rs`): `CallDepthExceeded { depth, max }` and `ReentrantCall { address: [u8; 32] }`.
- **VM state** (`zyanya-vm/src/vm.rs`): `pub const MAX_CALL_DEPTH: usize = 1024;`. Added `call_depth: usize` and `call_stack: Vec<[u8; 32]>` fields to `VM`.
- **Call-stack seeding:** On entry to `execute_stateful`, if `call_stack` is empty, the top-level contract address is pushed and `call_depth` is set to 1 — so a self-`CALL` is caught as reentrancy.
- **CALL opcode guards:** Before executing a child, checks `call_depth >= MAX_CALL_DEPTH` → `CallDepthExceeded`, and `call_stack.contains(target_addr)` → `ReentrantCall`.
- **Stack management:** Target is pushed onto `call_stack` and `call_depth` incremented before child execution. Both are unwound (pop + decrement) after `child_vm.execute_stateful` returns, before the `match` — ensuring unwind on both `Ok` and `Err` paths. The child VM receives `call_depth` and `call_stack.clone()` for propagation.

**Why it matters:** Recursive or reentrant call patterns are hard-rejected. Call depth is bounded at 1024, preventing stack-overflow DoS.

---

## Files that carry the change

| File | Finding(s) | What changed |
|------|-----------|--------------|
| `rpc/service/src/service.rs` | F-C-01 | Removed fallback arms in `deploy_contract_call` and `invoke_contract_call`; errors returned to caller |
| `consensus/src/pipeline/virtual_processor/processor.rs` | F-C-02, F-C-04 | Commit only on `outcome.success`; caller derivation from tx signer; `derive_caller_from_script_pub_key` helper |
| `consensus/src/model/stores/contract.rs` | F-C-02, F-C-03, F-C-04 | Deploy validates before cache insert; deposit credited on temp_cache only; buy rejects `deposit < cost` and deducts cost; sell fails closed; `caller` parameter added to `process_contract_tx` |
| `zyanya-vm/src/opcode.rs` | F-C-04 | `Caller` variant, tag `0x61`, gas cost 2, serialize/deserialize/display |
| `zyanya-vm/src/vm.rs` | F-C-04, F-C-05 | `caller`, `call_depth`, `call_stack` fields; `MAX_CALL_DEPTH=1024`; `CALLER` handler; CALL depth/reentrancy guards with proper unwind |
| `zyanya-vm/src/error.rs` | F-C-05 | `CallDepthExceeded`, `ReentrantCall` error variants |
| `zyanya-vm/src/lib.rs` | F-C-05 | Re-exports `MAX_CALL_DEPTH` |
| `zyanya-vm/src/assembler.rs` | F-C-04 | `CALLER` keyword + byte-size entry |
| `zyanya-vm/src/compiler/codegen.rs` | F-C-04 | `caller()` builtin (0 args, emits `CALLER`) |
| `zyanya-vm/src/token.rs` | F-C-04 | ASM template: `CALLER` checks for `transfer` and `mint`; test updates |
| `zyanya-vm/src/bonding_curve_token.rs` | F-C-04 | ASM template: `CALLER` checks for `init`, `transfer`, `buy`, `sell`; test updates |
| `bonding_curve.zcl` | F-C-04 | `init` owner-gated; `transfer`/`buy`/`sell` verify caller |
| `dex.zcl` | F-C-04 | `addLiquidity`/`removeLiquidity` verify caller |
| `specs/a3535b7c_contract-vm-critical-fixes.md` | All | Remediation plan / spec |
| `audit_reports/FINAL_AUDIT_REPORT.md` | All | Source audit report (from prior session `01566f2e`) |

---

## How to verify

1. **Compilation check (as run during the build):**
   ```bash
   source /root/.cargo/env && cargo check -p zyanya-vm -p consensus -p rpc/service 2>&1
   ```
   Expected: `Finished` with warnings only (pre-existing lifetime elision warnings), no errors.

2. **F-C-01 verification:** Grep for `process_contract_tx` or `write_contract_*` in `deploy_contract_call` / `invoke_contract_call` in `rpc/service/src/service.rs` — should find zero references. Both handlers should return `RpcError::RejectedTransaction` on mempool failure.

3. **F-C-02 verification:** In `consensus/src/pipeline/virtual_processor/processor.rs`, confirm `has_contract_changes` is only set when `outcome.success` is true. In `consensus/src/model/stores/contract.rs`, confirm Deploy validates bytecode before `cache.code.insert` and Invoke credits deposit to `temp_cache` only after `Ok(res)`.

4. **F-C-03 verification:** In `contract.rs`, confirm buy guard is `deposit_amount < cost` (not `deposit_amount > 0 && deposit_amount < cost`), and that `cost` is deducted from `temp_cache.balances`. Confirm sell returns `success: false` (fail-closed).

5. **F-C-04 verification:** Confirm `OpCode::Caller` exists in `opcode.rs` (tag `0x61`). Confirm `VM.caller` field and `CALLER` handler in `vm.rs`. Confirm `process_contract_tx` takes a `caller: u64` parameter. Confirm `derive_caller_from_script_pub_key` in `processor.rs`. Confirm `caller()` checks in `bonding_curve.zcl` and `dex.zcl`.

6. **F-C-05 verification:** Confirm `MAX_CALL_DEPTH = 1024` in `vm.rs`. Confirm `call_depth` and `call_stack` fields. Confirm `CallDepthExceeded` and `ReentrantCall` in `error.rs`. Confirm CALL opcode checks both before creating child VM, and unwinds (pop + decrement) after child returns.

---

## Notes

- **F-C-03 sell is disabled (fail-closed).** The consensus virtual processor lacks UTXO-output-creation plumbing for contract payouts. Rather than silently destroying the refund (the original bug), sell is rejected with `success: false` until proper payout plumbing is implemented. This is explicitly allowed by the remediation plan.
- **Caller identity is u64** (not full 32-byte address). This is consistent with existing `holder_u64` derivation used throughout the contract system. Collision resistance is tracked separately as F-H-05 (out of scope).
- **`call_contract_call` (read-only simulation)** uses `caller = 0` for the discarded-cache simulation. Caller-gated contracts will return 0 unless a caller is supplied — acceptable for read-only simulation.
- **No full build or tests were run** — only `cargo check` as specified.
- The `audit_reports/FINAL_AUDIT_REPORT.md` was produced by a prior session (`01566f2e`) and is not modified by this work.