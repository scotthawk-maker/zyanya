# Remediation Plan — Group D: Network/RPC fixes (F-C-10, F-C-11, F-C-12, F-C-13)

**Date:** 2026-08-18
**Scope:** CRITICAL findings F-C-10, F-C-11, F-C-12, F-C-13 from `audit_reports/FINAL_AUDIT_REPORT.md`.
**Verification gate:** `source /root/.cargo/env && cargo check -p rpc/service -p rpc/grpc -p rpc/wrpc 2>&1` (no full build, no tests).

---

## 0. Context & current state (read first)

- The audit report line numbers are **stale** for `rpc/service/src/service.rs`: the F-C-01 fix (remove the consensus-bypass fallback in `deploy_contract_call` / `invoke_contract_call`) has **already been applied**. Both handlers now submit via `submit_rpc_transaction` only. Do not re-introduce a fallback.
- Current contract handler locations in `rpc/service/src/service.rs`:
  - `deploy_contract_call` — line 586
  - `invoke_contract_call` — line 632
  - `get_contract_state_call` — line 679 (`.try_into().unwrap()` at **684**)
  - `get_contract_code_call` — line 690
  - `call_contract_call` — line 700 (`.try_into().unwrap()` at **750**)
  - `shutdown_call` — line 1146
- `RpcHash = zyanya_hashes::Hash` (`rpc/core/src/model/hash.rs`). `Hash::as_bytes(self) -> [u8; 32]` returns the array **by value** (`crypto/hashes/src/lib.rs:50`). This matters for F-C-11 (see below).
- The gRPC and wRPC server macros currently pass `None` as the `connection` argument to every `*_call` method (`rpc/macros/src/grpc/server.rs:76`, `rpc/macros/src/wrpc/server.rs:57`). This matters for F-C-13.

---

## 1. Shared constants (new)

Add a new module to `consensus/core/src/config/constants.rs` (accessible from both `zyanya_consensus_core` consumers — the RPC service and the consensus transaction validator):

```rust
pub mod contract {
    /// Maximum size of a deployed contract bytecode (1 MB).
    pub const MAX_CONTRACT_BYTECODE_SIZE: usize = 1024 * 1024;
    /// Maximum size of contract call data / parameters in bytes (64 KB).
    pub const MAX_CONTRACT_CALLDATA_SIZE: usize = 64 * 1024;
    /// Maximum number of u64 parameters (64 KB / 8).
    pub const MAX_CONTRACT_PARAMETERS: usize = MAX_CONTRACT_CALLDATA_SIZE / 8;
    /// Upper bound on the serialized contract payload before borsh deserialization.
    /// Covers the largest deploy (bytecode) plus invoke (parameters) plus fixed field overhead.
    pub const MAX_CONTRACT_PAYLOAD_SIZE: usize =
        MAX_CONTRACT_BYTECODE_SIZE + MAX_CONTRACT_CALLDATA_SIZE + 1024;
    /// Server-side cap on client-supplied max_gas (CPU DoS hardening).
    pub const MAX_CONTRACT_MAX_GAS: u64 = 100_000_000;
}
```

Import path: `zyanya_consensus_core::config::constants::contract::*`.

---

## 2. F-C-10 — Unbounded bytecode/calldata (memory DoS + VM gas bypass)

### 2a. RPC handlers — `rpc/service/src/service.rs`

Add `use zyanya_consensus_core::config::constants::contract::{MAX_CONTRACT_BYTECODE_SIZE, MAX_CONTRACT_CALLDATA_SIZE, MAX_CONTRACT_PARAMETERS, MAX_CONTRACT_MAX_GAS};` (or import the module).

1. **`deploy_contract_call` (line 586)** — at the very top, before building the payload:
   ```rust
   if request.bytecode.len() > MAX_CONTRACT_BYTECODE_SIZE {
       return Err(RpcError::General(format!(
           "bytecode size {} exceeds maximum {}",
           request.bytecode.len(), MAX_CONTRACT_BYTECODE_SIZE
       )));
   }
   if request.max_gas > MAX_CONTRACT_MAX_GAS {
       return Err(RpcError::General(format!("max_gas {} exceeds maximum {}", request.max_gas, MAX_CONTRACT_MAX_GAS)));
   }
   ```

