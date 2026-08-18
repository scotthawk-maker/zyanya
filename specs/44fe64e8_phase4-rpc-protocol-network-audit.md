# Phase 4 Security Audit Plan — RPC + Protocol + Network

**Target**: Zyanya blockchain (Kaspa/rusty-spectre fork) — all network-facing code (~29,000 lines).
**Output**: `audit_reports/phase4_findings.md`
**Date**: 2026-08-18

This phase is the *attack surface* audit: every byte that crosses a socket, gRPC
stream, or WebSocket is untrusted. The codebase is upstream Kaspa/rusty-spectre
renamed to `zyanya-*` crates, plus Zyanya-specific smart-contract RPC handlers
(`deploy`/`invoke`/`call`/`get_contract_*`) and a custom VM execution path.

Findings must be written to `audit_reports/phase4_findings.md` in the same format
as phases 1–3 (see `audit_reports/phase1_findings.md` and `phase3_findings.md`):
severity-ordered (CRITICAL → HIGH → MEDIUM → LOW), then by file path, with stable
IDs (`C-01`, `H-01`, …), `file:line`, description, code snippet, and recommended fix.
**Do not modify source code** — audit + report only.

---

## 1. Files to audit (with paths)

### 1.1 RPC — `rpc/` (18,837 lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 1 | `rpc/core/src/api/rpc.rs` | ~590 | RPC trait surface; which methods are exposed; `MAX_SAFE_WINDOW_SIZE`; contract method signatures |
| 2 | `rpc/core/src/api/ops.rs` | ~150 | RPC op enum (`RpcApiOps`); op numbering; `is_subscription` |
| 3 | `rpc/core/src/api/ctl.rs` | — | `RpcState`, control dispatch |
| 4 | `rpc/core/src/api/connection.rs` | — | `DynRpcConnection` |
| 5 | `rpc/core/src/api/notifications.rs` | — | notification op mapping |
| 6 | `rpc/core/src/error.rs` | ~150 | `RpcError`; error→response conversion; info leakage |
| 7 | `rpc/core/src/model/*.rs` | ~1,700 | request/response types; deserialization (borsh/serde); `hex_cnv.rs` unsafe block; `message.rs` |
| 8 | `rpc/core/src/convert/*.rs` | ~600 | consensus↔RPC conversions; `try_into`/`unwrap` panic vectors |
| 9 | `rpc/core/src/notify/*.rs` | ~400 | notification channel/collector/connection |
| 10 | `rpc/core/src/wasm/*.rs` | ~1,800 | WASM bindings; `message.rs` (1,730 lines) |
| 11 | `rpc/service/src/service.rs` | 1,492 | **ZYANYA CONTRACT RPC HANDLERS** + all `*_call` impls; input validation; safe-mode gating |
| 12 | `rpc/service/src/converter/*.rs` | ~340 | consensus/index/protocol converters |
| 13 | `rpc/macros/src/handler.rs` | 93 | handler name/type derivation |
| 14 | `rpc/macros/src/grpc/server.rs` | 150 | gRPC server interface macro |
| 15 | `rpc/macros/src/wrpc/*.rs` | ~590 | wRPC client/server/wasm macros |
| 16 | `rpc/grpc/core/src/lib.rs` | ~10 | `RPC_MAX_MESSAGE_SIZE` (1GB) |
| 17 | `rpc/grpc/core/src/ops.rs` | ~100 | `ZyanyadPayloadOps` enum; `to_error_response` |
| 18 | `rpc/grpc/core/src/channel.rs` | ~5 | notification channel |
| 19 | `rpc/grpc/core/src/convert/*.rs` | ~1,200 | protowire↔RPC conversions; `expect`/`unwrap` |
| 20 | `rpc/grpc/core/src/ext/*.rs` | ~200 | extensions |
| 21 | `rpc/grpc/core/src/macros.rs` | — | macros |
| 22 | `rpc/grpc/client/src/lib.rs` | ~900 | client connect/reconnect; message size; resolver |
| 23 | `rpc/grpc/client/src/resolver/*.rs` | ~300 | resolver id/matcher/queue |
| 24 | `rpc/grpc/client/src/client_pool.rs`, `connection_event.rs`, `error.rs`, `route.rs` | ~250 | client pool, routing |
| 25 | `rpc/grpc/server/src/connection_handler.rs` | ~250 | gRPC server listener; `RPC_MAX_MESSAGE_SIZE`; connection admission |
| 26 | `rpc/grpc/server/src/connection.rs` | ~400 | per-connection receive loop; routing; `enqueue` |
| 27 | `rpc/grpc/server/src/manager.rs` | — | connection manager; capacity limits |
| 28 | `rpc/grpc/server/src/request_handler/*.rs` | ~500 | `factory.rs` (method registration, queue sizes), `handler.rs`, `interface.rs`, `method.rs` |
| 29 | `rpc/grpc/server/src/adaptor.rs`, `collector.rs`, `error.rs`, `service.rs` | ~400 | adaptor/collector/service |
| 30 | `rpc/wrpc/server/src/server.rs` | 229 | WebSocket server; connection map; `connect`/`disconnect` |
| 31 | `rpc/wrpc/server/src/service.rs` | 213 | `MAX_WRPC_MESSAGE_SIZE` (128MB); bind/listen |
| 32 | `rpc/wrpc/server/src/connection.rs` | 184 | WebSocket connection; notify |
| 33 | `rpc/wrpc/server/src/router.rs` | 105 | wRPC router |
| 34 | `rpc/wrpc/server/src/address.rs` | 102 | address parsing |
| 35 | `rpc/wrpc/client/src/*.rs` | ~1,500 | wRPC client; `parse.rs` (URL parsing); `client.rs` (1GB message/frame size) |
| 36 | `rpc/wrpc/proxy/src/main.rs` | 104 | wRPC proxy; 1GB message size |
| 37 | `rpc/wrpc/wasm/src/*.rs` | ~1,600 | WASM wRPC client |

