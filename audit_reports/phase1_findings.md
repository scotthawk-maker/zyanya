# Phase 1 Findings — Custom Zyanya Code

Deep security audit of the 15 custom Zyanya modules listed in the Phase 1 plan.
All `file:line` references were re-verified against the current source tree.
Source code was **not modified** — this phase is audit + report only.

Findings are sorted by severity (CRITICAL → HIGH → MEDIUM → LOW), then by file path.
Stable IDs (`C-01`, `H-01`, …) are used so later phases can reference them.

---

## CRITICAL

### [C-01] RPC fallback bypasses consensus, persisting contract state without a mined transaction

- **File**: `rpc/service/src/service.rs:614-635` (deploy) and `:676-716` (invoke)
- **Description**: Both `deploy_contract_call` and `invoke_contract_call` first try to
  submit the transaction to the mempool via `submit_rpc_transaction`. If that call
  returns **any** error (mempool full, validation failure, network glitch, or simply a
  tx with no inputs that the mempool rejects), the handler falls back to executing the
  contract directly with `ContractProcessor::process_contract_tx` against a freshly
  constructed `ContractStateCache` and then **writes the resulting state to the durable
  consensus store** via `session.write_contract_code / write_contract_storage /
  write_contract_balance`. This means:
  - State is mutated and persisted **without** a mined/accepted transaction, without
    GhostDAG ordering, and without consensus validation.
  - There is **no authentication** on the RPC surface — any RPC client can trigger
    this path by submitting a tx that the mempool rejects (trivial: deploy/invoke txs
    built here have `inputs = []` and `outputs = []`, which the mempool routinely
    rejects for non-coinbase txs, making the fallback the *primary* path in practice).
  - The fallback for `invoke` even seeds `cache.fallback_storage` /
    `fallback_balance` from the live session, so it reads + writes real chain state.
  This is a direct consensus-bypass / unauthenticated-state-mutation primitive.