2. **`invoke_contract_call` (line 632)** — same pattern:
   ```rust
   if request.parameters.len() > MAX_CONTRACT_PARAMETERS {
       return Err(RpcError::General(format!(
           "parameters length {} exceeds maximum {}",
           request.parameters.len(), MAX_CONTRACT_PARAMETERS
       )));
   }
   if request.max_gas > MAX_CONTRACT_MAX_GAS { /* reject as above */ }
   ```

3. **`call_contract_call` (line 700)** — before constructing the VM / pushing calldata:
   ```rust
   if request.calldata.len() > MAX_CONTRACT_CALLDATA_SIZE {
       return Err(RpcError::General(format!(
           "calldata size {} exceeds maximum {}",
           request.calldata.len(), MAX_CONTRACT_CALLDATA_SIZE
       )));
   }
   if request.max_gas > MAX_CONTRACT_MAX_GAS { /* reject as above */ }
   ```

> Note: `max_gas` cap is part of the F-C-10 finding text ("set `max_gas = u64::MAX` to force the VM to run for an arbitrary duration"). It is included here as hardening; if you want to keep the change minimal, the bytecode/calldata checks are the mandatory part.

### 2b. Consensus validation — `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs`

In `check_contract_payload_in_isolation` (line ~157), **before** `ContractPayload::from_slice(&tx.payload)`:

```rust
use zyanya_consensus_core::config::constants::contract::{
    MAX_CONTRACT_BYTECODE_SIZE, MAX_CONTRACT_CALLDATA_SIZE, MAX_CONTRACT_PARAMETERS, MAX_CONTRACT_PAYLOAD_SIZE,
};

// Reject oversized payloads BEFORE borsh deserialization (borsh reads a u32 length
// prefix and allocates that many elements — an OOM vector).
if tx.payload.len() > MAX_CONTRACT_PAYLOAD_SIZE {
    return Err(TxRuleError::InvalidContractPayload(format!(
        "contract payload size {} exceeds maximum {}",
        tx.payload.len(), MAX_CONTRACT_PAYLOAD_SIZE
    )));
}
```

Then, after the existing `match payload { ... }` checks, add per-variant size checks:

- `Deploy(deploy)`: `if deploy.bytecode.len() > MAX_CONTRACT_BYTECODE_SIZE { return Err(TxRuleError::InvalidContractPayload("bytecode too large".to_string())); }`
- `Invoke(invoke)`: `if invoke.parameters.len() > MAX_CONTRACT_PARAMETERS { return Err(TxRuleError::InvalidContractPayload("too many parameters".to_string())); }`

> These are consensus rules (a node that rejects a tx another node accepts forks), so the constants must be identical across all nodes. Using shared constants in `zyanya_consensus_core` guarantees this. Do **not** make them per-node config.

---

## 3. F-C-11 — Panic vectors: `.try_into().unwrap()` on contract address

File: `rpc/service/src/service.rs`, lines **684** (`get_contract_state_call`) and **750** (`call_contract_call`).

Current code:
```rust
let addr_bytes: [u8; 32] = request.contract_address.as_bytes().try_into().unwrap();
```

**Key fact:** `Hash::as_bytes(self)` already returns `[u8; 32]` by value, so the `.try_into().unwrap()` is a redundant infallible conversion. The cleanest fix is to drop it entirely:

```rust
let addr_bytes: [u8; 32] = request.contract_address.as_bytes();
```

If you prefer to keep a defensive conversion (in case `as_bytes()` ever changes signature), use the fallible path with proper error handling instead of `unwrap()`:

```rust
let addr_bytes: [u8; 32] = request.contract_address
    .as_bytes()
    .try_into()
    .map_err(|_| RpcError::General("invalid contract address length".to_string()))?;
```

Apply the same change at both call sites (684 and 750). Do **not** leave any `.unwrap()`/`.expect()` on request-derived data in these handlers.

> Out of scope but noted: `service.rs:524` has `.expect("script public key is convertible into an address")` on request-derived addresses, and `service.rs:1442` has an `.unwrap()` inside a `#[cfg(test)]` module. The test one is fine; line 524 is a separate finding and should not be touched in this group unless you want to harden it (optional).

---

## 4. F-C-12 — 1 GB message size limits (memory exhaustion DoS)

