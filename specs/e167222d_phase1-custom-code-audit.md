# Phase 1 Security Audit Plan — Custom Zyanya Code

## Objective

Perform a deep, line-level security audit of the 15 custom Zyanya modules listed below,
validate and expand the scout's preliminary findings, and write a single findings report to:

```
audit_reports/phase1_findings.md
```

The report must contain, for **every** finding:

- **Severity**: `CRITICAL` / `HIGH` / `MEDIUM` / `LOW`
- **Location**: `file:line` (exact line numbers, verified against the current tree)
- **Description**: what the bug is and how it is triggered
- **Code snippet**: the offending lines (verbatim, trimmed)
- **Recommended fix**: concrete, minimal remediation

Order findings by severity (CRITICAL first), then by file.

---

## Audit scope (files, in priority order)

### CRITICAL tier

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 1 | `rpc/service/src/service.rs` | Consensus bypass via RPC fallback; unauthenticated state mutation; write path gating |
| 2 | `consensus/src/model/stores/contract.rs` | Custody accounting (buy/sell), fee burn, failed-tx state pollution, unsafe `as_ref` |
| 3 | `consensus/src/pipeline/virtual_processor/processor.rs` | Commit-on-failure of contract cache |
| 4 | `zyanya-vm/src/vm.rs` | CALL reentrancy / no call-depth limit, POW silent overflow, gas accounting |
| 5 | `bonding_curve.zcl`, `dex.zcl` | Missing auth (no msg.sender), u64 overflow in AMM math, reentrancy |

### HIGH tier

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 6 | `zyanya-vm/src/opcode.rs` | Jump-target validation, deserialization safety, byte-offset vs opcode-index confusion |
| 7 | `zyanya-vm/src/compiler/` (lexer, parser, codegen, ast, token, mod) | ZCL source injection, parser recursion DoS, codegen stack discipline |
| 8 | `zyanya-explorer/src/client.rs` | Off-chain cost math overflow, key coercion to 0/1, tx building, schnorr signing, base64 decode |
| 9 | `zyanya-explorer/src/api.rs` | Write-endpoint gating, silent key coercion, input validation |
| 10 | `zyanya-wallet/src/wallet_ops.rs` | UTXO selection overflow, `holder_u64` address collision, send/token/DEX ops |

### MEDIUM tier

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 11 | `zyanya-vm/src/gas.rs` | Gas metering saturation, refund semantics |
| 12 | `zyanya-vm/src/stack.rs` | Stack overflow/underflow, cross-call depth |
| 13 | `zyanya-vm/src/assembler.rs` | Label resolution, byte-offset arithmetic, duplicate labels |
| 14 | `zyanya-wallet/src/key_management.rs` | Key storage, mnemonic handling, signing, hex parsing |
| 15 | `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` | Contract payload validation gaps, missing bounds |

---

## Findings to validate and expand (scout + additional)

For each item: confirm the exact `file:line`, reproduce the trigger, and write it up in the
report format above. Items marked **[NEW]** were not in the scout list and must be added.

### 1. RPC fallback bypasses consensus on mempool failure — CRITICAL

- **File**: `rpc/service/src/service.rs`
- **Lines**: `deploy_contract_call` ~614–635; `invoke_contract_call` ~676–716
- **What to verify**: On `submit_rpc_transaction` error, both handlers fall back to
  `ContractProcessor::process_contract_tx` and then write state directly via
  `session.write_contract_code / write_contract_storage / write_contract_balance`.
  This executes and persists contract state **without** a mined transaction, without
  mempool validation, and without consensus. Confirm the fallback is reachable by any
  RPC client (no auth on the RPC surface) and that it mutates durable state.
- **Fix direction**: remove the fallback; return the mempool error. If a read-only
  simulation is desired, use a discarded cache (as `call_contract_call` does).

### 2. Contract custody accounting gap (buy does not deduct, sell does not credit) — CRITICAL

