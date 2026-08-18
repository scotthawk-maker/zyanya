# PR #2 Follow-Up Security Fixes — F-C-03, F-C-13, F-C-16

**Session:** `7ed7c305`
**Base commit:** `e674f02`
**Audit source:** `audit_reports/FINAL_AUDIT_REPORT.md`
**Scope:** Three CRITICAL-severity follow-up items flagged during the security review of PR #2. Each was previously mitigated with a stopgap (fail-closed guard, commented-out handshake, server-side HTML sanitization); this session implements the full fix for each.

---

## 1. Why this matters

The consolidated audit identified three findings whose initial remediation was incomplete:

| Finding | Original stopgap | This session's fix |
|---------|------------------|---------------------|
| **F-C-03** — Bonding-curve sell destroys the refund | Sell failed closed (`success: false`), disabling the entire sell path | Real UTXO payout output paying the seller |
| **F-C-13** — No auth on wRPC WebSocket | Handshake was a commented-out no-op; every method was open | Bearer-token enforcement on state-changing methods |
| **F-C-16** — Token metadata not cryptographically bound | Metadata was HTML-sanitized but unsigned; clients could tamper after signing | blake2b-256 `metadata_hash` embedded in the signed deploy payload |

Without these follow-ups, a token seller never received their ZYAN, any WebSocket client could call admin/contract methods, and an attacker could rewrite token metadata (name, social links, icon) after a deploy transaction was signed.

---

## 2. Fix 1 — F-C-03: UTXO payout plumbing for smart-contract sells

### What changed

The consensus contract processor and virtual state processor now create a real UTXO output paying the seller the refund amount when a contract sell/withdrawal (`entry_point == 5`) succeeds. Previously the sell branch returned `success: false` unconditionally because there was no plumbing to emit a UTXO output from the virtual processor.

### Files

#### `consensus/src/model/stores/contract.rs`

- `ContractExecutionOutcome` gained a `payout: Option<TransactionOutput>` field. Every existing outcome construction site (deploy, invoke, buy, failure paths) was updated to set `payout: None` except the sell path.
- `ContractProcessor::process_contract_tx` signature now takes `caller_script_public_key: Option<&ScriptPublicKey>` in addition to the existing `caller: u64`. The full script is needed to derive the seller's address for the payout output.
- The `entry_point == 5` (sell) branch was rewritten:
  - The refund amount is the VM return value (`ret_val`).
  - The refund is deducted from the contract's cached balance (`temp_cache.balances`) with `saturating_sub`, mirroring the buy-side cost deduction.
  - A payout `TransactionOutput` is built by resolving the caller's `ScriptPublicKey` to an address via `zyanya_txscript::extract_script_pub_key_address(spk, Prefix::Mainnet)` and then `zyanya_txscript::pay_to_address_script(&seller_address)`.
  - If the caller script is `None` or address extraction fails, the sell **fails closed** (`success: false`, no payout) — the code never pays to an unknown address.
  - On success, the temp cache is committed (`*cache = temp_cache`) and the outcome returns `success: true` with `payout: Some(...)`.
- `ContractStateCache` gained a `metadata_hash: HashMap<[u8; 32], [u8; 32]>` field (used by Fix 3, see below).
- `DbContractStore` gained a `metadata_hash_access: CachedDbAccess<Hash, [u8; 32]>` backed by the new `DatabaseStorePrefixes::ContractMetadataHash` (prefix 64). `commit_cache_batch` now persists metadata hashes, and a new `get_metadata_hash(&self, contract_address: Hash) -> StoreResult<[u8; 32]>` accessor was added.
- All `process_contract_tx` test call sites were updated to pass `None` for the caller script (tests do not exercise the sell path).

#### `consensus/src/pipeline/virtual_processor/processor.rs`

