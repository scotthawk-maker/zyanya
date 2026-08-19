# LOW Security Findings Batch L1 — Remediation (F-L-01 … F-L-47)

**Session:** `01b87849`
**Scope:** All LOW-severity findings from `audit_reports/FINAL_AUDIT_REPORT.md` §6, F-L-01 through F-L-47, skipping F-L-04 (verified safe).
**Verification gate:** `cargo check --workspace --all-targets` must pass after all edits.
**Diff size:** +820 / −153 across 50 tracked files (plus the plan document in `specs/`).

---

## 1. What changed and why it matters

This batch remediates 46 LOW-severity audit findings spanning the full codebase. The unifying theme is **replacing unsafe, panicking, or unchecked code paths with graceful, validated, or saturating equivalents**, plus closing a handful of information-leakage and DoS-hardening gaps. No public API is intentionally altered beyond the signature changes listed in §4.

The fixes fall into seven tiers (mirroring the remediation plan in `specs/01b87849_low-findings-batch-l1.md`):

### P1 — Consensus / VM correctness
| Finding | File | Fix |
|--------|------|-----|
| **F-L-01** | `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` | `check_transaction_outputs_count` error message now reports `tx.outputs.len()` / `self.max_tx_outputs` instead of the input fields. |
| **F-L-02** | `zyanya-vm/src/assembler.rs` | Label parsing reworked: a label must be the *only* token on the line (`:name` or `name:`). Lines like `:loop PUSH 1` now raise `AssemblerError::InvalidLabel` instead of swallowing the opcode. New `InvalidJumpTarget` error variant rejects numeric JUMP targets > 2³¹. |
| **F-L-03** | `zyanya-vm/src/stack.rs` | Documentation-only: comment notes cross-call depth is bounded by the F-C-05 call-depth limit. |
| **F-L-05** | `consensus/core/src/subnets.rs` | `is_builtin()` now includes `SUBNETWORK_ID_SMART_CONTRACT` so smart-contract transactions are validated by all nodes. |
| **F-L-06** | `consensus/src/processes/coinbase.rs` | `red_reward` accumulation uses `saturating_add` for both subsidy and fees, preventing silent overflow. |
| **F-L-07** | `consensus/src/processes/coinbase.rs` | `Vec::with_capacity` hint uses `saturating_add` + `saturating_mul` instead of plain `+` / `*`. |

### P2 — Key / secret handling
| Finding | File(s) | Fix |
|--------|---------|-----|
| **F-L-08** | `wallet/core/src/tx/fees.rs` | `Fees::try_from(&str)` returns an error for an unparseable non-empty string instead of silently mapping to 0. |
| **F-L-09** | `wallet/bip32/src/derivation_path.rs` | `FromStr` enforces a max of 100 path segments. |
| **F-L-10** | `wallet/bip32/src/child_number.rs`, `wallet/bip32/src/xkey.rs` | `from_bytes` now returns `Result` and validates the non-hardened index is `< HARDENED_FLAG`. Caller updated to `?`. |
| **F-L-11** | `wallet/bip32/src/xkey.rs`, `wallet/bip32/src/prefix.rs` | `ExtendedKey::from_str` cross-validates prefix chars vs decoded version via `Prefix::from_version` (now `pub(crate)`); mismatches return `Error::DecodeIssue`. Removes `from_parts_unchecked` usage. |
| **F-L-12** | `wallet/core/src/storage/local/mod.rs` | Three `static mut Option<String>` globals replaced with `OnceLock<String>`, eliminating all `unsafe` blocks. `set_default_*` functions are no longer `unsafe`. |
| **F-L-13** | `wallet/core/src/utxo/context.rs` | `unreachable!` / `panic!` in `promote` and `revive` replaced with `log_error!` + `continue`. |
| **F-L-14** | `wallet/core/src/derivation.rs` | `create_xpub_from_xprv` and `build_derivate_path` return `Err(Error::AccountKindFeature)` for unsupported account kinds instead of `panic!`. |
| **F-L-15** | `wallet/core/src/utxo/context.rs` | `calculate_balance` uses `fold` + `saturating_add` instead of `sum()`. |
| **F-L-16** | `wallet/core/src/account/variants/multisig.rs` | `validate()` rejects > 255 xpub keys; `sig_op_count` uses `unwrap_or(u8::MAX)` instead of `unwrap()`. |
| **F-L-17** | `wallet/core/src/account/mod.rs` | Test helper `bytes_str` uses safe `str::from_utf8().expect()` instead of `unsafe from_utf8_unchecked`. (gen1.rs was already fixed — verify-only.) |
| **F-L-18** | `wallet/core/src/encryption.rs` | `Encrypted` `Debug` impl prints `"[ENCRYPTED]"` instead of ciphertext hex. |
| **F-L-19** | `wallet/bip32/src/prefix.rs` | `From<NetworkId> for Prefix` maps `Testnet` to `Prefix::TPUB` (distinct from Devnet/Simnet `KTUB`). |
| **F-L-20** | `wallet/bip32/src/derivation_path.rs` | `Deserialize` visitor `expecting` message corrected to "a BIP32 derivation path string (e.g. m/44'/123456'/0')". |
| **F-L-46** | `rothschild/src/main.rs` | Private key printed via `println!` (stdout, not logging framework); `info!` log line redacts key to `[REDACTED]`. |
| **F-L-47** | `cli/src/wizards/wallet.rs` | `Secret::from(String)` replaces `Secret::new(s.trim().as_bytes().to_vec())`, letting the `zeroize` crate zero the intermediate `String`. Applied to wallet password, validation, and mnemonic passphrase prompts. |

