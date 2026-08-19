# Remediation Plan — Batch 6: RPC, wRPC, gRPC & Explorer Hardening

**Session (adw_id):** `37a839e0`
**Date:** 2026-08-18
**Scope:** HIGH findings F-H-04, F-H-23, F-H-24, F-H-25, F-H-26
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md`
**Verification gate:** `cargo check --workspace --all-targets` must pass.

---

## 1. Objective

Remediate five HIGH-severity findings in the RPC / wRPC / gRPC transport layer and the
block explorer:

| ID | Title | Severity |
|----|-------|----------|
| F-H-04 | Explorer `/api/compile` not write-gated + ZCL parser recursion DoS | HIGH |
| F-H-23 | wRPC server has no connection limit (unbounded WebSockets) | HIGH |
| F-H-24 | Inconsistent wRPC message size limits (proxy vs server) | HIGH |
| F-H-25 | gRPC conversion `expect()`/`unwrap()` panic vectors | HIGH |
| F-H-26 | Reflected XSS via API error messages rendered into `innerHTML` | HIGH |

---

## 2. Files to Audit & Vulnerability Categories

| File | Finding(s) | Vulnerability category |
|------|-----------|------------------------|
| `zyanya-explorer/src/api.rs` | F-H-04 | Missing authorization gate (write endpoint), DoS |
| `zyanya-vm/src/compiler/parser.rs` | F-H-04 | Unbounded recursion → stack overflow DoS |
| `rpc/wrpc/server/src/server.rs` | F-H-23 | Resource exhaustion (unbounded connections) |
| `rpc/wrpc/server/src/service.rs` | F-H-24 | Inconsistent message-size limit (shared constant) |
| `rpc/wrpc/proxy/src/main.rs` | F-H-24 | Inconsistent message-size limit (shared constant) |
| `rpc/grpc/core/src/convert/header.rs` | F-H-25 | Panic on conversion (`.expect()`) |
| `rpc/grpc/core/src/convert/message.rs` | F-H-25 | Panic on conversion (`.expect()`) |
| `zyanya-explorer/src/web.rs` | F-H-26 | Reflected XSS (untrusted error text → `innerHTML`) |

---

## 3. Priority Order (CRITICAL first)

All five are HIGH. Within the batch, order by exploitability / blast radius:

1. **F-H-04** — unauthenticated remote DoS on public explorers (parser stack overflow) +
   missing write gate. Fix first.
2. **F-H-23** — unauthenticated resource exhaustion (unbounded WebSocket connections).
3. **F-H-24** — memory-exhaustion DoS via oversized proxied wRPC messages (1 GB path).
4. **F-H-25** — panic vectors in gRPC conversion (task/connection drop DoS).
5. **F-H-26** — reflected XSS in explorer (client-side, but HIGH).

---

## 4. Detailed Remediation Plan

### 4.1 F-H-04 — Gate `/api/compile` + ZCL parser recursion depth limit

**Files:**
- `zyanya-explorer/src/api.rs` — `api_compile_contract_handler` (line 583)
- `zyanya-vm/src/compiler/parser.rs` — `Parser` struct (line 14), `ParserError` (line 6),
  `parse_expression` (line 200)

**Fix A — write-gate the endpoint (`api.rs`):**

`api_compile_contract_handler` currently does **not** call `check_write_enabled()`
(unlike every other write endpoint). Add the gate as the first statement, mirroring
`api_deploy_contract_handler` / `api_invoke_contract_handler`:

```rust
pub async fn api_compile_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<CompileContractReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.compile_contract(&payload.source) {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}
```

`check_write_enabled()` is already defined at `api.rs:313` and reads
`ZYANYA_EXPLORER_ENABLE_WRITE` (accepts `1` / `true`, case-insensitive). No new helper
needed.

**Fix B — recursion depth limit (`parser.rs`):**

The parser is recursive-descent: `parse_expression → parse_equality → parse_relational →
parse_additive → parse_multiplicative → parse_primary → (LParen) → parse_expression`.
Deeply nested input (`((((...))))`) recurses one Rust frame per nesting level and can
overflow the stack.

1. Add a new error variant to `ParserError` (line 6):

```rust
#[error("Maximum expression nesting depth ({0}) exceeded")]
MaxRecursionDepth(usize),
```

2. Add a `depth: usize` field to `struct Parser` (line 14) and initialize it in
   `Parser::new` (line 20): `Self { tokens, pos: 0, depth: 0 }`.

3. Add a module-level constant: `const MAX_PARSE_DEPTH: usize = 100;`

4. In `parse_expression` (line 200), increment the depth, reject beyond the limit, and
   decrement on every exit path:

```rust
fn parse_expression(&mut self) -> Result<Expression, ParserError> {
    self.depth += 1;
    if self.depth > MAX_PARSE_DEPTH {
        self.depth -= 1;
        return Err(ParserError::MaxRecursionDepth(MAX_PARSE_DEPTH));
    }
    let result = self.parse_equality();
    self.depth -= 1;
    result
}
```

> Note: `parse_expression` is the single recursion entry point (called from
> `parse_statement`, `parse_args`, and `parse_primary`'s `LParen` arm), so guarding it
> bounds all expression nesting. The limit of 100 is per the task brief (the report
> suggested 256; use **100** as instructed).

**Verification:** add a unit test in `parser.rs` `mod tests` that feeds
`"(".repeat(200) + "1" + ")".repeat(200)` (or a deeply nested expression) and asserts
`parse_program()` returns `Err(ParserError::MaxRecursionDepth(_))` rather than crashing.

---

### 4.2 F-H-23 — wRPC server connection limit

**File:** `rpc/wrpc/server/src/server.rs` (primary), `rpc/wrpc/server/src/service.rs`
(optional config plumbing)

**Current state:** `ServerInner` (line 40) holds `sockets: Mutex<HashMap<u64, Connection>>`
and `next_connection_id: AtomicU64`. `connect` (line 112) registers every connection with
no cap.

**Fix:**

1. Add a `max_connections: usize` field to `struct ServerInner` (line 40).

2. In `Server::new` (line ~50), initialize it. Default **256**. If plumbing through
   `Options` is desired, add `pub max_connections: usize` to `Options` in `service.rs`
   (line 19) with `Default` = 256, and read `options.max_connections` in `Server::new`.
   Otherwise hardcode `256` in `Server::new` with a named constant
   `const DEFAULT_MAX_CONNECTIONS: usize = 256;`.

3. In `connect` (line 112), reject new connections when the limit is reached. Do the
   check while holding the sockets lock so the count is authoritative:

```rust
pub async fn connect(&self, peer: &SocketAddr, messenger: Arc<Messenger>, auth_token: Option<String>) -> Result<Connection> {
    // Reject when the connection limit is reached (F-H-23)
    {
        let sockets = self.inner.sockets.lock()?;
        if sockets.len() >= self.inner.max_connections {
            return Err(WebSocketError::Other(format!(
                "wRPC connection limit reached ({})",
                self.inner.max_connections
            )).into());
        }
    }
    // ... existing connection setup ...
    self.inner.sockets.lock()?.insert(id, connection.clone());
    Ok(connection)
}
```

> `WebSocketError::Other(String)` is already used at `server.rs:131` and converts into
> `crate::error::Error` via `#[from]`. No new error variant is required.

