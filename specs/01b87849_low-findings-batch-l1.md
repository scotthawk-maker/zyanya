# Remediation Plan — LOW findings Batch L1 (F-L-01 … F-L-47)

**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md` §6 (LOW findings, lines ~1416–1692).
**Scope:** Apply minimal fixes to all LOW findings F-L-01 through F-L-47, **skipping F-L-04** (verified safe).
**Verification gate:** `cargo check --workspace --all-targets` must pass after all edits.

---

## 0. How to read this plan

Each finding below lists:
- **File** (exact path) and **line(s)** (from the audit report; line numbers may drift slightly after earlier edits — locate by the quoted code).
- **Current behavior** (verified against the working tree).
- **Minimal fix** (the smallest change that removes the hazard without changing public behavior more than necessary).

Findings already partially or fully addressed by earlier batches are marked **ALREADY DONE — VERIFY ONLY**; do not re-edit unless the quoted code is still present.

---

## 1. Priority order (all LOW; ordered by blast radius)

| Tier | Findings | Rationale |
|------|----------|-----------|
| P1 — consensus/VM correctness | F-L-01, F-L-02, F-L-05, F-L-06, F-L-07 | Touches consensus-critical code paths (tx validation, coinbase, subnet classification, assembler). |
| P2 — key/secret handling | F-L-08, F-L-09, F-L-10, F-L-11, F-L-12, F-L-13, F-L-14, F-L-15, F-L-16, F-L-17, F-L-18, F-L-19, F-L-20, F-L-46, F-L-47 | Private keys, derivation, encryption, wallet invariants. |
| P3 — network/P2P DoS | F-L-21, F-L-22, F-L-23, F-L-24, F-L-25, F-L-26, F-L-31, F-L-32 | Panics/overflow in hot P2P paths; unbounded channels; CLI input. |
| P4 — RPC/explorer | F-L-27, F-L-29, F-L-30, F-L-33 | RPC div-by-zero; explorer rate limiting, atomic writes, security headers. |
| P5 — DB/daemon/CLI | F-L-35, F-L-36, F-L-37, F-L-38 | Panics on DB/daemon/flag misuse. |
| P6 — mining/mempool | F-L-39, F-L-40, F-L-41 | Panics/eviction determinism in mining. |
| P7 — math/utils/wasm | F-L-28, F-L-34, F-L-42, F-L-43, F-L-44, F-L-45 | Unsafe UTF-8, LUT bounds, unwrap on parse. |

---

## 2. P1 — Consensus / VM correctness

### F-L-01 — `check_transaction_outputs_count` reports input fields in error
- **File:** `consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs` (~line 59–64)
- **Current:** `Err(TxRuleError::TooManyOutputs(tx.inputs.len(), self.max_tx_inputs))`
- **Fix:** `Err(TxRuleError::TooManyOutputs(tx.outputs.len(), self.max_tx_outputs))`

### F-L-02 — Assembler label parsing + numeric JUMP targets
- **File:** `zyanya-vm/src/assembler.rs` (pass 1 ~lines 40–70; `resolve_target` ~lines 232–238)
- **Current:**
  - Pass 1 treats any line starting **or ending** with `:` as a label; `:loop PUSH 1` becomes label `loop` and the opcode is swallowed.
  - `resolve_target` falls back to `parse_u64(arg)` and returns the raw number as a byte offset, inconsistent with the VM's opcode-index interpretation.
- **Fix (minimal):**
  1. In pass 1, a label definition is only valid when the **entire trimmed line** is a label (starts with `:` and contains no other tokens, or is exactly `name:` with no trailing tokens). If a line starts with `:` but has additional tokens, return a new/appropriate `AssemblerError` (e.g. `AssemblerError::InvalidLabel(line_num)` or reuse `UnknownOpcode`). Keep support for the existing `:loop` (line is only the label) and `loop:` forms.
  2. In `resolve_target`, when the argument parses as a numeric literal, validate it is a valid opcode index (i.e. `< opcodes.len()` at the point of resolution) — or, minimally, document/convert numeric targets to opcode indices consistently. Prefer: reject numeric targets that are out of range with `AssemblerError::UndefinedLabel`/a new `InvalidJumpTarget` variant. (Check the VM's `Jump`/`JumpIf` semantics in `zyanya-vm/src/opcode.rs` to confirm whether the operand is an opcode index; align the assembler to that.)

### F-L-03 — Cross-call stack depth unbounded
- **File:** `zyanya-vm/src/stack.rs` (~lines 5, 37–42, 48–50)
- **Status:** **ALREADY ADDRESSED by F-C-05 (call-depth limit).** No code change to `stack.rs`.
- **Action:** Add a one-line comment in `stack.rs` near the stack-size constant noting: *"Per-call operand stack is bounded to 1024; total cross-call depth is bounded by the call-depth limit enforced in the VM (F-C-05)."* Do not change behavior.

### F-L-05 — `is_builtin` does not include `SUBNETWORK_ID_SMART_CONTRACT`
- **File:** `consensus/core/src/subnets.rs` (~lines 58–62)
- **Current:** `*self == SUBNETWORK_ID_COINBASE || *self == SUBNETWORK_ID_REGISTRY`
- **Fix (per task instruction):** add the smart-contract subnet:
  `*self == SUBNETWORK_ID_COINBASE || *self == SUBNETWORK_ID_REGISTRY || *self == SUBNETWORK_ID_SMART_CONTRACT`
- **Note:** the audit report marked this "no change (by design)"; the task explicitly overrides it. Keep the doc comment accurate (smart-contract txs are now validated by all nodes).

### F-L-06 — Coinbase vesting `red_reward` accumulation unchecked
- **File:** `consensus/src/processes/coinbase.rs` (~line 162)
- **Current:** `red_reward += reward_data.subsidy + reward_data.total_fees;`
- **Fix:** `red_reward = red_reward.saturating_add(reward_data.subsidy).saturating_add(reward_data.total_fees);`
  (Task says `saturating_add`; do not change the function's return type.)

### F-L-07 — `Vec::with_capacity((mergeset_blues.len() + 1) * 13)` capacity hint
- **File:** `consensus/src/processes/coinbase.rs` (~line 125)
- **Current:** `Vec::with_capacity((ghostdag_data.mergeset_blues.len() + 1) * 13)`
- **Fix:** use checked arithmetic for the hint:
  `let cap = ghostdag_data.mergeset_blues.len().saturating_add(1).saturating_mul(13);`
  `let mut outputs = Vec::with_capacity(cap);`
  (Capacity is a hint only; saturating is sufficient and cannot panic.)

---

## 3. P2 — Key / secret handling

### F-L-08 — `Fees::try_from(&str)` maps invalid string to 0
- **File:** `wallet/core/src/tx/fees.rs` (~line 85)
- **Current:** `let fee = crate::utils::try_zyanya_str_to_sompi_i64(fee)?.unwrap_or(0);`
- **Fix:** propagate `None` as an error for a non-empty string:
  ```rust
  let fee = crate::utils::try_zyanya_str_to_sompi_i64(fee)?;
  match fee {
      Some(v) => Ok(Fees::from(v)),
      None => Err(crate::error::Error::String(format!("invalid fee string: {fee}"))),
  }
  ```
  (Pick the most appropriate existing `Error` variant if `Error::String` is not the right one — check `wallet/core/src/error.rs`.)

### F-L-09 — `DerivationPath::from_str` no length limit
- **File:** `wallet/bip32/src/derivation_path.rs` (~line 117)
- **Current:** `Ok(DerivationPath { path: path.map(str::parse).collect::<Result<_>>()? })`
- **Fix:** collect into a `Vec`, then enforce a cap (≤ 255 components):
  ```rust
  let path: Vec<ChildNumber> = path.map(str::parse).collect::<Result<_>>()?;
  if path.len() > 255 {
      return Err(Error::String("derivation path exceeds 255 components".into()));
  }
  Ok(DerivationPath { path })
  ```

### F-L-10 — `ChildNumber::from_bytes` skips validation
- **File:** `wallet/bip32/src/child_number.rs` (~lines 35–37, 68–71); caller `wallet/bip32/src/xkey.rs` (~line 86)
- **Current:** `from_bytes` returns `u32::from_be_bytes(bytes).into()` with no check; `From<u32>` is infallible.
- **Fix (minimal):** make `from_bytes` validate and return `Result<Self>`:
  ```rust
  pub fn from_bytes(bytes: [u8; Self::BYTE_SIZE]) -> Result<Self> {
      let n = u32::from_be_bytes(bytes);
      if n & !Self::HARDENED_FLAG >= Self::HARDENED_FLAG {
          return Err(Error::ChildNumber);
      }
      Ok(ChildNumber(n))
  }
  ```
  Update the single caller in `xkey.rs` to `ChildNumber::from_bytes(bytes[9..13].try_into()?)?`.
  Leave `From<u32>` as-is (infallible trait impl); add a doc comment noting the invariant that the non-hardened index must be `< HARDENED_FLAG`.

### F-L-11 — `ExtendedKey::from_str` no cross-validation of prefix vs version
- **File:** `wallet/bip32/src/xkey.rs` (~line 74)
- **Current:** builds `Prefix::from_parts_unchecked(chars, version)` after only validating `chars` is alphabetic.
- **Fix:** after decoding `version`, derive the expected prefix and compare:
  ```rust
  let expected = Prefix::from_version(version)?;   // from_version already validates
  if expected.as_str() != chars {
      return Err(Error::DecodeIssue);   // or a more specific error variant
  }
  let prefix = expected;
  ```
  (This removes the need for `from_parts_unchecked` in this path. `Prefix::from_version` is currently `fn` (private) — it is callable within the crate; if it is not visible from `xkey.rs`, make it `pub(crate)`.)

### F-L-12 — `static mut` storage globals
- **File:** `wallet/core/src/storage/local/mod.rs` (~lines 32–34)
- **Current:** three `static mut Option<String>` with `unsafe` accessors.
- **Fix:** replace with `std::sync::OnceLock<String>`:
  ```rust
  static DEFAULT_STORAGE_FOLDER: OnceLock<String> = OnceLock::new();
  // ... etc
  pub fn default_storage_folder() -> &'static str {
      DEFAULT_STORAGE_FOLDER.get_or_init(|| "~/.zyanya".to_string()).as_str()
  }
  ```
  Remove the `unsafe` blocks and `#[allow(static_mut_refs)]`. (If the values are ever mutated elsewhere, use `Mutex<String>` instead — but current code only initializes once.)