### 1.2 Protocol — `protocol/` (7,329 lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 38 | `protocol/p2p/src/handshake.rs` | ~90 | version/verack/ready exchange; timeouts |
| 39 | `protocol/p2p/src/common.rs` | ~180 | `ProtocolError`; `dequeue!`/`dequeue_with_timeout!` macros |
| 40 | `protocol/p2p/src/core/connection_handler.rs` | ~230 | P2P server; `P2P_MAX_MESSAGE_SIZE` (1GB); connect |
| 41 | `protocol/p2p/src/core/router.rs` | ~460 | message routing; overflow policy; `route_to_flow`; `subscribe` panics |
| 42 | `protocol/p2p/src/core/hub.rs` | ~250 | peer map; broadcast; duplicate/loopback handling |
| 43 | `protocol/p2p/src/core/adaptor.rs` | ~120 | `ConnectionInitializer`; connect_peer |
| 44 | `protocol/p2p/src/core/peer.rs`, `payload_type.rs` | — | peer key/identity |
| 45 | `protocol/p2p/src/convert/*.rs` | ~1,200 | protobuf↔core conversions; `messages.rs`, `net_address.rs`, `header.rs`, `tx.rs`, `utxo.rs`, `block.rs`, `ghostdag.rs`, `pruning.rs`, `subnets.rs`, `trusted.rs`, `option.rs`, `hash.rs`, `error.rs` |
| 46 | `protocol/p2p/src/convert/model/version.rs` | ~60 | `Version`; `MAX_USER_AGENT_LEN` (256) — **only applied on send, not receive** |
| 47 | `protocol/p2p/src/echo.rs` | ~200 | echo test initializer |
| 48 | `protocol/flows/src/flow_context.rs` | ~800 | `initialize_connection`; version validation; flow registration |
| 49 | `protocol/flows/src/flow_trait.rs`, `flowcontext/mod.rs` | — | flow trait; context |
| 50 | `protocol/flows/src/flowcontext/process_queue.rs` | ~60 | `ProcessQueue` |
| 51 | `protocol/flows/src/flowcontext/orphans.rs` | ~400 | orphan block pool; eviction; `max_orphans` |
| 52 | `protocol/flows/src/flowcontext/transactions.rs` | — | `MAX_INV_PER_TX_INV_MSG`; tx request tracking |
| 53 | `protocol/flows/src/service.rs` | — | flow service |
| 54 | `protocol/flows/src/v5/*.rs` (address, ping, request_*, blockrelay, ibd, txrelay) | ~2,500 | inv/getdata; tx relay; block relay; IBD; request handlers |
| 55 | `protocol/flows/src/v5/txrelay/flow.rs` | ~300 | **tx relay flooding**; `MAX_TPS_THRESHOLD`; spam counter |
| 56 | `protocol/flows/src/v5/blockrelay/flow.rs`, `handle_requests.rs` | ~500 | block relay; request handling |
| 57 | `protocol/flows/src/v5/ibd/*.rs` | ~600 | IBD negotiation/progress/streams |
| 58 | `protocol/flows/src/v6/*.rs` | ~250 | v6 flows |