### P3 — Network / P2P DoS
| Finding | File(s) | Fix |
|--------|---------|-----|
| **F-L-21** | `protocol/flows/src/v5/txrelay/flow.rs` | TPS calculation uses `1000u64.saturating_mul(...)` in both throttle sites. |
| **F-L-22** | `protocol/p2p/src/core/hub.rs` | `broadcast_to_some_peers` returns early when `num_peers == 0` instead of asserting. |
| **F-L-23** | `protocol/p2p/src/core/router.rs` | `Router::enqueue` returns `ProtocolError::Other(...)` when payload is `None` instead of asserting. |
| **F-L-24** | `components/addressmanager/src/lib.rs` | `connection_failed_count + 1` uses `saturating_add`. |
| **F-L-25** | `protocol/p2p/src/core/connection_handler.rs`, `protocol/p2p/src/core/router.rs` | Three `hub_sender.send(...).expect(...)` calls replaced with `if ... .is_err() { warn!(...) }` logging. |
| **F-L-26** | `protocol/p2p/src/core/connection_handler.rs`, `rpc/grpc/server/src/connection_handler.rs`, `rpc/wrpc/server/src/service.rs` | Server serve-error `panic!` arms replaced with `error!` logging (graceful task exit instead of process abort). |
| **F-L-31** | `mining/src/manager.rs`, `protocol/flows/src/flow_context.rs`, `mining/src/manager_tests.rs` | `UnboundedSender<Vec<TransactionId>>` → bounded `Sender<Vec<TransactionId>>` (capacity 1024). Send site uses `blocking_send` with a `warn!` on failure. |
| **F-L-32** | `zyanyad/src/args.rs` | `--uacomment` gains a `value_parser` enforcing ≤ 256 chars, printable ASCII, and no `/`. |

### P4 — RPC / Explorer
| Finding | File(s) | Fix |
|--------|---------|-----|
| **F-L-27** | `rpc/service/src/service.rs` | `DaaScoreTimestampEstimate` guards `score_between_headers == 0.0` before division, falling back to `time_between_headers`. |
| **F-L-29** | `zyanya-explorer/src/main.rs` | New `rate_limit` middleware: per-IP fixed-window limiter (100 req/sec). Explorer served with `into_make_service_with_connect_info::<SocketAddr>()` to extract peer IP. Returns 429 on excess. |
| **F-L-30** | `zyanya-explorer/src/client.rs` | New `write_atomic` helper (temp file + `rename`) used for metadata JSON and icon writes, preventing torn writes. |
| **F-L-33** | `zyanya-explorer/src/main.rs` | New `security_headers` middleware adds `X-Content-Type-Options: nosniff` and `X-Frame-Options: DENY`. |

### P5 — DB / Daemon / CLI
| Finding | File(s) | Fix |
|--------|---------|-----|
| **F-L-35** | `database/src/key.rs` | `DbKey` `Display` impl uses `self.path.get(0)` / `.get(1)` bounds-checked access instead of direct indexing. |
| **F-L-36** | `database/src/db.rs` | `delete_db` returns `Result<(), Box<dyn std::error::Error>>` instead of panicking on RocksDB destroy failure or invalid path encoding. |
| **F-L-37** | `zyanyad/src/args.rs`, `zyanyad/src/daemon.rs`, `testing/integration/src/common/daemon.rs` | `Args::network()` returns `Result<NetworkId, String>` instead of panicking on conflicting flags. All callers updated (`.unwrap()` in tests; `unwrap_or_else` + `exit(1)` in daemon). |
| **F-L-38** | `daemon/src/lib.rs`, `cli/src/modules/{node,miner}.rs`, `node-cli/src/modules/{node,miner}.rs` | `Daemons::zyanyad()` and `cpu_miner()` return `Option` instead of `.expect()`. Callers use `.ok_or(Error::custom(...))?`. |