### F-L-13 — `utxo/context.rs` `panic!`/`unreachable!`
- **File:** `wallet/core/src/utxo/context.rs` (~lines 382, 403)
- **Current:** `unreachable!("Error: promotion of the outgoing transaction!")` in `promote`; `panic!("Error: non-stasis utxo revival!")` in `revive`.
- **Fix:** replace with `log_error!` + `continue` (do not crash the wallet):
  - `promote`: replace `unreachable!(...)` with `log_error!("Error: promotion of the outgoing transaction!"); continue;`
  - `revive`: replace `panic!(...)` with `log_error!("Error: non-stasis utxo revival!"); continue;`
  (Both functions already return `Result<()>`; logging-and-continuing is the minimal non-crashing behavior. If a hard invariant is truly required, return `Err(...)` instead — but `continue` is the least disruptive.)

### F-L-14 — `create_xpub_from_xprv` / `build_derivate_path` panic on unsupported account kind
- **File:** `wallet/core/src/derivation.rs` (~lines 540, 561)
- **Current:** `_ => panic!("create_xpub_from_xprv not supported for account kind: {:?}", account_kind)` and `_ => panic!("build derivate path not supported for account kind: {:?}", account_kind)`
- **Fix:** return `Err(Error::AccountKindFeature)` in both `_` arms (variant already exists and is used in `multisig.rs`).