- `commit_utxo_state` now also captures the caller's full `ScriptPublicKey` into a parallel `tx_caller_scripts` map (alongside the existing `tx_callers` u64 map), keyed by `TransactionId`.
- The signature of `commit_utxo_state` changed: `mergeset_diff` and `multiset` are now `mut` so payout UTXOs can be appended.
- The `utxo_diffs_store.insert_batch` / `utxo_multisets_store.insert_batch` calls were **moved** to after contract processing so payout outputs can be added to the diff and multiset before they are persisted.
- After contract processing, collected payouts are turned into `UtxoEntry`s and inserted into `mergeset_diff.add` and the multiset (`multiset.add_utxo`), using the block's DAA score (`self.headers_store.get_daa_score(current)`) and a synthetic outpoint at index `tx.outputs.len() as u32` (contract txs have empty outputs today, so this is `0`).
- **Consensus commitment note (documented in code):** payouts are added *after* `verify_expected_utxo_state` has checked the header's `utxo_commitment`, so the current block header does not commit to the payout. However, the payout is a deterministic function of the block's contract transactions, so every node computes the identical diff + multiset and the virtual state stays consistent. The next block's header (built from the virtual multiset) does commit to it. The code comment warns not to re-verify the current header after adding payouts.

#### `utils/src/mem_size.rs`

- Added a blanket `MemSizeEstimator` impl for `[T; N]` (arrays), returning `size_of::<Self>()`. Required because `CachedDbAccess<Hash, [u8; 32]>` needs its value type to implement `MemSizeEstimator`.

### How to verify

- `cargo check` (the acceptance gate).
- A sell invoke (`entry_point == 5`) that succeeds now returns `success: true` with a populated `payout`. The virtual processor adds the payout UTXO to the block diff. The seller's address is derived from the first input's spent UTXO script.

---

## 3. Fix 2 — F-C-13: wRPC WebSocket bearer-token authentication

### What changed

The wRPC WebSocket server now enforces the same `ZYANYA_RPC_AUTH_TOKEN` bearer-token authentication that gRPC already enforces, but only for state-changing / admin methods. Read-only methods (`GetBlockCount`, `GetBlockDagInfo`, `GetInfo`, etc.) remain open. The token is extracted from the HTTP WebSocket upgrade request (`Authorization: Bearer <token>` header or `?token=<token>` query parameter), which required vendoring `workflow-websocket` to gain access to the upgrade request headers.

### Files

#### `vendor/workflow-websocket/` (new vendored crate)

- The entire `workflow-websocket` 0.18.0 crate was copied into `vendor/workflow-websocket/` and patched via `[patch.crates-io]` in the workspace `Cargo.toml`.
- `vendor/workflow-websocket/src/server/mod.rs`:
  - `handle_connection` switched from `accept_async_with_config` to `accept_hdr_async_with_config` with a `TokenExtractor` callback that implements `tungstenite::handshake::server::Callback::on_request`.
  - `TokenExtractor::on_request` calls `extract_token_from_request`, which checks (1) the `Authorization: Bearer <token>` header and (2) the `?token=<token>` query parameter, and stores the result in a peer-keyed `static LazyLock<Mutex<HashMap<SocketAddr, Option<String>>>>` registry (`HANDSHAKE_TOKENS`).
  - `pub fn take_handshake_token(peer: &SocketAddr) -> Option<String>` retrieves and removes the token for a peer, so the RPC handshake handler can pass it into the `Connection`.
- The vendored crate preserves all original features (`rustls-tls-webpki-roots`, `native-tls`, etc.) so `workflow-rpc` (which depends on `workflow-websocket`) continues to compile.

#### `Cargo.toml` (workspace root)

- Added `workflow-websocket` to `[workspace.dependencies]` with `default-features = false, features = ["rustls-tls-webpki-roots"]`.
- Added a `[patch.crates-io]` section redirecting `workflow-websocket` to `vendor/workflow-websocket`.

#### `rpc/core/src/api/ops.rs`

- Added `RpcApiOps::requires_auth(&self) -> bool`, mirroring the gRPC-side `ZyanyadPayloadOps::requires_auth()`. Returns `true` for: `DeployContract`, `InvokeContract`, `Shutdown`, `Ban`, `Unban`, `AddPeer`, `ResolveFinalityConflict`, `SubmitBlock`, `SubmitTransaction`, `SubmitTransactionReplacement`, and all `Notify*` subscription methods. Read-only methods return `false`.

#### `rpc/macros/src/wrpc/server.rs`

