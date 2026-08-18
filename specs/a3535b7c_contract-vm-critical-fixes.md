# Remediation Plan — Group A: Contract/VM CRITICAL Fixes (F-C-01 … F-C-05)

**Date:** 2026-08-18
**Scope:** Remediate the 5 CRITICAL contract/VM findings from `audit_reports/FINAL_AUDIT_REPORT.md`.
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md` §3 (F-C-01 through F-C-05).
**Verification:** `source /root/.cargo/env && cargo check -p zyanya-vm -p consensus -p rpc/service 2>&1` (no full build, no tests).

---

## 0. Priority order (all CRITICAL — do in this order)

1. **F-C-01** — RPC fallback consensus bypass (state persisted without a mined tx). *Highest: unauthenticated state-mutation primitive.*
2. **F-C-02** — Commit-on-failure (failed txs persist state / mint balance).
3. **F-C-03** — Bonding-curve custody accounting (buy never deducts, sell destroys refund).
4. **F-C-04** — No `msg.sender` / no authentication (theft primitive in token/bonding-curve/DEX).
5. **F-C-05** — `CALL` reentrancy + no call-depth limit (stack-overflow DoS + reentrancy).

F-C-04 and F-C-05 both touch `zyanya-vm`; implement the VM opcode/state changes together to avoid rework, then wire the consensus caller plumbing (F-C-04) and the contract `.zcl` updates.

---

## 1. F-C-01 — Remove RPC fallback that persists contract state without a mined tx

**File:** `rpc/service/src/service.rs`

**Current behavior (lines 586–645 `deploy_contract_call`, 647–716 `invoke_contract_call`):**
- Both handlers build an input-less/output-less contract tx, call `submit_rpc_transaction`, and on **any** `Err` fall back to `ContractProcessor::process_contract_tx` against a fresh `ContractStateCache`, then write `cache.code` / `cache.storage` / `cache.balances` to the durable store via `session.write_contract_*`. This persists state with no mined tx, no GhostDAG ordering, no consensus validation.

**Fix steps:**

1. In `deploy_contract_call` (line 614–634), delete the entire `Err(_) => { ... }` fallback arm. Replace the `match submit_res { ... }` with a single error-return path modeled on `submit_transaction_call` (lines 553–560):
   ```rust
   self.flow_context
       .submit_rpc_transaction(&session, tx.clone(), Orphan::Forbidden)
       .await
       .map_err(|err| {
           let err = RpcError::RejectedTransaction(tx.id(), err.to_string());
           debug!("{err}");
           err
       })?;
   ```
   Then return `Ok(DeployContractResponse { contract_address, transaction_id: tx.id(), gas_used: request.max_gas, success: true })`.

2. In `invoke_contract_call` (line 676–714), do the same: delete the `Err(_) => { ... }` fallback arm (including the `fallback_storage`/`fallback_balance` seeding, the `cache.code.insert`, and the `session.write_contract_*` loop). Return the mempool error via `RpcError::RejectedTransaction(tx.id(), err.to_string())`. On success return `Ok(InvokeContractResponse { transaction_id: tx.id(), gas_used: request.max_gas, return_value: None, success: true })`.

3. Remove now-dead code:
   - `invoke_contract_call`: the `let bytecode = session.get_contract_code(...)` fetch (lines 649–650) becomes unused — delete it. Keep `session` (still used by `submit_rpc_transaction`).
   - `deploy_contract_call`: `session` is still used by `submit_rpc_transaction`; keep it.
   - Confirm no remaining references to `ContractStateCache`/`ContractProcessor`/`write_contract_*` in these two handlers.

4. **Do NOT touch `call_contract_call` (line 749+).** It already executes against a discarded `RpcDbStateBackend` cache and never persists — that is the correct read-only simulation path. Leave it as-is (its `.try_into().unwrap()` at line 797 is F-C-11, out of scope for this group).

**Acceptance:** `deploy_contract_call` and `invoke_contract_call` never call `process_contract_tx` and never call `session.write_contract_*`. Any mempool rejection is returned to the caller as `RpcError::RejectedTransaction`.

---

## 2. F-C-02 — Failed contract txs must not commit state (commit-on-failure)

**Files:**
- `consensus/src/pipeline/virtual_processor/processor.rs` (lines 511–521)
- `consensus/src/model/stores/contract.rs` (Deploy ~213–218, Invoke ~281–283)

**Fix steps:**

### 2a. `processor.rs` — commit only on success

Current (line 511–513):
```rust
if processor.process_contract_tx(tx, &mut contract_cache).is_some() {
    has_contract_changes = true;
}
```
Change to:
```rust
if let Some(outcome) = processor.process_contract_tx(tx, &mut contract_cache) {
    if outcome.success {
        has_contract_changes = true;
    }
}
```
This alone is not sufficient (the shared `contract_cache` is mutated in place across all txs in the block), so the `contract.rs` changes below are required to keep failed mutations out of the cache.

### 2b. `contract.rs` — Deploy branch: validate before mutating, credit deposit only on success

Current (lines 213–218) inserts bytecode and credits `deposit_amount` **before** the `OpCode::deserialize_slice` validation at line 220, so an invalid-bytecode deploy still installs code.

Reorder so that:
1. Deserialize/validate `deploy.bytecode` first (`OpCode::deserialize_slice`). On `Err`, return the existing failure outcome (`success: false`) **without** touching `cache`.
2. Only after successful validation, insert `cache.code.insert(addr_bytes, deploy.bytecode.clone())`.
3. Move the `deposit_amount` credit to **after** successful validation (and, per F-C-03, reconcile it — see §3).

The Deploy branch does not run the VM (constructor is invoked separately via entry point 0), so a full `temp_cache` swap is not required here — simply do not mutate `cache` until success is established.

### 2c. `contract.rs` — Invoke branch: credit deposit on `temp_cache`, not `cache`

Current (lines 281–283) credits `deposit_amount` to `cache.balances` **before** execution, so a failed invoke still mints balance.

Move the deposit credit into the `temp_cache` path (the branch already clones `cache` into `temp_cache` at line 310 and swaps on success at line 356):
- Delete the pre-execution `cache.balances.insert(...)` at lines 281–283.
- After `vm.execute_stateful(...)` returns `Ok(res)` (i.e., execution succeeded), apply the deposit credit to `temp_cache` (before or as part of the buy/sell custody logic in §3), then `*cache = temp_cache`.

**Acceptance:** A failed deploy installs no bytecode; a failed invoke credits no balance; `processor.rs` only flags `has_contract_changes` when `outcome.success == true`.

---

## 3. F-C-03 — Bonding-curve custody accounting

**File:** `consensus/src/model/stores/contract.rs` (invoke branch, ~lines 281–283 deposit credit, 320–333 buy, 335–353 sell)

**Current bugs:**
- Buy (entry point 4): checks `deposit_amount > 0 && deposit_amount < cost` — so `deposit_amount == 0` passes and mints tokens for free. Nothing deducts `cost` from the contract balance.
- Sell (entry point 5): deducts `refund` from `temp_cache.balances` but creates **no** UTXO output paying the seller — the refund is destroyed.
- No reconciliation between `cache.balances` and `reserve` (storage key 2).

**Fix steps (in `process_contract_tx`, invoke branch, after `Ok(res)`):**

1. **Reject buy when `deposit_amount < cost` (including 0).** Change the buy guard (line 320–333) to:
   ```rust
   if invoke.entry_point == 4 {
       let cost = ret_val;
       if invoke.deposit_amount < cost {
           // insufficient deposit (covers deposit_amount == 0)
           return Some(ContractExecutionOutcome { /* ... */ success: false });
       }
   }
   ```

2. **Deduct `cost` from the contract balance on buy.** After the deposit is credited to `temp_cache.balances` (see §2c), subtract `cost`:
   ```rust
   let contract_bal = temp_cache.get_balance(&addr_bytes);
   // contract_bal must be >= cost (guaranteed by deposit >= cost after credit)
   temp_cache.balances.insert(addr_bytes, contract_bal.saturating_sub(cost));
   ```
   Net effect: buyer's `deposit_amount` is escrowed into the contract, `cost` is locked into the reserve (the contract code already does `sstore(2, reserve + cost)`), and the contract balance reflects the escrow.

3. **Sell: create a payout instead of destroying the refund.** Extend `ContractExecutionOutcome` (in `contract.rs`, ~line 180) with a payout field, e.g.:
   ```rust
   pub payouts: Vec<ContractPayout>,   // ContractPayout { script_public_key: ScriptPublicKey, amount: u64 }
   ```
   (or `seller: [u8; 32]` + `amount: u64` if the virtual processor will derive the script). On sell success (entry point 5), after deducting `refund` from `temp_cache.balances`, record the payout in the outcome instead of silently dropping it. The seller identity comes from the authenticated caller (F-C-04) — do **not** trust the `caller` parameter.

4. **Reconcile balances with reserve.** After buy/sell, assert/derive `cache.balances[contract] == reserve` (storage key 2) and keep them consistent. Concretely: on buy, `reserve += cost` (contract code) and `balance -= cost` (processor); on sell, `reserve -= refund` (contract code) and `balance -= refund` (processor). Add a debug assertion or explicit reconciliation so the two cannot drift.

5. **Virtual processor payout plumbing** (`consensus/src/pipeline/virtual_processor/processor.rs`): after processing contract txs in `commit_utxo_state`, for each successful outcome with `payouts`, add the payout UTXO output(s) to the block's UTXO diff so the seller actually receives the refund. This is the most involved part:
   - `commit_utxo_state` currently receives `mergeset_diff: UtxoDiff` and `multiset: MuHash` and writes them. The payout outputs must be added to `mergeset_diff` (and the multiset updated) **before** `utxo_diffs_store.insert_batch` / `utxo_multisets_store.insert_batch` (lines 478–479).
   - If full UTXO-diff plumbing is not feasible in this pass, **fail closed**: reject the sell (return `success: false`) when the payout cannot be created, so the refund is never silently destroyed. Do not ship the current "deduct and drop" behavior.

**Acceptance:** `deposit_amount == 0` buy is rejected; buy deducts `cost` from contract balance; sell records a payout (or fails closed) rather than destroying the refund; `balances` and `reserve` stay reconciled.

---

## 4. F-C-04 — Add authenticated caller (`msg.sender`) to the VM and contracts

**Files:**
- `zyanya-vm/src/opcode.rs` — new `OpCode::Caller`
- `zyanya-vm/src/vm.rs` — VM `caller` field + `Caller` execution
- `zyanya-vm/src/assembler.rs` — `CALLER` keyword
- `zyanya-vm/src/compiler/codegen.rs` — `caller()` builtin
- `zyanya-vm/src/error.rs` — (no new error needed; reuse existing)
- `consensus/src/model/stores/contract.rs` — pass caller into VM
- `consensus/src/pipeline/virtual_processor/processor.rs` — derive caller from tx signer
- `bonding_curve.zcl`, `dex.zcl` — verify caller before transfer/buy/sell/addLiquidity/removeLiquidity

### 4a. VM: add `Caller` opcode

1. `opcode.rs`:
   - Add variant `Caller` (no payload).
   - `base_gas_cost()` → `2`.
   - `serialize_slice` → emit `0x61` (next free tag after `CALL` = `0x60`).
   - `deserialize_slice` → `0x61 => opcodes.push(OpCode::Caller)`.
   - `Display` → `"CALLER"`.

2. `assembler.rs`:
   - Add `"CALLER" => OpCode::Caller` in the match.
   - Add `"CALLER"` to the 1-byte list in `opcode_line_byte_size`.

3. `vm.rs`:
   - Add `pub caller: u64` to the `VM` struct; initialize to `0` in `VM::new`.
   - Add a constructor/setter, e.g. `pub fn with_caller(gas_limit: u64, caller: u64) -> Self` (or set `vm.caller` directly before execution).
   - In `execute_stateful`, handle `OpCode::Caller => { self.stack.push(self.caller)?; self.pc += 1; }`.
   - In the `Call` opcode, propagate `self.caller` into the child VM (the tx submitter is constant across the call tree).

4. `compiler/codegen.rs`:
   - Add a `caller()` builtin (0 args) in `generate_expression`'s `Expression::Call` match that emits `CALLER`. This lets ZCL write `let sender = caller();`.

### 4b. Consensus: derive and pass the caller

1. `contract.rs` `process_contract_tx`: add a `caller: u64` parameter (or a `caller: [u8; 32]` if you prefer full-address identity — see note). Set `vm.caller = caller` before `vm.execute_stateful(...)`.

2. `processor.rs` `commit_utxo_state`: derive the caller from the **transaction signer** — the owner of the first input's UTXO entry `script_public_key`:
   - Resolve the first input's UTXO entry (the input's `previous_outpoint` → UTXO set). This requires the UTXO view to be available in `commit_utxo_state`; if it is not, thread the resolved caller (or a `tx_id → caller` map) from `calculate_utxo_state` (which has `selected_parent_utxo_view`, line 440) into `commit_utxo_state`.
   - Convert the `script_public_key` to an address via `zyanya_txscript::extract_script_pub_key_address`, then to the u64 holder key used by contracts: `u64::from_le_bytes(address.payload[0..8])` (mirror `zyanya-wallet/src/wallet_ops.rs::holder_u64`, lines 279–285). **Use the same derivation everywhere** so the CALLER value matches the holder keys the contracts already use.
   - Pass that u64 into `process_contract_tx`.

3. `rpc/service/src/service.rs` `call_contract_call` (read-only): pass `caller = 0` (or a caller derived from the request if one is added). Note in a comment that read-only calls with caller-gated contracts will return 0 unless a caller is supplied — acceptable for a discarded-cache simulation.

**Design note (caller identity width):** The operand stack is `u64`, and existing contracts key holders by u64 (`holder_u64`). The pragmatic, consistent choice is a u64 caller. If you want collision resistance, switch holder keys to `[u8; 32]` (out of scope here; F-H-05 tracks that separately). Keep u64 for this pass.

### 4c. Update contracts to enforce the caller

`bonding_curve.zcl`:
- `transfer(from, to, amount)`: add `let sender = caller(); if (from != sender) { return 0; }` before the balance check.
- `buy(caller, tokens_to_mint)`: add `let sender = caller(); if (caller != sender) { return 0; }`.
- `sell(caller, tokens_in)`: add `let sender = caller(); if (caller != sender) { return 0; }`.
- `init(slope)`: gate to owner — add an owner check (e.g. `let sender = caller(); if (sender != OWNER) { return 0; }` with an owner constant, or reject re-init once initialized). At minimum, prevent arbitrary re-init that resets slope/supply/reserve.

`dex.zcl`:
- `addLiquidity(caller, amountA, amountB)`: `let sender = caller(); if (caller != sender) { return 0; }`.
- `removeLiquidity(caller, lpAmount)`: `let sender = caller(); if (caller != sender) { return 0; }`.
- `swap` has no caller param (it operates on reserves) — leave as-is unless you add a caller check for fee accounting.

Also update the hand-written assembly templates if they are the deployed bytecode path: `zyanya-vm/src/token.rs` (`TOKEN_CONTRACT_ASM_TEMPLATE`) and `zyanya-vm/src/bonding_curve_token.rs` (`BONDING_CURVE_ASM`) — add `CALLER` checks to `transfer`/`mint` (token) and `transfer`/`buy`/`sell` (bonding curve) so the on-chain bytecode matches the `.zcl` sources. Gate `mint` (token entry point 3) and `init` (bonding curve entry point 0) to the owner.

**Acceptance:** `CALLER` opcode round-trips through serialize/deserialize/assembler; VM pushes the caller; `process_contract_tx` receives and forwards the signer-derived caller; contracts reject `from != caller` / `caller != sender`.

---

## 5. F-C-05 — `CALL` reentrancy guard + call-depth limit

**File:** `zyanya-vm/src/vm.rs` (Call opcode, lines 236–272), `zyanya-vm/src/error.rs`

**Fix steps:**

1. `error.rs`: add variants:
   - `CallDepthExceeded { depth: usize, max: usize }` (or a simple `CallDepthExceeded`).
   - `ReentrantCall { address: [u8; 32] }` (or `ReentrantCall`).

2. `vm.rs`:
   - Add `pub const MAX_CALL_DEPTH: usize = 1024;`.
   - Add fields to `VM`: `call_depth: usize` (init 0) and `call_stack: Vec<[u8; 32]>` (init empty).
   - In `execute_stateful`, seed the call stack with the top-level contract: if `self.call_stack.is_empty()`, push `*contract_address` and set `call_depth = 1` (so a self-`CALL` is caught as reentrancy).
   - In `OpCode::Call(target_addr)`:
     1. `if self.call_depth >= MAX_CALL_DEPTH { return Err(VMError::CallDepthExceeded { depth: self.call_depth, max: MAX_CALL_DEPTH }); }`
     2. `if self.call_stack.contains(target_addr) { return Err(VMError::ReentrantCall { address: *target_addr }); }`
     3. Push `*target_addr` onto `self.call_stack`, increment `self.call_depth`.
     4. Create the child VM with `VM::new(forward_gas)`, then set `child_vm.call_depth = self.call_depth` and `child_vm.call_stack = self.call_stack.clone()` and `child_vm.caller = self.caller`.
     5. Execute the child; on completion (Ok or Err), pop `self.call_stack` and decrement `self.call_depth` (use a guard/`let _ =` pattern or explicit pop after the match so it always unwinds).
     6. Keep the existing gas refund and return-value push behavior.

3. Keep the existing `get_code`/`deserialize_slice` error handling (push 0, continue) for those specific failures; only depth/reentrancy are hard rejections.

**Acceptance:** A contract that `CALL`s itself (or a mutually-recursive pair) is rejected with `ReentrantCall`; recursion beyond 1024 frames is rejected with `CallDepthExceeded`; the call stack unwinds correctly on both success and error paths.

---

## 6. Cross-cutting notes & risks for the builder

- **Order of implementation:** F-C-01 first (removes the only non-consensus caller of `process_contract_tx`), then F-C-02/F-C-03 (both in `contract.rs` + `processor.rs`), then F-C-04/F-C-05 (VM + consensus plumbing + `.zcl`). This minimizes rework and keeps `cargo check` green between steps.
- **`process_contract_tx` signature change (F-C-04):** adding a `caller` parameter touches its two call sites (`processor.rs:511` and the now-removed RPC fallback) plus the unit tests in `contract.rs` (`#[cfg(test)] mod tests`, lines ~436–590). Update the test call sites to pass a caller (e.g. `1`) so `cargo check -p consensus` stays clean.
- **`VM` struct change (F-C-04/F-C-05):** adding `caller`, `call_depth`, `call_stack` fields is additive; `VM::new` keeps working. Existing tests in `zyanya-vm` (`lib.rs`, `token.rs`, `bonding_curve_token.rs`, `assembler.rs`, `compiler/mod.rs`) should still compile; update any that assert exact `VMResult`/stack behavior if the new opcode/fields change it.
- **Caller derivation plumbing (F-C-04):** the trickiest part is resolving the first input's UTXO entry inside `commit_utxo_state`. If the UTXO view is not reachable there, thread a `HashMap<TransactionId, u64>` (tx → caller) computed in `calculate_utxo_state` (where `selected_parent_utxo_view` exists) into `commit_utxo_state`. Do **not** add a caller field to the contract payload (spoofable).
- **F-C-03 payout UTXO:** if creating the payout output in the UTXO diff proves too invasive, fail closed (reject sell) rather than leaving the refund-destroyed behavior. Document the chosen approach in the code.
- **Do not run full build or tests** — only `cargo check -p zyanya-vm -p consensus -p rpc/service`. Fix all compile errors (and any new warnings that indicate dead code from the F-C-01 removal).
- **Out of scope (do not fix here):** F-C-06 … F-C-16, F-H-*, F-M-*, F-L-* (including F-C-11's `.try_into().unwrap()` in `call_contract_call` and F-H-05's `holder_u64` collision). Keep changes minimal and targeted to the 5 findings.

---

## 7. Verification checklist

1. `source /root/.cargo/env && cargo check -p zyanya-vm -p consensus -p rpc/service 2>&1` exits 0.
2. `deploy_contract_call` / `invoke_contract_call` contain no `process_contract_tx` and no `session.write_contract_*`.
3. `processor.rs` commits only on `outcome.success`.
4. `contract.rs` Deploy validates before `cache.code.insert`; Invoke credits deposit on `temp_cache` only.
5. Buy rejects `deposit_amount < cost` (incl. 0) and deducts `cost`; sell records a payout (or fails closed).
6. `CALLER` opcode present in `opcode.rs`/`assembler.rs`/`vm.rs`; `caller()` builtin in `codegen.rs`; contracts check the caller.
7. `CALL` enforces `MAX_CALL_DEPTH` and rejects reentrant targets.