### F-L-15 — `calculate_balance` uses plain `sum()`
- **File:** `wallet/core/src/utxo/context.rs` (~lines 480–481)
- **Current:** `context.mature.iter().map(|e| e.as_ref().amount).sum()` and `context.pending.values().map(|e| e.as_ref().amount).sum()`
- **Fix:** checked/saturating fold (function returns `Balance`, not `Result` — keep the signature):
  ```rust
  let mature: u64 = context.mature.iter().fold(0u64, |acc, e| acc.saturating_add(e.as_ref().amount));
  let pending: u64 = context.pending.values().fold(0u64, |acc, e| acc.saturating_add(e.as_ref().amount));
  ```
  (Task says "checked_add fold"; `saturating_add` inside `fold` is the minimal non-panicking equivalent that preserves the return type. If you prefer strictness, use `try_fold(0u64, |acc, e| acc.checked_add(e.as_ref().amount)).unwrap_or(u64::MAX)`.)

### F-L-16 — `sig_op_count` panics on >255 xpub keys
- **File:** `wallet/core/src/account/variants/multisig.rs` (~lines 53–61 `validate`, 209–211 `sig_op_count`)
- **Current:** `u8::try_from(self.xpub_keys.len()).unwrap()`
- **Fix (minimal):**
  1. In `Payload::validate()` add: `if self.xpub_keys.len() > u8::MAX as usize { return Err(Error::InvalidArgument("xpub_keys exceeds 255".to_string())); }`
  2. In `sig_op_count`, keep the `u8` return but make it non-panicking: `u8::try_from(self.xpub_keys.len()).unwrap_or(u8::MAX)`.
  (Related identical patterns exist in `variants/bip32watch.rs:165` and `variants/watchonly.rs:229` — out of scope for this batch, but note them in the commit message if you touch them.)

