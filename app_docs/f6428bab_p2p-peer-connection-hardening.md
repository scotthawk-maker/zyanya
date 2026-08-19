# Batch 4 — P2P Protocol & Peer Connection Hardening (F-H-16, F-H-17, F-H-18, F-H-19)

Session `f6428bab` · 7 files changed (+515 −20) · remediation of four HIGH security findings from `audit_reports/FINAL_AUDIT_REPORT.md`.

## What changed and why it matters

Four independent HIGH findings were remediated in the P2P stack. None of the changes alter any public API signature; all limits use in-crate default constants.

### F-H-16 — Bound `user_agent` on receive (memory DoS + log flooding)

`protocol/p2p/src/convert/messages.rs`

The `MAX_USER_AGENT_LEN` (256 byte) bound was only enforced on the *send* path (`Version::add_user_agent`). The receive path (`Version::try_from`) cloned the incoming `user_agent` verbatim, allowing a malicious peer to ship an arbitrarily large string and trigger memory exhaustion and log flooding.

**Fix:** A private `truncate_user_agent` helper truncates the received string to `MAX_USER_AGENT_LEN` on a UTF-8 char boundary (plain `String::truncate` panics on a non-char boundary, so the helper walks backwards to the nearest valid boundary). The receive path now calls it instead of cloning. Truncation (rather than rejection) was chosen to stay consistent with the send-path behavior and avoid disconnecting peers whose agent is only slightly oversized.

### F-H-17 — Timestamp cast + `time_offset` arithmetic wraparound

Files: `protocol/p2p/src/convert/messages.rs`, `protocol/p2p/src/convert/error.rs`, `protocol/flows/src/flow_context.rs`

Two related integer issues:

1. `Version::try_from` did `msg.timestamp as u64` directly. A negative `i64` would wrap to a huge `u64`, and an extreme timestamp could panic the `time_offset` subtraction in debug builds / wrap in release.
2. `flow_context.rs` computed `time_offset = unix_now() as i64 - peer_version_message.timestamp` with plain subtraction, which overflows on a malicious extreme timestamp.

**Fix (messages.rs):** Before casting, `Version::try_from` now rejects `msg.timestamp < 0` and timestamps whose `abs_diff` from local `unix_now()` exceeds `MAX_TIMESTAMP_DRIFT_MS` (24 h, generous to tolerate clock skew). Both return the new `ConversionError::InvalidTimestamp(i64)` variant.

**Fix (error.rs):** Added `InvalidTimestamp(i64)` to `ConversionError` with `#[error("Invalid timestamp {0}")]`.

**Fix (flow_context.rs):** `time_offset` now uses `(unix_now() as i64).saturating_sub(peer_version_message.timestamp)`, removing the wraparound/panic vector at the point of computation. The `try_into()` validation in `messages.rs` still rejects the connection after the saturating computation completes.

### F-H-18 — P2P connection limit + per-IP rate limiting (resource exhaustion)

`protocol/p2p/src/core/connection_handler.rs`

Inbound P2P connections were entirely unbounded — an attacker could exhaust file descriptors, memory, and CPU by opening unlimited concurrent streams. Outbound connections are already bounded by `outbound_target` config, so this fix targets the inbound `message_stream` handler.

**Fix:** Two new limits, both enforced before the router is built:

- **Global cap:** `MAX_CONNECTIONS = 128` concurrent inbound connections, gated by a `tokio::sync::Semaphore`. A `ConnectionGuardStream<S>` wraps the outgoing `ReceiverStream` and holds an `OwnedSemaphorePermit`; the permit is released when tonic drops the stream on connection close, so the slot lifetime exactly matches the connection lifetime.
- **Per-IP rate limit:** `MAX_CONNECTIONS_PER_IP_PER_MINUTE = 10` new connections per IP within a 60-second sliding window (`CONNECTION_RATE_LIMIT_WINDOW`). Implemented as a `Mutex<HashMap<IpAddress, VecDeque<Instant>>>`; each access evicts expired entries from the front before checking/pushing. `std::sync::Mutex` is used because the critical section is short and contains no `.await`.

Both limits return `tonic::Status::resource_exhausted` on violation. New `ConnectionHandler` fields (`connection_slots`, `per_ip_connections`) are initialized in `ConnectionHandler::new` without changing its signature.

### F-H-19 — Peer identity spoofing + duplicate eviction

Files: `utils/src/networking.rs`, `protocol/flows/src/flow_context.rs`, `protocol/p2p/src/core/hub.rs`

Two related identity-trust issues:

1. The router's peer identity was set from the *self-declared* `peer_version.id`, so any peer could claim any PeerId — including one already in use, which the Hub would then evict.
2. `Hub::insert_new_router` silently closed the *existing* peer when a duplicate key arrived, enabling identity-spoofing-driven eviction of an established peer.