Goal: reduce all transports to **64 MB or less**, make the RPC transports consistent, and set `max_encoding_message_size` on servers.

### 4a. gRPC — `rpc/grpc/core/src/lib.rs:8`

```rust
/// Maximum decoded gRPC message size to send and receive (64 MB)
pub const RPC_MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;
```

### 4b. gRPC server — `rpc/grpc/server/src/connection_handler.rs:129-139`

Add `max_encoding_message_size` (outbound cap) alongside the existing `max_decoding_message_size`:

```rust
let protowire_server = RpcServer::new(connection_handler)
    .accept_compressed(CompressionEncoding::Gzip)
    .send_compressed(CompressionEncoding::Gzip)
    .max_decoding_message_size(RPC_MAX_MESSAGE_SIZE)
    .max_encoding_message_size(RPC_MAX_MESSAGE_SIZE);
```

> tonic applies `max_decoding_message_size` **after** gzip decompression, so this also bounds decompressed size (compression-bomb mitigation). No need to disable gzip.

### 4c. gRPC client — `rpc/grpc/client/src/lib.rs:570`

Already uses `RPC_MAX_MESSAGE_SIZE`; it picks up the new 64 MB value automatically. Optionally add `.max_encoding_message_size(RPC_MAX_MESSAGE_SIZE)` for symmetry (client outbound).

### 4d. wRPC server — `rpc/wrpc/server/src/service.rs:16`

```rust
static MAX_WRPC_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64MB (was 128MB)
```

### 4e. wRPC proxy — `rpc/wrpc/proxy/src/main.rs:99`

```rust
let config = WebSocketConfig { max_message_size: Some(64 * 1024 * 1024), ..Default::default() };
```

### 4f. wRPC client — `rpc/wrpc/client/src/client.rs:443-444`

```rust
let ws_config = WebSocketConfig {
    max_message_size: Some(64 * 1024 * 1024),
    max_frame_size: Some(64 * 1024 * 1024),
    accept_unmasked_frames: false,
    resolver: Some(self.inner.clone()),
    ..Default::default()
};
```

### 4g. P2P — `protocol/p2p/src/core/connection_handler.rs:45`

The finding also covers P2P (1 GB). Reduce it (audit recommends 32 MB for P2P):

```rust
const P2P_MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB (was 1GB)
```

Add `.max_encoding_message_size(P2P_MAX_MESSAGE_SIZE)` to both the server (`:78`) and client (`:118`) tonic builders.

> The task's explicit file list is `rpc/grpc/` and `rpc/wrpc/`; P2P is included here because the finding text and "all transports" wording cover it. If the builder is told to stay strictly within the RPC crates, 4g can be deferred — but it is recommended for completeness.

---

## 5. F-C-13 — No authentication on state-changing RPC methods

Goal: optional bearer-token auth. Read-only methods stay open; **deploy / invoke / shutdown / stop / start** (and other admin/state-changing methods) require auth when a token is configured. Fail **closed** when a token is configured but not supplied.

### 5a. Config field — `consensus/core/src/config/mod.rs`

Add to `Config`:

```rust
/// Optional bearer token required for state-changing RPC methods.
/// When `None`, authentication is disabled (current behavior).
pub rpc_auth_token: Option<String>,
```

Initialize to `None` in `Config::with_perf` (the `Self { ... }` literal).

### 5b. Read token from environment — `zyanyad/src/args.rs` (`apply_to_config`)

```rust
config.rpc_auth_token = std::env::var("ZYANYA_RPC_AUTH_TOKEN").ok().filter(|s| !s.is_empty());
```

(Optionally also add a `--rpc-auth-token` CLI arg; env var is sufficient per the task and is the minimal change.)

### 5c. Error variant — `rpc/core/src/error.rs`

Add to `RpcError`:

```rust
#[error("Unauthorized: missing or invalid bearer token")]
Unauthorized,
```

### 5d. gRPC server context — `rpc/grpc/server/src/connection_handler.rs`

Add `pub rpc_auth_token: Option<String>` to `ServerContext` and populate it in `ServerContext::new` (or in `ConnectionHandler::new`). The token value comes from the `Config` already available where `GrpcService`/`ConnectionHandler` is constructed (`zyanyad/src/daemon.rs` passes `config` into `GrpcService::new`). Thread `config.rpc_auth_token.clone()` through.

