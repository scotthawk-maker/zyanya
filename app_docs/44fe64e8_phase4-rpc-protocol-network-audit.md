# Phase 4 Security Audit — RPC + Protocol + Network Attack Surface

**Session**: `44fe64e8`
**Date**: 2026-08-18
**Status**: Completed and reviewed (APPROVED)

## What changed and why it matters

This session produced a deep security audit of all network-facing code in the
Zyanya blockchain codebase (~29,000 lines across 66 source files in 15
modules). The audit covers the complete attack surface — every byte that
crosses a socket, gRPC stream, or WebSocket connection — and identifies 29
security findings (5 CRITICAL, 10 HIGH, 7 MEDIUM, 7 LOW).

The single deliverable is **`audit_reports/phase4_findings.md`** (886 lines).
No source code was modified; this phase is audit + report only.

The findings matter because the network-facing code is the primary attack
surface for any blockchain node. The 5 CRITICAL findings alone describe
exploitable paths for consensus bypass, unauthenticated state mutation,
memory exhaustion DoS, and unauthenticated access to all admin and
smart-contract methods. Several of these are re-verified from Phase 1 with
confirmed current line numbers.

## Files that carry the work

| File | Lines | Role |
|------|-------|------|
| `audit_reports/phase4_findings.md` | 886 | **Primary deliverable** — full audit report with 29 findings |
| `specs/44fe64e8_phase4-rpc-protocol-network-audit.md` | 239 | Audit plan / spec (produced by the planner stage) |
| `adws/adw_data/sessions/44fe64e8/context_handoff/plan.md` | 239 | Context-handoff copy of the plan |
| `adws/adw_data/sessions/44fe64e8/context_handoff/review.md` | 56 | Reviewer's spot-verification report (APPROVED) |

All other changed files in the diff are session metadata (agent maps,
builder/reviewer PI session logs, events, database files) — not audit content.

## Audit scope

### RPC modules (18,837 lines)
- `rpc/core/src/` — method definitions, types, error handling, model/convert/notify/wasm (10,093 lines)
- `rpc/service/src/` — handler implementations including Zyanya contract RPC handlers (1,837 lines)
- `rpc/macros/src/` — RPC macros (883 lines)
- `rpc/grpc/core/src/` — gRPC core, protowire (3,000 lines)
- `rpc/grpc/client/src/` — gRPC client (1,450 lines)
- `rpc/grpc/server/src/` — gRPC server (2,245 lines)
- `rpc/wrpc/server/src/` — WebSocket RPC server (877 lines)
- `rpc/wrpc/client/src/` — WebSocket RPC client (1,517 lines)
- `rpc/wrpc/proxy/src/` — WebSocket proxy (131 lines)
- `rpc/wrpc/wasm/src/` — WASM WebSocket (1,595 lines)

### Protocol modules (7,329 lines)
- `protocol/p2p/src/` — P2P handshake, peer messaging (2,786 lines)
- `protocol/flows/src/` — P2P flows, inv/getdata, transaction relay (4,543 lines)

### Component modules (2,074 lines)
- `components/addressmanager/src/` — peer address management (980 lines)
- `components/connectionmanager/src/` — connection management (340 lines)
- `components/consensusmanager/src/` — consensus manager (754 lines)

## Findings summary

### CRITICAL (5)

| ID | Title | File |
|----|-------|------|
| C-01 | RPC fallback bypasses consensus, persisting contract state without a mined transaction | `rpc/service/src/service.rs:614-635, 676-716` |
| C-02 | Unbounded bytecode/calldata/max_gas — memory DoS and VM gas bypass | `rpc/service/src/service.rs:589, 660, 770, 773` |
| C-03 | Panic vectors in contract RPC handlers — `.try_into().unwrap()` on contract address | `rpc/service/src/service.rs:691, 733, 797` |
| C-04 | 1 GB message size limits across all transports — memory exhaustion DoS | `rpc/grpc/core/src/lib.rs:8`, `protocol/p2p/src/core/connection_handler.rs:45`, others |
| C-05 | No authentication on any RPC method — admin and contract methods remotely accessible | `rpc/service/src/service.rs`, `rpc/core/src/api/ops.rs:117-164`, `rpc/wrpc/server/src/service.rs:70-86` |

**C-01** is the most severe: on mempool rejection, the `deploy_contract_call`
and `invoke_contract_call` handlers fall back to direct contract execution and
**write the resulting state to the durable consensus store** — without a mined
transaction, without GhostDAG ordering, and without consensus validation. Since
the constructed transactions have no inputs and no outputs, the mempool
routinely rejects them, making the fallback the *primary* path in practice.
There is no authentication on the RPC surface, so any network client can
trigger this.

**C-04** notes that gRPC, P2P, and wRPC proxy all accept messages up to 1 GB,
with gzip compression accepted on all transports — a classic compression-bomb
vector. `max_encoding_message_size` is never set on any server, so outbound
responses are also unbounded.

**C-05** documents that the wRPC handshake is commented out (`// TODO - discuss
and implement handshake`), meaning any WebSocket client can immediately issue
any RPC including `Shutdown`, `Ban`, `AddPeer`, and all contract methods.

### HIGH (10)