- **Code** (`service.rs:614-635`):
  ```rust
  let submit_res = self.flow_context.submit_rpc_transaction(&session, tx.clone(), Orphan::Forbidden).await;
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
- **Recommended fix**: Remove the fallback entirely; return the mempool/RPC error to the
  caller. If a read-only simulation is desired, use a discarded cache as
  `call_contract_call` (`:749+`) already does — never persist. Any state-changing
  operation must go through the transaction pool and consensus.

---

### [C-02] Failed contract transactions still commit state (commit-on-failure)

- **File**: `consensus/src/pipeline/virtual_processor/processor.rs:511-521` and
  `consensus/src/model/stores/contract.rs:213` (Deploy) / `:281` (Invoke)
- **Description**: In the virtual processor, every contract tx is passed to
  `ContractProcessor::process_contract_tx`, and the caller commits the cache whenever the
  return value `.is_some()` — **regardless of `outcome.success`**:
  ```rust
  // processor.rs:511
  if processor.process_contract_tx(tx, &mut contract_cache).is_some() {
      has_contract_changes = true;
  }
  // processor.rs:520
  if has_contract_changes {
      self.contract_store.commit_cache_batch(&mut batch, &contract_cache).unwrap();
  }
  ```
  `process_contract_tx` returns `Some(outcome)` even when `success == false`, so a
  **failed** contract tx persists all cache mutations made before/during execution.
  Compounding this:
  - **Deploy branch** (`contract.rs:213`): `cache.code.insert(addr_bytes, deploy.bytecode)`
    happens *before* any execution attempt. A deploy with invalid bytecode (or one that
    fails later) still permanently installs the contract code.
  - **Invoke branch** (`contract.rs:281-283`): `deposit_amount` is credited to
    `cache.balances` *before* execution. A failed invoke still credits the deposit,
    meaning the caller's ZYAN is created out of thin air for the contract.
  - Only the Invoke **success** path uses a `temp_cache` (`:310`) and swaps it in on
    success (`:356`); the Deploy branch has no such guard, and the pre-execution
    mutations in both branches are on the live `cache`, not `temp_cache`.
  Result: failed deploys install bytecode; failed invokes mint balance to contracts;
  all of it is committed by the virtual processor.
- **Code** (`contract.rs:213-218`):
  ```rust
  cache.code.insert(addr_bytes, deploy.bytecode.clone());
  if deploy.deposit_amount > 0 {
      let current_balance = cache.get_balance(&addr_bytes);
      cache.balances.insert(addr_bytes, current_balance.saturating_add(deploy.deposit_amount));
  }
  ```
- **Recommended fix**: (1) In `processor.rs`, commit only when
  `outcome.map(|o| o.success).unwrap_or(false)`. (2) In `process_contract_tx`, perform
  *all* cache mutations on a `temp_cache` and swap into the real `cache` only on
  `success == true` — mirror the existing Invoke `temp_cache` pattern in the Deploy
  branch and move the `deposit_amount` credit to after a successful execution.

---

### [C-03] Bonding-curve custody accounting gap: buy does not deduct ZYAN, sell destroys refund

- **File**: `consensus/src/model/stores/contract.rs:213-217` (deploy deposit credit),
  `:281-283` (invoke deposit credit), `:320-333` (buy), `:335-353` (sell)
- **Description**: The consensus layer models contract ZYAN custody via a
  `cache.balances` map, but the accounting is broken in both directions:
  - **Buy (entry_point 4, `:320-333`)**: The contract code computes `cost` and adds it
    to `reserve` (storage key 2), and the processor checks
    `deposit_amount >= cost` (`:322`). However, the `deposit_amount` was already
    *credited* to `cache.balances` at `:281-283` before execution, and **nothing ever
    deducts `cost` from the contract balance**. The buyer's ZYAN is never escrowed —
    `deposit_amount` is a free-form payload field, not a UTXO output locked to the
    contract — so the contract "reserve" grows while no real ZYAN is held. A buyer can
    set `deposit_amount = 0` and still mint tokens (the check at `:322` only rejects
    when `deposit_amount > 0 && deposit_amount < cost`, i.e. `deposit_amount == 0`
    *passes*).
  - **Sell (entry_point 5, `:335-353`)**: The processor deducts `refund` from
    `temp_cache.balances` (`:352`), but **no UTXO output is created paying the seller**.
    The seller's refund is simply destroyed from the contract balance — the seller
    receives nothing, and the ZYAN vanishes.
  There is no reconciliation between `balances` and `reserve`, and no real UTXO value
  transfer on either side. The entire bonding-curve economy is uncollateralized.
- **Code** (`contract.rs:281-283`):
  ```rust
  if invoke.deposit_amount > 0 {
      let current_balance = cache.get_balance(&addr_bytes);
      cache.balances.insert(addr_bytes, current_balance.saturating_add(invoke.deposit_amount));
  }
  ```
  (`contract.rs:320-333` — buy checks `deposit_amount >= cost` but never deducts `cost`;
  `:335-353` — sell deducts `refund` from `balances` but creates no payout UTXO.)
- **Recommended fix**: Model custody as real UTXO value transfer: the invoke tx must
  include an output locking `cost` sompi to the contract address on buy, and the block
  processor must create an output paying `refund` to the seller on sell. Reconcile
  `cache.balances` with `reserve` (key 2) on every buy/sell. Reject buy when
  `deposit_amount < cost` *including* the `deposit_amount == 0` case.

---

### [C-04] No `msg.sender` / no authentication in token, bonding-curve, and DEX contracts

- **Files**: `bonding_curve.zcl:18-26` (transfer), `:40-52` (buy), `:54-72` (sell);
  `dex.zcl:17-36` (addLiquidity), `:49-66` (swap), `:74-97` (removeLiquidity);
  `zyanya-vm/src/token.rs` (transfer `:80-100`, mint `:115-130`);
  `zyanya-vm/src/bonding_curve_token.rs` (transfer, buy, sell)
- **Description**: The VM has no notion of an authenticated caller (`msg.sender`).
  Every function that should be sender-gated takes the "caller" / "from" as a
  **caller-supplied parameter** with no check that the transaction signer owns that
  identity:
  - `transfer(from, to, amount)` — anyone can pass any `from` and drain that holder's
    entire balance. This is a direct theft primitive.
  - `buy(caller, tokens_to_mint)` and `sell(caller, tokens_in)` — the caller is a
    parameter; anyone can mint tokens to / sell tokens from any address.
  - `mint` in `token.rs` is callable by anyone at any time (entry point 3), with no
    owner check — unlimited inflation.
  - `init` (entry point 0) in the bonding curve is callable by anyone at any time,
    resetting the slope and zeroing supply/reserve — a griefing/destroy primitive.
  - DEX `addLiquidity(caller, …)`, `removeLiquidity(caller, lpAmount)` — caller is a
    parameter; anyone can claim/remove another user's LP tokens.
  The consensus layer passes only `entry_point` and `parameters` to the VM
  (`contract.rs:307-309`); the transaction signer is never propagated as an
  authenticated identity.
- **Code** (`bonding_curve.zcl:18-26`):
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
- **Recommended fix**: Introduce an authenticated caller identity in the VM (derive
  `msg.sender` from the transaction signer and expose it via a new opcode or a reserved
  stack slot). Enforce `from == msg.sender` in `transfer`, `caller == msg.sender` in
  buy/sell/addLiquidity/removeLiquidity, and gate `mint`/`init` to the contract owner.

---

### [C-05] VM `CALL` opcode has no reentrancy guard and no call-depth limit

- **File**: `zyanya-vm/src/vm.rs:236-272`
- **Description**: `OpCode::Call` spawns a child `VM` that shares the **same `&mut S
  StateBackend`** as the caller (`:265`):
  ```rust
  match child_vm.execute_stateful(&target_opcodes, target_addr, state) {
  ```
  Two critical problems:
  1. **No call-depth limit**: A contract can `CALL` itself (or a mutually-recursive
     pair) indefinitely. Each `CALL` recurses into `execute_stateful`, consuming a Rust
     stack frame. Deep recursion → Rust stack overflow → **node process crash (DoS)**.
     Gas does bound *work*, but the gas is forwarded to the child (`:240`), so a single
     tx with large `max_gas` can recurse thousands of frames before running out of gas.
  2. **No reentrancy guard**: The child shares the same `state` backend, so a callee
     can re-enter the caller mid-execution and mutate shared storage. This is the
     classic reentrancy pattern exploitable against the bonding-curve and DEX contracts
     (which read-then-write storage without locks). Combined with [C-04] (no auth), a
     malicious contract called during a buy/sell/swap can manipulate reserves.
- **Code** (`vm.rs:236-272`):
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
- **Recommended fix**: Add a call-depth counter (threaded through `execute_stateful` or
  stored on the VM) and reject `CALL` beyond a limit (e.g. 1024). Add a per-contract
  reentrancy guard ("entered" flag in state) and reject re-entry. Consider giving the
  child a *copy* of state for read-only calls, or a journal that commits atomically on
  success.

---

## HIGH

### [H-01] `POW` opcode uses `wrapping_pow` — silent overflow in financial math

- **File**: `zyanya-vm/src/vm.rs:128-131`
- **Description**: `Add`, `Sub`, and `Mul` all use `checked_*` and return
  `VMError::ArithmeticOverflow` on overflow. `Pow` uses `a.wrapping_pow(b as u32)`,
  which **silently wraps** modulo `2^64`. A contract can produce a wrapped result that
  flows into bonding-curve/DEX math without any error, yielding incorrect (and
  exploitable) financial values. The exponent is also cast `b as u32` (truncating a u64
  exponent) and the extra gas `1 + (b as u64 / 32)` is computed on the *truncated*
  value used for gas, while the *full* `b` is used for the cast — minor inconsistency.
- **Code** (`vm.rs:128-131`):
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
- **Recommended fix**: Use `a.checked_pow(b as u32)` and return
  `VMError::ArithmeticOverflow` on `None`. Reject exponents that exceed `u32::MAX` (or
  cap and charge gas) instead of silently truncating.

---

### [H-02] DEX constant-product math overflows u64 on-chain (griefing/DoS) and off-chain (wrong quotes)

- **Files**: `dex.zcl:55` (`resB * amountIn * 997`), `:62` (`resA * amountIn * 997`);
  `bonding_curve.zcl:44` (`slope * (2 * supply * k + k * k)`),
  `:62` (`slope * (2 * supply * k - k * k)`);
  off-chain mirror in `zyanya-explorer/src/client.rs:897` and `:1045`
- **Description**:
  - **On-chain**: The VM `MUL` is checked (`vm.rs:99-104`), so overflow *reverts* the
    transaction. But the DEX formula `resB * amountIn * 997` overflows u64 for realistic
    reserves: e.g. `resB = 10^9`, `amountIn = 10^9`, `997` → `~10^21 ≫ u64::MAX
    (1.8·10^19)`. Any swap against a moderately-sized pool reverts, making the DEX
    unusable (griefing/DoS). The bonding-curve `2 * supply * k + k * k` overflows for
    large supplies/k as well.
  - **Off-chain (`client.rs:897`)**: `let cost = slope.saturating_mul(2 * S * k + k *
    k) / 2;` — here `2 * S * k` and `k * k` are **plain u64 arithmetic** (not
    `saturating_`/`checked_`), so they silently wrap in release builds. The
    `saturating_mul` only guards the outer `slope *` product. A user can be quoted a
    wildly wrong `cost` (or `refund` at `:1045`), then sign a tx that the node rejects
    or that charges a different amount — funds mispriced.
- **Code** (`client.rs:897`):
  ```rust
  let cost = slope.saturating_mul(2 * S * k + k * k) / 2;
  ```
  (`client.rs:1045`): `let refund = if S >= k { slope.saturating_mul(2 * S * k - k * k) / 2 } else { 0 };`
  (`dex.zcl:55`): `let num = resB * amountIn * 997;`
- **Recommended fix**: Use `u128` intermediates (or `checked_*` math) in both the ZCL
  contracts and the explorer client. In the VM, consider a `MUL128`/wide-mul opcode or
  reorder the AMM formula to divide before multiplying (`(resB * amountIn / den) * 997`
  with rounding) to keep intermediates within u64. In `client.rs`, compute `2*S*k` and
  `k*k` as `u128` and reject overflow explicitly.

---

### [H-03] Jump-target validation: byte-offset vs opcode-index confusion allows mid-opcode targets

- **File**: `zyanya-vm/src/opcode.rs:266-278` (remap loop) and `zyanya-vm/src/vm.rs:146-163`
- **Description**: Jump targets are serialized as **byte offsets** (`opcode.rs:222`,
  `:230`). During deserialization, a `byte_to_opcode` map records the byte offset of
  each opcode start (`:187`). The remap loop (`:266-278`) converts a `Jump`/`JumpIf`
  target to an opcode index **only if** the byte offset exactly matches a recorded
  opcode start. If a target points **into the middle of a multi-byte opcode** (e.g.
  inside a `PUSH` 8-byte payload or a `CALL` 32-byte address), it is *left as a raw
  byte offset* and never rejected. Later, the VM (`vm.rs:150-152`) checks
  `*target >= code.len()` where `code.len()` is the **opcode count**, not the byte
  length — so a raw byte offset (which can be larger than the opcode count) is
  rejected as `InvalidJumpTarget`, *or*, if the byte offset happens to be ≤ opcode
  count, it is treated as a valid opcode index and **jumps to an unrelated opcode**.
  This is a deserialization-safety / control-flow integrity bug.
- **Code** (`opcode.rs:266-278`):
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
- **Recommended fix**: After the remap loop, validate that every `Jump`/`JumpIf` target
  resolved to a known opcode boundary; return `VMError::InvalidJumpTarget` (or a new
  `DeserializationError`) for any target that did not resolve. Alternatively, serialize
  jump targets as opcode indices directly.

---

### [H-04] Explorer `/api/compile` endpoint is not write-gated and accepts arbitrary ZCL source (parser-recursion DoS)

- **Files**: `zyanya-explorer/src/api.rs:587-595` (`api_compile_contract_handler`),
  `zyanya-vm/src/compiler/parser.rs` (recursive-descent, no depth limit),
  `zyanya-vm/src/compiler/codegen.rs:224-241` (`call` builtin)
- **Description**: `api_compile_contract_handler` does **not** call
  `check_write_enabled()`, so it is open on all deployments including public explorers.
  It accepts arbitrary `source` text and runs the full ZCL pipeline (lexer → parser →
  codegen → assembler). The parser is recursive-descent
  (`parse_expression → parse_equality → … → parse_primary`) with **no recursion depth
  limit**; deeply nested expressions like `(((((((…)))))))` recurse one frame per
  nesting level and can overflow the Rust stack → **remote DoS crash**. Separately, the
  `call` builtin in codegen (`codegen.rs:235-238`) takes an `Expression::Variable` as
  the address and emits `CALL <name>` verbatim, which the assembler will reject as an
  invalid address — no injection path into opcodes was found, but the unbounded
  recursion is the primary risk.
- **Code** (`api.rs:587-595`):
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
- **Recommended fix**: Gate the compile endpoint behind `check_write_enabled()` (or a
  separate compile-enabled flag). Add a recursion-depth counter in the parser and
  reject beyond a limit (e.g. 256). Bound the input `source` length (e.g. 64 KiB).
  Consider running compilation in a `std::thread` with a fixed stack size and a timeout.

---

### [H-05] Wallet `holder_u64` derives a u64 from the first 8 bytes of the address — balance collision / theft

- **File**: `zyanya-wallet/src/wallet_ops.rs:279-285`
- **Description**: `holder_u64` reduces a 32-byte address to a u64 by reading its first
  8 bytes (`u64::from_le_bytes(address.payload[0..8])`). Two distinct addresses that
  share their first 8 payload bytes map to the **same storage key** in the token
  contract. Since the token contract has no authentication ([C-04]) and uses
  `sload(holder)/sstore(holder, …)`, a collision means two different users share one
  balance slot — an attacker who finds or crafts a colliding address can spend another
  user's tokens. Even without malice, legitimate users can silently merge balances.
  The 8-byte (64-bit) space means collisions become likely (birthday bound) at ~2^32
  addresses — feasible on a busy chain.
- **Code** (`wallet_ops.rs:279-285`):
  ```rust
  pub fn holder_u64(address: &Address) -> u64 {
      if address.payload.len() >= 8 {
          u64::from_le_bytes(address.payload[0..8].try_into().unwrap_or([0u8; 8]))
      } else {
          1
      }
  }
  ```
- **Recommended fix**: Use the full address (or a hash of it) as the storage key. If
  the VM storage key must be u64, derive it as `u64::from_be_bytes(hash(address)[0..8])`
  and document the collision risk, or switch the storage key type to `[u8; 32]`.

---

### [H-06] UTXO selection uses unchecked addition for `total_balance` / `selected_amount`

- **File**: `zyanya-wallet/src/wallet_ops.rs:158` and `:204`
- **Description**: In `get_zyan_balance`, `total_balance += entry.utxo_entry.amount`
  (`:158`) is plain `u64` addition with no overflow check. In `send_zyan`,
  `selected_amount += entry.amount` (`:204`) is likewise unchecked. While individual
  UTXO amounts are bounded by `MAX_SOMPI`, a wallet with many UTXOs whose sum exceeds
  `u64::MAX` would silently wrap, causing `total_balance < required` to be computed
  incorrectly — potentially selecting too few UTXOs and producing an invalid tx, or
  computing a wrong change amount. In practice the total supply is below `u64::MAX`,
  so this is lower-likelihood than the other findings, but it is still undefined
  behavior (wrapping in release) and should be defensive.
- **Code** (`wallet_ops.rs:158`):
  ```rust
  total_balance += entry.utxo_entry.amount;
  ```
- **Recommended fix**: Use `total_balance.checked_add(entry.utxo_entry.amount)` and
  handle overflow (cap at `u64::MAX` or return an error). Apply the same to
  `selected_amount`.

---

## MEDIUM

### [M-01] Explorer silently coerces invalid storage keys / addresses to 0 or 1

- **Files**: `zyanya-explorer/src/api.rs:155-159` (`api_contract_state_handler`),
  `zyanya-explorer/src/client.rs:960` (`parse_u64_key(...).unwrap_or(1)` — buy),
  `:1110` (`unwrap_or(1)` — sell), `:1321` (`token_in.parse::<u64>().unwrap_or(0)`)
- **Description**: In `api_contract_state_handler`, an invalid hex/decimal `key` query
  param silently becomes `0` via `unwrap_or(0)`, so a typo'd key queries (and in write
  paths, mutates) **storage slot 0** — which in the token/bonding-curve contracts holds
  *total supply*, and in the DEX holds *reserveA*. In `client.rs`, the buy/sell
  builders coerce the user's address-derived key to `1` on parse failure (`:960`,
  `:1110`), so a malformed address maps to holder `1` (often the owner) — a
  misattribution. The DEX `swap_on_dex` coerces an unknown `token_in` string to `0`
  (`:1321`), silently swapping the wrong side. All of these mask errors instead of
  returning 400.
- **Code** (`api.rs:155-159`):
  ```rust
  let key_val = query.key.as_deref().map(|k| {
      let clean = k.trim();
      if let Some(rest) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
          u64::from_str_radix(rest, 16).unwrap_or(0)
      } else {
          clean.parse::<u64>().unwrap_or(0)
      }
  }).unwrap_or(0);
  ```
  (`client.rs:960`): `let buyer_u64 = parse_u64_key(&user_address.to_string()).unwrap_or(1);`
- **Recommended fix**: Propagate parse errors — return `400 Bad Request` with a clear
  message instead of `unwrap_or(0)`/`unwrap_or(1)`. For buy/sell, require a valid
  address and fail closed.

---

### [M-02] Gas metering: `consume` saturates (can mask overflow), and failing child `CALL` burns all forwarded gas (griefing)

- **File**: `zyanya-vm/src/gas.rs:28-36` (`consume`), `:56-58` (`refund`);
  `zyanya-vm/src/vm.rs:240` (forward-gas consume), `:267-270` (refund only on success)
- **Description**:
  - `consume` uses `saturating_add` (`gas.rs:30`). If `used_gas + amount` overflows
    u64, it saturates to `u64::MAX` and then the `> gas_limit` check trips with
    `requested = u64::MAX` — the error message is misleading but the tx does revert, so
    this is safe-ish. However, `refund` (`:56`) uses `saturating_sub`, so a bug that
    refunds more than `used_gas` would silently clamp to 0 rather than underflow —
    masking a logic error where a contract could net-negative gas.
  - In the `CALL` path (`vm.rs:240`), `forward_gas` is consumed up front. On child
    success, the unused gas is refunded (`:267-268`). On child **failure** (`:269`),
    **no refund** is given — the entire `forward_gas` is burned. A contract that calls
    a gas-guzzling/always-failing callee can be used to burn a victim's gas (griefing),
    and more importantly there is no check that `forward_gas <= remaining_gas` —
    `consume(forward_gas)` will simply `OutOfGas` if it exceeds the limit, which is
    fine, but the all-or-nothing burn on failure is asymmetric and harsh.
- **Code** (`vm.rs:265-270`):
  ```rust
  match child_vm.execute_stateful(&target_opcodes, target_addr, state) {
      Ok(res) => {
          let unused = child_vm.gas_meter.gas_limit().saturating_sub(child_vm.gas_meter.used_gas());
          self.gas_meter.refund(unused);
          self.stack.push(res.return_value.unwrap_or(0))?;
      }
      Err(_) => { self.stack.push(0)?; }   // no refund — all forward_gas burned
  }
  ```
- **Recommended fix**: Consider refunding unused gas on failure too (the child's
  `used_gas` is still consumed; only the *unused* portion should be refunded). Document
  the forward-gas semantics. Replace `saturating_add` in `consume` with `checked_add`
  and return `OutOfGas` on overflow for a clearer error.

---

### [M-03] ZCL parser is recursive-descent with no depth limit (DoS via deep nesting)

- **File**: `zyanya-vm/src/compiler/parser.rs:143-262` (expression parsers)
- **Description**: `parse_expression → parse_equality → parse_relational →
  parse_additive → parse_multiplicative → parse_primary` recurses one Rust frame per
  precedence level, and `parse_primary` handles `LParen → parse_expression` (`:244`),
  so each parenthesized sub-expression adds 6 frames. Deeply nested input
  (`((((…))))`) overflows the Rust stack and crashes the process. When reached via the
  explorer `/api/compile` endpoint ([H-04], unauthenticated), this is a remote DoS.
- **Code** (`parser.rs:243-247`):
  ```rust
  TokenKind::LParen => {
      self.advance();
      let expr = self.parse_expression()?;
      self.expect_kind(&TokenKind::RParen, ") after nested expression")?;
      Ok(expr)
  }
  ```
- **Recommended fix**: Thread a `depth` counter through the parse functions and return
  `ParserError` beyond a limit (e.g. 256). Alternatively, run the parser in a
  `Builder`-spawned thread with a fixed (e.g. 8 MiB) stack.

---

### [M-04] Wallet `from_secret_hex` uses `trim_start_matches("0x")` which strips repeated prefixes

- **File**: `zyanya-wallet/src/key_management.rs:96`
- **Description**: `hex_str.trim().trim_start_matches("0x")` — `trim_start_matches`
  strips **all** leading occurrences of the pattern `"0x"`, not just one. An input like
  `"0x0x<64hex>"` (or a pasted key with a stray prefix) becomes `"<64hex>"` silently,
  and `"0x0x0x..."` strips three times. More subtly, a key starting with bytes that
  form `"0x"` after trim (unlikely but possible with leading zeros in a malformed
  paste) could be over-stripped, yielding a different 32-byte secret than intended — a
  user restores the wrong key and loses access to funds. The intent was almost
  certainly `strip_prefix("0x")`.
- **Code** (`key_management.rs:96`):
  ```rust
  let clean = hex_str.trim().trim_start_matches("0x");
  ```
- **Recommended fix**: Use `hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim())`
  to strip at most one prefix. Reject input whose length after stripping is not 64 hex
  chars.

---

### [M-05] `tx_validation_in_isolation` has no size/length bounds on contract bytecode or parameters (memory/CPU DoS)

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:173-203`
  (`check_contract_payload_in_isolation`)