- The generated per-method handler closure now inserts an auth check after the verbose log and before the `rpc_service` call:
  ```rust
  if #rpc_api_ops::#handler.requires_auth() {
      if let Some(expected) = server_ctx.rpc_auth_token() {
          if connection_ctx.auth_token() != Some(expected) {
              return Err(ServerError::Text("Unauthorized".to_string()));
          }
      }
  }
  ```
  When no token is configured (`rpc_auth_token() == None`), the check is a no-op and all methods remain open — matching gRPC behavior.

#### `rpc/wrpc/server/src/service.rs`

- `Options` gained `pub rpc_auth_token: Option<String>` (defaults to `None`).
- The `handshake` implementation replaced the commented-out greeting stub with a call to `workflow_websocket::server::take_handshake_token(peer)`, passing the extracted token into `self.server.connect(peer, messenger, auth_token)`.

#### `rpc/wrpc/server/src/server.rs`

- `Server::connect` now accepts `auth_token: Option<String>` and forwards it to `Connection::new`.
- Added `Server::rpc_auth_token(&self) -> Option<&str>` accessor (reads from `options.rpc_auth_token`).

#### `rpc/wrpc/server/src/connection.rs`

- `ConnectionInner` gained `pub auth_token: Option<String>`.
- `Connection::new` now takes `auth_token: Option<String>`.
- Added `Connection::auth_token(&self) -> Option<&str>` accessor used by the macro-generated auth check.

#### `rpc/wrpc/server/Cargo.toml`

- Added `workflow-websocket.workspace = true` dependency.

#### `rpc/wrpc/proxy/src/main.rs`

- The proxy's `WrpcServerOptions` construction now explicitly sets `rpc_auth_token: None` (the proxy does not enforce auth itself; it forwards to gRPC which does).

#### `zyanyad/src/daemon.rs`

- `config.rpc_auth_token` is cloned before `config` is moved into the gRPC service, and threaded into the `WrpcServerOptions { rpc_auth_token: rpc_auth_token.clone(), .. }` construction.

### How to verify

- `cargo check`.
- With `ZYANYA_RPC_AUTH_TOKEN` set, a wRPC client calling `SubmitTransaction` without a matching `Authorization: Bearer` header (or `?token=` query) receives `Unauthorized`. A client supplying the correct token succeeds. `GetBlockCount` succeeds regardless of token.

---

## 4. Fix 3 — F-C-16: Cryptographic metadata binding

### What changed

Token metadata (name, symbol, description, twitter, telegram, website, icon_uri) is now hashed with blake2b-256 and the resulting `metadata_hash` is embedded in the `DeployContractPayload`, making it part of the signed deploy transaction. On retrieval, the stored metadata is re-hashed and compared to the committed hash; mismatches (tampered metadata) are rejected.

### Files

#### `consensus/core/src/tx.rs`

- `DeployContractPayload` gained `pub metadata_hash: [u8; 32]` (blake2b-256 of sanitized metadata; `[0; 32]` for non-token deploys). This changes borsh serialization and therefore deploy tx ids — acceptable pre-launch.
- The `test_contract_payload_borsh` test was updated to set `metadata_hash: [0u8; 32]`.

#### `database/src/registry.rs`

- Added `DatabaseStorePrefixes::ContractMetadataHash = 64` for persistent storage of committed metadata hashes.

#### `consensus/src/model/stores/contract.rs`

- `ContractStateCache` gained `metadata_hash: HashMap<[u8; 32], [u8; 32]>`.
- In the `Deploy` branch of `process_contract_tx`, after successful bytecode validation, `cache.metadata_hash.insert(addr_bytes, deploy.metadata_hash)` records the committed hash.
- `DbContractStore` gained `metadata_hash_access: CachedDbAccess<Hash, [u8; 32]>`, committed in `commit_cache_batch`, with a `get_metadata_hash` accessor.
- Test `DeployContractPayload` constructors updated to set `metadata_hash: [0u8; 32]`.

#### `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`

- Test `DeployContractPayload` constructor updated to set `metadata_hash: [0u8; 32]`.

#### `rpc/service/src/service.rs`

- `deploy_contract_call` sets `metadata_hash: [0u8; 32]` for raw/non-token deploys (RPC-driven token deploys carry no metadata in this path).