- **File**: `consensus/src/model/stores/contract.rs`
- **Lines**: deposit credit ~281–283; buy check ~320–333; sell check ~335–353
- **What to verify**:
  - `invoke.deposit_amount` is credited to `cache.balances` before execution, but the
    deposit is a payload field, not a UTXO locked to the contract — the buyer's ZYAN is
    never actually escrowed to the contract address.
  - Buy (entry_point 4) checks `deposit_amount >= cost` but never deducts `cost` from the
    contract balance; the contract's `reserve` (key 2) and the `balances` map diverge.
  - Sell (entry_point 5) deducts `refund` from the contract balance but never creates a
    UTXO output paying the seller — the seller's refund is destroyed.
- **Fix direction**: model custody as real UTXO value transfer (contract-owned outputs),
  or make deposit/refund explicit in the tx input/output set and reconcile `balances`
  with `reserve`.

### 3. Failed contract txs still commit state — CRITICAL [NEW]

- **File**: `consensus/src/pipeline/virtual_processor/processor.rs` ~511–521 and
  `consensus/src/model/stores/contract.rs` (Deploy ~213, Invoke ~281)
- **What to verify**: `process_contract_tx` returns `Some(outcome)` even when
  `outcome.success == false`, and the caller only checks `.is_some()` before setting
  `has_contract_changes = true` and committing the cache. Additionally, the Deploy branch
  inserts `cache.code` and the Invoke branch credits `deposit_amount` to `cache.balances`
  **before** execution, so a failed deploy persists bytecode and a failed invoke persists
  the deposit.
- **Fix direction**: only commit when `success == true`; perform all cache mutations on a
  temp cache and swap in only on success (mirror the Invoke `temp_cache` pattern in Deploy).

### 4. VM CALL opcode: no reentrancy guard, no call-depth limit — CRITICAL

- **File**: `zyanya-vm/src/vm.rs` ~236–273
- **What to verify**:
  - `OpCode::Call` spawns a child `VM` sharing the same `&mut state`; a contract can call
    itself (or any contract) recursively with no depth limit and no reentrancy lock.
  - Deep recursion → Rust stack overflow → node process crash (DoS).
  - Reentrancy: a callee can re-enter the caller mid-execution and mutate shared storage
    (classic reentrancy, exploitable by the bonding-curve/DEX contracts).
- **Fix direction**: add a call-depth limit (e.g. 1024) and/or a reentrancy guard
  (per-contract "entered" flag), and reject calls that exceed it.

### 5. POW opcode uses `wrapping_pow` (silent overflow) — HIGH

- **File**: `zyanya-vm/src/vm.rs` ~125–131
- **What to verify**: `a.wrapping_pow(b as u32)` silently wraps on overflow, unlike
  `Add/Sub/Mul` which use `checked_*`. A contract can produce a wrapped result that is
  then used in financial math (e.g. bonding curve) without error.
- **Fix direction**: use `checked_pow` and return `VMError::ArithmeticOverflow`.

### 6. No msg.sender / no authentication in token & bonding-curve contracts — CRITICAL

- **Files**: `bonding_curve.zcl` (transfer ~14–24, buy ~37–55, sell ~58–88),
  `dex.zcl` (addLiquidity ~14–36, swap ~39–60, removeLiquidity ~63–86),
  `zyanya-vm/src/token.rs` (transfer/mint), `zyanya-vm/src/bonding_curve_token.rs`
- **What to verify**:
  - `transfer(from, to, amount)` takes `from` as a caller-supplied parameter with no
    check that the caller owns `from` — anyone can drain any holder.
  - `mint` (token.rs) and `init` (bonding curve) are callable by anyone at any time.
  - The VM has no notion of `msg.sender`; the consensus layer passes `entry_point` and
    `parameters` only.
- **Fix direction**: introduce an authenticated caller identity (derive from tx signer)
  and enforce ownership in the contracts; gate `mint`/`init`.

### 7. DEX / bonding-curve math can overflow u64 — HIGH