- **Description**: The deploy branch checks `bytecode.is_empty()`, `max_gas != 0`,
  `gas_price != 0`, and `tx.gas == max_gas`, but imposes **no upper bound** on
  `bytecode` size or `parameters` length. An attacker can submit a deploy with a
  multi-megabyte bytecode (or an invoke with millions of parameters) that passes
  isolation validation, then forces the node to deserialize and (attempt to) execute
  it — a memory/CPU DoS, especially in the mempool and during block validation. The
  invoke branch has the same gap for `parameters`.
- **Code** (`tx_validation_in_isolation.rs:183-193`):
  ```rust
  zyanya_consensus_core::tx::ContractPayload::Deploy(deploy) => {
      if deploy.bytecode.is_empty() { return Err(...); }
      if deploy.max_gas == 0 { return Err(...); }
      if deploy.gas_price == 0 { return Err(...); }
      if tx.gas != deploy.max_gas { return Err(...); }
      // no size bound on deploy.bytecode
  }
  ```
- **Recommended fix**: Add `MAX_CONTRACT_BYTECODE_SIZE` and
  `MAX_CONTRACT_PARAMETERS` constants and reject txs exceeding them. Bound
  `deposit_amount` to `MAX_SOMPI`.

---

## LOW

### [L-01] `check_transaction_outputs_count` reports `inputs.len()` / `max_tx_inputs` instead of outputs fields (copy-paste bug)