#### `zyanya-explorer/Cargo.toml`

- Added `blake2b_simd = { workspace = true }`.

#### `zyanya-explorer/src/client.rs`

- Added `compute_metadata_hash(m: &TokenMetadata) -> [u8; 32]`: hashes each field (name, symbol, description, twitter, telegram, website, icon_uri) with a u64-LE length prefix to avoid field-boundary ambiguity; `None` fields hash as empty strings.
- `RpcClientManager` gained `metadata_hash_store: Arc<tokio::sync::Mutex<HashMap<String, [u8; 32]>>>`.
- **Build path** (`build_unsigned_deploy_token_tx`):
  - The icon URI is now computed *before* the payload (previously it was computed after). The icon filename uses a blake2b-16 hash of the icon base64 data instead of the contract address (which is only known after the tx id is computed), ensuring determinism before the payload is assembled.
  - Metadata is constructed from the request, sanitized via `sanitize_metadata`, hashed via `compute_metadata_hash`, and the resulting `metadata_hash` is included in the `DeployContractPayload`.
  - The sanitized metadata values (not the raw request values) are placed into `SignableTxData` so the client signs and returns the same values that were hashed.
- **Submit path** (`submit_signed_tx`):
  - After signature verification, the deploy payload is deserialized from `data.tx.payload`. If it is a `Deploy`, the `SignableTxData` metadata fields are reconstructed into a `TokenMetadata`, sanitized (idempotent), re-hashed, and compared to `deploy.metadata_hash`. A mismatch returns an error ("Metadata hash mismatch ... Metadata may have been tampered with after signing.") before the tx is submitted or metadata is saved.
- **Storage / retrieval**:
  - `save_token_metadata` now sanitizes, computes the committed hash, and stores it in `metadata_hash_store` (keyed by both the address and its lowercased form) alongside the metadata.
  - `get_token_metadata` recomputes the hash from the stored metadata and compares it to the committed hash in `metadata_hash_store`. If a committed hash exists but does not match, the function logs a warning and returns `None` (tampered metadata is never served). If no committed hash is recorded (legacy/unsigned metadata), the metadata is returned as-is.

### How to verify

- `cargo check`.
- Deploy a token via the explorer; the unsigned tx payload contains a non-zero `metadata_hash`. Modifying any metadata field in the `SignableTxData` JSON after signing produces a hash mismatch at submit time and the submission is rejected. Modifying the stored metadata on disk causes `get_token_metadata` to return `None`.

---

## 5. Supporting artifacts

- `specs/7ed7c305_payout-auth-metadata.md` — the 287-line implementation plan/spec written before coding, covering all three fixes with attack-surface analysis, implementation order, and verification steps.

---

## 6. Verification

The acceptance gate is `cargo check` (no full build or tests):

```bash
source /root/.cargo/env && cargo check 2>&1
```

Watch for:
- Missing `metadata_hash` in any `DeployContractPayload` literal (the field was added to the struct, so every constructor must set it).
- The macro-generated auth check referencing `rpc_auth_token()` / `auth_token()` before those methods exist on `Server` / `Connection`.
- The vendored `workflow-websocket` compiling under the workspace toolchain (the `[patch.crates-io]` path must preserve the feature set pulled in by `workflow-rpc`).

---

## 7. Notes for future agents

- The F-C-03 payout is added to the UTXO diff *after* header verification. This is deterministic and consensus-safe, but the current block header does not commit to the payout. Do not attempt to re-verify the current header after adding payouts — the next block's header covers it.
- The F-C-13 vendored `workflow-websocket` is a full copy of the 0.18.0 crate with a minimal patch to `server/mod.rs` (switching to `accept_hdr_async_with_config` and adding the `HANDSHAKE_TOKENS` registry). If `workflow-rpc` is upgraded, the vendored copy may need to be re-synced.
- The F-C-16 `metadata_hash` field on `DeployContractPayload` changes borsh serialization and therefore all deploy tx ids. This is acceptable pre-launch but would be a consensus break post-launch.
- `RpcApiOps::requires_auth()` is maintained separately from the gRPC-side `ZyanyadPayloadOps::requires_auth()`. If new state-changing methods are added, both must be updated.