### 5e. Extract token from gRPC metadata — `rpc/grpc/server/src/connection_handler.rs` (`message_stream`)

In `message_stream`, before building the `Connection`, read the `authorization` header:

```rust
let auth_token = request
    .metadata()
    .get("authorization")
    .and_then(|v| v.to_str().ok())
    .map(|v| v.strip_prefix("Bearer ").unwrap_or(v).to_string());
```

Pass `auth_token` into `Connection::new(...)`.

### 5f. Store token on the connection — `rpc/grpc/server/src/connection.rs`

- Add `auth_token: Option<String>` to `Inner`.
- Add a parameter to `Connection::new(...)` and store it.
- Add an accessor:
  ```rust
  pub fn auth_token(&self) -> Option<&str> { self.inner.auth_token.as_deref() }
  ```

### 5g. Enforce in the router — `rpc/grpc/server/src/connection.rs` (`route_to_handler`, line ~173)

After computing `rpc_op`, before routing:

```rust
if rpc_op.requires_auth() {
    match &self.server_context.rpc_auth_token {
        None => {}
        Some(expected) => {
            if connection.auth_token() != Some(expected.as_str()) {
                // Fail closed: return an error response to the client.
                let mut response = ZyanyadResponse {
                    id: request.id,
                    payload: Some(rpc_op.to_error_response(RpcError::Unauthorized.into())),
                };
                connection.enqueue(response).await?;
                return Ok(());
            }
        }
    }
}
```

Add a helper on `ZyanyadPayloadOps` (in `rpc/grpc/core/src/ops.rs`) or a free function listing the state-changing ops:

```rust
pub fn requires_auth(&self) -> bool {
    matches!(
        self,
        ZyanyadPayloadOps::DeployContract
            | ZyanyadPayloadOps::InvokeContract
            | ZyanyadPayloadOps::Shutdown
            | ZyanyadPayloadOps::Ban
            | ZyanyadPayloadOps::Unban
            | ZyanyadPayloadOps::AddPeer
            | ZyanyadPayloadOps::ResolveFinalityConflict
            | ZyanyadPayloadOps::SubmitBlock
            | ZyanyadPayloadOps::SubmitTransaction
            | ZyanyadPayloadOps::SubmitTransactionReplacement
            // notification start/stop (state-changing subscription commands)
            | ZyanyadPayloadOps::StopNotifyingUtxosChanged
            | ZyanyadPayloadOps::StartNotifyingUtxosChanged
            | ZyanyadPayloadOps::StopNotifyingPruningPointUtxoSetOverride
            | ZyanyadPayloadOps::StopNotifyingVirtualChainChanged
            | ZyanyadPayloadOps::StopNotifyingVirtualDaaScoreChanged
    )
}
```

> The mandatory minimum per the task is **deploy / invoke / stop / start**. The full admin set above is recommended. Verify the exact `ZyanyadPayloadOps` variant names against `rpc/grpc/core/src/ops.rs` before compiling.

### 5h. Service-layer defense-in-depth (optional but recommended) — `rpc/service/src/service.rs`

Add a helper to `RpcCoreService`:

```rust
fn require_auth(&self, supplied: Option<&str>) -> RpcResult<()> {
    match &self.config.rpc_auth_token {
        None => Ok(()),
        Some(expected) => match supplied {
            Some(t) if constant_time_eq(t.as_bytes(), expected.as_bytes()) => Ok(()),
            _ => Err(RpcError::Unauthorized),
        }
    }
}
```

Call `self.require_auth(connection.and_then(|c| c.auth_token()))?` at the top of `deploy_contract_call`, `invoke_contract_call`, and `shutdown_call`.

To make the token reach the service you must also:
1. Extend `RpcConnection` (`rpc/core/src/api/connection.rs`) with a defaulted method:
   ```rust
   fn auth_token(&self) -> Option<&str> { None }
   ```
2. Implement `RpcConnection` for the gRPC `Connection` (add `impl RpcConnection for Connection { fn id(&self) -> u64 { ... } fn auth_token(&self) -> Option<&str> { self.auth_token() } }`).
3. Change the gRPC server macro (`rpc/macros/src/grpc/server.rs:76`) from `server_ctx.core_service.#fn_call(None, request)` to `server_ctx.core_service.#fn_call(Some(&connection), request)` (and rename the ignored `_: #connection_ctx_type` binding to `connection`).