### F-L-17 — `unsafe from_utf8_unchecked` in gen1 + test code
- **File:** `wallet/core/src/compat/gen1.rs` (~line 18) and `wallet/core/src/account/mod.rs` (~line 856)
- **Status:** `gen1.rs` **ALREADY FIXED** (uses `String::from_utf8(decrypted)?`). **VERIFY ONLY.**
- **Remaining fix:** `account/mod.rs` test helper `bytes_str` still has `unsafe { std::str::from_utf8_unchecked(&hex) }.to_string()`. Replace with `std::str::from_utf8(&hex).expect("hex output is valid UTF-8").to_string()`.

### F-L-18 — `Encrypted` `Debug` prints ciphertext hex
- **File:** `wallet/core/src/encryption.rs` (~line 197)
- **Current:** `.field("payload", &self.payload.to_hex())`
- **Fix:** `.field("payload", &"[ENCRYPTED]")` (or a length-only field, e.g. `&self.payload.len()`).

### F-L-19 — `From<NetworkId> for Prefix` maps all non-mainnet to `KTUB`
- **File:** `wallet/bip32/src/prefix.rs` (~line 215)
- **Current:** `Mainnet => KPUB`, `Devnet/Simnet/Testnet => KTUB`.
- **Fix (per task "add testnet mapping"):** map testnet to the distinct testnet prefix:
  ```rust
  NetworkType::Mainnet => Prefix::KPUB,
  NetworkType::Testnet => Prefix::TPUB,
  NetworkType::Devnet  => Prefix::KTUB,
  NetworkType::Simnet  => Prefix::KTUB,
  ```
  (This makes testnet distinguishable from devnet/simnet. If the intent was instead to give devnet/simnet their own prefixes, confirm before deviating — the minimal, unambiguous change is the `Testnet => TPUB` mapping.)

### F-L-20 — `DerivationPath` deserialize `expecting` copy-paste error
- **File:** `wallet/bip32/src/derivation_path.rs` (~line 35)
- **Current:** `"a string containing list of permissions separated by a '+'"`
- **Fix:** `"a BIP32 derivation path string (e.g. m/44'/123456'/0')"`

### F-L-46 — Rothschild logs private key
- **File:** `rothschild/src/main.rs` (~lines 195–199 and the `log_message` block ~lines 200–215)
- **Current:** `info!("Generated private key {} ...", sk.display_secret(), ...)` and `log_message` includes `schnorr_key.display_secret()`.
- **Fix:**
  1. Generated-key case: print the key to **stdout** via `println!` (not the logging framework) so it is not persisted to log files, and keep the address in the `info!` log.
  2. `log_message`: replace `schnorr_key.display_secret()` with `"[REDACTED]"` (keep the from-address).
  - Never pass `display_secret()` to `info!`/`warn!`/`debug!`.

