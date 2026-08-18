# Plan — PR #2 follow-up security fixes

Three follow-up items from the consolidated audit (`audit_reports/FINAL_AUDIT_REPORT.md`):

1. **F-C-03 FOLLOW-UP** — UTXO payout plumbing for smart-contract sells.
2. **F-C-13 FOLLOW-UP** — wRPC WebSocket bearer-token authentication.
3. **F-C-16 FOLLOW-UP** — Cryptographic metadata binding for token metadata.

All three are CRITICAL-severity findings. Implement in the order below (each is independent; order chosen so consensus changes land first and are easiest to `cargo check` in isolation).

---

## Fix 1 — F-C-03 FOLLOW-UP: UTXO payout plumbing for smart-contract sells

### Priority: CRITICAL

### Files
- `consensus/src/pipeline/virtual_processor/processor.rs`
- `consensus/src/model/stores/contract.rs`
- `consensus/core/src/tx.rs` (only if a new shared type is added; otherwise no change)

### Current state
- `ContractProcessor::process_contract_tx` (in `contract.rs`) handles `entry_point == 5` (sell) by **failing closed**: it returns `success: false` because no UTXO-output-creation plumbing exists for contract payouts.
- Buy (`entry_point == 4`) already deducts `cost` from the contract balance correctly.
- The virtual processor (`processor.rs::commit_utxo_state`) derives a `u64` caller identity from the first input's UTXO `script_public_key` (`derive_caller_from_script_pub_key`) but **discards the full script/address**, so there is no way to build a payout output today.

### Changes

#### A. `consensus/src/model/stores/contract.rs`
1. Add a payout field to the outcome so the processor can communicate a payout back to the virtual processor:
   ```rust
   pub struct ContractExecutionOutcome {
       // ... existing fields ...
       pub payout: Option<zyanya_consensus_core::tx::TransactionOutput>,
   }
   ```
   (`TransactionOutput { value, script_public_key }` already exists in `consensus/core/src/tx.rs`.)
2. Change `process_contract_tx` signature to also receive the caller's full script public key (needed to build the P2PKH/P2PK payout script):
   ```rust
   pub fn process_contract_tx(
       &self,
       tx: &Transaction,
       cache: &mut ContractStateCache,
       caller: u64,
       caller_script_public_key: Option<&zyanya_consensus_core::tx::ScriptPublicKey>,
   ) -> Option<ContractExecutionOutcome>
   ```
3. In the `Invoke` branch, replace the `entry_point == 5` fail-closed block with real payout logic:
   - The sell refund is the VM return value (`ret_val`).
   - Deduct `refund` from the contract balance (`temp_cache.balances`) so the contract's ZYAN custody decreases by the amount paid out (mirrors the buy-side `cost` deduction).
   - Build the payout output using the caller's address:
     ```rust
     let seller_address = zyanya_txscript::extract_script_pub_key_address(
         caller_script_public_key, zyanya_addresses::Prefix::Mainnet)?; // fail closed on Err
     let script_public_key = zyanya_txscript::pay_to_address_script(&seller_address);
     payout = Some(TransactionOutput::new(refund, script_public_key));
     ```
     Note: Zyanya's standard single-key output script is `pay_to_address_script` (P2PK for `Version::PubKey`); the prompt's "P2PKH" is loose terminology — use `pay_to_address_script`.
   - Set `payout` on the returned `ContractExecutionOutcome` and keep `success: true`.
   - If `caller_script_public_key` is `None` or address extraction fails, fail closed (`success: false`, no payout) — never pay to an unknown address.
4. Update the two `DeployContractPayload` test constructors in this file if `DeployContractPayload` gains a field (see Fix 3) — set `metadata_hash: [0u8; 32]`.

#### B. `consensus/src/pipeline/virtual_processor/processor.rs`
1. In `commit_utxo_state`, while building `tx_callers`, also capture the caller's full `ScriptPublicKey` into a parallel map:
   ```rust
   let mut tx_caller_scripts: HashMap<TransactionId, ScriptPublicKey> = HashMap::new();
   // inside the existing loop, when the first input's UTXO entry is found:
   tx_caller_scripts.insert(tx.id(), entry.script_public_key.clone());
   ```
