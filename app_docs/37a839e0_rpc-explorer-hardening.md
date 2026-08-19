# Batch 6 Remediation — RPC, wRPC, gRPC & Explorer Hardening (F-H-04, F-H-23, F-H-24, F-H-25, F-H-26)

**Session (adw_id):** `37a839e0`
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md`
**Verification gate:** `cargo check --workspace --all-targets`
**Diff base:** `0c1db89`
**Net change:** +475 / -16 across 9 tracked files

This write-up documents the security remediation work that shipped under this batch. It
describes *what changed*, *why each change matters*, *where the change lives*, and *how to
verify it*. A companion remediation plan with full implementation guidance is preserved
at `specs/37a839e0_rpc-explorer-hardening.md`.

---

## Summary

Five HIGH-severity findings were remediated in a single batch. All five are exploitable
without authentication against public-facing surfaces (the block explorer's HTTP API and
embedded HTML, the wRPC WebSocket server, and the gRPC conversion layer that handles
peer-supplied protowire data). The fixes are deliberately minimal and scoped per finding:
no consensus, VM-execution, or wallet code was touched.

| ID | Title | Surface | Fix shape |
|----|-------|---------|-----------|
| F-H-04 | Explorer `/api/compile` not write-gated + ZCL parser recursion DoS | HTTP | Gate endpoint + cap parser recursion |
| F-H-23 | wRPC server has no connection limit | WebSocket | Reject connections past 256 |
| F-H-24 | Inconsistent wRPC message-size limits | wRPC proxy + server | Single shared `pub const` |
| F-H-25 | gRPC conversion `expect()` panic vectors | gRPC send path | Non-panicking fallbacks |
| F-H-26 | Reflected XSS via API error strings in `innerHTML` | Embedded HTML | `escapeHtml()` wrap of all error sites |

---

## What Changed and Why

### F-H-04 — Gate `/api/compile` and cap ZCL parser recursion

Two distinct defects under one finding:

1. **Missing write gate.** `api_compile_contract_handler` was the only write-style handler
   in `zyanya-explorer/src/api.rs` that did **not** call `check_write_enabled()`. Every
   other mutating handler (`api_deploy_contract_handler`, `api_invoke_contract_handler`,
   etc.) is gated behind the `ZYANYA_EXPLORER_ENABLE_WRITE` env var. A public explorer that
   had not opted into write mode could still compile arbitrary ZCL source — which is
   expensive work an attacker could hammer at scale. The gate is now the first statement
   of the handler, mirroring its siblings.

2. **Parser stack-overflow DoS.** The ZCL parser in `zyanya-vm/src/compiler/parser.rs` is
   recursive-descent. `parse_expression` recurses through `parse_equality →
   parse_relational → … → parse_primary`, and `parse_primary`'s parenthesised arm calls
   back into `parse_expression`. A payload of `(((…)))` nested a few hundred deep recurses
   one Rust stack frame per level and overflows the stack. A new
   `ParserError::MaxRecursionDepth(usize)` variant, a `depth: usize` counter on `Parser`,
   a `MAX_PARSE_DEPTH = 100` constant, and increment/decrement bookkeeping in
   `parse_expression` bound all expression nesting. The limit of 100 follows the task
   brief (the audit report suggested 256; the prompt's value wins). A unit test
   `test_parser_rejects_deeply_nested_expression` feeds 200-deep parentheses and asserts
   the parser returns `MaxRecursionDepth(100)` rather than crashing.

### F-H-23 — wRPC connection limit

`rpc/wrpc/server/src/server.rs`'s `connect()` accepted every incoming WebSocket with no
upper bound — a textbook resource-exhaustion DoS: an unauthenticated attacker opens
thousands of connections, each holding a `Connection` in `sockets` and a messenger/task
in runtime memory. A `max_connections: usize` field was added to `ServerInner`, initialized
to a named `DEFAULT_MAX_CONNECTIONS = 256` in `Server::new`. `connect()` now takes the
`sockets` mutex, reads `len()`, and rejects with `WebSocketError::Other("wRPC connection
limit reached (…)")` when the cap is hit *before* allocating a connection id. `disconnect`
already removes entries on close, so the count drains naturally. The check-then-insert is
not strictly atomic across concurrent connects (a small overshoot under load is possible),
which is acceptable for DoS mitigation.

### F-H-24 — Shared wRPC message-size constant

The wRPC proxy and server both already used a 64 MB message-size limit, but the proxy
hardcoded the magic number `64 * 1024 * 1024` while the server held a *private* `static
MAX_WRPC_MESSAGE_SIZE` in `service.rs`. That privacy is exactly how drift happens: a future
change to one would silently diverge from the other (the audit report caught the proxy at
1 GB and the server at 128 MB before prior batches normalised both to 64 MB). The fix is a
single source of truth: `service.rs` now declares `pub const MAX_WRPC_MESSAGE_SIZE: usize
= 64 * 1024 * 1024;` (changed from `static` to `const` and made `pub`), and
`proxy/src/main.rs` imports and uses it. The proxy no longer has any literal message-size
value.

### F-H-25 — gRPC conversion panic vectors

The gRPC conversion layer in `rpc/grpc/core/src/convert/` had three `.expect()` calls on
the *send* path (rpc_core → protowire), implemented by the `from!` macro which returns
`Self` rather than `Result` — so `?` propagation isn't available there. Each `expect()`
panics the task/connection on malformed input and constitutes a remote DoS vector (an
attacker who can submit a bad timestamp or non-UTF-8 extra data crashes the conversion
path). The fixes replace panics with non-panicking fallbacks:

- `header.rs` (two sites): `item.timestamp.try_into().expect("…")` →
  `item.timestamp.try_into().unwrap_or(i64::MAX)`. An out-of-range u64 timestamp is
  clamped rather than crashing.
- `message.rs`: `String::from_utf8(item.extra_data.clone()).expect("…")` →
  `String::from_utf8_lossy(&item.extra_data).into_owned()`. Invalid UTF-8 bytes are
  replaced with U+FFFD rather than crashing.

The receive path (protowire → rpc_core, the actual untrusted input direction) already
uses `try_from!` with `?` and returns `RpcResult`; it was not changed. The prompt scoped
F-H-25 to `rpc/grpc/core/src/convert/`; `protocol/p2p/src/convert/net_address.rs` (also
cited in the report) was left untouched.

### F-H-26 — Reflected XSS via API error messages

The block explorer serves three large HTML pages with embedded JavaScript
(`LAUNCH_HTML`, `TOKEN_HTML`, `DAG_HTML`) in `zyanya-explorer/src/web.rs`. Several
`catch`/error branches interpolate server-returned error strings directly into
`innerHTML`. Server error messages can echo user input (e.g. an invalid address), so an
attacker can inject `<img onerror=…>` or `<script>` payloads that execute in the victim's
session — a reflected XSS.

`TOKEN_HTML` already had an `escapeHtml()` helper from a prior batch. The two missing
pages now define one too (`LAUNCH_HTML` and `DAG_HTML`), and **every** dynamic error
interpolation across all three pages is wrapped in `escapeHtml(...)`:

- `LAUNCH_HTML`: `Unsigned tx build error`, `Submission Error`, `Network/Crypto error`.
- `TOKEN_HTML`: `Buy failed`, `Sell failed`, and both `Network error` branches.
- `DAG_HTML`: `Failed to load block details`.

The `escapeHtml` helper escapes `& < > " '` (the five characters that matter for HTML and
attribute-context injection). Non-error `innerHTML` sites (block data, token lists,
social links) were left alone — those are tracked under other findings (F-C-14 / F-C-15 /
F-M-30 / F-M-36) and prior batches.

---

## Files That Carry the Change

| File | Finding(s) | Change |
|------|------------|--------|
| `zyanya-explorer/src/api.rs` | F-H-04 | `api_compile_contract_handler` now calls `check_write_enabled()` first. |
| `zyanya-vm/src/compiler/parser.rs` | F-H-04 | New `MaxRecursionDepth` error variant, `depth` field, `MAX_PARSE_DEPTH = 100`, depth bookkeeping in `parse_expression`, unit test. |
| `rpc/wrpc/server/src/server.rs` | F-H-23 | `max_connections` field on `ServerInner`, `DEFAULT_MAX_CONNECTIONS = 256`, rejection in `connect()`. |
| `rpc/wrpc/server/src/service.rs` | F-H-24 | `MAX_WRPC_MESSAGE_SIZE` changed from private `static` to `pub const`. |
| `rpc/wrpc/proxy/src/main.rs` | F-H-24 | Imports and uses the shared `MAX_WRPC_MESSAGE_SIZE` constant; no more literal. |
| `rpc/grpc/core/src/convert/header.rs` | F-H-25 | Two `.expect()` → `.unwrap_or(i64::MAX)` on timestamp conversion. |
| `rpc/grpc/core/src/convert/message.rs` | F-H-25 | `.expect()` → `String::from_utf8_lossy(...)` on extra_data. |
| `zyanya-explorer/src/web.rs` | F-H-26 | New `escapeHtml()` helpers in `LAUNCH_HTML` and `DAG_HTML`; all eight error `innerHTML` sites wrapped. |
| `specs/37a839e0_rpc-explorer-hardening.md` | (all) | Full remediation plan / builder brief — 382 lines, preserved for traceability. |

---

## How to Use and Verify

### Build gate

```bash
cargo check --workspace --all-targets
```

This is the required gate per the task brief and must pass on a clean checkout.

### Targeted regression checks

```bash
# F-H-04: compile endpoint is now gated
grep -n "check_write_enabled" zyanya-explorer/src/api.rs

# F-H-04: parser depth limit exists
grep -n "MAX_PARSE_DEPTH\|MaxRecursionDepth" zyanya-vm/src/compiler/parser.rs

# F-H-23: connection limit present
grep -n "max_connections\|DEFAULT_MAX_CONNECTIONS" rpc/wrpc/server/src/server.rs

# F-H-24: one shared constant, no 1 GB leftovers under rpc/wrpc/
grep -rn "MAX_WRPC_MESSAGE_SIZE" rpc/wrpc/
grep -rn "1024 \* 1024 \* 1024" rpc/wrpc/   # should be empty

# F-H-25: no panicking expect/unwrap left in production convert code
# (only matches inside #[cfg(test)] mod tests are acceptable)
grep -rn "\.expect\|\.unwrap()\|panic!" rpc/grpc/core/src/convert/*.rs

# F-H-26: no unescaped error interpolation remains
grep -n "innerHTML = .*error\|innerHTML = .*err\." zyanya-explorer/src/web.rs
```

### Parser depth unit test

```bash
cargo test -p zyanya-vm --lib parser::tests::test_parser_rejects_deeply_nested_expression
```

should pass (it feeds 200-deep `(((…)))` and asserts `Err(MaxRecursionDepth(100))`
rather than a stack overflow).

### Runtime behaviour to confirm

- An explorer started **without** `ZYANYA_EXPLORER_ENABLE_WRITE` should return the
  write-disabled error from `POST /api/compile` (previously it would compile).
- A wRPC server at 256 live connections should reject the 257th with a
  `wRPC connection limit reached (256)` error; closing one should allow a new one in.
- Submitting a ZCL program with > 100 levels of expression nesting should yield a
  `Maximum expression nesting depth (100) exceeded` parser error, not a crash.

---

## Notes and Scope Boundaries

- The 1 GB / 128 MB divergence originally flagged in the audit had already been
  normalised to 64 MB by prior batches; this batch's F-H-24 work is *only* the
  shared-constant refactor that prevents future drift.
- `rpc/wrpc/client/src/client.rs` also hardcodes 64 MB and was **not** changed — it is
  out of scope per the prompt (proxy + server only). Its value must not be changed.
- `protocol/p2p/src/convert/net_address.rs` is cited in the audit report under the
  panic-vector theme but is **out of scope** for F-H-25 (prompt scopes to
  `rpc/grpc/core/src/convert/`).
- F-H-25 deliberately stops at non-panicking fallbacks rather than refactoring the
  `from!` macro to `TryFrom`. The macro's `impl_into_zyanyad_request_ex!` family ripples
  through every request type and the client `call()` method; a full Result-ification
  belongs in a separate batch. The fallbacks eliminate the panic vectors with minimal
  blast radius, matching the audit's note that "the specific instances appear safe due to
  upstream length checks."
- The wRPC connection-limit check holds the sockets mutex only for the `len()` read; a
  small concurrent overshoot past 256 is possible. This is acceptable for DoS
  mitigation. Strict atomicity would require holding the lock across check + insert,
  which was not done to keep the change minimal.
- All eight F-H-26 error sites were verified wrapped; non-error `innerHTML` interpolations
  were intentionally left for their own findings.