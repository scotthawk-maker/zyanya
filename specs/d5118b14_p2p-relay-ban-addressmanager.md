# Remediation Plan — Batch 5: P2P Relay, Ban Logic & Address Manager (F-H-20, F-H-21, F-H-22)

**Date:** 2026-08-18
**Scope:** Remediate three HIGH findings from `audit_reports/FINAL_AUDIT_REPORT.md`:
- **F-H-20** — Transaction relay: no ban/disconnect for spam peers (amplification)
- **F-H-21** — Address manager poisoning: no source validation, eviction can displace good peers
- **F-H-22** — Address manager `is_banned`: u64 underflow on future timestamp (ban bypass)

**Definition of done:** `cargo check --workspace --all-targets` passes; each fix has a unit test where feasible.

---

## Priority order

1. **F-H-22** (do first) — trivial, self-contained, and F-H-20's ban mechanism depends on `is_banned` being correct.
2. **F-H-21** — address manager hardening (source validation + eviction protection).
3. **F-H-20** — tx relay rate limit + ban (depends on a working `ban`/`is_banned`).

---

## Files to modify

| File | Findings | Change |
|------|----------|--------|
| `components/addressmanager/src/lib.rs` | F-H-21, F-H-22 | `is_banned` saturating_sub; `add_address` trust param + port validation; `keep_limit` eviction protection; connected-peer tracking |
| `protocol/flows/src/v5/txrelay/flow.rs` | F-H-20 | per-peer tx relay rate limit + spam ban |
| `protocol/flows/src/v5/address.rs` | F-H-21 | pass `trusted=false` for peer-advertised addresses |
| `protocol/flows/src/flow_context.rs` | F-H-21 | pass `trusted=true` for connected-peer addresses |
| `components/connectionmanager/src/lib.rs` | F-H-21 | pass `trusted=true` for DNS seeds; sync connected-peer set |

---

## F-H-22 — `is_banned` u64 underflow (ban bypass)

**File:** `components/addressmanager/src/lib.rs` (`is_banned`, ~line 306)

**Current code:**
```rust
pub fn is_banned(&mut self, ip: IpAddress) -> bool {
    const MAX_BANNED_TIME: u64 = 24 * 60 * 60 * 1000;
    match self.banned_address_store.get(ip.into()).unwrap_option() {
        Some(timestamp) => {
            if unix_now() - timestamp.0 > MAX_BANNED_TIME {   // underflows if timestamp.0 > unix_now()
                self.unban(ip);
                false
            } else {
                true
            }
        }
        None => false,
    }
}
```

**Fix:** replace the raw subtraction with `saturating_sub`. A future timestamp now yields `elapsed == 0`, which is `<= MAX_BANNED_TIME`, so the peer stays banned (no bypass) until the clock catches up.

```rust
pub fn is_banned(&mut self, ip: IpAddress) -> bool {
    const MAX_BANNED_TIME: u64 = 24 * 60 * 60 * 1000;
    match self.banned_address_store.get(ip.into()).unwrap_option() {
        Some(timestamp) => {
            let elapsed = unix_now().saturating_sub(timestamp.0);
            if elapsed > MAX_BANNED_TIME {
                self.unban(ip);
                false
            } else {
                true
            }
        }
        None => false,
    }
}
```

**Test:** add a unit test that inserts a ban with a future timestamp (e.g. `unix_now() + 10_000`) and asserts `is_banned` returns `true` (not bypassed). Also assert a ban older than `MAX_BANNED_TIME` returns `false` and is removed.

---

## F-H-21 — Address manager poisoning

**File:** `components/addressmanager/src/lib.rs` (plus call sites listed above)

Three sub-fixes:

### 1. Source validation — `add_address(address, trusted)`

Change the signature and reject invalid ports. Untrusted addresses (peer-advertised via `AddressesMessage`) start with a higher `connection_failed_count` so they are evicted before trusted addresses and removed after a single failure. This avoids a DB schema change (the `Entry` struct is bincode-serialized; adding a field would break existing DBs).

Add constants near the existing `MAX_CONNECTION_FAILED_COUNT`:
```rust
/// Initial failure count for addresses learned from trusted sources
/// (connected peers, DNS seeds, local addresses).
const TRUSTED_INITIAL_FAILED_COUNT: u64 = 1;
/// Initial failure count for addresses learned from untrusted sources
/// (peer-advertised AddressesMessage). Equal to MAX_CONNECTION_FAILED_COUNT so
/// untrusted addresses are evicted first and dropped after one failure.
const UNTRUSTED_INITIAL_FAILED_COUNT: u64 = MAX_CONNECTION_FAILED_COUNT;
```