2. Pass `tx_caller_scripts.get(&tx.id())` into `process_contract_tx`.
3. Collect payouts from successful outcomes and add them to the block's UTXO diff and multiset **before** they are written to the store:
   ```rust
   let mut payouts: Vec<(TransactionId, TransactionOutput)> = Vec::new();
   // after process_contract_tx returns Some(outcome) with outcome.success:
   if let Some(payout) = outcome.payout {
       payouts.push((tx.id(), payout));
   }
   ```
   Then, before `utxo_diffs_store.insert_batch` / `utxo_multisets_store.insert_batch`:
   ```rust
   for (tx_id, output) in &payouts {
       let outpoint = TransactionOutpoint::new(*tx_id, /* index */ tx.outputs.len() as u32);
       let entry = UtxoEntry::new(output.value, output.script_public_key.clone(), block_daa_score, false);
       mergeset_diff.add.insert(outpoint, entry);          // UtxoDiff.add is a public UtxoCollection
       multiset.add_utxo(&outpoint, &entry);               // MuHashExtensions::add_utxo
   }
   ```
   - `block_daa_score` is the DAA score of the block being committed; obtain it from the block header (`self.headers_store.get_daa_score(current)` or thread it in — `commit_utxo_state` already has `current: Hash`).
   - Use `tx.outputs.len() as u32` as the output index so the synthetic payout outpoint never collides with a real output (contract txs are built with empty outputs today, so this is `0`).
   - `MuHashExtensions` is already imported in `processor.rs` (`use zyanya_consensus_core::muhash::MuHashExtensions` is available via the existing imports; add if missing).

#### Consensus-commitment note (important)
The payout output is added **after** `verify_expected_utxo_state` has checked the header's `utxo_commitment`. This means the current block's header does not commit to the payout, but the payout is a **deterministic function of the block's contract txs**, so every node computes the identical diff + multiset and the virtual state stays consistent. The next block's header (built from the virtual multiset) *does* commit to it. Do not attempt to re-verify the current header after adding payouts. Document this deviation in a code comment.

