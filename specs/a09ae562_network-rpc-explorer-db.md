# Remediation Plan — MEDIUM Findings Batch M2 (F-M-23 … F-M-39)

**Date:** 2026-08-18
**Scope:** Network, RPC, Explorer, Database MEDIUM findings from `audit_reports/FINAL_AUDIT_REPORT.md`.
**Goal:** Apply the fixes listed in the task brief, then confirm `cargo check --workspace --all-targets` passes.

## Verification

```bash
cargo check --workspace --all-targets
```

Run from `/root/zyanya-audit`. Fix any compile errors introduced by signature changes (noted per finding below).

---

## Priority order (highest first)

All findings in this batch are MEDIUM. Ordering below reflects exploitability/impact within the batch.

| # | Finding | Area | Rationale |
|---|---------|------|-----------|
| 1 | F-M-28 | RPC | Error responses leak internal state to unauthenticated clients |
| 2 | F-M-30 | Explorer | Reflected XSS in DAG visualization modal |
| 3 | F-M-31 | Explorer | No CORS → CSRF on state-changing endpoints |
| 4 | F-M-24 | Network | Tx-request bandwidth amplification |
| 5 | F-M-29 | Network | Oversized inv messages (128K tx ids) |
| 6 | F-M-32 | Notify | Unbounded channels → OOM |
| 7 | F-M-33 | Explorer | Unvalidated icon upload (disk exhaustion / polyglot) |
| 8 | F-M-35 | Explorer | World-readable /tmp fallback for metadata |
| 9 | F-M-37 | Explorer | Pagination integer overflow |
| 10 | F-M-38 | Database | Panic on DB open failure |
| 11 | F-M-39 | Config | Unbounded `ram_scale` f64 |
| 12 | F-M-25 | Address mgr | Panic on `WeightedError` |
| 13 | F-M-26 | wRPC | Panic on poisoned mutex |
| 14 | F-M-27 | wRPC | Panic on notification serialization failure |
| 15 | F-M-23 | RPC | `unsafe from_utf8_unchecked` in hex conversion |
| 16 | F-M-34 | Database | Unsafe pointer cast in registry |
| 17 | F-M-36 | Explorer | Reflected XSS via error message (verify — likely already escaped) |

---

## Detailed fixes

### F-M-23 — `unsafe { str::from_utf8_unchecked }` in hex conversion

- **File:** `rpc/core/src/model/hex_cnv.rs` (impl `ToRpcHex for &[u8]`, ~line 26)
- **Current:** `let result = unsafe { str::from_utf8_unchecked(&hex) }; result.to_string()`
- **Fix:** Replace with safe conversion:
  ```rust
  let result = String::from_utf8(hex).expect("hex output is always ASCII");
  result
  ```
  (or `String::from_utf8(hex).unwrap()`). `faster_hex::hex_encode` only emits ASCII, so this is sound and removes the `unsafe` block. Keep the existing `use std::str;` only if still needed; otherwise drop it.

### F-M-24 — `RequestTransactionsFlow` unlimited tx requests

- **File:** `protocol/flows/src/v5/txrelay/flow.rs` (`RequestTransactionsFlow::start_impl`, ~lines 340-360)
- **Current:** loops over every requested tx id with no cap.
- **Fix:** Add a per-request cap of 100 tx ids:
  ```rust
  /// Maximum number of transaction IDs honored per RequestTransactionsMessage.
  const MAX_TX_PER_REQUEST: usize = 100;
  ```
  In `start_impl`, after `let tx_ids: Vec<_> = msg.try_into()?;`:
  ```rust
  if tx_ids.len() > MAX_TX_PER_REQUEST {
      zyanya_core::warn!("peer {} requested {} txs; truncating to {}", self.router, tx_ids.len(), MAX_TX_PER_REQUEST);
  }
  for transaction_id in tx_ids.into_iter().take(MAX_TX_PER_REQUEST) {
      // ... existing body unchanged
  }
  ```
  (Optionally, mirror the existing F-H-20 sliding-window ban pattern in this file if a stricter per-peer rate limit is desired; the required minimum is the 100-tx-per-request cap.)

### F-M-25 — `RandomWeightedIterator::new` panics on `WeightedError`