- **Files**: `dex.zcl` (swap `resB * amountIn * 997` ~44–48), `bonding_curve.zcl`
  (buy `slope * (2*supply*k + k*k)` ~41–43), and off-chain mirror in
  `zyanya-explorer/src/client.rs` ~897 and ~1045
- **What to verify**:
  - On-chain: `MUL` is checked in the VM, so overflow reverts the tx (DoS / griefing),
    but the DEX constant-product formula overflows u64 for realistic reserves
    (e.g. 1e9 * 1e9 * 997 ≈ 1e21 > u64::MAX).
  - Off-chain: `2 * S * k + k * k` and `2 * S * k - k * k` in `client.rs` are plain u64
    arithmetic — silent wrap in release builds → wrong cost/refund quoted to users.
- **Fix direction**: use u128 intermediates (or checked math) in both the contracts and
  the explorer; reject overflow explicitly.

### 8. Explorer silently coerces invalid keys to 0 — MEDIUM

- **Files**: `zyanya-explorer/src/api.rs` ~155–159 (`api_contract_state_handler`),
  `zyanya-explorer/src/client.rs` ~960, ~1110 (`parse_u64_key(...).unwrap_or(1)`),
  ~1321 (`token_in.parse::<u64>().unwrap_or(0)`)
- **What to verify**: invalid hex/decimal keys silently become `0` (or `1`), so a typo'd
  address/key queries or mutates the wrong storage slot instead of erroring.
- **Fix direction**: propagate parse errors (return 400) instead of `unwrap_or`.

### 9. Jump-target validation / byte-offset vs opcode-index confusion — HIGH

- **File**: `zyanya-vm/src/opcode.rs` `deserialize_slice` ~150–200 (remap loop)
- **What to verify**: jump targets are serialized as byte offsets, then remapped to opcode
  indices only if the offset exactly matches a recorded opcode start. A target pointing
  into the middle of a multi-byte opcode (e.g. inside a `PUSH` payload) is left as a raw
  byte offset and later compared against `code.len()` (opcode count) in the VM — either
  rejected or, worse, jumping to an unintended opcode index.
- **Fix direction**: validate that every jump target resolves to a valid opcode boundary
  at deserialization time; reject otherwise.

### 10. ZCL compiler: source injection & parser recursion DoS — MEDIUM

- **Files**: `zyanya-vm/src/compiler/lexer.rs`, `parser.rs`, `codegen.rs`
- **What to verify**:
  - `codegen.rs` `call` builtin (~150–170): `Expression::Variable` address is emitted
    verbatim as `CALL <name>` and fails assembly; `Expression::Number` is formatted as a
    64-hex address. Confirm no injection path from ZCL source into assembly labels/opcodes.
  - `parser.rs` is recursive-descent with no depth limit; deeply nested expressions
    (`((((...))))`) can overflow the stack. The explorer `/api/compile` endpoint
    (`api.rs` `api_compile_contract_handler`) is **not** gated by `check_write_enabled`
    and accepts arbitrary source → remote DoS.
- **Fix direction**: add recursion/depth limits in the parser; bound input size on the
  compile endpoint.

### 11. Gas metering saturation & refund semantics — MEDIUM

- **File**: `zyanya-vm/src/gas.rs` (`consume` saturating_add, `refund` saturating_sub)
- **What to verify**: `consume` saturates on overflow (safe-ish), but the CALL path
  (`vm.rs` ~240, ~267–268) consumes `forward_gas` up front and refunds only on success;
  a failing child call burns the full `forward_gas` (griefing). Confirm no path lets a
  contract mint/refund more gas than consumed.
- **Fix direction**: document/limit forward-gas; consider refunding on failure too.

### 12. Stack overflow/underflow — LOW (verify)

- **File**: `zyanya-vm/src/stack.rs`
- **What to verify**: `push`/`pop`/`dup`/`swap` are bounds-checked and `MAX_STACK_DEPTH`
  is 1024. Confirm no unchecked indexing. Note the cross-call stack is **not** bounded
  (see finding 4).

