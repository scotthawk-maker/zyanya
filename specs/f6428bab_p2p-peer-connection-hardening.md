# Remediation Plan — Batch 4: P2P Protocol & Peer Connection Hardening

**Findings:** F-H-16, F-H-17, F-H-18, F-H-19 (all HIGH)
**Source:** `audit_reports/FINAL_AUDIT_REPORT.md` (§4, lines 906–1000)
**Goal:** Apply the four fixes, then verify `cargo check --workspace --all-targets` passes.

---

## 1. Scope & Summary

| Finding | Issue | Primary file(s) |
|---------|-------|-----------------|
| F-H-16 | `user_agent` not bounded on receive → memory DoS + log flooding | `protocol/p2p/src/convert/messages.rs` |
| F-H-17 | `i64 → u64` timestamp cast + `time_offset` arithmetic wraparound | `protocol/p2p/src/convert/messages.rs`, `protocol/flows/src/flow_context.rs` |
| F-H-18 | No P2P connection limit or per-IP rate limiting → resource exhaustion | `protocol/p2p/src/core/connection_handler.rs` |
| F-H-19 | Self-declared `PeerId` trusted; duplicate connection evicts existing peer | `protocol/flows/src/flow_context.rs`, `protocol/p2p/src/core/hub.rs`, `utils/src/networking.rs` |

All four fixes are independent and can be applied in any order. Priority order (highest risk first): **F-H-18 → F-H-16 → F-H-17 → F-H-19**.

---

## 2. Files to Modify

| # | File | Findings | Change |
|---|------|----------|--------|
| 1 | `protocol/p2p/src/convert/messages.rs` | F-H-16, F-H-17 | Bound `user_agent` on receive; validate timestamp before cast |
| 2 | `protocol/p2p/src/convert/error.rs` | F-H-17 | Add `InvalidTimestamp` error variant |
| 3 | `protocol/flows/src/flow_context.rs` | F-H-17, F-H-19 | `saturating_sub` for `time_offset`; derive identity from connection |
| 4 | `protocol/p2p/src/core/connection_handler.rs` | F-H-18 | Add max-connections + per-IP rate limit |
| 5 | `protocol/p2p/src/core/hub.rs` | F-H-19 | Reject duplicate new peer instead of evicting existing |
| 6 | `utils/src/networking.rs` | F-H-19 | Add `PeerId::from_socket_addr` helper |

---

## 3. Detailed Fixes

### 3.1 F-H-16 — Bound `user_agent` on receive

**File:** `protocol/p2p/src/convert/messages.rs`

`MAX_USER_AGENT_LEN` (256) is defined in `protocol/p2p/src/convert/model/version.rs:9` and is only enforced on the send path (`Version::add_user_agent`). The receive path (`Version::try_from`) clones `msg.user_agent` verbatim.

**Change:**

1. Extend the existing import to bring in the constant:
   ```rust
   use super::{
       error::ConversionError,
       model::{
           trusted::{TrustedDataEntry, TrustedDataPackage},
           version::{Version, MAX_USER_AGENT_LEN},
       },
       option::TryIntoOptionEx,
   };
   ```

2. Add a private helper that truncates safely on a UTF-8 char boundary (plain `String::truncate` panics if the cut lands mid-codepoint):
   ```rust
   /// Truncate a user-agent string to `MAX_USER_AGENT_LEN` bytes on a UTF-8 char boundary.
   fn truncate_user_agent(user_agent: String) -> String {
       if user_agent.len() <= MAX_USER_AGENT_LEN {
           return user_agent;
       }
       let mut end = MAX_USER_AGENT_LEN;
       while !user_agent.is_char_boundary(end) {
           end -= 1;
       }
       let mut truncated = user_agent;
       truncated.truncate(end);
       truncated
   }
   ```

3. In `Version::try_from`, replace `user_agent: msg.user_agent.clone()` with:
   ```rust
   user_agent: truncate_user_agent(msg.user_agent),
   ```

**Decision:** Truncate (not reject) to stay consistent with the send path and avoid disconnecting peers with slightly oversized agents. Rejection is an acceptable alternative if preferred, but truncation is the recommended behavior.

---

### 3.2 F-H-17 — Timestamp validation + saturating `time_offset`

**Files:** `protocol/p2p/src/convert/messages.rs`, `protocol/p2p/src/convert/error.rs`, `protocol/flows/src/flow_context.rs`