### F-L-47 — CLI wizard secrets trimmed with `as_bytes().to_vec()`
- **File:** `cli/src/wizards/wallet.rs` (~lines 68, 108, 112)
- **Current:** `Secret::new(term.ask(...).await?.trim().as_bytes().to_vec())` — the intermediate `String` is not zeroized.
- **Fix:** bind the intermediate `String`, extract trimmed bytes, then zeroize the `String` (the `zeroize` crate provides `impl Zeroize for String`):
  ```rust
  let mut raw = term.ask(true, "Enter wallet encryption password: ").await?;
  let secret_bytes = raw.trim().as_bytes().to_vec();
  raw.zeroize();
  let wallet_secret = Secret::new(secret_bytes);
  ```
  Apply the same pattern to the validation prompt and the bip39 passphrase prompt. (Ensure `zeroize::Zeroize` is in scope; `cli` already depends on `zeroize` via the workspace — verify.)

---

## 4. P3 — Network / P2P DoS

### F-L-21 — TPS throttle math overflow
- **File:** `protocol/flows/src/v5/txrelay/flow.rs` (~lines 175 and 384)
- **Current:** `1000 * snapshot_delta.low_priority_tx_counts`
- **Fix:** `1000u64.saturating_mul(snapshot_delta.low_priority_tx_counts)` in **both** locations.

### F-L-22 — `Hub::broadcast_to_some_peers` asserts `num_peers > 0`
- **File:** `protocol/p2p/src/core/hub.rs` (~line 136)
- **Current:** `assert!(num_peers > 0);`
- **Fix:** `if num_peers == 0 { return; }` (early return).

### F-L-23 — `Router::enqueue` uses `assert!`
- **File:** `protocol/p2p/src/core/router.rs` (~lines 405–406)
- **Current:** `assert!(msg.payload.is_some(), "Zyanyad P2P message should always have a value");`
- **Fix:** return an error instead:
  ```rust
  if msg.payload.is_none() {
      return Err(ProtocolError::Other("Zyanyad P2P message should always have a value"));
  }
  ```

### F-L-24 — `connection_failed_count + 1` overflow
- **File:** `components/addressmanager/src/lib.rs` (~line 301)
- **Current:** `let new_count = self.address_store.get(address).connection_failed_count + 1;`
- **Fix:** `let new_count = self.address_store.get(address).connection_failed_count.saturating_add(1);`

### F-L-25 — `hub_sender.send().expect()` in P2P paths
- **File:** `protocol/p2p/src/core/connection_handler.rs` (~lines 150, 275) and `protocol/p2p/src/core/router.rs` (~line 451)
- **Current:** `.send(HubEvent::...).await.expect("hub receiver should never drop before senders")`
- **Fix:** replace `.expect(...)` with a `match`/`if let Err` that logs a warning and continues (or closes the connection):
  ```rust
  if self.hub_sender.send(HubEvent::NewPeer(router.clone())).await.is_err() {
      warn!("hub receiver dropped; peer notification skipped");
  }
  ```
  For `router.rs` `close()`, log and continue (the router is closing anyway).

### F-L-26 — P2P / gRPC / wRPC server panic on serve error
- **File:** `protocol/p2p/src/core/connection_handler.rs` (~line 79), `rpc/grpc/server/src/connection_handler.rs` (~line 160), `rpc/wrpc/server/src/service.rs` (~line 155)
- **Current:** `Err(err) => panic!("... stopped with error: {err:?}")` (and wRPC bind error panic).
- **Fix (minimal):** replace `panic!` with `error!`/`warn!` logging and return/continue so the task ends without aborting the process:
  ```rust
  Err(err) => error!("P2P Server {serve_address} stopped with error: {err:?}"),
  ```
  Do the same for the gRPC and wRPC serve/bind error arms. (Full graceful-shutdown signaling is a larger change; logging-and-exit is the minimal fix that removes the panic.)

### F-L-31 — `UnboundedSender` for tx IDs in mining manager
- **File:** `mining/src/manager.rs` (~lines 39, 632, 932, 784); callers `protocol/flows/src/flow_context.rs` (~line 579) and `mining/src/manager_tests.rs` (~lines 1030, 1054)
- **Current:** `tokio::sync::mpsc::UnboundedSender<Vec<TransactionId>>`; channel created with `unbounded_channel()`.
- **Fix:** switch to a bounded channel:
  1. `manager.rs`: change the import and both `revalidate_high_priority_transactions` signatures to `mpsc::Sender<Vec<TransactionId>>`; at the send site (~line 784) use `transaction_ids_sender.blocking_send(valid_ids)` (sync context inside `spawn_blocking`) and log on `Err` (receiver dropped/full).
  2. `flow_context.rs`: `let (tx, mut rx) = mpsc::channel(1024);` (pick a named const, e.g. `REVALIDATION_CHANNEL_CAPACITY`).
  3. `manager_tests.rs`: replace `unbounded_channel()` with `mpsc::channel(1024)` (or the same const); `blocking_recv`/`try_recv` still work on a bounded receiver.