- **File**: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:59-64`
- **Description**: When the output count exceeds the limit, the error is constructed
  with `tx.inputs.len()` and `self.max_tx_inputs` — the **input** count and input
  limit — instead of `tx.outputs.len()` and `self.max_tx_outputs`. The check itself is
  correct (it compares `tx.outputs.len() > self.max_tx_outputs`), but the error
  payload is wrong, which misleads diagnostics and any tooling that surfaces the error.
- **Code** (`tx_validation_in_isolation.rs:59-64`):
  ```rust
  fn check_transaction_outputs_count(&self, tx: &Transaction) -> TxResult<()> {
      if tx.outputs.len() > self.max_tx_outputs {
          return Err(TxRuleError::TooManyOutputs(tx.inputs.len(), self.max_tx_inputs));
      }
      Ok(())
  }
  ```
- **Recommended fix**: `Err(TxRuleError::TooManyOutputs(tx.outputs.len(), self.max_tx_outputs))`.

---

### [L-02] `unsafe` `AsRef<[u8]>` for `#[repr(C)] ContractStorageKey` is fragile UB

- **File**: `consensus/src/model/stores/contract.rs:22-40`
- **Description**: `ContractStorageKey` is `#[repr(C)]` with a `[u8; 32]` + `u64`. The
  `AsRef` impl casts `&self` to `&[u8]` via `from_raw_parts(self as *const Self as
  *const u8, size_of::<Self>())`. The layout is currently correct (32-byte array
  followed by 8-byte u64, naturally aligned, no padding → 40 bytes), but this is
  **undefined behavior** if a field is added/reordered such that padding is inserted —
  the raw byte slice would include padding bytes and the RocksDB key would change
  semantics silently. It is also technically UB to cast a `&repr(C) struct` to `&[u8]`
  without going through a pointer provenance chain in all cases.