### 13. Assembler label resolution — LOW

- **File**: `zyanya-vm/src/assembler.rs`
- **What to verify**: labels are byte offsets; numeric `JUMP` targets are parsed as byte
  offsets (`resolve_target` ~230). A label on the same line as an opcode
  (`:loop PUSH 1`) is mis-parsed as a label name containing the opcode. Confirm no way to
  smuggle an out-of-bounds target past `opcode_line_byte_size`.

### 14. Wallet key management — MEDIUM

- **File**: `zyanya-wallet/src/key_management.rs`
- **What to verify**: `save_to_file` sets 0600 (good); `from_secret_hex` uses
  `trim_start_matches("0x")` (strips repeated prefix); `load_from_file` reads whole file.
  Confirm no plaintext key leakage in logs/history and that `sign_transaction` verifies
  before returning.

### 15. Wallet ops: UTXO selection & holder_u64 collision — HIGH

- **File**: `zyanya-wallet/src/wallet_ops.rs`
- **What to verify**:
  - `holder_u64` (~200) derives a u64 from the first 8 bytes of the address payload —
    two distinct addresses sharing those 8 bytes map to the same token holder → balance
    collision / theft.
  - UTXO selection `selected_amount += entry.amount` (~150) is unchecked addition
    (bounded by MAX_SOMPI in practice, but confirm).
  - `send_zyan` change/fee math and coinbase-maturity skip logic.

### 16. tx_validation_in_isolation: payload validation gaps — MEDIUM

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`
  `check_contract_payload_in_isolation` ~180–230
- **What to verify**: deploy/invoke checks cover `max_gas != 0`, `gas_price != 0`,
  `tx.gas == max_gas`, non-empty bytecode — but **no** limit on `bytecode` size,
  `parameters` length, or `deposit_amount` bounds. A tx with a huge bytecode or millions
  of parameters is a memory/CPU DoS. Also note the copy-paste bug in
  `check_transaction_outputs_count` (~60) reporting `tx.inputs.len()` / `max_tx_inputs`
  instead of outputs.
- **Fix direction**: add size/length bounds; fix the error fields.

### 17. Unsafe block in `ContractStorageKey::as_ref` — LOW

- **File**: `consensus/src/model/stores/contract.rs` ~38–46
- **What to verify**: `#[repr(C)]` struct cast to `&[u8]` via `from_raw_parts` with
  `size_of::<Self>()` (40 bytes). Layout is currently correct (32-byte array + u64, no
  padding), but it is fragile UB if the struct changes. Confirm no padding and document.

---

## Execution steps for the builder

1. Create `audit_reports/` if missing.
2. Work through the files in the priority order above. For each file:
   - Read the full file (use `offset`/`limit` for large files).
   - Verify every finding's `file:line` against the current tree (line numbers above are
     from the audit snapshot and may drift — re-grep to confirm).
   - Reproduce the trigger mentally or with a minimal test where feasible.
   - Record the finding in the report format (severity, file:line, description, snippet, fix).
3. Add any additional findings discovered in the listed categories (integer overflow,
   reentrancy, key handling, consensus bypass, DoS, input validation/deserialization,
   unsafe blocks, crypto correctness).
4. Sort the final report by severity (CRITICAL → HIGH → MEDIUM → LOW), then by file path.
5. Do **not** modify source code — this phase is audit + report only.

## Report skeleton

```markdown
# Phase 1 Findings — Custom Zyanya Code

## CRITICAL
### [C-01] <title>
- **File**: <path>:<line>
- **Description**: ...
- **Code**:
  ```rust
  ...
  ```
- **Recommended fix**: ...

## HIGH
### [H-01] ...

## MEDIUM
### [M-01] ...

## LOW
### [L-01] ...
```

Use stable IDs (`C-01`, `H-01`, ...) so later phases can reference them.