### F-L-32 — No input validation on `--uacomment`
- **File:** `zyanyad/src/args.rs` (~lines 345–350)
- **Current:** `Arg::new("user_agent_comments").long("uacomment").action(ArgAction::Append).require_equals(true)` with no validation.
- **Fix:** add a `value_parser` that rejects values > 256 chars (and, minimally, non-printable/`/` characters):
  ```rust
  .value_parser(clap::builder::ValueParser::new(|s: &str| -> Result<String, String> {
      if s.len() > 256 { return Err("uacomment must be ≤ 256 characters".into()); }
      if !s.chars().all(|c| c.is_ascii_graphic() || c == ' ') { return Err("uacomment must be printable ASCII".into()); }
      if s.contains('/') { return Err("uacomment must not contain '/'".into()); }
      Ok(s.to_string())
  }))
  ```
  (Task minimum is the 256-char limit; the printable-ASCII and `/` checks are cheap and match the audit report.)

---

## 5. P4 — RPC / Explorer

### F-L-27 — `DaaScoreTimestampEstimate` division by zero
- **File:** `rpc/service/src/service.rs` (~lines 967–972)
- **Current:** `((time_between_headers as f64) * (score_between_query_and_header / score_between_headers)) as u64` where `score_between_headers` can be `0.0`.
- **Fix:** guard before the division:
  ```rust
  let time_adjustment = if score_between_headers == 0.0 {
      time_between_headers   // or 0; use the header timestamp delta directly
  } else {
      ((time_between_headers as f64) * (score_between_query_and_header / score_between_headers)) as u64
  };
  ```

### F-L-29 — No rate limiting on explorer API
- **File:** `zyanya-explorer/src/main.rs` (~lines 67–101, router construction ~lines 145–190)
- **Current:** no rate limiting middleware.
- **Fix (minimal, no new dependency):** add a per-IP fixed-window limiter (100 req/sec) as an axum middleware:
  - State: `Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>` (or `tokio::sync::Mutex`).
  - On each request, extract the peer IP via `axum::extract::ConnectInfo<SocketAddr>`; reset the window if > 1s elapsed; if count > 100 return `StatusCode::TOO_MANY_REQUESTS`.
  - Wire it with `.layer(middleware::from_fn_with_state(limiter_state, rate_limit))` and serve with `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())`.
  - Apply to `/api/*` routes (or globally — simpler and acceptable for this batch).

### F-L-30 — Non-atomic metadata file writes
- **File:** `zyanya-explorer/src/client.rs` (~lines 406, 435)
- **Current:** `std::fs::write(&self.metadata_path, &json)` (and icon write at ~435).
- **Fix:** add a helper and use it for the metadata JSON (and icon if convenient):
  ```rust
  fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
      let tmp = path.with_extension("tmp");
      std::fs::write(&tmp, data)?;
      std::fs::rename(&tmp, path)
  }
  ```
  Keep the existing `/tmp` fallback behavior on error.

### F-L-33 — Explorer missing security headers
- **File:** `zyanya-explorer/src/main.rs` (~lines 67–101, router construction)
- **Current:** only the `cors_same_origin` middleware.
- **Fix:** add a `security_headers` middleware (or extend the existing one) that inserts:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  Layer it on the router: `.layer(middleware::from_fn(security_headers))`.
  (Task minimum is these two headers; optionally add `Content-Security-Policy: default-src 'self'` and `Strict-Transport-Security` per the audit report.)

---

## 6. P5 — DB / Daemon / CLI

### F-L-35 — `DbKey` Display indexes without bounds check
- **File:** `database/src/key.rs` (~lines 97–119)
- **Current:** `self.path[0]`, `self.path[1]` indexed based on `prefix_len` checks.
- **Fix:** use `self.path.get(0)` / `self.path.get(1)` and only write when `Some(...)`; skip the segment when `None`.