- **File:** `components/addressmanager/src/lib.rs` (`RandomWeightedIterator::new`, ~lines 470-478; also `next()` ~line 490)
- **Current:** `Err(e) => panic!("{e}")` in the constructor.
- **Fix:** Return the error instead of panicking:
  ```rust
  pub fn new(weights: Vec<f64>, addresses: Vec<NetAddress>) -> Result<Self, WeightedError> {
      assert_eq!(weights.len(), addresses.len());
      let remaining = weights.iter().filter(|&&w| w > 0.0).count();
      let weighted_index = match WeightedIndex::new(weights) {
          Ok(index) => Some(index),
          Err(WeightedError::NoItem) => None,
          Err(e) => return Err(e),
      };
      Ok(Self { weighted_index, remaining, addresses })
  }
  ```
- **Ripple (required):**
  - `Store::iterate_prioritized_random_addresses` (~line 476) → return `Result<impl ExactSizeIterator<Item = NetAddress>, WeightedError>` and propagate with `?`.
  - `AddressManager::iterate_prioritized_random_addresses` (~line 315) → same `Result<...>` signature, propagate.
  - Caller `components/connectionmanager/src/lib.rs:175` → handle the `Result` (log the error and return from `handle_outbound_connections`, or treat as empty iterator).
  - Tests at `components/addressmanager/src/lib.rs:579,583,652` → update to `RandomWeightedIterator::new(...).unwrap()` / handle `Result`.
- **Also fix** the same `Err(e) => panic!("{e}")` in `Iterator::next` (`update_weights` error arm): log and set `self.weighted_index = None` instead of panicking.

### F-M-26 — wRPC `disconnect` uses `.lock().unwrap()`

- **File:** `rpc/wrpc/server/src/server.rs` (`Server::disconnect`, ~line 180)
- **Current:** `self.inner.sockets.lock().unwrap().remove(&connection.id());`
- **Fix:** Change `disconnect` to return `Result<(), WebSocketError>` and use `map_err`:
  ```rust
  pub async fn disconnect(&self, connection: Connection) -> Result<(), WebSocketError> {
      // ... existing unregister_listener / grpc_client cleanup unchanged ...
      self.inner
          .sockets
          .lock()
          .map_err(|e| WebSocketError::Other(format!("sockets mutex poisoned: {e}")))?
          .remove(&connection.id());
      Ok(())
  }
  ```
- **Ripple:** caller `rpc/wrpc/server/src/service.rs:90` (`self.server.disconnect(ctx).await;`) → `let _ = self.server.disconnect(ctx).await;` (or log the error). `WebSocketError` is already imported in `server.rs` (used in `connect`).

### F-M-27 — wRPC notification serialization `.unwrap()`

- **File:** `rpc/wrpc/server/src/connection.rs` (`ConnectionT::into_message`, ~lines 159-161)
- **Current:** `Self::create_serialized_notification_message(...).unwrap()`
- **Constraint:** the `ConnectionT` trait (`notify/src/connection.rs:15`) returns `Self::Message` (not `Result`), so the error cannot be propagated without a wider trait change. Do **not** change the trait.
- **Fix:** Replace `.unwrap()` with a match that logs and returns a benign message:
  ```rust
  fn into_message(notification: &Self::Notification, encoding: &Self::Encoding) -> Self::Message {
      let op: RpcApiOps = notification.event_type().into();
      match Self::create_serialized_notification_message(encoding.clone().into(), op, Serializable(notification.clone())) {
          Ok(msg) => msg,
          Err(err) => {
              workflow_log::log_error!("Failed to serialize wRPC notification: {err}");
              Message::Close(None) // benign fallback; verify exact variant from workflow_rpc::server::prelude::*
          }
      }
  }
  ```
  Verify the `Message` type/variant (`Message::Close(None)` is used in `workflow-rpc` server code; if unavailable, use an empty `Message::Text`).

### F-M-28 — RPC error responses leak internal state