- **Code** (`contract.rs:36-40`):
  ```rust
  fn as_ref(&self) -> &[u8] {
      unsafe {
          std::slice::from_raw_parts(
              self as *const Self as *const u8,
              std::mem::size_of::<Self>(),
          )
      }
  }
  ```
- **Recommended fix**: Replace the `unsafe` cast with a safe, explicit serialization:
  `[&self.contract_address, &self.key.to_le_bytes()].concat()` or
  `self.contract_address.iter().chain(self.key.to_le_bytes()).copied().collect()`. If
  the raw layout is required for RocksDB key compatibility, add a compile-time
  assertion (`assert_eq!(size_of::<ContractStorageKey>(), 40)`,
  `assert_eq!(offset_of!(…), …)`) and a comment documenting the invariant.

---

### [L-03] Assembler: label-on-same-line-as-opcode is mis-parsed; numeric `JUMP` targets are raw byte offsets

- **File**: `zyanya-vm/src/assembler.rs:87-96` (label detection), `:232-238`
  (`resolve_target`)
- **Description**:
  - A line like `:loop PUSH 1` is detected as a label because `trimmed.starts_with(':')`
    is true, and `trim_matches(':')` yields `"loop PUSH 1"` — the opcode is swallowed
    into the label name and silently dropped (the `PUSH 1` is lost). This is a
    footgun, though not directly exploitable.
  - `resolve_target` (`:232-238`) falls back to `parse_u64(arg)` for numeric jump
    targets and treats the result as a **byte offset** (since labels are byte offsets
    in pass 1). This is then stored in `OpCode::Jump(usize)`, which the VM interprets
    as an **opcode index** after `deserialize_slice` remaps label-based targets. A
    hand-written numeric `JUMP 9` intending "opcode index 9" is actually treated as
    "byte offset 9" and, if it coincides with a `PUSH` payload byte, behaves
    incorrectly. The two interpretations are inconsistent.