**Part A — validate before casting (`messages.rs`):**

In `Version::try_from`, replace `timestamp: msg.timestamp as u64` with a validated conversion:
```rust
timestamp: {
    if msg.timestamp < 0 {
        return Err(ConversionError::InvalidTimestamp(msg.timestamp));
    }
    // Optional hardening: reject timestamps that drift too far from local time.
    // `unix_now()` returns milliseconds since epoch (u64).
    let now = zyanya_core::time::unix_now() as i64;
    if msg.timestamp.abs_diff(now) > MAX_TIMESTAMP_DRIFT_MS {
        return Err(ConversionError::InvalidTimestamp(msg.timestamp));
    }
    msg.timestamp as u64
},
```

Add a module-level constant (24h in milliseconds — generous to tolerate clock skew):
```rust
/// Maximum allowed drift (ms) between a peer's reported timestamp and local time.
const MAX_TIMESTAMP_DRIFT_MS: u64 = 24 * 60 * 60 * 1000;
```

> Note: `zyanya_core::time::unix_now` is already used by `convert/model/version.rs`, so the dependency is available. If the drift check is considered too strict for the deployment, the negative-timestamp check alone is the mandatory minimum; the drift check is defense-in-depth.

**Part B — add error variant (`error.rs`):**

Add to `ConversionError`:
```rust
#[error("Invalid timestamp {0}")]
InvalidTimestamp(i64),
```

**Part C — saturating arithmetic (`flow_context.rs:714`):**

Replace:
```rust
let time_offset = unix_now() as i64 - peer_version_message.timestamp;
```
with:
```rust
let time_offset = (unix_now() as i64).saturating_sub(peer_version_message.timestamp);
```

This prevents `i64` overflow/underflow (panic in debug, wrap in release) when a peer sends an extreme timestamp. The `try_into()` validation in Part A still rejects the connection afterward, but the saturating arithmetic removes the wraparound/panic vector at the point of computation.

---

### 3.3 F-H-18 — P2P connection limit + per-IP rate limit

**File:** `protocol/p2p/src/core/connection_handler.rs`

**Design:** Limits apply to **inbound** connections in `message_stream` (the attack vector). Outbound connections are already bounded by the `outbound_target` config. Use a `tokio::sync::Semaphore` for the global cap and a `Mutex<HashMap<IpAddress, VecDeque<Instant>>>` sliding window for per-IP rate limiting.

**Step 1 — constants:**
```rust
/// Maximum number of concurrent inbound P2P connections.
const MAX_CONNECTIONS: usize = 128;
/// Maximum number of new inbound connections allowed per IP per minute.
const MAX_CONNECTIONS_PER_IP_PER_MINUTE: usize = 10;
/// Sliding window for the per-IP connection rate limit.
const CONNECTION_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
```

**Step 2 — new fields on `ConnectionHandler`:**
```rust
pub struct ConnectionHandler {
    hub_sender: MpscSender<HubEvent>,
    initializer: Arc<dyn ConnectionInitializer>,
    counters: Arc<TowerConnectionCounters>,
    /// Global inbound connection slots (released when a connection closes).
    connection_slots: Arc<tokio::sync::Semaphore>,
    /// Per-IP timestamps of recent inbound connections (sliding window).
    per_ip_connections: Arc<Mutex<HashMap<IpAddress, VecDeque<Instant>>>>,
}
```

Initialize in `ConnectionHandler::new` (signature unchanged):
```rust
Self {
    hub_sender,
    initializer,
    counters,
    connection_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS)),
    per_ip_connections: Arc::new(Mutex::new(HashMap::new())),
}
```

**Step 3 — stream guard that releases the slot on connection close:**

The `message_stream` handler returns a `Response<Stream>`. Tonic drops that stream when the connection ends, so holding the semaphore permit inside the stream releases the slot exactly when the connection is torn down.

```rust
/// Wraps the outgoing stream and holds a connection-slot permit so the slot is
/// released when the connection (and thus the stream) is dropped.
struct ConnectionGuardStream<S> {
    inner: S,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<S: futures::Stream + Unpin> futures::Stream for ConnectionGuardStream<S> {
    type Item = S::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}
```

**Step 4 — enforce limits in `message_stream`:**