New `add_address`:
```rust
pub fn add_address(&mut self, address: NetAddress, trusted: bool) {
    if address.ip.is_loopback() || address.ip.is_unspecified() {
        debug!("[Address manager] skipping local address {}", address.ip);
        return;
    }
    // Reject invalid ports (port 0 is never a valid peer address).
    if address.port == 0 {
        debug!("[Address manager] skipping address with invalid port {}", address);
        return;
    }
    if self.address_store.has(address) {
        return;
    }
    let initial_count = if trusted { TRUSTED_INITIAL_FAILED_COUNT } else { UNTRUSTED_INITIAL_FAILED_COUNT };
    self.address_store.set(address, initial_count);
}
```

> Note: privileged-port rejection (`port < 1024`) is optional hardening; the default P2P port is 18111. Only port 0 is rejected here to avoid breaking custom/testnet ports.

### 2. Protect known-good peers from eviction + never evict connected peers

Add a `connected: HashSet<AddressKey>` field to the `Store` struct in the `address_store_with_cache` module, initialized empty in `Store::new`.

Add to `Store`:
```rust
pub fn set_connected(&mut self, connected: HashSet<NetAddress>) {
    self.connected = connected.into_iter().map(|a| a.into()).collect();
}
```

Update `remove_by_key` to also clear the connected marker:
```rust
fn remove_by_key(&mut self, key: AddressKey) {
    self.addresses.remove(&key);
    self.connected.remove(&key);
    self.db_store.remove(key).unwrap()
}
```

Rewrite `keep_limit` to (a) skip connected peers, (b) evict the highest `connection_failed_count` among the rest, and (c) only fall back to connected peers if every address is connected:
```rust
fn keep_limit(&mut self) {
    while self.addresses.len() > MAX_ADDRESSES {
        let to_remove = self
            .addresses
            .iter()
            .filter(|(key, _)| !self.connected.contains(key))
            .max_by(|a, b| (a.1).connection_failed_count.cmp(&(b.1).connection_failed_count))
            .or_else(|| {
                self.addresses.iter().max_by(|a, b| (a.1).connection_failed_count.cmp(&(b.1).connection_failed_count))
            })
            .map(|(key, _)| *key)
            .unwrap();
        self.remove_by_key(to_remove);
    }
}
```

Expose a public `set_connected` on `AddressManager`:
```rust
pub fn set_connected(&mut self, connected: HashSet<NetAddress>) {
    self.address_store.set_connected(connected);
}
```

### 3. Call-site updates

- `protocol/flows/src/v5/address.rs:58` (ReceiveAddressesFlow — arbitrary peer-advertised addresses):
  ```rust
  amgr_lock.add_address(NetAddress::new(ip, port), false)
  ```
- `protocol/flows/src/flow_context.rs:770` (outbound connection — actually connected):
  ```rust
  address_manager.add_address(router.net_address().into(), true);
  ```
- `protocol/flows/src/flow_context.rs:774` (peer self-advertised address from a connected peer):
  ```rust
  address_manager.add_address(peer_ip_address, true);
  ```
- `components/connectionmanager/src/lib.rs:305` (DNS seeder):
  ```rust
  amgr_lock.add_address(NetAddress::new(addr.ip().into(), addr.port()), true);
  ```
- `components/connectionmanager/src/lib.rs` `handle_event` — sync the connected set right after building `peer_by_address`:
  ```rust
  let connected: HashSet<NetAddress> = peer_by_address.keys().map(|addr| (*addr).into()).collect();
  self.address_manager.lock().set_connected(connected);
  ```
  (`NetAddress` and `HashSet` are already imported in this file.)

### 4. Tests (F-H-21)

- Update the existing `test_network_distribution_weighting` call to `am_guard.add_address(NetAddress::new(..), true)`.
- Add a test: fill the store past `MAX_ADDRESSES` with a mix of trusted (count 1) and untrusted (count 3) addresses; assert untrusted addresses are evicted first and trusted/known-good (count 0) addresses survive.
- Add a test: mark an address as connected via `set_connected`, overfill the store, and assert the connected address is never evicted.

---

## F-H-20 — Transaction relay: rate limit + ban for spam peers

**File:** `protocol/flows/src/v5/txrelay/flow.rs`

The `RelayTransactionsFlow` is instantiated per peer, so per-peer counters live on the struct. When a peer exceeds the rate or spam threshold, ban its IP via the address manager and return `Err(ProtocolError::MisbehavingPeer(...))` — the `Flow::launch` wrapper in `flow_trait.rs` already disconnects the peer on any `Err`.

### 1. Constants

```rust
/// Maximum number of transactions a peer may relay per second before being banned.
const MAX_TX_RELAY_PER_SECOND: u64 = 100;
/// Sliding window (ms) for the tx relay rate limit.
const TX_RELAY_WINDOW_MS: u64 = 1000;
/// Maximum number of spam/non-standard transactions tolerated per window before banning.
const MAX_SPAM_TXS_PER_WINDOW: u64 = 100;
/// Sliding window (ms) for the spam counter.
const SPAM_WINDOW_MS: u64 = 60_000;
```