- **Code** (`assembler.rs:232-238`):
  ```rust
  fn resolve_target(arg: &str, labels: &HashMap<String, usize>, line_num: usize) -> Result<usize, AssemblerError> {
      let clean_arg = arg.trim_start_matches(':');
      if let Some(&target) = labels.get(clean_arg) { Ok(target) }
      else if let Ok(val) = parse_u64(arg) { Ok(val as usize) }
      else { Err(AssemblerError::UndefinedLabel(arg.to_string(), line_num)) }
  }
  ```
- **Recommended fix**: Reject label lines that contain additional tokens after the
  label name. Document that numeric `JUMP` targets are byte offsets (or switch them to
  opcode indices consistently). Add a test for `:label OPCODE` mis-parsing.

---

### [L-04] Stack bounds checking is correct, but cross-call stack depth is unbounded (see [C-05])

- **File**: `zyanya-vm/src/stack.rs:5` (`MAX_STACK_DEPTH = 1024`), `:37-42` (`push`),
  `:48-50` (`pop`), `:53-57` (`dup`), `:60-66` (`swap`)
- **Description**: The operand stack is correctly bounds-checked: `push` rejects at
  `max_depth` (`:37`), `pop` returns `StackUnderflow` on empty (`:49`),
  `dup`/`swap` require ≥1/≥2 elements. No unchecked indexing was found. The only gap is
  that **each `CALL` spawns a fresh `VM` with its own 1024-deep stack** (`vm.rs:260`),
  so the *per-call* stack is bounded, but the *call-chain* depth is not — that is
  covered by [C-05] (no call-depth limit). Recorded here for completeness.
