# Phase 5 Security Audit Plan — Explorer + Utils + Remaining Modules

**Target**: Zyanya blockchain (Kaspa/rusty-spectre fork) — modules not covered in Phases 1–4.
**Output**: `audit_reports/phase5_findings.md`
**Date**: 2026-08-18

This phase covers the block explorer (web/HTTP attack surface), the database
layer, daemon/node entry points, math/utils, indexes, the notification system,
mining (block template + mempool), metrics, WASM bindings, CLI, and the
simulation/rothschild/testing crates.

Findings must be written to `audit_reports/phase5_findings.md` in the same
format as phases 1–4 (see `audit_reports/phase1_findings.md` … `phase4_findings.md`):
severity-ordered (CRITICAL → HIGH → MEDIUM → LOW), then by file path, with stable
IDs (`C-01`, `H-01`, …), `file:line`, description, code snippet, and recommended fix.
**Do not modify source code** — audit + report only. Re-verify every `file:line`
against the current tree before writing it.

> **Note on the prompt's "stratum" claim**: there is **no Stratum server** in
> `mining/src/`. The only `stratum` references are a doc comment in
> `consensus/pow/src/wasm.rs:77` and marketing text in `zyanya-explorer/src/web.rs`
> (lines ~2507, 2585, 4425) referencing an external `zyanya-pool` binary that is
> **not present in this repo**. The builder must record this in the report (the
> "Stratum protocol vulnerabilities" focus area is N/A for this tree) and instead
> audit the actual mining surface: block-template builder/selector, mempool, and
> the `get_block_template` RPC path.

---

## 1. Files to audit (with paths)

### 1.1 Explorer — `zyanya-explorer/` (8,172 lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 1 | `zyanya-explorer/src/web.rs` | 5,881 | **XSS**: ~70 `innerHTML` sinks; token metadata (name/symbol/description/twitter/telegram/website) rendered unescaped; error strings echoed into `innerHTML`; SVG injection via `fetch(...).then(t => container.innerHTML = t)` |
| 2 | `zyanya-explorer/src/client.rs` | 1,572 | gRPC client + tx building; **bonding-curve integer overflow**; `submit_signed_tx` trusts attacker-controlled metadata; `decode_base64`; `save_token_icon`/`save_token_metadata` file writes |
| 3 | `zyanya-explorer/src/api.rs` | 592 | Axum handlers; `check_write_enabled` gating gaps; `token_icon_handler` path handling; `api_dag_handler` `limit + offset` overflow; `api_compile_contract_handler` ungated |
| 4 | `zyanya-explorer/src/main.rs` | 127 | IPv6-only listener; route table; no rate limiting / no auth on write endpoints |