### P6 — Mining / Mempool
| Finding | File | Fix |
|--------|------|-----|
| **F-L-39** | `mining/src/model/topological_sort.rs` | `assert_eq!` → `debug_assert_eq!` (removes release-build panic). |
| **F-L-40** | `mining/src/mempool/model/orphan_pool.rs` | `get_random_low_priority_orphan` uses `rand::seq::IteratorRandom::choose` for true random eviction. |
| **F-L-41** | `mining/src/mempool/check_transaction_standard.rs` | `assert!` → `debug_assert!` for the contextual-mass invariant. |

### P7 — Math / Utils / WASM
| Finding | File(s) | Fix |
|--------|---------|-----|
| **F-L-28** | `wasm/core/src/events.rs` | `// SAFETY:` comment added to `unsafe impl Send for Sink` documenting single-threaded WASM usage. |
| **F-L-34** | `wasm/core/src/types.rs` | `unsafe from_utf8_unchecked` → safe `str::from_utf8(&hex).expect(...)`. |
| **F-L-42** | `math/src/uint.rs` | Already fixed (verify-only); safe `from_utf8` in both `LowerHex` and `Display`. |
| **F-L-43** | `math/src/uint.rs` | `debug_assert!` bounds checks added before every `DEC_DIGITS_LUT` index read (3 sites). |
| **F-L-44** | `utils/src/hex.rs` | `str::from_utf8(buff).unwrap()` → `str::from_utf8(buff).map_err(D::Error::custom)?`. |
| **F-L-45** | `utils/src/networking.rs` | `IpNet::from_str(curr_net).unwrap()` → `.expect("unroutable_nets contains invalid CIDR")`. |

---

## 2. Files carrying the change

### Consensus / VM
- `consensus/core/src/subnets.rs` — `is_builtin` subnet list.
- `consensus/src/processes/coinbase.rs` — capacity hint + red_reward saturation.
- `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` — error field fix.
- `zyanya-vm/src/assembler.rs` — label parsing + jump-target validation.
- `zyanya-vm/src/stack.rs` — documentation comment (F-L-03).

### Wallet / BIP32
- `wallet/bip32/src/child_number.rs` — `from_bytes` validation.
- `wallet/bip32/src/derivation_path.rs` — segment cap + deserialize message.
- `wallet/bip32/src/prefix.rs` — `from_version` visibility + testnet mapping.
- `wallet/bip32/src/xkey.rs` — prefix cross-validation + `from_bytes` caller.
- `wallet/core/src/account/mod.rs` — test helper safe UTF-8.
- `wallet/core/src/account/variants/multisig.rs` — xpub key count guard + `sig_op_count`.
- `wallet/core/src/derivation.rs` — panic → error on unsupported account kind.
- `wallet/core/src/encryption.rs` — `Debug` redaction.
- `wallet/core/src/storage/local/mod.rs` — `static mut` → `OnceLock`.
- `wallet/core/src/tx/fees.rs` — invalid-fee error propagation.
- `wallet/core/src/utxo/context.rs` — panic → log+continue, balance saturating fold.
- `cli/src/wizards/wallet.rs` — `Secret::from(String)` zeroization.

### Protocol / P2P
- `protocol/flows/src/flow_context.rs` — bounded channel.
- `protocol/flows/src/v5/txrelay/flow.rs` — saturating TPS math.
- `protocol/p2p/src/core/connection_handler.rs` — hub send + serve error logging.
- `protocol/p2p/src/core/hub.rs` — early return on zero peers.
- `protocol/p2p/src/core/router.rs` — enqueue error + hub send logging.
- `components/addressmanager/src/lib.rs` — saturating fail count.

### RPC / Explorer
- `rpc/service/src/service.rs` — div-by-zero guard.
- `rpc/grpc/server/src/connection_handler.rs` — serve-error logging.
- `rpc/wrpc/server/src/service.rs` — serve/bind error logging.
- `zyanya-explorer/src/main.rs` — rate limiter + security headers middleware.
- `zyanya-explorer/src/client.rs` — atomic file writes.