**Notes:**
- The check-then-insert is not strictly atomic (two concurrent connects can both pass),
  but the lock is held for the authoritative `len()` read; a small overshoot under
  concurrent load is acceptable for a DoS mitigation. If strictness is required, hold the
  lock across the check and insert.
- `disconnect` (line 147) already removes entries, so the count decreases on close.

---

### 4.3 F-H-24 — Shared wRPC message-size constant (64 MB)

**Files:**
- `rpc/wrpc/server/src/service.rs` — `MAX_WRPC_MESSAGE_SIZE` (line 16)
- `rpc/wrpc/proxy/src/main.rs` — `WebSocketConfig` (line ~99)

**Current state:** Both are already 64 MB, but the proxy hardcodes the magic number
`64 * 1024 * 1024` while the server uses a private `static`. The remaining work is to
make the limit a single shared constant.

**Fix:**

1. In `service.rs` (line 16), change the private `static` to a public `const`:

```rust
pub const MAX_WRPC_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64MB
```

2. In `proxy/src/main.rs`, import the constant and use it:

```rust
use zyanya_wrpc_server::service::{Options, ZyanyaRpcHandler, MAX_WRPC_MESSAGE_SIZE};
// ...
let config = WebSocketConfig { max_message_size: Some(MAX_WRPC_MESSAGE_SIZE), ..Default::default() };
```

