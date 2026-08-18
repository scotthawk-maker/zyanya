# Remediation — MEDIUM Security Findings Batch M2 (F-M-23 through F-M-39)

**Session:** a09ae562
**Scope:** Network, RPC, Explorer, Database MEDIUM findings from `audit_reports/FINAL_AUDIT_REPORT.md`.
**Base commit:** fca4ca1 (+670 −44 across 21 files)
**Verification:** `cargo check --workspace --all-targets` must pass.

---

## What changed and why it matters

Seventeen MEDIUM-severity findings were remediated in this batch. Each fix is annotated in-code with its finding ID (e.g. `// F-M-23: ...`). The fixes fall into four thematic groups:

### 1. Removing panics and `unsafe` (defense-in-depth)

| Finding | File(s) | Change |
|---|---|---|
| **F-M-23** | `rpc/core/src/model/hex_cnv.rs` | Replaced `unsafe { str::from_utf8_unchecked(&hex) }` with safe `String::from_utf8(hex).expect("hex output is always ASCII")`. `faster_hex::hex_encode` only emits ASCII bytes, so the unsafe shortcut was sound but unnecessary; the safe path eliminates the UB risk if the upstream contract is ever violated. |
| **F-M-25** | `components/addressmanager/src/lib.rs`, `components/connectionmanager/src/lib.rs` | `RandomWeightedIterator::new` and `Store::iterate_prioritized_random_addresses` now return `Result<_, WeightedError>` instead of `panic!`-ing on a non-recoverable `WeightedError`. The `Iterator::next` `update_weights` error arm now logs via `crate::log_weighted_error` and degrades to an empty iterator (`self.weighted_index = None`) instead of panicking. The sole caller (`ConnectionManager::handle_outbound_connections`) handles the `Result` by logging and skipping that connection round. |
| **F-M-26** | `rpc/wrpc/server/src/server.rs`, `rpc/wrpc/server/src/service.rs` | `Server::disconnect` now returns `Result<(), WebSocketError>` and uses `lock().map_err(...)` instead of `.lock().unwrap()`, so a poisoned sockets mutex no longer crashes the server task. The `RpcHandler::disconnect` caller logs the error. |
| **F-M-27** | `rpc/wrpc/server/src/connection.rs` | `ConnectionT::into_message` (which returns `Self::Message`, not `Result`) replaced its `.unwrap()` on notification serialization with a `match` that logs via `workflow_log::log_error!` and falls back to a benign `Message::Close(None)`. The trait signature is unchanged. |
| **F-M-34** | `database/src/registry.rs` | The `AsRef<[u8]> for DatabaseStorePrefixes` impl retained its `unsafe` pointer cast but gained a detailed `SAFETY:` comment block documenting why the `#[repr(u8)]` + `Copy` layout makes the cast valid, what is read, and that the returned slice borrows from `self`. |
| **F-M-38** | `database/src/db/conn_builder.rs` | Added `ConnBuilderError` (`FdBudget`, `RocksDb`, `NonUtf8Path`) via `thiserror`. All three `ConnBuilder::build` methods now return `Result<Arc<DB>, ConnBuilderError>` and use `?` / `map_err` instead of `.unwrap()` on the RocksDB open and the `to_str()` path conversion. |

### 2. Limiting amplification and leakage (network / RPC hardening)

| Finding | File(s) | Change |
|---|---|---|
| **F-M-24** | `protocol/flows/src/v5/txrelay/flow.rs` | `RequestTransactionsFlow::start_impl` now caps honored transaction IDs per `RequestTransactionsMessage` to `MAX_TX_PER_REQUEST = 100`, logging and truncating when exceeded. Prevents a malicious peer from forcing unbounded transaction relay (bandwidth amplification). |
| **F-M-28** | `rpc/core/src/error.rs`, `rpc/grpc/core/src/ops.rs`, `rpc/wrpc/server/src/error.rs` | New `RpcError::sanitize()` method collapses internal-detail variants (`ConsensusError`, `MiningManagerError`, `RpcSubsystem`, etc.) to `General("internal error")` and strips `RejectedTransaction` rejection reasons, while user-facing variants pass through unchanged. Applied at the gRPC response boundary (`to_error_response` macro now calls `error.sanitize()` and logs the full error first) and the wRPC boundary (manual `From<RpcError> for Error` that calls `.sanitize()`). Operators retain full diagnostics via the server-side `log::error!` added before sanitization. |
| **F-M-29** | `protocol/flows/src/flowcontext/transactions.rs` | `MAX_INV_PER_TX_INV_MSG` reduced from `131_072` to `10_000`, limiting the size of transaction-inventory messages a node will process. |
| **F-M-32** | `notify/src/notifier.rs`, `notify/src/subscriber.rs` | Both notification channels changed from `Channel::unbounded()` to `Channel::bounded(1000)`, preventing unbounded queue growth / OOM when a broadcaster or subscriber cannot keep up. Send paths already use `try_send`, so a full channel returns an error instead of blocking. |

### 3. Explorer web-security fixes (XSS, CORS, upload validation, file perms, overflow)