### F-L-36 — `delete_db` panics on RocksDB destroy failure
- **File:** `database/src/db.rs` (~lines 42–43)
- **Current:** `pub fn delete_db(db_dir: PathBuf)` with `.expect("DB is expected to be deletable")` and `db_dir.to_str().unwrap()`.
- **Fix:** change signature to `pub fn delete_db(db_dir: PathBuf) -> Result<(), rocksdb::Error>` (or a boxed error) and propagate:
  ```rust
  let path = db_dir.to_str().ok_or_else(|| /* error */)?;
  <DBWithThreadMode<MultiThreaded>>::destroy(&options, path)?;
  Ok(())
  ```
  Update callers to handle the `Result` (log-and-continue is acceptable).

### F-L-37 — `network()` panics on conflicting flags
- **File:** `zyanyad/src/args.rs` (~lines 170–180)
- **Current:** `_ => panic!("only a single net should be activated")`
- **Fix:** change signature to `pub fn network(&self) -> Result<NetworkId, String>` and return `Err("only a single net should be activated".into())` for the conflicting case. Update callers to print the error and exit(1) (or propagate).

### F-L-38 — Daemon `.expect()` calls
- **File:** `daemon/src/lib.rs` (~lines 99, 105)
- **Current:** `zyanyad()` and `cpu_miner()` use `.expect(...)` when the daemon is `None`.
- **Fix:** return `Option<Arc<...>>` (or `Result`) instead of panicking. Callers already exist for the `try_*` variants; update the 4 call sites (`node-cli/src/modules/node.rs:99`, `node-cli/src/modules/miner.rs:102`, `cli/src/modules/node.rs:99`, `cli/src/modules/miner.rs:102`) to handle `None` (print a helpful "daemon not configured" message). Alternatively, keep the `zyanyad()`/`cpu_miner()` names but return `Result` and update callers with `?`/match.

---

## 7. P6 — Mining / Mempool

### F-L-39 — Topological sort asserts no cycles
- **File:** `mining/src/model/topological_sort.rs` (~line 52)
- **Current:** `assert_eq!(sorted.len(), self.len(), "by definition, cryptographically no cycle can exist in a DAG of transactions");`
- **Note:** the `TopologicalSort` trait (`topological_sort(self) -> Self`) is **dead code** — production uses `TopologicalIter`/`TopologicalIntoIter` (imported in `manager.rs:17`). The assert is in the dead method.
- **Fix (minimal):** change `assert_eq!` to `debug_assert_eq!` (removes the release-build panic without changing the `-> Self` signature). Optionally delete the dead trait, but that is a larger cleanup — not required.

### F-L-40 — Orphan pool eviction not truly random
- **File:** `mining/src/mempool/model/orphan_pool.rs` (~lines 259–261)
- **Current:** `self.all_orphans.values().find(|x| x.priority == Priority::Low)` (always the first).
- **Fix:** use `rand` (already a workspace dep of `mining`):
  ```rust
  use rand::seq::IteratorRandom;
  fn get_random_low_priority_orphan(&self) -> Option<&MempoolTransaction> {
      self.all_orphans.values().filter(|x| x.priority == Priority::Low).choose(&mut rand::thread_rng())
  }
  ```

### F-L-41 — `check_transaction_standard_in_context` uses `assert!`
- **File:** `mining/src/mempool/check_transaction_standard.rs` (~line 175)
- **Current:** `assert!(contextual_mass > 0, "expected to be set by consensus");`
- **Fix:** `debug_assert!(contextual_mass > 0, "expected to be set by consensus");` (task explicitly says `debug_assert!`).

---

## 8. P7 — Math / Utils / WASM

### F-L-28 — Unsafe `impl Send` for WASM `Sink`
- **File:** `wasm/core/src/events.rs` (~line 38)
- **Current:** `unsafe impl Send for Sink {}` with no safety comment.
- **Fix (minimal):** add a `// SAFETY:` comment documenting that `Sink` is only used in single-threaded WASM contexts where `js_sys::Function`/`Object` are not shared across threads. Optionally gate with `#[cfg(target_arch = "wasm32")]` if non-WASM builds do not need it (verify before gating — it may be required by non-WASM test builds).