**Notes:**
- `rpc/wrpc/client/src/client.rs` (line ~443) also hardcodes 64 MB. It is out of scope
  for this batch (prompt lists proxy + server only), but the builder may optionally
  switch it to the shared constant for consistency if the import is available from the
  client crate. Do **not** change its value.
- Confirm no other wRPC path still uses 1 GB (the report's 1 GB proxy/client values have
  already been lowered in prior batches; re-verify with a grep for `1024 * 1024 * 1024`
  under `rpc/wrpc/`).

---

### 4.4 F-H-25 — gRPC conversion `expect()`/`unwrap()` panic vectors

**Files:**
- `rpc/grpc/core/src/convert/header.rs` — lines 18, 35
- `rpc/grpc/core/src/convert/message.rs` — line 159

**Context:** The `from!` macro (send path, rpc_core → protowire) implements `From` and
returns `Self` (not `Result`), so `?` propagation is not available there. The receive
path (protowire → rpc_core, the actual untrusted input) already uses `try_from!` with
`?` and returns `RpcResult`. The three remaining `.expect()` calls are on the send path
but still constitute panic vectors; replace them with non-panicking conversions.

**Fix:**

1. `header.rs:18` and `header.rs:35` — `u64` timestamp → protobuf `int64`:

```rust
// before
timestamp: item.timestamp.try_into().expect("timestamp is always convertible to i64"),
// after
timestamp: item.timestamp.try_into().unwrap_or(i64::MAX),
```

2. `message.rs:159` — `Vec<u8>` extra_data → protobuf `string`:

```rust
// before
extra_data: String::from_utf8(item.extra_data.clone()).expect("extra data has to be valid UTF-8"),
// after
extra_data: String::from_utf8_lossy(&item.extra_data).into_owned(),
```

3. **Audit sweep:** after the two edits, re-run the grep below and confirm the only
   remaining `.expect()`/`.unwrap()`/`panic!`/`unreachable!`/`assert!` hits in
   `rpc/grpc/core/src/convert/` are inside `#[cfg(test)] mod tests` (header.rs:108+,
   message.rs:1257+). `.unwrap_or_default()` / `.unwrap_or(0)` are non-panicking and
   must be left as-is.

```bash
grep -rn "\.expect\|\.unwrap\|panic!\|unreachable!\|assert!\|assert_eq!" rpc/grpc/core/src/convert/*.rs
```

**Notes:**
- Do **not** attempt a full `From` → `TryFrom` refactor of the request-serialization
  macros (`impl_into_zyanyad_request_ex!` in `zyanyad.rs`) in this batch; it ripples
  through every request type and the client `call()` method. The non-panicking fallbacks
  above eliminate the panic vectors with minimal blast radius, which matches the
  finding's "the specific instances appear safe due to upstream length checks" note.
- `protocol/p2p/src/convert/net_address.rs:47-50` is cited in the report but is **out of
  scope** for this batch (prompt scopes F-H-25 to `rpc/grpc/core/src/convert/`).

---

### 4.5 F-H-26 — Reflected XSS via API error messages in `innerHTML`

**File:** `zyanya-explorer/src/web.rs`

**Context:** Server error strings can include user input (e.g. an invalid address echoed
back), and the embedded JS interpolates them into `innerHTML` without escaping. An
`escapeHtml()` helper already exists in two script blocks (lines 1228 and 3328), but the
deploy-token page (`LAUNCH_HTML`, script at 2879–3106) and the DAG page (`DAG_HTML`,
script at 5363–5929) do **not** define it.

**Sites to fix (current line numbers):**