### Mining / Mempool
- `mining/src/manager.rs` — bounded channel sender.
- `mining/src/manager_tests.rs` — bounded channel in tests.
- `mining/src/mempool/check_transaction_standard.rs` — `debug_assert!`.
- `mining/src/mempool/model/orphan_pool.rs` — random eviction.
- `mining/src/model/topological_sort.rs` — `debug_assert_eq!`.

### Database / Daemon / CLI
- `database/src/db.rs` — `delete_db` returns `Result`.
- `database/src/key.rs` — bounds-checked `DbKey` Display.
- `daemon/src/lib.rs` — `Option`-returning daemon accessors.
- `zyanyad/src/args.rs` — `network()` returns `Result` + `--uacomment` validation.
- `zyanyad/src/daemon.rs` — `network()` caller updated.
- `cli/src/modules/{node,miner}.rs` — daemon accessor error handling.
- `node-cli/src/modules/{node,miner}.rs` — daemon accessor error handling.
- `testing/integration/src/common/daemon.rs` — `network()` caller updated.
- `rothschild/src/main.rs` — private key redaction.

### Math / Utils / WASM
- `math/src/uint.rs` — LUT index `debug_assert!` bounds.
- `utils/src/hex.rs` — safe UTF-8 in deserialize.
- `utils/src/networking.rs` — `expect` with message.
- `wasm/core/src/events.rs` — `SAFETY` comment.
- `wasm/core/src/types.rs` — safe `from_utf8`.

### Plan document
- `specs/01b87849_low-findings-batch-l1.md` — full remediation plan (467 lines).

---

## 3. Signature changes (breaking for in-crate callers)

These five findings changed function signatures; all in-tree callers were updated in the same batch:

| Finding | Signature | Callers updated |
|---------|-----------|-----------------|
| F-L-10 | `ChildNumber::from_bytes` → `Result<Self>` | `xkey.rs` |
| F-L-31 | `UnboundedSender` → `Sender` (bounded) | `flow_context.rs`, `manager_tests.rs` |
| F-L-36 | `delete_db` → `Result<(), Box<dyn Error>>` | grep callers |
| F-L-37 | `Args::network()` → `Result<NetworkId, String>` | `daemon.rs`, `testing/integration/.../daemon.rs` |
| F-L-38 | `Daemons::zyanyad()` / `cpu_miner()` → `Option` | `cli/src/modules/*`, `node-cli/src/modules/*` |

---

## 4. How to verify

```bash
# Full workspace check — must pass with zero errors.
cargo check --workspace --all-targets
```

Targeted spot-checks per tier:

```bash
# P1 — consensus / VM
cargo check -p zyanya-consensus-core -p zyanya-consensus -p zyanya-vm

# P2 — wallet / BIP32
cargo check -p zyanya-wallet-bip32 -p zyanya-wallet-core

# P3 — P2P / protocol
cargo check -p zyanya-p2p-core -p zyanya-protocol-flows

# P4 — RPC / explorer
cargo check -p zyanya-rpc-service -p zyanya-explorer

# P5 — DB / daemon / CLI
cargo check -p zyanya-database -p zyanya-daemon -p zyanyad

# P6 — mining
cargo check -p zyanya-mining

# P7 — math / utils / WASM
cargo check -p zyanya-math -p zyanya-utils -p zyanya-wasm-core
```

---

## 5. Notes for downstream agents

- **F-L-04** was intentionally skipped (verified safe in the audit report).
- **F-L-03**, **F-L-17 (gen1.rs)**, and **F-L-42** were already addressed by earlier batches; this batch adds documentation comments or verifies only — no behavioral change.
- The remediation plan lives at `specs/01b87849_low-findings-batch-l1.md` and contains the full per-finding rationale and execution checklist.
- Related identical `u8::try_from(...).unwrap()` patterns exist in `wallet/core/src/account/variants/bip32watch.rs` and `variants/watchonly.rs` — out of scope for this batch but worth a future pass.
- `Prefix::from_version` was changed from `fn` to `pub(crate) fn` to support the F-L-11 cross-validation without making it fully public.
- `set_default_storage_folder` / `set_default_wallet_file` are no longer `unsafe` — any external FFI callers (e.g. `wasm-bindgen` exports) calling them with `unsafe { }` wrappers should drop the `unsafe` block.