- **Recommended fix**: No change needed to `stack.rs` itself; resolve via the
  call-depth limit in [C-05].

---

## Summary

| ID | Severity | File | One-line |
|----|----------|------|----------|
| C-01 | CRITICAL | `rpc/service/src/service.rs:614` | RPC fallback persists contract state without consensus |
| C-02 | CRITICAL | `consensus/.../processor.rs:511` | Failed contract txs still commit state |
| C-03 | CRITICAL | `consensus/.../contract.rs:281` | Bonding-curve custody: buy never deducts, sell destroys refund |
| C-04 | CRITICAL | `bonding_curve.zcl:18` | No `msg.sender` / no auth in token, bonding-curve, DEX |
| C-05 | CRITICAL | `zyanya-vm/src/vm.rs:236` | `CALL` has no reentrancy guard / no call-depth limit |
| H-01 | HIGH | `zyanya-vm/src/vm.rs:128` | `POW` uses `wrapping_pow` (silent overflow) |
| H-02 | HIGH | `dex.zcl:55` / `client.rs:897` | DEX/bonding-curve math overflows u64 |
| H-03 | HIGH | `zyanya-vm/src/opcode.rs:266` | Jump-target byte-offset vs opcode-index confusion |
| H-04 | HIGH | `zyanya-explorer/src/api.rs:587` | `/api/compile` unauthenticated + parser-recursion DoS |
| H-05 | HIGH | `zyanya-wallet/src/wallet_ops.rs:279` | `holder_u64` 8-byte truncation → balance collision/theft |
| H-06 | HIGH | `zyanya-wallet/src/wallet_ops.rs:158` | UTXO selection unchecked u64 addition |
| M-01 | MEDIUM | `zyanya-explorer/src/api.rs:155` | Invalid keys silently coerced to 0/1 |
| M-02 | MEDIUM | `zyanya-vm/src/gas.rs:28` | Gas saturating math + failing `CALL` burns all forwarded gas |
| M-03 | MEDIUM | `zyanya-vm/src/compiler/parser.rs:143` | Parser recursion unbounded (DoS) |
| M-04 | MEDIUM | `zyanya-wallet/src/key_management.rs:96` | `trim_start_matches("0x")` over-strips prefix |
| M-05 | MEDIUM | `consensus/.../tx_validation_in_isolation.rs:173` | No size bounds on bytecode/parameters (DoS) |
| L-01 | LOW | `consensus/.../tx_validation_in_isolation.rs:59` | Outputs-count error reports input fields (copy-paste) |
| L-02 | LOW | `consensus/.../contract.rs:36` | `unsafe` `AsRef` for `repr(C)` key is fragile UB |
| L-03 | LOW | `zyanya-vm/src/assembler.rs:87` | Label-on-same-line mis-parsed; numeric JUMP = byte offset |
| L-04 | LOW | `zyanya-vm/src/stack.rs:5` | Stack bounds OK; cross-call depth unbounded (see C-05) |