| Line | Script block | Message |
|------|-------------|---------|
| 3045 | LAUNCH_HTML | `Unsigned tx build error: ${unsignedData.error \|\| 'Failed'}` |
| 3097 | LAUNCH_HTML | `Submission Error: ${submitData.error \|\| 'Submit failed'}` |
| 3101 | LAUNCH_HTML | `Network/Crypto error: ${err.message}` |
| 3451 | TOKEN_HTML | `Buy failed: ${data.error \|\| 'Unknown error'}` |
| 3454 | TOKEN_HTML | `Network error: ${err.message}` |
| 3479 | TOKEN_HTML | `Sell failed: ${data.error \|\| 'Unknown error'}` |
| 3482 | TOKEN_HTML | `Network error: ${err.message}` |
| 5776 | DAG_HTML | `Failed to load block details: ${err.message}` |

**Fix:**

1. **LAUNCH_HTML script (2879–3106):** add the same `escapeHtml` helper (copy the
   implementation from line 1228) near the top of the script, then wrap every dynamic
   error value:

```js
statusEl.innerHTML = `<span style="color: var(--burn-red)">Unsigned tx build error: ${escapeHtml(unsignedData.error || 'Failed')}</span>`;
statusEl.innerHTML = `<span style="color: var(--burn-red)">Submission Error: ${escapeHtml(submitData.error || 'Submit failed')}</span>`;
statusEl.innerHTML = `<span style="color: var(--burn-red)">Network/Crypto error: ${escapeHtml(err.message)}</span>`;
```

2. **TOKEN_HTML script (3323–3487):** `escapeHtml` already exists (line 3328). Wrap the
   four buy/sell error values (lines 3451, 3454, 3479, 3482) with `escapeHtml(...)`.

3. **DAG_HTML script (5363–5929):** add the `escapeHtml` helper and wrap the block-detail
   error at line 5776:

```js
body.innerHTML = `<div style="color: var(--side-red); padding: 1rem;">Failed to load block details: ${escapeHtml(err.message)}</div>`;
```

4. **Sweep:** re-run the grep below and confirm no remaining `innerHTML` interpolation of
   `error` / `err.message` / `data.error` / `submitData.error` / `unsignedData.error` is
   unescaped:

```bash
grep -n "innerHTML = .*error\|innerHTML = .*err\.\|innerHTML = .*\.message\|innerHTML = .*data\.error\|innerHTML = .*submitData\|innerHTML = .*unsignedData" zyanya-explorer/src/web.rs
```

**Notes:**
- The `escapeHtml` helper must escape `& < > " '` (the existing implementation does).
- Do not change the non-error `innerHTML` sites (block data, token lists, social links)
  in this batch — those are covered by F-C-14/F-C-15/F-M-30/F-M-36 and prior batches.
  Only the error-message sites listed above are in scope for F-H-26.

---

## 5. Verification

After applying all fixes:

```bash
cargo check --workspace --all-targets
```

Additional targeted checks (optional but recommended):

```bash
# F-H-04: compile endpoint gated
grep -n "check_write_enabled" zyanya-explorer/src/api.rs

# F-H-23: connection limit present
grep -n "max_connections" rpc/wrpc/server/src/server.rs

# F-H-24: single shared constant, no 1GB leftovers
grep -rn "MAX_WRPC_MESSAGE_SIZE\|1024 \* 1024 \* 1024" rpc/wrpc/

# F-H-25: no panicking expect/unwrap left in production convert code
grep -rn "\.expect\|\.unwrap\|panic!\|unreachable!" rpc/grpc/core/src/convert/*.rs

# F-H-26: no unescaped error interpolation
grep -n "innerHTML = .*error\|innerHTML = .*err\." zyanya-explorer/src/web.rs
```

Run the parser unit test added in 4.1 to confirm the depth limit returns an error rather
than overflowing the stack.

---

## 6. Notes for the Builder

- The working tree is currently clean on branch `security-fixes`. Prior batches have
  already lowered the wRPC proxy/client message sizes to 64 MB and added `escapeHtml`
  helpers in two of the three explorer script blocks — do not re-do that work; only apply
  the deltas described above.
- F-H-24's remaining work is **only** the shared-constant refactor (both values are
  already 64 MB).
- F-H-25's remaining work is **only** the three `.expect()` sites; the receive path is
  already `Result`-based.
- Keep changes minimal and scoped per finding. Do not touch consensus, VM execution, or
  wallet code in this batch.
- If `cargo check` surfaces unrelated pre-existing warnings/errors, do not fix them here
  unless they block the gate; report them instead.