### 1.3 Components — `components/` (2,074 lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 59 | `components/addressmanager/src/lib.rs` | 624 | **address manager poisoning**; `add_address`; `ban`/`unban`; `MAX_ADDRESSES` (4096); weighted iterator |
| 60 | `components/addressmanager/src/stores/address_store.rs` | 120 | DB key encoding; `ADDRESS_KEY_SIZE` |
| 61 | `components/addressmanager/src/stores/banned_address_store.rs` | 109 | ban store; `MAX_BANNED_TIME` |
| 62 | `components/addressmanager/src/port_mapping_extender.rs` | 93 | UPnP |
| 63 | `components/connectionmanager/src/lib.rs` | 340 | **connection management**; outbound/inbound limits; DNS seeding; `ban` |
| 64 | `components/consensusmanager/src/lib.rs` | 238 | consensus manager; staging commit |
| 65 | `components/consensusmanager/src/session.rs` | 486 | `ConsensusSessionOwned`; `unguarded_session`; contract store accessors |
| 66 | `components/consensusmanager/src/batch.rs` | 30 | block batch |

---

## 2. Attack surfaces & vulnerability categories per file

### 2.1 CRITICAL — Zyanya contract RPC handlers (unauthenticated state mutation)

**Files**: `rpc/service/src/service.rs` (lines ~586–810), `rpc/core/src/api/rpc.rs` (~485–556), `rpc/core/src/api/ops.rs` (160–164), `rpc/grpc/core/src/ops.rs` (DeployContract..CallContract), `components/consensusmanager/src/session.rs` (contract accessors).

- **Consensus bypass / unauthenticated state write** (`deploy_contract_call` ~614–635, `invoke_contract_call` ~676–716): on mempool rejection the handler falls back to direct `ContractProcessor::process_contract_tx` and **persists** `write_contract_code`/`write_contract_storage`/`write_contract_balance` to the durable consensus store with no mined transaction, no GhostDAG ordering, no auth. (Phase 1 `C-01` — re-verify and expand; this phase must confirm the exact current line numbers and any new variants.)
- **No authentication on any RPC surface**: `deploy`/`invoke`/`call`/`get_contract_*` are reachable over gRPC and wRPC with no auth. `Shutdown`, `Ban`, `Unban`, `AddPeer` are also unauthenticated admin methods.
- **Unbounded input**: `request.bytecode`, `request.parameters`, `request.calldata` are `Vec<u8>` with no size cap → memory DoS and VM gas bypass (bytecode size is not bounded before `OpCode::deserialize_slice`).
- **Panic vectors**: `request.contract_address.as_bytes().try_into().unwrap()` at lines ~691, ~733, ~797 — verify `RpcHash::as_bytes()` is always 32 bytes; if the type ever changes or a malformed hash arrives, this panics the handler task.
- **`call_contract_call`** (~780–810): `calldata.chunks_exact(8)` pushes unbounded values onto the VM stack; `max_gas` is client-supplied (no cap) → CPU DoS.
- **`get_contract_state_call` / `get_contract_code_call`**: read-only but unauthenticated; confirm no write path.

### 2.2 CRITICAL/HIGH — Message size limits & memory allocation (DoS)

**Files**: `rpc/grpc/core/src/lib.rs` (`RPC_MAX_MESSAGE_SIZE = 1GB`), `rpc/grpc/server/src/connection_handler.rs:139`, `rpc/grpc/client/src/lib.rs:570`, `protocol/p2p/src/core/connection_handler.rs` (`P2P_MAX_MESSAGE_SIZE = 1GB`), `rpc/wrpc/server/src/service.rs` (`MAX_WRPC_MESSAGE_SIZE = 128MB`), `rpc/wrpc/client/src/client.rs:443-444` (1GB), `rpc/wrpc/proxy/src/main.rs:99` (1GB).

- 1GB decoding limits allow a single peer to force ~1GB allocations per message; combined with gzip decompression (compression bombs) this is a memory-exhaustion DoS.
- Check whether `max_encoding_message_size` is set anywhere (it is not in the server paths) — outbound responses are unbounded.
- Check gzip decompression limits (tonic `CompressionEncoding::Gzip` accepted on both P2P and gRPC servers).