- **Files:** `rpc/core/src/error.rs` (add sanitizer), `rpc/grpc/core/src/ops.rs` (apply at gRPC boundary), `rpc/wrpc/server/src/error.rs` (apply at wRPC boundary)
- **Fix:**
  1. Add a method to `RpcError` in `rpc/core/src/error.rs`:
     ```rust
     impl RpcError {
         /// Returns a version safe to send to unauthenticated RPC clients.
         /// Internal-detail variants are collapsed to generic messages; the
         /// full error must be logged server-side before calling this.
         pub fn sanitize(self) -> RpcError {
             use RpcError::*;
             match self {
                 RejectedTransaction(id, _) => RejectedTransaction(id, "transaction rejected".to_string()),
                 ConsensusError(_) | MiningManagerError(_) | NotificationError(_)
                 | ConsensusClient(_) | WasmError(_) | SerdeWasmBindgen(_) | ScriptClassError(_)
                 | AddressError(_) | NetworkTypeError(_) | NetworkIdError(_) | NodeIdError(_)
                 | SubnetParsingError(_) | UtxoReturnAddressNotFound(_) | RpcSubsystem(_) => {
                     General("internal error".to_string())
                 }
                 other => other, // user-facing variants pass through unchanged
             }
         }
     }
     ```
     (Adjust the exact variant list to match the enum; keep user-facing variants such as `TransactionNotFound`, `Unauthorized`, `UnavailableInSafeMode`, `NoUtxoIndex`, `WindowSizeExceedingMaximum`, `InvalidGetBlocksRequest`, `CoinbasePayloadLengthAboveMax`, `InvalidBlock`, `MergerNotFound`, `IpHasPermanentConnection`, `IpIsNotBanned`, `SubmitBlockError`, `NotImplemented`, `UnsupportedFeature`, `MissingRpcFieldError`, `InvalidRpcScriptClass`, parse errors, `InconsistentMempoolTxQuery`, `RpcCtlDispatchError` unchanged.)
  2. gRPC boundary — `rpc/grpc/core/src/ops.rs` `to_error_response` macro: change
     `[<$variant_name ResponseMessage>]::from(error)` → `[<$variant_name ResponseMessage>]::from(error.sanitize())`.
  3. wRPC boundary — `rpc/wrpc/server/src/error.rs`: replace the `#[from]` on the `RpcError` variant with a manual `From<RpcError> for Error` that calls `.sanitize()`:
     ```rust
     impl From<RpcError> for Error {
         fn from(err: RpcError) -> Self { Error::RpcError(err.sanitize()) }
     }
     ```
     (remove `#[from]` from the `RpcError(#[from] RpcError)` variant).
  4. Log the full error server-side before sanitizing at the call sites (e.g., in `rpc/grpc/server/src/connection.rs` where `to_error_response` is invoked, and in the wRPC handler path) so operators retain diagnostics.

### F-M-29 — `MAX_INV_PER_TX_INV_MSG = 131_072`

- **File:** `protocol/flows/src/flowcontext/transactions.rs:16`
- **Fix:** `pub(crate) const MAX_INV_PER_TX_INV_MSG: usize = 10_000;`

### F-M-30 — Reflected XSS via block data in DAG visualization modal

- **File:** `zyanya-explorer/src/web.rs` (DAG page script; tooltip ~5757-5764, modal ~5786-5792)
- **Context:** `escapeHtml()` is already defined in this script block (~line 5377).
- **Fix:** Escape every interpolated block field:
  - Tooltip (`tooltip.innerHTML`): wrap `node.short_hash`, each parent in `node.parents.map(...)`, and `node.selected_parent` with `escapeHtml(...)`.
  - Modal (`body.innerHTML`): wrap `block.hash`, each parent in `block.parents.map(p => escapeHtml(p)).join('<br>')`, and `block.selected_parent` with `escapeHtml(...)`.
  - Example: `${node.parents ? node.parents.map(p => escapeHtml(p.substring(0, 6))).join(', ') : 'None'}` and `${escapeHtml(block.selected_parent || 'None')}`.

### F-M-31 — No CORS restrictions on explorer API