### 1.2 Utils + Database (5,000+ lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 5 | `database/src/` (access.rs, writer.rs, key.rs, item.rs, set_access.rs, utils.rs, registry.rs, cache.rs, db.rs, db/conn_builder.rs, errors.rs, lib.rs) | 1,597 | **unsafe** in `registry.rs:87`; `unwrap`/`expect` on DB open/destroy; key encoding; `delete_range` bounds; cache eviction |
| 6 | `daemon/src/` (lib.rs, zyanyad/mod.rs, cpu_miner/mod.rs, error.rs, imports.rs, result.rs) | 834 | daemon process management; binary location; `expect` on `Daemons::zyanyad()`/`cpu_miner()`; config passthrough |
| 7 | `zyanyad/src/` (args.rs, daemon.rs, main.rs, lib.rs) | 1,288 | **config parsing** (`deny_unknown_fields`, `serde(default)`); `ram_scale` f64 unbounded; `async_threads`/`max_tracked_addresses` bounds; `network()` panic; config-file TOML injection |
| 8 | `core/src/` (log/*, task/*, time.rs, signals.rs, panic.rs, core.rs, assert.rs, console.rs, zyanyad_env.rs, service.rs, lib.rs) | 965 | logging; panic hooks; signal handling; task runtime |
| 9 | `math/src/` (uint.rs, int.rs, lib.rs, wasm.rs) | 1,560 | **unsafe `from_utf8_unchecked`** in `uint.rs:757,816`; `Display`/`LowerHex` LUT indexing; `overflowing_*` correctness; `int.rs` signed math |
| 10 | `utils/src/` (networking.rs, hex.rs, serde_bytes*, channel.rs, expiring_cache.rs, sync/*, fd_budget.rs, mem_size.rs, sim.rs, git.rs, sysinfo.rs, rand/seq.rs, refs.rs, triggers.rs, vec.rs, hashmap.rs, iter.rs, any.rs, as_slice.rs, binary_heap.rs, arc.rs, lib.rs) | 3,213 | `networking.rs` `IpNet::from_str(...).unwrap()`; `PrefixBucket` `try_into().expect`; `hex.rs`; `serde_bytes` deserialization; `channel.rs`; `sync/rwlock.rs`/`semaphore.rs`; `fd_budget.rs` |
| 11 | `utils/tower/src/` (middleware.rs, counters.rs, lib.rs) | 85 | tower middleware; counters |

### 1.3 Indexes + Notify (7,000+ lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 12 | `indexes/core/src/` (notification.rs, indexed_utxos.rs, connection.rs, notifier.rs, lib.rs) | 193 | notification types; indexed UTXO model |
| 13 | `indexes/processor/src/` (processor.rs, service.rs, errors.rs, lib.rs) | 377 | index processor; block processing; error handling |
| 14 | `indexes/utxoindex/src/` (index.rs, stores/*, update_container.rs, core/*, testutils/*) | 1,181 | UTXO index; store manager; supply tracking; `expect`/`unwrap` on store access |
| 15 | `notify/src/` (collector.rs, subscriber.rs, scope.rs, subscription/*, events.rs, converter.rs, root.rs, error.rs, connection.rs, address/tracker.rs, notification.rs, notifier.rs, listener.rs, broadcaster.rs, lib.rs) | 5,980 | **unbounded channels** (`subscriber.rs:75`, `notifier.rs:293`); `address/tracker.rs:367` panic on counter underflow; `downcast_ref().unwrap()` in `notification.rs`; subscription capacity; broadcaster backpressure |

### 1.4 Mining (8,625 lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 16 | `mining/src/` (manager.rs, block_template/*, mempool/*, model/*, feerate/*, monitor.rs, cache.rs, lib.rs, errors.rs) | 8,438 | **block template manipulation** (`manager.rs:80` `get_block_template`, `block_template/builder.rs`, `selector.rs`); coinbase/extra-data from `MinerData`; mempool `check_transaction_standard.rs`; `replace_by_fee.rs`; orphan pool; frontier search tree; `topological_sort.rs` |
| 17 | `mining/errors/src/` (manager.rs, mempool.rs, block_template.rs, lib.rs) | 187 | error types |

### 1.5 Other (13,000+ lines)

| # | Path | Lines | What to look for |
|---|------|-------|------------------|
| 18 | `metrics/core/src/` (data.rs, lib.rs, error.rs, result.rs) + `metrics/perf_monitor/src/` | 1,440 | counter overflow; perf monitor sampling |
| 19 | `simpa/src/` (main.rs, simulator/*) | 850 | simulation harness (not production) |
| 20 | `wasm/src/lib.rs` + `wasm/core/src/` (hex.rs, events.rs, types.rs, lib.rs) | 506 | **WASM bindings**: `hex.rs` `ColorRange::try_from` (u32 start/end, `Range` construction, `row_width(0)`); JS-exposed panics; `events.rs` |
| 21 | `testing/integration/src/` | 6,648 | test harness (informational; may reveal attack patterns) |
| 22 | `rothschild/src/main.rs` | 566 | rothschild tool |
| 23 | `cli/src/` + `node-cli/src/` (cli.rs, modules/*, wizards/*, extensions/*, helpers.rs, matchers.rs, notifier.rs, error.rs) | 5,745 | **CLI argument injection**; wallet password/mnemonic handling (`wizards/account.rs`, `wizards/import.rs`); `modules/rpc.rs`, `modules/node.rs`, `modules/miner.rs`; `Secret` handling; shell/command execution |

---

## 2. Attack surfaces & vulnerability categories per file

### 2.1 CRITICAL — Stored XSS in the block explorer (token metadata → `innerHTML`)

**Files**: `zyanya-explorer/src/web.rs` (sinks), `zyanya-explorer/src/client.rs:1225-1234` and `:667-676` (source), `zyanya-explorer/src/api.rs` (write endpoints).

- **Source of attacker-controlled data**: `TokenMetadata { name, symbol, description, twitter, telegram, website, icon_uri }` is persisted to `token-metadata.json` by `save_token_metadata` (`client.rs:259-280`). It is written from:
  - `submit_signed_tx` (`client.rs:1225-1234`) — the `unsigned_tx` hex blob is **fully client-supplied**; `data.name/symbol/description/twitter/telegram/website` are deserialized from it and saved **verbatim** after a successful submit. No sanitization, no length cap.
  - `deploy_bonding_curve_token` (`client.rs:667-676`).
- **Sinks (unescaped `innerHTML`)**:
  - `web.rs:3334-3339` — `socialsHtml.push(\`<a href="${meta.twitter}" ...>\`)` then `social-links.innerHTML = socialsHtml.join('')`. A `twitter` value of `" onmouseover="alert(1)` or `javascript:alert(1)` executes in the victim's browser. **This is the clearest CRITICAL XSS.**
  - `web.rs:1389-1391` — tokens table renders `t.name` / `t.symbol` into `innerHTML`.
  - `web.rs:2343` — `cardsGrid.innerHTML = gridHtml` (token cards).
  - `web.rs:3316-3339` — token page `token-name`/`token-desc` use `innerText` (safe) but `social-links` uses `innerHTML` (unsafe).
  - `web.rs:1271, 1319, 1351, 1364, 1419, 1466, 2971, 5683, 5704-5724` — other `innerHTML` sinks fed by API data.
- **Impact**: any visitor to `/token/:address`, `/explorer`, or `/tools` executes attacker JS → session/credential theft (the explorer has no cookies of its own, but it is a same-origin gateway to the node RPC and can be used for phishing, keylogging, or driving the write endpoints from the victim's browser).
- **Fix**: never build HTML by string concatenation; use `textContent`/`createElement`/`setAttribute`, or a DOM-sanitizer (DOMPurify) for any HTML that must be rendered; validate/escape metadata on write and on read; enforce length limits and a URL scheme allow-list (`https:` only) for `twitter/telegram/website`.

### 2.2 CRITICAL/HIGH — Reflected XSS via error strings echoed into `innerHTML`

**Files**: `zyanya-explorer/src/web.rs:3030, 3073, 3082, 3086, 3396-3430`, `zyanya-explorer/src/client.rs` (error formatting), `zyanya-explorer/src/api.rs` (error JSON).

- `statusEl.innerHTML = \`... ${data.error || 'Unknown error'} ...\`` and `... ${err.message} ...` render server/client error text as HTML. Many error strings interpolate user input, e.g. `format!("Invalid contract address: {}", e)` (`client.rs:602`), `format!("Invalid bytecode hex: {}", e)`, `format!("Invalid signature hex for input {}: {}", i, e)`.
- An attacker can submit a request whose error path echoes a payload (e.g. a malformed address/hex containing `<img src=x onerror=...>`), and the response is rendered into `innerHTML` in the caller's browser → reflected XSS.
- **Fix**: use `textContent` for all status/error messages; never interpolate server strings into HTML; sanitize error text server-side.

### 2.3 HIGH — Bonding-curve integer overflow in explorer tx building (financial)

**Files**: `zyanya-explorer/src/client.rs:897` (buy) and `:1045` (sell).

- `let cost = slope.saturating_mul(2 * S * k + k * k) / 2;` — the inner `2 * S * k + k * k` is **plain `u64` arithmetic** and overflows for large `S` (on-chain `total_supply`) and `k` (attacker-supplied `amount`). In release builds this wraps to a small value, so the buyer pays a fraction of the true bonding-curve cost.
- `let refund = if S >= k { slope.saturating_mul(2 * S * k - k * k) / 2 } else { 0 };` — `2 * S * k` overflows the same way; the `S >= k` guard only prevents the subtraction from underflowing, not the multiplication from overflowing.
- Also `total_in += utxo_entry.amount` (`client.rs:749, 912, 1062`) is unchecked accumulation (bounded by the break condition, but verify).
- **Fix**: use `u128` (or `checked_mul`/`checked_add` with explicit error) for all bonding-curve math; reject `amount`/`supply`/`slope` above a sane cap; add unit tests for overflow boundaries.

### 2.4 HIGH — `submit_signed_tx` trusts client-supplied metadata and re-inits contracts

**Files**: `zyanya-explorer/src/client.rs:1180-1245`.

- The `unsigned_tx` field is a hex-encoded JSON `SignableTxData` blob. The server deserializes it and trusts `data.contract_address`, `data.slope`, `data.name`, `data.symbol`, `data.description`, `data.twitter`, `data.telegram`, `data.website`, `data.icon_uri` **without re-deriving them from the signed transaction**.
- After signature verification, `client.invoke_contract(contract_hash, 0, vec![data.slope], 100_000, 1, 0)` re-initializes the contract with the client-supplied `slope` (only for `Deploy` payloads, but the payload is also client-supplied).
- `save_token_metadata(&data.contract_address, metadata)` persists attacker-controlled strings → feeds the 2.1 stored XSS.
- **Fix**: derive `contract_address` from the tx id (as `build_unsigned_deploy_token_tx` already does) instead of trusting the blob; validate metadata fields (length, charset, URL scheme); do not re-init from client data.

### 2.5 HIGH — Explorer write endpoints are an unauthenticated gateway to node RPC

**Files**: `zyanya-explorer/src/api.rs:313-330` (`check_write_enabled`), `:340-590` (write handlers), `zyanya-explorer/src/main.rs` (routes).

- `check_write_enabled` only checks the `ZYANYA_EXPLORER_ENABLE_WRITE` env var — **no authentication, no rate limiting, no per-IP controls**. When enabled (required for the token-deploy feature), any internet client can call `deploy_contract`, `invoke_contract`, `call_contract`, `submit_signed_tx`, `token_transfer`, `swap_on_dex`, `unsigned-*` endpoints, which proxy to the node's gRPC RPC.
- This **amplifies Phase 4 `C-01`** (unauthenticated contract RPC with consensus-bypass fallback): the explorer exposes that primitive to the public web.
- `api_compile_contract_handler` (`api.rs:583-590`) is **not gated** by `check_write_enabled` at all — arbitrary `source` is compiled by `zyanya_vm::Compiler::compile` with no size limit → CPU/memory DoS and a compiler-bug attack surface.
- **Fix**: require authentication (API key / signed requests) for all write endpoints; rate-limit; cap request body sizes; gate `compile` too; consider removing the write surface from public deployments entirely.

### 2.6 MEDIUM — Explorer path handling & pagination

**Files**: `zyanya-explorer/src/api.rs:35-50` (`token_icon_handler`), `:200-210` (`api_dag_handler`), `zyanya-explorer/src/client.rs:1500-1530` (`decode_base64`), `:270-290` (`save_token_icon`).

- `token_icon_handler` uses `Path::new(&filename).file_name()` which strips `../` — verify this is sufficient (it is on Linux; note Windows `\` is irrelevant here) and that `icons_dir` is not attacker-controlled via env.
- `api_dag_handler`: `limit = ...min(100)`, but `offset` is unbounded; `limit + offset` can overflow `usize` (panic in debug, wrap in release → wrong `get_dag_graph` limit). Use `saturating_add`/`checked_add`.
- `decode_base64` silently skips invalid chars and `=` padding (lenient); `icon_base64` has no explicit size cap (bounded only by axum's default 2 MB body limit) → disk DoS via many large icons. Enforce a max decoded size and reject invalid base64.
- `save_token_icon` writes `{address}.png` — `address` is a hash in current callers, but verify no caller passes raw user input.

### 2.7 MEDIUM — Database layer: unsafe block, panics, key encoding

**Files**: `database/src/registry.rs:87`, `database/src/db/conn_builder.rs:117-137`, `database/src/db.rs:42-43`, `database/src/key.rs`, `database/src/access.rs:166`, `database/src/set_access.rs:145`.

- `registry.rs:87` `std::slice::from_ref(unsafe { &*(self as *const Self as *const u8) })` — sound only because the enum is `#[repr(u8)]` and size 1 (asserted in a test). Document/verify no variant ever exceeds `u8` and the `Separator = u8::MAX` value cannot collide with a real prefix.
- `conn_builder.rs` `.to_str().unwrap()` and `DB::open(...).unwrap()` — panics on non-UTF-8 path or open failure (startup-only, but a malformed `appdir` from config/CLI is a crash vector).
- `db.rs:42-43` `destroy(...).expect(...)` — same.
- `access.rs:166` / `set_access.rs:145` `delete_range(from.unwrap(), to.unwrap())` — verify `from <= to` invariant is enforced by callers (RocksDB panics/UB on inverted ranges).
- `key.rs` `DbKey` — verify `prefix_len` accounting and that `Display` (`self.path[0]`, `self.path[1]`) cannot index out of bounds on empty paths.

### 2.8 MEDIUM — Daemon/node configuration vulnerabilities

**Files**: `zyanyad/src/args.rs` (whole file), `zyanyad/src/daemon.rs`, `daemon/src/zyanyad/mod.rs`, `daemon/src/lib.rs`.

- `ram_scale: f64` (`args.rs:80`) — no bounds check; `0.0`, negative, or `NaN`/`inf` values flow into memory-allocation bounds (`config.ram_scale`). Verify downstream consumers handle these (allocation of 0 or absurd sizes).
- `async_threads: usize`, `max_tracked_addresses: usize`, `rpc_max_clients: usize`, `inbound_limit: usize` — verify upper bounds are enforced (the `max-tracked-addresses` help text claims a `Tracker::MAX_ADDRESS_UPPER_BOUND` cap; confirm it is actually applied, not just documented).
- `network()` (`args.rs:170-180`) panics if more than one of `--mainnet/--testnet/--devnet/--simnet` is set — config error, not attacker-reachable, but note.
- Config file is parsed with `deny_unknown_fields` + `serde(default)` — good; verify no field accepts a value that later causes a panic (e.g. `testnet_suffix`).
- `daemon/src/lib.rs` `Daemons::zyanyad()`/`cpu_miner()` use `.expect(...)` — panics if accessed before init (internal invariant).
- `daemon/src/zyanyad/mod.rs` and `cpu_miner/mod.rs` — process spawn/args; check for command-injection if any user string is passed to a shell (should be `Command::arg` only).

### 2.9 MEDIUM — Notification system DoS (unbounded channels, panics)

**Files**: `notify/src/subscriber.rs:75`, `notify/src/notifier.rs:293`, `notify/src/address/tracker.rs:367`, `notify/src/notification.rs:28-33`, `notify/src/broadcaster.rs`, `notify/src/subscription/single.rs`.

- `Channel::unbounded()` for subscriber `incoming` and notifier `notification_channel` — a slow/stalled consumer lets memory grow without bound → DoS. Verify whether any backpressure exists downstream (broadcaster → listener).
- `address/tracker.rs:367` `panic!("Address tracker is trying to decrease an address counter that is already at zero")` — verify this cannot be triggered by a malicious sequence of subscribe/unsubscribe/UTXO events.
- `notification.rs:28-33` `downcast_ref::<...>().unwrap()` — panics if a subscription type mismatches (internal invariant; verify event routing cannot produce a mismatch from external input).
- `notifier.rs:306` `index.try_into().unwrap()` and `assert!(iter.next().is_none(), ...)` — startup/internal; note.
- `subscription/single.rs` `UtxosChangedSubscriptionData::with_capacity` — verify capacity is bounded by `max_tracked_addresses` and cannot be driven to a huge allocation.

### 2.10 MEDIUM — Mining block template manipulation & mempool

**Files**: `mining/src/manager.rs:80-150` (`get_block_template`), `mining/src/block_template/builder.rs`, `mining/src/block_template/selector.rs`, `mining/src/mempool/check_transaction_standard.rs`, `mining/src/mempool/replace_by_fee.rs`, `mining/src/mempool/model/orphan_pool.rs`, `mining/src/model/topological_sort.rs`.

- `get_block_template(consensus, miner_data)` — `MinerData` (coinbase address, extra data, pay address) arrives via the RPC `get_block_template` request. Verify: coinbase address is validated; `extra_data` is length-bounded; the template cannot be manipulated to produce an invalid/oversized block or to steal fees.
- `BlockTemplateBuilder::modify_block_template` / `build_block_template` — check coinbase output construction, mass/feerate arithmetic, and that `maximum_build_block_template_attempts` bounds the loop.
- `check_transaction_standard.rs` (585 lines) — standardness checks (script size, sig op count, dust, mass); look for bypasses that let non-standard txs into the mempool.
- `replace_by_fee.rs` — RBF logic; verify fee-delta arithmetic and eviction cannot be gamed.
- `orphan_pool.rs` / `frontier.rs` / `search_tree.rs` — unbounded growth / eviction correctness.
- `topological_sort.rs` / `topological_index.rs` — cycle handling; panic on malformed dependency graphs.

### 2.11 LOW/MEDIUM — Math utilities: unsafe UTF-8 and overflow correctness

**Files**: `math/src/uint.rs:757, 816`, `math/src/int.rs`, `math/src/lib.rs`.

- `uint.rs:757` and `:816` `unsafe { core::str::from_utf8_unchecked(...) }` — sound because the buffers are hex/decimal ASCII, but verify the `Display` LUT indexing (`DEC_DIGITS_LUT[d1]`, `d1+1`, `d2`, `d2+1`) cannot go out of bounds for any `u64`/`u128`/`u256` value (indices must stay < 200).
- `overflowing_*` implementations (`uint.rs:67-210`) — verify carry/borrow propagation and shift-by-`BITS` edge cases (`s >= Self::BITS`).
- `int.rs` (142 lines) — signed arithmetic; check `as` casts and negation of `MIN`.
- `lib.rs` — `TryFromIntError` conversions; `as_u128`/`as_u64` truncation paths.

### 2.12 LOW — Utils: networking parsing, serde, channels, fd budget

**Files**: `utils/src/networking.rs:60, 120`, `utils/src/hex.rs`, `utils/src/serde_bytes*/`, `utils/src/channel.rs`, `utils/src/sync/rwlock.rs`, `utils/src/sync/semaphore.rs`, `utils/src/fd_budget.rs`, `utils/src/expiring_cache.rs`.

- `networking.rs:60` `try_into().expect("Slice with incorrect length")` — safe (fixed 8-byte slice) but note.
- `networking.rs:120` `IpNet::from_str(curr_net).unwrap()` — verify `curr_net` is a hardcoded constant, not attacker input.
- `hex.rs` — hex encode/decode; check odd-length and invalid-char handling.
- `serde_bytes*` deserializers — check for panics on malformed input (these are used on RPC/DB boundaries).
- `channel.rs`, `sync/rwlock.rs`, `semaphore.rs` — concurrency primitives; look for deadlock/poisoning panics.
- `fd_budget.rs` — file-descriptor accounting; check for underflow/overflow.

### 2.13 LOW — WASM bindings: JS-triggerable panics

**Files**: `wasm/core/src/hex.rs:88-140`, `wasm/core/src/events.rs`, `wasm/core/src/types.rs`, `wasm/src/lib.rs`.

- `hex.rs` `ColorRange::try_from` — `object.get_u32("start")`/`get_u32("end")`; `start..end` with `end < start` is an empty range (no panic) but `HexViewBuilder::add_colors` with out-of-range indices may panic; `row_width(0)` may panic. These are browser-side DoS only (WASM), not server-side — rate LOW.
- `replace_char` uses `.chars().next().unwrap_or(' ')` — safe.
- `wasm/src/lib.rs` — feature-gated re-exports; no unsafe of note; confirm no `wasm_bindgen` export leaks secrets.

### 2.14 LOW — CLI: argument injection & credential handling

**Files**: `cli/src/cli.rs`, `cli/src/modules/rpc.rs`, `cli/src/modules/node.rs`, `cli/src/modules/miner.rs`, `cli/src/wizards/account.rs`, `cli/src/wizards/import.rs`, `cli/src/helpers.rs`, `node-cli/src/*` (mirror).

- Wallet password/mnemonic via `Secret` and `term.ask(true, ...)` — verify secrets are zeroized and never logged/displayed.
- `modules/rpc.rs` / `node.rs` — check for shell command construction from user input (should use `Command::arg`); check RPC URL parsing.
- `modules/miner.rs` — mining config passthrough; CPU percent parsing.
- `cli` and `node-cli` are near-identical (both 5,745 lines) — audit once, note the duplication.

### 2.15 LOW — Metrics, simpa, rothschild, integration tests

**Files**: `metrics/core/src/data.rs`, `metrics/perf_monitor/src/*`, `simpa/src/*`, `rothschild/src/main.rs`, `testing/integration/src/*`.

- `metrics/core/src/data.rs` (895 lines) — counter arithmetic; check for overflow in rate/delta computations.
- `simpa` / `rothschild` / `testing/integration` — not production attack surface; skim for unsafe/panic patterns and note any that reveal consensus assumptions.

---

## 3. Priority order (CRITICAL first)

1. **CRITICAL** — Stored XSS via token metadata → `innerHTML` (`web.rs:3334-3339` and other sinks; source `client.rs:1225-1234`).
2. **CRITICAL/HIGH** — Reflected XSS via error strings in `innerHTML` (`web.rs:3030-3430`).
3. **HIGH** — Bonding-curve `u64` overflow in explorer tx building (`client.rs:897, 1045`).
4. **HIGH** — `submit_signed_tx` trusts client-supplied metadata + re-init (`client.rs:1180-1245`).
5. **HIGH** — Explorer write endpoints = unauthenticated gateway to node RPC (amplifies Phase 4 C-01); ungated `compile` endpoint (`api.rs:313-330, 583-590`).
6. **MEDIUM** — Explorer path handling & pagination overflow (`api.rs:35-50, 200-210`; `client.rs:1500-1530`).
7. **MEDIUM** — Database layer unsafe/panics/key encoding (`database/src/registry.rs:87`, `conn_builder.rs`, `db.rs`, `key.rs`).
8. **MEDIUM** — Daemon/node config (`ram_scale`, thread/address bounds, `network()` panic) (`zyanyad/src/args.rs`).
9. **MEDIUM** — Notification system DoS (unbounded channels, tracker panic) (`notify/src/*`).
10. **MEDIUM** — Mining block-template manipulation & mempool standardness/RBF (`mining/src/*`).
11. **LOW** — Math unsafe UTF-8 & overflow correctness (`math/src/uint.rs`).
12. **LOW** — Utils networking/serde/channel/fd-budget (`utils/src/*`).
13. **LOW** — WASM bindings JS-triggerable panics (`wasm/core/src/hex.rs`).
14. **LOW** — CLI argument injection & credential handling (`cli/src/*`, `node-cli/src/*`).
15. **LOW** — Metrics/simpa/rothschild/integration tests.

---

## 4. Methodology / steps for the builder

1. **Explorer XSS deep-dive first**: enumerate every `innerHTML`/`outerHTML`/`insertAdjacentHTML`/`document.write` sink in `web.rs` (grep already shows ~70). For each, trace the data source: is it (a) a hex hash (safe), (b) token metadata (attacker-controlled), or (c) an error string (attacker-influenced)? Confirm the stored-XSS chain end-to-end: `submit_signed_tx` → `save_token_metadata` → `token-metadata.json` → `/api/tokens` + `/token/:address` → `innerHTML`.
2. **Verify the reflected-XSS chain**: pick an endpoint whose error path interpolates user input (e.g. `client.rs:602` `Invalid contract address: {e}`) and confirm the JS renders `data.error` into `innerHTML`.
3. **Bonding-curve math**: reproduce the `2 * S * k + k * k` overflow with concrete `S`/`k` values; confirm release-mode wrapping; check the same pattern in `build_unsigned_sell_tx` and any other cost/refund math.
4. **`submit_signed_tx` trust analysis**: confirm `contract_address`/`slope`/metadata are taken from the blob rather than re-derived; assess the re-init `invoke_contract` call.
5. **Write-endpoint gating**: enumerate which handlers call `check_write_enabled` and which do not (`api_compile_contract_handler` does not); confirm no auth/rate-limit anywhere; tie to Phase 4 C-01.
6. **Database layer**: read `registry.rs`, `key.rs`, `conn_builder.rs`, `db.rs`, `access.rs`/`set_access.rs` `delete_range`; classify each `unwrap`/`expect`/`unsafe` as attacker-reachable vs startup-only.
7. **Config**: read `zyanyad/src/args.rs` fully; verify `ram_scale`/`async_threads`/`max_tracked_addresses` bounds are enforced downstream (grep consumers of `config.ram_scale`, `Tracker::MAX_ADDRESS_UPPER_BOUND`).
8. **Notify**: read `subscriber.rs`, `notifier.rs`, `broadcaster.rs`, `address/tracker.rs`, `subscription/single.rs`; confirm unbounded-channel growth and the tracker underflow panic reachability.
9. **Mining**: read `manager.rs::get_block_template`, `block_template/builder.rs`, `selector.rs`, `mempool/check_transaction_standard.rs`, `replace_by_fee.rs`, `orphan_pool.rs`; verify `MinerData` validation and standardness bypasses. Record that no Stratum server exists in-tree.
10. **Math/utils/wasm/cli**: verify the unsafe blocks and LUT indexing in `uint.rs`; check `networking.rs` unwraps; check WASM `hex.rs` panics; check CLI secret handling and command construction.
11. **Write findings** to `audit_reports/phase5_findings.md` in the established format. Re-verify every `file:line` against the current tree. Do not modify source.

## 5. Output format (per finding)

```
### [C-01] <title>
- **File**: `path:line` (and related lines)
- **Description**: ...
- **Code**: ```rust ... ```
- **Recommended fix**: ...
```

Order: CRITICAL → HIGH → MEDIUM → LOW, then by file path. Use stable IDs (`C-01`, `H-01`, `M-01`, `L-01`) so later phases can reference them.