### 2.3 HIGH — P2P handshake & peer identity (eclipse / spoofing)

**Files**: `protocol/p2p/src/handshake.rs`, `protocol/p2p/src/core/connection_handler.rs`, `protocol/p2p/src/core/hub.rs`, `protocol/p2p/src/core/router.rs`, `protocol/p2p/src/core/peer.rs`, `protocol/flows/src/flow_context.rs` (~700–780), `protocol/p2p/src/convert/model/version.rs`, `protocol/p2p/src/convert/messages.rs`.

- **`user_agent` not bounded on receive**: `Version::try_from` (`convert/messages.rs:44-60`) clones `msg.user_agent` verbatim; `MAX_USER_AGENT_LEN` truncation only happens in `add_user_agent` (send path). A peer can send a multi-MB user_agent (up to the 1GB message cap) → memory DoS and log flooding (`{version_message:?}` debug logs).
- **`timestamp` cast**: `msg.timestamp as u64` (`convert/messages.rs:50`) — negative i64 timestamps wrap to huge u64; `time_offset = unix_now() as i64 - peer_version_message.timestamp` (`flow_context.rs:714`) can underflow/overflow i64.
- **No handshake rate limiting / connection cap**: `message_stream` (`connection_handler.rs:200-220`) accepts every inbound connection and spawns a router + full flow set; no per-IP limit, no max-connections check at the P2P layer (gRPC has a manager, P2P does not).
- **Peer identity spoofing**: `router.set_identity(peer_version.id)` trusts the peer's self-declared `PeerId`; `Hub` keys on `(PeerId, IP)` — verify duplicate/loopback checks (`flow_context.rs:717-723`) and whether a peer can claim another peer's id to evict it (`hub.rs insert_new_router` closes the previous router on duplicate key).
- **`response_id` routing** (`router.rs:372-380`): any peer can set `response_id` to target arbitrary registered routes; verify flows validate the response matches the request (txrelay does check `transaction_id != request.req`).

### 2.4 HIGH — Transaction relay flooding & mempool DoS

**Files**: `protocol/flows/src/v5/txrelay/flow.rs`, `protocol/flows/src/flowcontext/transactions.rs`, `protocol/flows/src/flowcontext/process_queue.rs`.

- `MAX_INV_PER_TX_INV_MSG` bound on invs; `invs_channel_size = 4096`; `txs_channel_size = MAX_INV_PER_TX_INV_MSG`.
- `request_transactions` throttling (`MAX_TPS_THRESHOLD = 3000`) — verify the `limit`/`overage` arithmetic (`saturating_sub`) and that `requests.len() >= limit as usize` cannot be bypassed.
- `spam_counter` only logs every 100 spam txs; no ban/disconnect for spam peers (TODO in code).
- `RequestTransactionsFlow` responds to any `RequestTransactionsMessage` with any tx in the mempool — no rate limit on the responder side → amplification.
- `ProcessQueue` unbounded growth if `enqueue_chunk` is fed unbounded input (verify callers bound it).

### 2.5 HIGH — Address manager poisoning & connection management

**Files**: `components/addressmanager/src/lib.rs`, `components/addressmanager/src/stores/*.rs`, `components/connectionmanager/src/lib.rs`.

- `add_address` accepts any non-loopback/non-unspecified `NetAddress` with no port validation, no source validation, no rate limit → attacker can flood the address store (bounded to `MAX_ADDRESSES = 4096` via `keep_limit`, but eviction is by `connection_failed_count` max — verify eviction logic and that a flood can displace good peers).
- `mark_connection_failure`: `connection_failed_count + 1` (u64) — overflow only after 2^64 failures (note as LOW/theoretical).
- `RandomWeightedIterator::new` (`lib.rs:~500`): `panic!("{e}")` on `WeightedError` other than `NoItem` (e.g. `InvalidWeight` from NaN/negative). Weights are `64f64.powf(...)` (always positive) — verify no path produces NaN/negative (LOW).
- `ban`/`unban`/`is_banned`: `unix_now() - timestamp.0` in `is_banned` — if `timestamp.0` is in the future (clock skew) this underflows u64 → permanent ban bypass (verify).
- `connectionmanager`: `handle_inbound_connections` disconnects random inbound peers above `inbound_limit`; `dns_seed_single` adds all resolved addresses without validation; `ban` skips IPs with permanent connections.

### 2.6 HIGH — gRPC deserialization & request handling