- **File:** `zyanya-explorer/src/main.rs` (router construction ~lines 67-101)
- **Fix:** Add a restrictive CORS layer. Preferred (no new dependency): an axum middleware via `axum::middleware::from_fn` that:
  - Reads `Origin` and `Host` headers; reflects `Access-Control-Allow-Origin` **only** when the Origin's host:port matches the request `Host` (same-origin). Never emit `*`.
  - Handles `OPTIONS` preflight: return `204` with `Access-Control-Allow-Methods: GET, POST, OPTIONS` and `Access-Control-Allow-Headers: Content-Type` when same-origin.
  - Apply with `.layer(middleware::from_fn(cors_same_origin))` on the `Router`.
  - Alternative (if preferred): add the `cors` feature to the workspace `tower-http` dependency (`Cargo.toml` line 276) and use `tower_http::cors::CorsLayer` with a same-origin allowlist. `tower-http 0.5.2` is already in `Cargo.lock`.
- **Note:** CORS alone does not fully stop CSRF for simple POSTs; the required minimum per the brief is the same-origin `Access-Control-Allow-Origin` header. Do not set `Access-Control-Allow-Origin: *`.

### F-M-32 — Unbounded notification channels

- **Files:** `notify/src/subscriber.rs:75`, `notify/src/notifier.rs:293`
- **Fix:** `Channel::unbounded()` → `Channel::bounded(1000)` in both places.
- **Verified safe:** both send paths use `try_send` (`subscriber.rs:116`, `notifier.rs:473`), and `TrySendError` already converts to `Error::ChannelSendError` (`notify/src/error.rs`), so a full channel returns an error instead of blocking/deadlocking. No further changes required.

### F-M-33 — Token icon upload has no validation

- **File:** `zyanya-explorer/src/client.rs` (`save_token_icon`, ~lines 360-375)
- **Fix:** After `let decoded = decode_base64(base64_data)?;` add:
  ```rust
  const MAX_ICON_SIZE: usize = 1024 * 1024; // 1 MB
  const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
  if decoded.len() > MAX_ICON_SIZE {
      return Err("token icon exceeds 1 MB limit".to_string());
  }
  if !decoded.starts_with(&PNG_MAGIC) {
      return Err("token icon must be a PNG image".to_string());
  }
  ```

### F-M-34 — Unsafe pointer cast in database registry

- **File:** `database/src/registry.rs` (`impl AsRef<[u8]> for DatabaseStorePrefixes`, ~line 87)
- **Current:** `std::slice::from_ref(unsafe { &*(self as *const Self as *const u8) })`
- **Fix:** Use the safe alternative (enum is `#[repr(u8)]` + `Copy`):
  ```rust
  impl AsRef<[u8]> for DatabaseStorePrefixes {
      fn as_ref(&self) -> &[u8] {
          // SAFETY: DatabaseStorePrefixes is #[repr(u8)] and Copy, so `*self as u8`
          // is a valid single-byte value; from_ref yields a 1-byte slice.
          std::slice::from_ref(&(*self as u8))
      }
  }
  ```

### F-M-35 — Fallback to world-readable /tmp for metadata

- **File:** `zyanya-explorer/src/client.rs` (fallback writes ~lines 293-294, 302, 353-354, 365-369)
- **Fix:** Use `0600` permissions on fallback files and `0700` on fallback dirs. Replace `std::fs::write("/tmp/...", ...)` with `OpenOptions`:
  ```rust
  use std::os::unix::fs::OpenOptionsExt; // explorer is a native server binary
  fn write_private(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
      let mut f = std::fs::OpenOptions::new()
          .write(true).create(true).truncate(true)
          .mode(0o600)
          .open(path)?;
      use std::io::Write;
      f.write_all(data)
  }
  ```
  - Metadata fallback (`/tmp/zyanya-token-metadata.json`): use `write_private`.
  - Icon fallback dir (`/tmp/zyanya-token-icons`): create with `std::fs::DirBuilder::new().recursive(true).mode(0o700).create(...)` (Unix), then `write_private` for the icon file.
  - Keep the existing primary-path behavior unchanged; only harden the fallback.
  - If non-Unix compilation must be preserved, gate `.mode(...)` behind `#[cfg(unix)]` with a plain `OpenOptions` fallback.

### F-M-36 — Reflected XSS via error messages in DAG block modal

- **File:** `zyanya-explorer/src/web.rs` (~line 5798)
- **Current state:** the modal error handler already uses `escapeHtml(err.message)`:
  `body.innerHTML = `<div ...>Failed to load block details: ${escapeHtml(err.message)}</div>`;`