Insert after the `remote_address` extraction and before building the router:
```rust
let ip: IpAddress = remote_address.ip().into();

// Per-IP rate limit (sliding window).
{
    let mut map = self.per_ip_connections.lock().unwrap();
    let now = Instant::now();
    let entry = map.entry(ip).or_default();
    while entry.front().map_or(false, |t| now.duration_since(*t) > CONNECTION_RATE_LIMIT_WINDOW) {
        entry.pop_front();
    }
    if entry.len() >= MAX_CONNECTIONS_PER_IP_PER_MINUTE {
        return Err(TonicStatus::resource_exhausted("per-IP connection rate limit exceeded"));
    }
    entry.push_back(now);
}

// Global connection limit.
let permit = match self.connection_slots.clone().try_acquire_owned() {
    Ok(permit) => permit,
    Err(_) => return Err(TonicStatus::resource_exhausted("max connections reached")),
};
```

Then change the final return to wrap the stream with the guard:
```rust
let stream = ReceiverStream::new(outgoing_receiver).map(Ok);
let guarded = ConnectionGuardStream { inner: stream, _permit: permit };
Ok(Response::new(Box::pin(guarded) as Self::MessageStreamStream))
```

**Step 5 — imports to add:**
```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Instant;
use zyanya_utils::networking::{IpAddress, NetAddress};
```
(`Pin` is already imported; `Duration` is already imported; `futures` is already a dependency via `futures::FutureExt`.)

**Notes:**
- `std::sync::Mutex` is used because the critical section is short and synchronous (no `.await` while holding the lock). `parking_lot::Mutex` is an acceptable alternative.
- The `per_ip_connections` map grows one entry per distinct IP. Lazy pruning on access bounds memory per entry; a periodic cleanup of empty entries is an optional follow-up, not required for this fix.
- `Instant::duration_since` is safe here because only `Instant::now()` values are ever pushed (monotonic).

---

### 3.4 F-H-19 — Derive peer identity from connection; reject duplicate eviction

**Files:** `utils/src/networking.rs`, `protocol/flows/src/flow_context.rs`, `protocol/p2p/src/core/hub.rs`

**Part A — deterministic identity helper (`utils/src/networking.rs`):**

Add `use sha2::{Digest, Sha256};` to the imports (sha2 is already a dependency of `zyanya-utils`), then add to `impl PeerId`:
```rust
/// Derive a deterministic peer identity from a socket address (IP:port).
///
/// This binds the peer identity to the actual connection endpoint so that peers
/// cannot spoof arbitrary self-declared identities during the P2P handshake.
pub fn from_socket_addr(addr: &SocketAddr) -> Self {
    let mut hasher = Sha256::new();
    hasher.update(addr.to_string().as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Self(Uuid::from_bytes(bytes))
}
```
(`SocketAddr` and `Uuid` are already imported in this file.)

**Part B — use derived identity (`flow_context.rs:716-725`):**

Replace:
```rust
let peer_version: Version = peer_version_message.try_into()?;
router.set_identity(peer_version.id);
// Avoid duplicate connections
if self.hub.has_peer(router.key()) {
    return Err(ProtocolError::PeerAlreadyExists(router.key()));
}
// And loopback connections...
if self.node_id == router.identity() {
    return Err(ProtocolError::LoopbackConnection(router.key()));
}
```
with:
```rust
let peer_version: Version = peer_version_message.try_into()?;
// Loopback detection: compare the self-declared id against our own node id.
// (The self-declared id is NOT trusted for keying/identity purposes.)
if self.node_id == peer_version.id {
    return Err(ProtocolError::LoopbackConnection(router.key()));
}
// Derive the peer identity from the connection endpoint (IP:port) instead of
// trusting the self-declared PeerId, preventing identity spoofing.
router.set_identity(PeerId::from_socket_addr(&router.net_address()));
// Avoid duplicate connections
if self.hub.has_peer(router.key()) {
    return Err(ProtocolError::PeerAlreadyExists(router.key()));
}
```

`PeerId` is already imported in `flow_context.rs`. Loopback detection is preserved by comparing the *declared* id to `self.node_id` before overwriting the router identity.

**Part C — reject new duplicate instead of evicting existing (`hub.rs:79-84`):**