**Files**: `rpc/grpc/server/src/connection.rs`, `rpc/grpc/server/src/request_handler/*.rs`, `rpc/grpc/core/src/convert/*.rs`, `rpc/grpc/core/src/ops.rs`.

- `route_to_handler` (`connection.rs:170-200`): `request.payload.as_ref().unwrap()` after a `is_none` check — safe but fragile; `get_or_subscribe` creates a bounded channel per op (`method.queue_size()`, default 256) and spawns `method.tasks()` handlers — verify queue sizes and that `DropIfFull` (SubmitBlock) is the only backpressure path; other ops use `Enqueue` (async send) which can grow memory if handlers stall.
- `convert/*.rs` `expect`/`unwrap` on conversions (e.g. `header.rs:18` `timestamp.try_into().expect(...)`, `message.rs:159` `String::from_utf8(...).expect(...)`) — verify these cannot be triggered by untrusted input (timestamp i64→u64, extra_data UTF-8).
- `ZyanyadPayloadOps::to_error_response` — verify every op has a response message and no panic on unknown op.
- `interface.rs:54` `panic!("RPC method {op:?} is declared multiple times")` — startup-only, not attacker-reachable (note).

### 2.7 HIGH — WebSocket (wRPC) connection handling

**Files**: `rpc/wrpc/server/src/server.rs`, `rpc/wrpc/server/src/service.rs`, `rpc/wrpc/server/src/connection.rs`, `rpc/wrpc/server/src/router.rs`, `rpc/wrpc/proxy/src/main.rs`.

- `Server::connect` (`server.rs:100-130`): no connection limit; `sockets: Mutex<HashMap<u64, Connection>>` grows unbounded; `next_connection_id.fetch_add` wraps at u64::MAX (theoretical).
- `handshake` (`service.rs:70-90`): **handshake is commented out** — no greeting/negotiation; any client can connect and issue any RPC.
- `MAX_WRPC_MESSAGE_SIZE = 128MB` server-side vs 1GB client/proxy — inconsistent; proxy (`main.rs:99`) allows 1GB.
- `disconnect` (`server.rs:150-165`): `self.inner.sockets.lock().unwrap()` — a poisoned mutex panics (LOW).
- `connection.rs:161` `create_serialized_notification_message(...).unwrap()` — serialization failure panics (LOW).

### 2.8 MEDIUM — Serialization/deserialization panic vectors & unsafe

**Files**: `rpc/core/src/model/hex_cnv.rs`, `rpc/core/src/model/*.rs`, `rpc/core/src/convert/*.rs`, `protocol/p2p/src/convert/*.rs`, `rpc/wrpc/client/src/parse.rs`.

- `hex_cnv.rs:26` `unsafe { str::from_utf8_unchecked(&hex) }` — safe only because `faster_hex::hex_encode` emits ASCII; verify no other caller.
- `FromRpcHex for Vec<u8>` (`hex_cnv.rs:44-52`): `hex_str.len() / 2` — odd-length input is rejected by `hex_decode` (returns Err) but the allocation is already made; verify no panic on odd length.
- `protocol/p2p/src/convert/net_address.rs:48-50`: `chunk.try_into().expect(...)` and `Ipv6Addr::from(<[u16;8]>::try_from(octets).unwrap())` — verify length checks precede these.
- `rpc/wrpc/client/src/parse.rs`: URL/host parsing — verify no panic on malformed input (IPv6 bracket handling, port parsing).
- borsh/serde deserialization of request types — verify no `unwrap` on untrusted bytes in `rpc/core/src/model/*.rs`.

### 2.9 MEDIUM — RPC method exposure & safe-mode gating

**Files**: `rpc/service/src/service.rs` (safe-mode checks ~1102–1219, 1404), `rpc/core/src/api/rpc.rs`, `rpc/core/src/api/ops.rs`.

- Enumerate every method reachable without auth: `Shutdown` (133), `Ban` (139), `Unban` (140), `AddPeer` (124), `SubmitBlock` (117), `SubmitTransaction` (125), `SubmitTransactionReplacement` (146), `ResolveFinalityConflict` (132), plus the 5 contract ops (160–164).
- Verify `unsafe_rpc` gating: `submit_transaction_call` (~541-542) `allow_orphan`; `get_block_template`/`get_utxos_by_addresses`/`get_balances_by_addresses`/`get_coin_supply`/`get_daa_score_timestamp_estimate` safe-mode checks; `window_size > MAX_SAFE_WINDOW_SIZE` (1102); blanket `UtxosChanged` subscription restriction (1404).
- Confirm the gating is enforced on **both** gRPC and wRPC paths (both call the same `RpcCoreService`, but verify the proxy path does not bypass).