### Attack surfaces / vulnerability categories
- Consensus integrity (deterministic UTXO/multiset mutation).
- Value-transfer correctness (seller actually receives refund; no silent destruction of ZYAN).
- Integer handling (use `saturating_sub` for the balance deduction; refund is already bounded by the contract's `reserve` check in `bonding_curve.zcl`).

---

## Fix 2 — F-C-13 FOLLOW-UP: wRPC WebSocket authentication

### Priority: CRITICAL

### Files
- `rpc/wrpc/server/src/service.rs` (Options, handshake)
- `rpc/wrpc/server/src/lib.rs` (module re-exports, if a new module is added)
- `rpc/wrpc/server/src/server.rs` (Server: store `rpc_auth_token`, `rpc_auth_token()` accessor, pass token into `Connection`)
- `rpc/wrpc/server/src/connection.rs` (Connection: store `auth_token`, `auth_token()` accessor)
- `rpc/core/src/api/ops.rs` (add `RpcApiOps::requires_auth()`)
- `rpc/macros/src/wrpc/server.rs` (emit per-method auth check)
- `zyanyad/src/daemon.rs` (thread `config.rpc_auth_token` into `WrpcServerOptions`)
- Vendored `workflow-websocket` (see token transport below)

### Current state
- gRPC enforces bearer-token auth on state-changing methods via `ZyanyadPayloadOps::requires_auth()` + `Connection::auth_token()` (`rpc/grpc/server/src/connection.rs:186`, `rpc/grpc/core/src/ops.rs:121`).
- wRPC has **no** auth: `handshake` in `service.rs` is a no-op (the greeting handshake is commented out), and the wRPC router dispatches every method unauthenticated.

### Changes

#### A. Per-method auth enforcement (core, in-repo)
1. `rpc/core/src/api/ops.rs` — add a `requires_auth()` method to `RpcApiOps` mirroring `ZyanyadPayloadOps::requires_auth()`:
   ```rust
   pub fn requires_auth(&self) -> bool {
       matches!(self,
           RpcApiOps::DeployContract | RpcApiOps::InvokeContract
           | RpcApiOps::Shutdown | RpcApiOps::Ban | RpcApiOps::Unban
           | RpcApiOps::AddPeer | RpcApiOps::ResolveFinalityConflict
           | RpcApiOps::SubmitBlock | RpcApiOps::SubmitTransaction
           | RpcApiOps::SubmitTransactionReplacement
           | RpcApiOps::NotifyUtxosChanged | RpcApiOps::NotifyPruningPointUtxoSetOverride
           | RpcApiOps::NotifyVirtualChainChanged | RpcApiOps::NotifyVirtualDaaScoreChanged
           | RpcApiOps::NotifyBlockAdded | RpcApiOps::NotifyNewBlockTemplate
           | RpcApiOps::NotifyFinalityConflict | RpcApiOps::NotifySinkBlueScoreChanged)
   }
   ```
   Read-only methods (`GetBlockCount`, `GetBlockDagInfo`, `GetInfo`, etc.) stay open.
2. `rpc/wrpc/server/src/service.rs` — add `pub rpc_auth_token: Option<String>` to `Options` and default it to `None` in `Default`.
3. `rpc/wrpc/server/src/server.rs` — store `rpc_auth_token` in `ServerInner` (from `Options`), add:
   ```rust
   pub fn rpc_auth_token(&self) -> Option<&str> { self.inner.options.rpc_auth_token.as_deref() }
   ```
   Change `connect()` to accept `auth_token: Option<String>` and pass it to `Connection::new`.
4. `rpc/wrpc/server/src/connection.rs` — add `auth_token: Option<String>` to `ConnectionInner`, thread through `Connection::new`, add:
   ```rust
   pub fn auth_token(&self) -> Option<&str> { self.inner.auth_token.as_deref() }
   ```
5. `rpc/macros/src/wrpc/server.rs` — in the generated method handler, insert the auth check after the verbose log and before the `rpc_service` call:
   ```rust
   if #rpc_api_ops::#handler.requires_auth() {
       if let Some(expected) = server_ctx.rpc_auth_token() {
           if connection_ctx.auth_token() != Some(expected) {
               return Err(ServerError::Text("Unauthorized".to_string()));
           }
       }
   }
   ```
   (`ServerError` is already in scope in the generated closure; `server_ctx` is `Server`, `connection_ctx` is `Connection`.)
6. `zyanyad/src/daemon.rs` — in the `WrpcServerOptions { ... }` construction (around line 624), add `rpc_auth_token: config.rpc_auth_token.clone()`.

#### B. Token transport — headers or query parameter
The `workflow-websocket` server currently uses `accept_async_with_config`, which discards the HTTP upgrade request, so headers/query are not visible to `RpcHandler::handshake`. To satisfy "check the auth token in the WebSocket connection headers or query parameter":

1. Vendor `workflow-websocket` (copy `~/.cargo/registry/src/.../workflow-websocket-0.18.0` into `vendor/workflow-websocket`) and add to the workspace `Cargo.toml`:
   ```toml
   [patch.crates-io]
   workflow-websocket = { path = "vendor/workflow-websocket" }
   ```
2. In `vendor/workflow-websocket/src/server/mod.rs`, change `handle_connection` to use `tokio_tungstenite::accept_hdr_async_with_config` with a callback that extracts the token:
   - `Authorization: Bearer <token>` header, or
   - `?token=<token>` query parameter from `req.uri().query()`.
   Store the extracted token in a peer-keyed registry (e.g. a `static Mutex<HashMap<SocketAddr, Option<String>>>`) and expose `pub fn take_handshake_token(peer: &SocketAddr) -> Option<String>`.
3. In `rpc/wrpc/server/src/service.rs::handshake`, call `workflow_websocket::server::take_handshake_token(peer)` and pass the result into `self.server.connect(peer, messenger, auth_token)`.

Fallback if vendoring proves too invasive: read the token from the first WebSocket message via the already-stubbed `handshake::greeting` pattern (store it on the `Connection`), and keep the per-method check. Note this deviates from "headers or query parameter" and requires client cooperation, so prefer the vendored patch.

### Attack surfaces / vulnerability categories
- Unauthenticated state-changing RPC (SubmitBlock/SubmitTransaction/Shutdown/Ban/AddPeer) over WebSocket.
- Consensus bypass via unauthenticated `SubmitBlock`/`SubmitTransaction`.
- Network DoS (admin methods remotely reachable).
- Token comparison (use constant-time comparison if feasible; at minimum compare `Option<&str>` equality as gRPC does).

---

## Fix 3 — F-C-16 FOLLOW-UP: Cryptographic metadata binding

### Priority: CRITICAL

### Files
- `zyanya-explorer/src/client.rs`
- `consensus/src/model/stores/contract.rs`
- `consensus/core/src/tx.rs` (add `metadata_hash` to `DeployContractPayload`)
- `rpc/service/src/service.rs` (set `metadata_hash: [0u8; 32]` in `deploy_contract_call`)
- Test constructors: `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`, `consensus/core/src/tx.rs`

### Current state
- Token metadata (`name`, `symbol`, `description`, `twitter`, `telegram`, `website`, `icon_uri`) is sanitized server-side (`sanitize_metadata` strips `< > " ' \``) but is **not** bound to the signed transaction. `submit_signed_tx` verifies signatures only over `data.tx`/`data.entries`, then saves the (tamperable) metadata.

### Changes

#### A. `consensus/core/src/tx.rs`
1. Add a field to `DeployContractPayload`:
   ```rust
   pub struct DeployContractPayload {
       pub bytecode: Vec<u8>,
       pub max_gas: u64,
       pub gas_price: u64,
       pub deposit_amount: u64,
       pub metadata_hash: [u8; 32],   // blake2b-256 of sanitized token metadata; [0;32] for non-token deploys
   }
   ```
   This changes borsh serialization (and therefore deploy tx ids) — acceptable pre-launch.
2. Update the `test_contract_payload_borsh` test to set `metadata_hash: [0u8; 32]`.

#### B. `consensus/src/model/stores/contract.rs`
1. Add `pub metadata_hash: HashMap<[u8; 32], [u8; 32]>` to `ContractStateCache` (default empty).
2. In the `Deploy` branch of `process_contract_tx`, after successful bytecode validation, store the committed hash:
   ```rust
   cache.metadata_hash.insert(addr_bytes, deploy.metadata_hash);
   ```
3. Add a `metadata_hash_access: CachedDbAccess<Hash, [u8; 32]>` to `DbContractStore` with a new `DatabaseStorePrefixes` variant (e.g. `ContractMetadataHash`), commit it in `commit_cache_batch`, and add `pub fn get_metadata_hash(&self, contract_address: Hash) -> StoreResult<[u8; 32]>`.
   - Requires adding the prefix variant in `database/src/registry.rs` (or wherever `DatabaseStorePrefixes` is defined).
4. Update the two test `DeployContractPayload` constructors to set `metadata_hash: [0u8; 32]`.

#### C. `zyanya-explorer/src/client.rs`
1. Add a deterministic hash helper (blake2b-256, length-prefixed to avoid field-boundary ambiguity):
   ```rust
   fn compute_metadata_hash(m: &TokenMetadata) -> [u8; 32] {
       let mut h = blake2b_simd::Params::new().hash_length(32).to_state();
       for field in [m.name.as_deref(), m.symbol.as_deref(), m.description.as_deref(),
                     m.twitter.as_deref(), m.telegram.as_deref(), m.website.as_deref(),
                     m.icon_uri.as_deref()] {
           let bytes = field.unwrap_or("").as_bytes();
           h.update(&(bytes.len() as u64).to_le_bytes());
           h.update(bytes);
       }
       *h.finalize().as_array()
   }
   ```
   Add `blake2b_simd = { workspace = true }` to `zyanya-explorer/Cargo.toml` (it is already a workspace dependency).
2. **Sanitize before hashing.** The hash must bind the *stored* (sanitized) metadata. In `build_unsigned_deploy_token_tx`, build a `TokenMetadata` from the request, run `sanitize_metadata(&mut metadata)`, compute `metadata_hash`, and include it in `DeployContractPayload`. Keep the sanitized values in `SignableTxData` so the client signs/returns the same values.
3. In `submit_signed_tx`:
   - Reconstruct `TokenMetadata` from `SignableTxData`, run `sanitize_metadata` (idempotent), recompute `metadata_hash`.
   - For `ContractPayload::Deploy`, extract `deploy.metadata_hash` and compare to the recomputed hash; **reject** (`return Err(...)`) on mismatch before saving metadata.
   - Store the committed hash alongside the metadata (see below).
4. `save_token_metadata` / `get_token_metadata`:
   - Persist the committed `metadata_hash` with the metadata (extend the stored JSON shape or a parallel map).
   - On retrieval (`get_token_metadata`), recompute the hash from the stored metadata and compare to the committed hash; return `None` (or a verification error) on mismatch so tampered metadata is never served.
   - The committed hash is the one verified against the signed deploy tx at submit time; if the on-chain getter from (B) is wired to an RPC later, prefer querying it.

#### D. `rpc/service/src/service.rs`
- In `deploy_contract_call`, set `metadata_hash: [0u8; 32]` in the `DeployContractPayload` (raw/non-token deploys carry no metadata). Optionally add `metadata_hash` to `DeployContractRequest` (with its custom `Serializer`/`Deserializer` in `rpc/core/src/model/message.rs`) if RPC-driven token deploys must carry metadata — not required for the explorer path, which builds the tx locally.

#### E. `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`
- Update the test `DeployContractPayload` constructor to set `metadata_hash: [0u8; 32]`.

### Attack surfaces / vulnerability categories
- Metadata spoofing / integrity (metadata now covered by the tx signature via the payload hash).
- Stored XSS (indirect: tampered metadata can no longer be persisted with a valid signature).
- Hash construction (length-prefix fields; sanitize-then-hash so the committed hash matches stored bytes).

---

## Implementation order

1. **Fix 3 payload field first** (`consensus/core/src/tx.rs` + all `DeployContractPayload` constructors), because it touches the most call sites and is a pure compile-surface change. Run `cargo check` after.
2. **Fix 1** (`contract.rs` + `processor.rs`) — payout plumbing.
3. **Fix 2** (`rpc/core/src/api/ops.rs`, wRPC server files, macro, daemon, vendored `workflow-websocket`).
4. **Fix 3 explorer side** (`zyanya-explorer/src/client.rs` + `Cargo.toml`).

## Verification

```bash
source /root/.cargo/env && cargo check 2>&1
```

- Fix any compilation errors only. Do **not** run full build or tests.
- Watch for: missing `metadata_hash` in any `DeployContractPayload` literal; the macro-generated auth check referencing `rpc_auth_token()`/`auth_token()` before those methods exist; the vendored `workflow-websocket` patch compiling under the workspace toolchain.

## Risks / notes for the builder

- **Fix 1 consensus subtlety**: payouts are added to the UTXO diff + multiset after header verification. This is deterministic and consensus-safe, but the current block header does not commit to the payout. Add a code comment; do not re-verify the header.
- **Fix 1 address derivation**: reuse `derive_caller_from_script_pub_key`'s address extraction, but keep the full `ScriptPublicKey`/`Address` (do not reduce to `u64`) for the payout script. Fail closed if extraction fails.
- **Fix 2 vendoring**: the `[patch.crates-io]` path must preserve `workflow-websocket`'s feature set (it is pulled in via `workflow-rpc`). If the vendored patch is too risky, fall back to the greeting-message transport and note the deviation.
- **Fix 3 hash domain**: hash the *sanitized* metadata and length-prefix each field. `blake2b_simd` is already a workspace dep; add it to `zyanya-explorer/Cargo.toml`.
- Do not run tests; `cargo check` is the acceptance gate.