### 2. Struct fields

Add to `RelayTransactionsFlow`:
```rust
/// Track the number of spam txs coming from this peer
spam_counter: u64,
/// Start of the current spam counting window
spam_window_start: u64,
/// Number of transactions relayed by this peer in the current window
tx_relay_count: u64,
/// Start of the current tx relay rate-limit window
tx_relay_window_start: u64,
```

Update `new` to initialize them:
```rust
Self {
    ctx, router, invs_route, msg_route,
    spam_counter: 0,
    spam_window_start: unix_now(),
    tx_relay_count: 0,
    tx_relay_window_start: unix_now(),
}
```

### 3. Ban helper

```rust
fn ban_peer(&self, reason: &str) -> ProtocolError {
    let ip = self.router.net_address().ip().into();
    self.ctx.address_manager.lock().ban(ip);
    ProtocolError::MisbehavingPeer(reason.to_owned())
}
```

### 4. Rate limit in `receive_transactions`

After the `transactions` vector is populated (before/after insert — before is fine), enforce the per-second relay rate:
```rust
// Per-peer tx relay rate limit (F-H-20)
let now = unix_now();
if now.saturating_sub(self.tx_relay_window_start) >= TX_RELAY_WINDOW_MS {
    self.tx_relay_window_start = now;
    self.tx_relay_count = 0;
}
self.tx_relay_count += transactions.len() as u64;
if self.tx_relay_count > MAX_TX_RELAY_PER_SECOND {
    return Err(self.ban_peer(&format!(
        "peer {} exceeded tx relay rate limit ({} tx/sec)",
        self.router, MAX_TX_RELAY_PER_SECOND
    )));
}
```

### 5. Ban on spam in the insert-results loop

Replace the current spam branch:
```rust
Err(MiningManagerError::MempoolError(RuleError::RejectSpamTransaction(_)))
| Err(MiningManagerError::MempoolError(RuleError::RejectNonStandard(..))) => {
    let now = unix_now();
    if now.saturating_sub(self.spam_window_start) >= SPAM_WINDOW_MS {
        self.spam_window_start = now;
        self.spam_counter = 0;
    }
    self.spam_counter += 1;
    if self.spam_counter > MAX_SPAM_TXS_PER_WINDOW {
        return Err(self.ban_peer(&format!(
            "peer {} sent {} spam/non-standard txs",
            self.router, self.spam_counter
        )));
    }
    if self.spam_counter % 100 == 0 {
        zyanya_core::warn!("Peer {} has shared {} spam/non-standard txs ({:?})", self.router, self.spam_counter, res);
    }
}
```

### 6. Tests (F-H-20)

- Add a unit test for the window-reset/rate-limit logic. If the counting logic is extracted into a small pure helper (e.g. `fn update_rate_window(count: &mut u64, window_start: &mut u64, now: u64, window_ms: u64) -> u64`), test it directly; otherwise test via the existing `check_tx_throttling`-style pattern.
- Note: `RequestTransactionsFlow` responder amplification is the related finding **F-M-24** (out of scope for this batch, but worth a follow-up).

---

## Verification

1. `cargo check --workspace --all-targets` — must pass.
2. `cargo test -p zyanya-addressmanager` (or the crate's actual package name) — new/updated tests pass.
3. `cargo test -p zyanya-flows` (or equivalent) — tx relay tests pass.
4. Grep for any remaining `add_address(` call sites to confirm all pass the `trusted` argument.

> Package names: confirm exact crate names via `Cargo.toml` (`components/addressmanager/Cargo.toml`, `protocol/flows/Cargo.toml`) before running targeted tests.

---

## Risks / notes for the builder

- **No DB schema change:** trust is encoded via `connection_failed_count` (untrusted = `MAX_CONNECTION_FAILED_COUNT`), not a new `Entry` field, to avoid breaking bincode-serialized address-store data.
- **`set_connected` sync cadence:** the connection manager syncs the connected set each `handle_event` iteration (30s ticker + forced iterations). Eviction is rare, so this is sufficient; `mark_connection_success` sets count to 0 which independently protects freshly-connected peers from eviction.
- **`ban_peer` uses `address_manager.ban`** (synchronous, parking_lot mutex) rather than `connection_manager.ban` (async). The flow disconnects itself by returning `Err`, which `Flow::launch` turns into `router.close()`.
- **Do not change `Entry` serialization** or the `AddressKey`/`NetAddress` types.
- Keep the existing `spam_counter % 100` warning log for observability after adding the ban threshold.