### 2.10 LOW — Integer overflow/underflow in math & misc

**Files**: `rpc/service/src/service.rs` (DAA timestamp estimate ~960-980), `protocol/flows/src/v5/txrelay/flow.rs` (TPS math), `components/addressmanager/src/lib.rs` (ban timestamp), `protocol/flows/src/flow_context.rs` (time_offset).

- `get_daa_score_timestamp_estimate_call`: `(curr_daa_score - header.daa_score).checked_mul(...)` uses `checked_mul` but `curr_daa_score - header.daa_score` is a plain `u64` subtraction guarded by `header.daa_score <= curr_daa_score` — verify.
- `check_tx_throttling`: `1000 * snapshot_delta.low_priority_tx_counts` can overflow u64 at extreme counts (LOW).
- `time_offset` i64 arithmetic (`flow_context.rs:714`).

---

## 3. Priority order (CRITICAL first)

1. **CRITICAL** — Zyanya contract RPC handlers: consensus bypass, unauthenticated state mutation, unbounded bytecode/calldata, panic vectors (`rpc/service/src/service.rs`).
2. **CRITICAL** — Message size limits (1GB) + gzip decompression bombs across gRPC/P2P/wRPC.
3. **HIGH** — P2P handshake: unbounded `user_agent` on receive, timestamp cast, no connection/rate limits, peer-id spoofing/duplicate eviction.
4. **HIGH** — Transaction relay flooding / mempool DoS / responder amplification.
5. **HIGH** — Address manager poisoning + connection manager (flood, eviction, ban bypass).
6. **HIGH** — gRPC request handling: queue growth, conversion `expect`/`unwrap`, error mapping.
7. **HIGH** — wRPC: no handshake, no connection limit, inconsistent message sizes.
8. **MEDIUM** — Serialization/deserialization panic vectors & unsafe blocks.
9. **MEDIUM** — RPC method exposure & safe-mode gating (admin methods unauthenticated).
10. **LOW** — Integer overflow/underflow in math, poisoned-mutex panics, theoretical u64 wraps.

---

## 4. Methodology / steps for the builder

1. **Re-verify Phase 1 `C-01`** first (contract RPC fallback) — confirm current line numbers in `rpc/service/src/service.rs` and check for new variants (e.g. `call_contract_call` write path, `get_contract_*`).
2. **Trace the full request path** for each transport: gRPC (`connection_handler.rs` → `connection.rs` → `request_handler/*` → `RpcCoreService`) and wRPC (`service.rs` → `server.rs` → `router.rs` → `RpcCoreService`), and the proxy path (`proxy/main.rs` → gRPC client). Confirm which methods are reachable and whether any auth/safe-mode gate is bypassed.
3. **Enumerate all `unwrap`/`expect`/`panic!`/`unreachable!`/`unsafe`** in the listed files and classify each as attacker-reachable vs startup-only. (A grep already shows the hotspots; verify each.)
4. **Check every message-size constant** and whether `max_encoding_message_size` / gzip decompression limits are set; assess memory-exhaustion impact.
5. **P2P handshake deep-dive**: read `handshake.rs`, `flow_context.rs::initialize_connection`, `convert/messages.rs`, `convert/model/version.rs`; test the `user_agent`/`timestamp`/`PeerId` claims.
6. **Tx relay & orphan pool**: read `txrelay/flow.rs`, `flowcontext/transactions.rs`, `flowcontext/orphans.rs`, `process_queue.rs`; verify bounds and throttling arithmetic.
7. **Address/connection managers**: read `addressmanager/src/lib.rs` + stores, `connectionmanager/src/lib.rs`; verify poisoning, eviction, ban logic.
8. **Write findings** to `audit_reports/phase4_findings.md` in the established format (severity-ordered, stable IDs, `file:line`, description, code snippet, recommended fix). Re-verify every `file:line` against the current tree. Do not modify source.

## 5. Output format (per finding)

```
### [C-01] <title>
- **File**: `path:line` (and related lines)
- **Description**: ...
- **Code**: ```rust ... ```
- **Recommended fix**: ...
```

Order: CRITICAL → HIGH → MEDIUM → LOW, then by file path. Use stable IDs (`C-01`, `H-01`, `M-01`, `L-01`) so later phases can reference them.