> If 5h is skipped, 5a–5g still fully satisfy F-C-13 for the gRPC transport (the task's listed file scope). wRPC routes through the same `RpcCoreService`; with 5h in place and wRPC still passing `None`, wRPC state-changing calls fail closed when a token is configured (safe). Implementing a wRPC token transport (e.g., a first-message auth or query param) is a follow-up and is **not** required by this task.

### 5i. Constant-time comparison

Use a constant-time compare to avoid a timing side channel on the token. If the `subtle` crate is already a dependency, use `subtle::ConstantTimeEq`. Otherwise a small manual XOR-accumulate loop is acceptable. Plain `==` is acceptable for a first pass but note the timing caveat.

---

## 6. Priority order (CRITICAL first)

1. **F-C-10** — unbounded bytecode/calldata (remote memory DoS + VM gas bypass). Both RPC and consensus layers.
2. **F-C-12** — 1 GB message limits (trivial remote memory exhaustion across every transport).
3. **F-C-13** — no auth on state-changing RPC (remote admin/consensus mutation).
4. **F-C-11** — panic vectors (latent; currently infallible but must be cleaned up).

Rationale: F-C-10 and F-C-12 are directly exploitable DoS primitives with no preconditions. F-C-13 is a remote state-mutation primitive (and compounds F-C-01). F-C-11 is currently latent (the conversion is infallible today) but is a one-line cleanup.

---

## 7. Files touched (summary)

| File | Findings |
|------|----------|
| `consensus/core/src/config/constants.rs` | F-C-10 (new shared constants) |
| `consensus/core/src/config/mod.rs` | F-C-13 (config field) |
| `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` | F-C-10 (consensus size checks) |
| `rpc/service/src/service.rs` | F-C-10 (RPC size checks), F-C-11 (unwrap removal), F-C-13 (require_auth helper, optional) |
| `rpc/core/src/error.rs` | F-C-13 (`Unauthorized` variant) |
| `rpc/core/src/api/connection.rs` | F-C-13 (trait method, optional) |
| `rpc/grpc/core/src/lib.rs` | F-C-12 (64 MB constant) |
| `rpc/grpc/core/src/ops.rs` | F-C-13 (`requires_auth` helper) |
| `rpc/grpc/server/src/connection_handler.rs` | F-C-12 (encoding size), F-C-13 (token extraction + ServerContext) |
| `rpc/grpc/server/src/connection.rs` | F-C-13 (token storage + router enforcement) |
| `rpc/grpc/client/src/lib.rs` | F-C-12 (inherits constant; optional encoding size) |
| `rpc/wrpc/server/src/service.rs` | F-C-12 (64 MB) |
| `rpc/wrpc/proxy/src/main.rs` | F-C-12 (64 MB) |
| `rpc/wrpc/client/src/client.rs` | F-C-12 (64 MB) |
| `protocol/p2p/src/core/connection_handler.rs` | F-C-12 (32 MB + encoding size) |
| `zyanyad/src/args.rs` | F-C-13 (env var read) |
| `rpc/macros/src/grpc/server.rs` | F-C-13 (pass connection, optional) |

---

## 8. Verification

After all edits:

```bash
source /root/.cargo/env && cargo check -p rpc/service -p rpc/grpc -p rpc/wrpc 2>&1
```

Fix any compilation errors. Do **not** run a full build or tests.

Common pitfalls to watch:
- `ZyanyadPayloadOps` variant names in `requires_auth()` must match `rpc/grpc/core/src/ops.rs` exactly.
- `ServerContext` is `#[derive(Clone)]`; adding `Option<String>` is fine, but update every construction site (`ServerContext::new`).
- `Connection::new` is called in `connection_handler.rs`; update the call to pass the new `auth_token` argument.
- If 5h is implemented, the `RpcConnection` trait change must not break the wRPC `Connection` (it does not implement `RpcConnection`; the wRPC macro still passes `None`, which is fine).
- `Hash::as_bytes()` returns `[u8; 32]` by value — after F-C-11 the code should not call `.try_into()` on it at all.