Replace `insert_new_router`:
```rust
async fn insert_new_router(&self, new_router: Arc<Router>) {
    let mut peers = self.peers.write();
    if peers.contains_key(&new_router.key()) {
        // A peer with this key already exists. Reject the NEW connection rather
        // than evicting the existing one (prevents identity-spoofing eviction).
        warn!("P2P, Hub event loop, rejecting new peer with duplicate key: {}", new_router.key());
        drop(peers);
        new_router.close().await;
        return;
    }
    peers.insert(new_router.key(), new_router);
}
```

`warn!` is already imported. `new_router.close()` emits `HubEvent::PeerClosing`, which the event loop already handles safely (it only removes the entry when `Arc::ptr_eq` matches, so the existing peer is untouched).

---

## 4. Vulnerability Categories per File

| File | Categories |
|------|-----------|
| `protocol/p2p/src/convert/messages.rs` | Input validation / deserialization safety (unbounded string), integer overflow/underflow (i64→u64 cast) |
| `protocol/p2p/src/convert/error.rs` | Error handling (new validation error variant) |
| `protocol/flows/src/flow_context.rs` | Integer overflow/underflow (time_offset), peer identity spoofing / consensus-adjacent identity trust |
| `protocol/p2p/src/core/connection_handler.rs` | Network-level DoS (resource exhaustion), rate limiting |
| `protocol/p2p/src/core/hub.rs` | Peer identity spoofing / duplicate eviction |
| `utils/src/networking.rs` | Cryptographic implementation correctness (deterministic identity derivation) |

---

## 5. Priority Order

1. **F-H-18** (CRITICAL-adjacent DoS): unbounded inbound connections exhaust FDs/memory/CPU. Apply first.
2. **F-H-16**: unbounded `user_agent` → memory + log flooding. Small, self-contained.
3. **F-H-17**: integer wraparound / debug-build panic on malicious timestamp.
4. **F-H-19**: identity spoofing / duplicate eviction (lower exploitability — UUID guessing is impractical — but still a correctness fix).

---

## 6. Verification

1. `cargo check --workspace --all-targets` — must pass (required by the task).
2. Targeted checks (fast feedback):
   - `cargo check -p zyanya-p2p-lib -p zyanya-p2p-flows -p zyanya-utils --all-targets`
3. Targeted tests (if added):
   - `cargo test -p zyanya-utils peer_id` (deterministic `from_socket_addr`)
   - `cargo test -p zyanya-p2p-lib` (user_agent truncation + timestamp validation, if unit tests are added)
4. `cargo clippy -p zyanya-p2p-lib -p zyanya-p2p-flows -p zyanya-utils --all-targets` (optional, no new warnings).

---

## 7. Recommended Unit Tests

- **F-H-16** (`messages.rs`): `Version::try_from` truncates a >256-byte `user_agent`; a multi-byte UTF-8 string whose 256th byte is mid-codepoint does not panic and is truncated to a valid boundary.
- **F-H-17** (`messages.rs`): negative `timestamp` returns `Err(InvalidTimestamp)`; timestamp beyond ±24h returns `Err(InvalidTimestamp)`; a valid timestamp converts correctly.
- **F-H-19** (`utils/src/networking.rs`): `PeerId::from_socket_addr` is deterministic for the same `SocketAddr` and differs for different ports/IPs.
- **F-H-18**: extract the per-IP sliding-window check into a small testable helper (e.g., `fn check_per_ip_rate_limit(&self, ip: IpAddress) -> Result<(), TonicStatus>`) and unit-test the window/limit behavior directly.

---

## 8. Risks & Notes for the Builder

- **F-H-16 truncation:** do NOT use bare `String::truncate(MAX_USER_AGENT_LEN)` — it panics on a non-char boundary. Use the char-boundary-safe helper above.
- **F-H-17 drift check:** if the ±24h window is deemed too strict for the target network, keep the negative-timestamp rejection (mandatory) and drop only the drift check. The `saturating_sub` in `flow_context.rs` is required regardless.
- **F-H-18:** the semaphore permit must be held for the full connection lifetime. Do not drop it immediately after `message_stream` returns — wrap the response stream (as shown) so tonic releases it on connection close.
- **F-H-19:** keep the loopback check on the *declared* id (`peer_version.id`) before overwriting `router.identity()`; otherwise self-connection detection is lost.
- Do not change the `ConnectionHandler::new` / `Adaptor` / `P2pService` signatures — the limits use in-crate defaults (128 / 10-per-minute) per the task brief.