| Finding | File(s) | Change |
|---|---|---|
| **F-M-30** | `zyanya-explorer/src/web.rs` | All interpolated block fields in the DAG visualization tooltip and modal (`node.short_hash`, parent hashes, `node.selected_parent`, `block.hash`, `block.parents`, `block.selected_parent`) are now wrapped in `escapeHtml(...)`. The existing `escapeHtml()` helper (~line 5377) is reused. Closes reflected XSS via block data. |
| **F-M-31** | `zyanya-explorer/src/main.rs` | New `cors_same_origin` axum middleware reflects the request `Origin` as `Access-Control-Allow-Origin` only when the origin's authority matches the request `Host` (same-origin). Handles `OPTIONS` preflight with `204` + `Access-Control-Allow-Methods` / `Access-Control-Allow-Headers`. Never emits `*`. Applied via `.layer(middleware::from_fn(cors_same_origin))` on the router. |
| **F-M-33** | `zyanya-explorer/src/client.rs` | `save_token_icon` now validates the decoded upload: rejects icons > 1 MB (`MAX_ICON_SIZE`) and requires a PNG magic-byte header (`PNG_MAGIC`). Prevents polyglot / disk-exhaustion abuse via token icon uploads. |
| **F-M-35** | `zyanya-explorer/src/client.rs` | New `write_private` (0600 file mode) and `create_private_dir` (0700 dir mode) helpers, gated with a `FileModePrivate` trait that is a no-op on non-Unix targets. The `/tmp/zyanya-token-metadata.json` fallback and the `/tmp/zyanya-token-icons` fallback directory now use these private permissions so token metadata/icons are not world-readable. |
| **F-M-36** | `zyanya-explorer/src/web.rs` | Verified — the DAG block modal error handler already used `escapeHtml(err.message)`. No code change was required; recorded as confirmed-safe. |
| **F-M-37** | `zyanya-explorer/src/api.rs` | `api_dag_handler` now uses `limit.checked_add(offset)` and returns `400 BAD_REQUEST` with a JSON error on overflow, instead of silently wrapping `limit + offset`. |

### 4. Config validation

| Finding | File(s) | Change |
|---|---|---|
| **F-M-39** | `zyanyad/src/args.rs` | `Args::parse` now validates `ram_scale` at parse time: must be finite, `> 0.0`, and `< 1000.0`. Returns a clap `ValueValidation` error otherwise, preventing unbounded mempool/memory scaling from a misconfigured or adversarial CLI value. |

### Supporting artifact

A full remediation plan (`specs/a09ae562_network-rpc-explorer-db.md`, +324 lines) was committed alongside the code, documenting each finding's rationale, exact code location, fix strategy, ripple effects, and builder notes.

---

## Files that carry the change

| File | Findings |
|---|---|
| `components/addressmanager/src/lib.rs` | F-M-25 |
| `components/connectionmanager/src/lib.rs` | F-M-25 (caller) |
| `database/src/db/conn_builder.rs` | F-M-38 |
| `database/src/registry.rs` | F-M-34 |
| `notify/src/notifier.rs` | F-M-32 |
| `notify/src/subscriber.rs` | F-M-32 |
| `protocol/flows/src/flowcontext/transactions.rs` | F-M-29 |
| `protocol/flows/src/v5/txrelay/flow.rs` | F-M-24 |
| `rpc/core/src/error.rs` | F-M-28 |
| `rpc/core/src/model/hex_cnv.rs` | F-M-23 |
| `rpc/grpc/core/src/ops.rs` | F-M-28 |
| `rpc/wrpc/server/src/connection.rs` | F-M-27 |
| `rpc/wrpc/server/src/error.rs` | F-M-28 |
| `rpc/wrpc/server/src/server.rs` | F-M-26 |
| `rpc/wrpc/server/src/service.rs` | F-M-26 (caller) |
| `specs/a09ae562_network-rpc-explorer-db.md` | plan document |
| `zyanya-explorer/src/api.rs` | F-M-37 |
| `zyanya-explorer/src/client.rs` | F-M-33, F-M-35 |
| `zyanya-explorer/src/main.rs` | F-M-31 |
| `zyanya-explorer/src/web.rs` | F-M-30, F-M-36 (verified) |
| `zyanyad/src/args.rs` | F-M-39 |

---

## How to verify

1. **Compile check (required gate):**
   ```bash
   cargo check --workspace --all-targets
   ```
   The signature changes in F-M-25, F-M-26, F-M-27, and F-M-38 were propagated to all callers and tests; the workspace should compile cleanly.

2. **Targeted spot-checks:**
   - F-M-23: grep for `from_utf8_unchecked` in `rpc/core/src/model/hex_cnv.rs` — should be gone.
   - F-M-25: `RandomWeightedIterator::new` and `iterate_prioritized_random_addresses` return `Result`; the connection manager caller handles `Err`.
   - F-M-28: `RpcError::sanitize` exists; gRPC `to_error_response` calls `.sanitize()`; wRPC `Error::from(RpcError)` calls `.sanitize()`.
   - F-M-31: `/api/*` responses from a same-origin browser include `Access-Control-Allow-Origin`; cross-origin requests do not.
   - F-M-33: an icon upload that is not a PNG or exceeds 1 MB returns an error string.
   - F-M-37: requesting `/api/dag` with `limit` and `offset` that sum past `usize::MAX` returns HTTP 400.
   - F-M-39: `zyanyad --ram-scale 0` or `--ram-scale 5000` fails at parse time with a clap error.

3. **Audit trail:** each in-code fix is tagged with its finding ID (`F-M-NN:`) in a comment, making it straightforward to grep for and re-verify against the audit report.