| ID | Title | File |
|----|-------|------|
| H-01 | P2P `user_agent` not bounded on receive — memory DoS and log flooding | `protocol/p2p/src/convert/messages.rs:50-53` |
| H-02 | P2P timestamp cast `i64` → `u64` and time_offset arithmetic — integer wraparound | `protocol/p2p/src/convert/messages.rs:50`, `protocol/flows/src/flow_context.rs:714` |
| H-03 | No P2P connection limit or per-IP rate limiting — resource exhaustion | `protocol/p2p/src/core/connection_handler.rs:202-223` |
| H-04 | Peer identity spoofing — self-declared `PeerId` trusted; duplicate eviction | `protocol/flows/src/flow_context.rs:717-725`, `protocol/p2p/src/core/hub.rs:79-84` |
| H-05 | Transaction relay: no ban/disconnect for spam peers — amplification | `protocol/flows/src/v5/txrelay/flow.rs:229-237, 271-292` |
| H-06 | Address manager poisoning — no source validation, eviction can displace good peers | `components/addressmanager/src/lib.rs:254-264, 386-392` |
| H-07 | Address manager `is_banned` — u64 underflow on future timestamp (ban bypass) | `components/addressmanager/src/lib.rs:306-316` |
| H-08 | wRPC server has no connection limit — unbounded WebSocket connections | `rpc/wrpc/server/src/server.rs:112-145` |
| H-09 | Inconsistent wRPC message size limits — proxy allows 1 GB while server allows 128 MB | `rpc/wrpc/server/src/service.rs:18`, `rpc/wrpc/proxy/src/main.rs:99` |
| H-10 | gRPC conversion `expect()` / `unwrap()` on untrusted input — panic vectors | `rpc/grpc/core/src/convert/header.rs:18`, `message.rs:159`, `protocol/p2p/src/convert/net_address.rs:47-50` |

### MEDIUM (7)

| ID | Title | File |
|----|-------|------|
| M-01 | `unsafe { str::from_utf8_unchecked }` in hex conversion — sound but fragile | `rpc/core/src/model/hex_cnv.rs:26` |
| M-02 | `RequestTransactionsFlow` responds to unlimited tx requests — no per-peer rate limit | `protocol/flows/src/v5/txrelay/flow.rs:282-300` |
| M-03 | `RandomWeightedIterator::new` panics on `WeightedError` — not `NoItem` | `components/addressmanager/src/lib.rs:470-478` |
| M-04 | wRPC `disconnect` uses `.lock().unwrap()` — panic on poisoned mutex | `rpc/wrpc/server/src/server.rs:160` |
| M-05 | wRPC notification serialization `.unwrap()` — panic on serialization failure | `rpc/wrpc/server/src/connection.rs:159-161` |
| M-06 | RPC error responses may leak internal state information | `rpc/core/src/error.rs` |
| M-07 | `MAX_INV_PER_TX_INV_MSG = 131,072` — very large inv messages allowed | `protocol/flows/src/flowcontext/transactions.rs:16` |

### LOW (7)

| ID | Title | File |
|----|-------|------|
| L-01 | TPS throttle math can overflow on extreme counts | `protocol/flows/src/v5/txrelay/flow.rs:143, 313` |
| L-02 | `Hub::broadcast_to_some_peers` asserts `num_peers > 0` — panic on misuse | `protocol/p2p/src/core/hub.rs:136` |
| L-03 | `Router::enqueue` uses `assert!` for payload presence — panic on internal bug | `protocol/p2p/src/core/router.rs:405-406` |
| L-04 | `connection_failed_count + 1` can theoretically overflow u64 | `components/addressmanager/src/lib.rs:271-277` |
| L-05 | `hub_sender.send().expect()` in multiple P2P paths — panic if hub receiver drops | `protocol/p2p/src/core/connection_handler.rs:129, 218` |
| L-06 | `P2P Server` and `gRPC Server` panic on serve error — no graceful degradation | `protocol/p2p/src/core/connection_handler.rs:79`, `rpc/grpc/server/src/connection_handler.rs:160` |
| L-07 | `DaaScoreTimestampEstimate` — division by zero if headers have equal DAA scores | `rpc/service/src/service.rs:967-972` |

## How to verify

1. **Read the report**: `audit_reports/phase4_findings.md` — 886 lines, 29 `### [...]` finding headers, ordered CRITICAL → HIGH → MEDIUM → LOW, then by file path.

2. **Spot-check line numbers**: The reviewer performed spot-verification of all CRITICAL and HIGH findings against the current source tree. For example:
   - `grep -n 'try_into().unwrap()' rpc/service/src/service.rs` returns exactly lines 691, 733, 797 (C-03).
   - `rpc/grpc/core/src/lib.rs:8` contains `RPC_MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 1024` (C-04).
   - `rpc/wrpc/server/src/service.rs:70-80` contains the commented-out handshake (C-05).

3. **Verify severity ordering**: Confirm findings are grouped C-01..C-05, H-01..H-10, M-01..M-07, L-01..L-07.

4. **Confirm no source modification**: `git diff` should show only new files (the audit report and session metadata), no changes to existing source code.

5. **Cross-reference Phase 1**: C-01 notes "(Re-verified from Phase 1 C-01)" — confirm the line numbers in `rpc/service/src/service.rs` match the Phase 1 finding.

## Methodology

The builder agent (GLM-5.2) followed the 8-step plan from the planner stage:
1. Re-verified Phase 1 C-01 (contract RPC fallback) with current line numbers.
2. Traced full request paths for gRPC, wRPC, and proxy transports.
3. Enumerated all `unwrap`/`expect`/`panic!`/`unreachable!`/`unsafe` in the 66 listed files.
4. Checked every message-size constant and `max_encoding_message_size`/gzip settings.
5. Performed P2P handshake deep-dive (user_agent, timestamp, PeerId).
6. Analyzed tx relay and orphan pool bounds/throttling arithmetic.
7. Reviewed address/connection manager poisoning, eviction, and ban logic.
8. Wrote findings in the established format with verified file:line references.

The reviewer (GLM-5.2) independently spot-verified 10 key claims against the
source tree and confirmed all requirements were met. Verdict: APPROVED.