### F-L-34 — Unsafe `from_utf8_unchecked` in WASM hex
- **File:** `wasm/core/src/types.rs` (~line 52)
- **Current:** `let result = unsafe { str::from_utf8_unchecked(&hex) };`
- **Fix:** `let result = str::from_utf8(&hex).expect("hex output is always valid UTF-8");`

### F-L-42 — Unsafe `from_utf8_unchecked` in math uint Display/LowerHex
- **File:** `math/src/uint.rs` (~lines 757, 816)
- **Status:** **ALREADY FIXED** — both `LowerHex` and `Display` now use safe `str::from_utf8(...).expect(...)`. **VERIFY ONLY** (grep for `from_utf8_unchecked` in `math/src/uint.rs` returns nothing).

### F-L-43 — Display LUT indexing bounds
- **File:** `math/src/uint.rs` (~lines 783–804)
- **Current:** `DEC_DIGITS_LUT[d1]`, `[d1+1]`, `[d2]`, `[d2+1]` with implicit `rem < 10_000` guarantee.
- **Fix:** add `debug_assert!(d1 < 200 && d2 < 200);` before the LUT reads (and the same for the `n < 100` branch indices) to catch any future `div_rem_u64` regression.

### F-L-44 — `hex.rs` deserialize `unwrap()` on UTF-8
- **File:** `utils/src/hex.rs` (~line 31)
- **Current:** `T::from_hex(str::from_utf8(buff).unwrap()).map_err(D::Error::custom)`
- **Fix:** `T::from_hex(str::from_utf8(buff).map_err(D::Error::custom)?).map_err(D::Error::custom)`

### F-L-45 — `networking.rs` `unwrap()` on IP parsing
- **File:** `utils/src/networking.rs` (~line 121)
- **Current:** `if IpNet::from_str(curr_net).unwrap().contains(&self.0) {`
- **Fix:** parse the hardcoded list once and fail loudly with a clear message (still a programming error, but no silent `unwrap` in a hot path):
  ```rust
  use std::sync::LazyLock;
  static UNROUTABLE_NETS: LazyLock<Vec<IpNet>> = LazyLock::new(|| {
      ["10.0.0.0/8", /* ...full list... */]
          .iter().map(|s| IpNet::from_str(s).expect("unroutable_nets contains invalid CIDR")).collect()
  });
  // then: UNROUTABLE_NETS.iter().any(|net| net.contains(&self.0))
  ```
  (Minimal alternative: keep the loop but replace `.unwrap()` with `.expect("unroutable_nets contains invalid CIDR")`.)

---

## 9. Skipped / already-addressed summary

| ID | Status |
|----|--------|
| F-L-03 | Add note only (addressed by F-C-05). |
| F-L-04 | **SKIP** (verified safe). |
| F-L-17 (gen1.rs) | Already fixed — verify only; fix remaining test helper in `account/mod.rs`. |
| F-L-42 | Already fixed — verify only. |

---

## 10. Execution order & verification

1. Work through tiers P1 → P7 in order. Each finding is independent; commit per tier (or per file) for clean review.
2. After each tier, run `cargo check -p <affected-crate>` to catch signature ripples early (especially F-L-10, F-L-31, F-L-36, F-L-37, F-L-38 which change signatures).
3. Final gate: `cargo check --workspace --all-targets` must pass.
4. Do **not** run `cargo fmt`/`clippy` as part of this batch unless a check fails; keep diffs minimal.

## 11. Signature-change ripple checklist (do these together)

- **F-L-10** `ChildNumber::from_bytes -> Result`: update `wallet/bip32/src/xkey.rs:86`.
- **F-L-31** bounded channel: update `mining/src/manager.rs` (import + 2 signatures + send site), `protocol/flows/src/flow_context.rs:579`, `mining/src/manager_tests.rs:1030,1054`.
- **F-L-36** `delete_db -> Result`: find and update all callers (grep `delete_db(`).
- **F-L-37** `network() -> Result`: update all callers (grep `.network()` in `zyanyad/`).
- **F-L-38** daemon accessors: update `node-cli/src/modules/{node,miner}.rs` and `cli/src/modules/{node,miner}.rs`.