**Fix (utils/src/networking.rs):** Added `PeerId::from_socket_addr(&SocketAddr)`, which derives a deterministic 16-byte identity by SHA-256 hashing `addr.to_string()` and taking the first 16 bytes as a UUID. This binds the peer identity to the actual connection endpoint so it cannot be spoofed. `sha2` was already a `zyanya-utils` dependency.

**Fix (flow_context.rs):** Loopback detection now compares the *declared* id (`peer_version.id`) against `self.node_id` *before* the identity is overwritten — preserving self-connection detection. Then `router.set_identity(PeerId::from_socket_addr(&router.net_address()))` replaces the previous `router.set_identity(peer_version.id)`, so the router key (and thus the Hub's peer map key) is derived from the connection, not the peer's claim.

**Fix (hub.rs):** `insert_new_router` now checks `self.peers.read().contains_key(&new_router.key())` first. If the key already exists, the *new* connection is closed and a warning is logged — the existing peer is untouched. The read guard is dropped before the `.await` on `new_router.close()` so the future stays `Send`. `new_router.close()` emits `HubEvent::PeerClosing`, which the event loop already handles safely (it only removes the entry when `Arc::ptr_eq` matches, so the established peer is never affected).

## Files carrying the change

| File | Findings | Role |
|------|----------|------|
| `protocol/p2p/src/convert/messages.rs` | F-H-16, F-H-17 | `truncate_user_agent` helper; timestamp validation before `i64 → u64` cast; `MAX_TIMESTAMP_DRIFT_MS` constant |
| `protocol/p2p/src/convert/error.rs` | F-H-17 | New `InvalidTimestamp(i64)` error variant |
| `protocol/flows/src/flow_context.rs` | F-H-17, F-H-19 | `saturating_sub` for `time_offset`; loopback check on declared id before identity overwrite; derived identity via `PeerId::from_socket_addr` |
| `protocol/p2p/src/core/connection_handler.rs` | F-H-18 | `MAX_CONNECTIONS` semaphore + `ConnectionGuardStream` permit holder; per-IP sliding-window rate limit; new struct fields |
| `protocol/p2p/src/core/hub.rs` | F-H-19 | `insert_new_router` rejects the new duplicate instead of evicting the existing peer |
| `utils/src/networking.rs` | F-H-19 | `PeerId::from_socket_addr` deterministic identity helper (SHA-256 → first 16 bytes) |
| `specs/f6428bab_p2p-peer-connection-hardening.md` | all | New spec document describing the four fixes in detail (375 lines) |

## How to use and verify

The fixes are transparent to callers — no public signatures changed and no configuration was added; the limits use in-crate defaults (128 connections, 10 per IP per minute, 256-byte user agent, 24 h timestamp drift).

**Required verification (from the task brief):**

```sh
cargo check --workspace --all-targets
```

**Targeted / faster feedback:**

```sh
cargo check -p zyanya-p2p-lib -p zyanya-p2p-flows -p zyanya-utils --all-targets
cargo clippy -p zyanya-p2p-lib -p zyanya-p2p-flows -p zyanya-utils --all-targets
```

**Suggested follow-up unit tests (not yet added by this session):**

- `messages.rs`: `Version::try_from` truncates a >256-byte `user_agent`; a multi-byte UTF-8 string whose 256th byte is mid-codepoint does not panic; negative / out-of-drift timestamps return `Err(InvalidTimestamp)`.
- `utils/src/networking.rs`: `PeerId::from_socket_addr` is deterministic for the same `SocketAddr` and differs across ports/IPs.
- `connection_handler.rs`: extract the per-IP sliding-window check into a small testable helper and unit-test the window/limit behavior directly.

## Notes / risks

- **F-H-16:** never replace the helper with bare `String::truncate(MAX_USER_AGENT_LEN)` — it panics on a non-char boundary. The char-boundary-safe walk is required.
- **F-H-17:** the ±24 h drift check is defense-in-depth; if it is ever deemed too strict for a deployment, the negative-timestamp rejection and the `saturating_sub` are the mandatory minimums. Do not remove `saturating_sub`.
- **F-H-18:** the semaphore permit must live for the whole connection. It is held inside `ConnectionGuardStream`, which tonic drops on connection close — do not move the permit out of the stream.
- **F-H-19:** keep the loopback check on the *declared* id (`peer_version.id`) before calling `set_identity`; otherwise self-connection detection is lost.
- The `per_ip_connections` map grows one entry per distinct IP and is pruned lazily on access; a periodic cleanup of empty entries is an optional follow-up, not required here.