- **Action:** Verify this is the only error path in `showBlockModal` and that `escapeHtml` is in scope (it is, defined ~line 5377). If already escaped, no code change is required — record as verified. If any other unescaped `err.message`/dynamic value is found in the DAG modal, wrap it with `escapeHtml(...)`.

### F-M-37 — Integer overflow in DAG pagination (`limit + offset`)

- **File:** `zyanya-explorer/src/api.rs` (`api_dag_handler`, ~lines 181-183)
- **Current:** `client.get_dag_graph(limit + offset)` with unbounded `offset`.
- **Fix:** Use checked addition and return 400 on overflow:
  ```rust
  let limit = pagination.limit.unwrap_or(20).min(100);
  let offset = pagination.offset.unwrap_or(0);
  let end = match limit.checked_add(offset) {
      Some(v) => v,
      None => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "pagination limit + offset overflow" }))).into_response(),
  };
  match client.get_dag_graph(end).await { ... }
  ```

### F-M-38 — Database connection builder panics on DB open failure

- **File:** `database/src/db/conn_builder.rs` (three `build()` methods, ~lines 117, 126, 137)
- **Fix:**
  1. Add an error type (in this file or a small `error` module):
     ```rust
     #[derive(Debug, thiserror::Error)]
     pub enum ConnBuilderError {
         #[error("fd budget error: {0}")]
         FdBudget(#[from] zyanya_utils::fd_budget::Error),
         #[error("rocksdb open error: {0}")]
         RocksDb(#[from] rocksdb::Error),
         #[error("database path is not valid UTF-8")]
         NonUtf8Path,
     }
     ```
  2. Change all three `build()` signatures to `Result<Arc<DB>, ConnBuilderError>`.
  3. Replace `self.db_path.to_str().unwrap()` with `self.db_path.to_str().ok_or(ConnBuilderError::NonUtf8Path)?`.
  4. Replace `<DBWithThreadMode<MultiThreaded>>::open(&opts, path).unwrap()` with `...open(&opts, path).map_err(ConnBuilderError::RocksDb)?` (or `?` via `From`).
  5. The `default_opts!` macro already uses `?` on `acquire_guard`; with `From<fd_budget::Error>` this still compiles.
- **Ripple:** callers use `.build().unwrap()` (`database/src/utils.rs` macros, `testing/integration/src/consensus_integration_tests.rs:1830`) — `ConnBuilderError` must implement `Debug` (it does via `thiserror`). No caller signature changes required.

### F-M-39 — Unbounded `ram_scale` f64

- **Files:** `zyanyad/src/args.rs` (field ~line 80, parse ~line 464), `mining/src/mempool/config.rs` (`apply_ram_scale`, ~lines 126-129)
- **Fix:** Validate at parse time in `Args::parse` (after the `Args { ... }` struct is built, before `Ok(args)`):
  ```rust
  if !args.ram_scale.is_finite() || args.ram_scale <= 0.0 || args.ram_scale >= 1000.0 {
      return Err(clap::Error::raw(
          clap::error::ErrorKind::ValueValidation,
          format!("--ram-scale must be > 0 and < 1000, got {}", args.ram_scale),
      ));
  }
  ```
  (Per the brief: must be `> 0` and `< 1000`.)
- **Optional hardening:** in `apply_ram_scale`, keep the existing `.min(1.0)` clamp; the parse-time validation is the required fix.

---

## Notes for the builder

- Several findings reference stale line numbers in the report; the current locations are given above. Re-locate by symbol/pattern if lines drift.
- F-M-36 appears already fixed (error message already escaped at `web.rs:5798`); verify rather than blindly re-edit.
- F-M-25, F-M-26, F-M-38 change function signatures — update all callers/tests listed, then run the full check.
- F-M-27 cannot return an error without changing the `ConnectionT` trait; use log + benign fallback message.
- F-M-28: log the full error server-side before sanitizing so operators keep diagnostics.
- F-M-31: never emit `Access-Control-Allow-Origin: *`; same-origin reflection only.
- After all edits, run `cargo check --workspace --all-targets` and fix any fallout.
