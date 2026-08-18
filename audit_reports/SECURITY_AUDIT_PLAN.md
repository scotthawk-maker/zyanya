# Zyanya Blockchain — Security Audit Plan (Builder Agent)

**Target**: Zyanya (`rusty-spectre` / Kaspa fork) — Rust workspace, 1041 `.rs` files.
**Date**: 2026-08-18
**Working dir**: `/root/zyanya-audit`

This plan is grounded in a first-pass source review. It lists the files to audit,
the vulnerability categories to check per file, and a priority order (CRITICAL first).
Prior findings already exist in `audit_reports/` (phase1–5 + FINAL_AUDIT_REPORT.md);
this plan is a fresh, independent pass and should be cross-referenced against those.

---

## 0. Priority Order (summary)

| Tier | Severity | Modules |
|------|----------|---------|
| T0 | CRITICAL | `zyanya-vm` (VM/opcode), `consensus` contract store + virtual processor, `math` |
| T1 | HIGH | `crypto/txscript`, `wallet/keys`, `wallet/bip32`, `wallet/psst`, `rpc/*`, `protocol/*` |
| T2 | MEDIUM | `mining/mempool`, `database`, `crypto/hashes`, `zyanya-vm/compiler`, `consensus/core` tx/hashing |
| T3 | LOW/INFO | `zyanya-explorer`, `zyanya-query`, `utils`, `metrics`, `notify` |

---

## 1. Files to Audit (with paths) + Vulnerability Categories

### T0 — CRITICAL

#### 1.1 `zyanya-vm/src/vm.rs` — VM execution engine
Categories: integer overflow/underflow, reentrancy, gas accounting, control-flow integrity.
- `OpCode::Pow` (line ~153): `a.wrapping_pow(b as u32)` — **silent wrap** on overflow and
  **exponent truncation** (`b as u32`). Gas charged on full `b` (`1 + b/32`) but exponent
  truncated to 32 bits → gas/cost mismatch. Verify determinism and DoS surface.
- `OpCode::Call` (line ~250): reentrancy guard is `call_stack.contains(target_addr)` — a
  *static* same-contract guard, not a full reentrancy guard. Verify it cannot be bypassed
  via a diamond call path (A→B→C→A is blocked, but A→B→C→B? verify). Check `call_stack`
  clone/unwind correctness on error paths (is `pop()` always reached?).
- `OpCode::Call` gas forwarding: `forward_gas` popped then `consume(forward_gas)`; child VM
  gets `VM::new(forward_gas)`. Verify refund math (`saturating_sub`) cannot over-refund.
- `OpCode::Return`: `self.stack.pop().ok()` swallows underflow — verify intended.
- `OpCode::Jump`/`JumpIf`: target validated against `code.len()` (opcode count), but
  deserialization remaps byte-offsets→opcode-indices only for valid opcode starts (see 1.2).
- `caller` is a `u64` (truncated address) — see 1.4. Weak `msg.sender` identity.

#### 1.2 `zyanya-vm/src/opcode.rs` — opcode (de)serialization
Categories: input validation, deserialization safety, control-flow integrity.
- `deserialize_slice`: jump targets are byte offsets remapped to opcode indices via
  `byte_to_opcode` HashMap. Targets that do NOT land on an opcode start byte are left as raw
  byte offsets, then compared against `code.len()` (opcode count) at runtime → **jump target
  validation gap** (arbitrary jump into middle of an immediate). Verify and harden.
- `serialize_slice`/`deserialize_slice`: `*target as u64` / `u64 as usize` casts — verify
  no truncation on 32-bit targets (wasm).
- Bounds checks use `cursor + 8 > bytes.len()` — verify no `usize` overflow (cursor bounded,
  but confirm).
- `byte_to_opcode` is a `HashMap` used only for lookup (not iteration) — confirm no
  non-determinism.

#### 1.3 `consensus/src/model/stores/contract.rs` — contract state store
Categories: unsafe code, consensus determinism, serialization, integer overflow.
- `ContractStorageKey::as_ref` (line ~37): `unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }`.
  Relies on `#[repr(C)]` layout `[u8;32]` + `u64` having **no padding** (offset 32 is 8-aligned,
  so currently no padding). **Fragile**: any field-size change introduces uninitialized padding
  bytes into the RocksDB key → non-deterministic keys / consensus fork. Replace with explicit
  byte serialization (e.g., `[u8;40]` built field-by-field).
- `commit_cache_batch` iterates `HashMap` (`code`, `storage`, `balances`, `metadata_hash`).
  Write order is non-deterministic. Verify whether RocksDB `WriteBatch` order affects final
  state / any state hash / merkle root. If any downstream hash depends on order → CRITICAL.
- `process_contract_tx`: gas fee math uses `saturating_mul`/`saturating_add`/`saturating_sub`
  — verify no silent value loss (e.g., `burned = total_fee/2`, `miner = total_fee - burned`).
- Deploy charges `gas_used: 0` but `gas_fee = max_gas * gas_price`; error paths charge full
  `max_gas`. Verify fee/burn accounting is consensus-consistent and not exploitable.
- `derive_contract_address` uses `TransactionSigningHash` — verify collision resistance and
  that `index` is always 0 (deploy output index).

#### 1.4 `consensus/src/pipeline/virtual_processor/processor.rs` — virtual processor
Categories: consensus logic bypass, auth, integer overflow, state commitment.
- `derive_caller_from_script_pub_key` (line ~1356): **msg.sender = first 8 bytes of address
  payload as LE u64**. 64-bit truncation → caller-identity collision / spoofing. An attacker
  who can grind an address whose first 8 payload bytes match a victim's can impersonate them
  in contract auth (`CALLER ... EQ`). HIGH. Recommend full 32-byte identity or hash.
- Inconsistency: `derive_caller_from_script_pub_key` returns `0` on empty/error, while
  `zyanya-wallet/src/wallet_ops.rs::holder_u64` returns `1` on empty → divergent identity.
- Payout logic (F-C-03): sell payout builds `TransactionOutput` from `caller_script_public_key`
  using hardcoded `Prefix::Mainnet`. Verify testnet/devnet correctness; verify payout value
  (`refund`) cannot exceed contract balance (uses `saturating_sub` — confirm no underflow
  silently pays out more than escrowed).
- Payout UTXOs are added **after** `verify_expected_utxo_state` (header utxo_commitment not
  covering them). Verify the determinism argument holds and no node can diverge.
- `tx_callers`/`tx_caller_scripts` are `HashMap`s keyed by tx id — verify no iteration-order
  dependence in downstream processing.

#### 1.5 `math/src/uint.rs` — big-integer arithmetic (Uint128/192/256)
Categories: integer overflow/underflow (consensus-critical).
- `Add`/`Sub`/`Mul`/`Shl`/`Shr` operator impls (lines ~530–690) use `overflowing_*` and guard
  with **`debug_assert!(!carry)` only**. In release builds `debug_assert!` is compiled out →
  **silent wrap**. `[profile.release] overflow-checks = true` does NOT help because
  `overflowing_*` never panics. Verify every call site is provably non-overflowing, or replace
  with real checked/panicking arithmetic. This affects difficulty, blue work, mass, fees.
- `div_rem`/`div_rem_u64`: verify division-by-zero handling (panic vs. error) and that it is
  consensus-consistent.
- `unsafe { str::from_utf8_unchecked }` in `to_hex`/`from_hex` paths (lines ~757, 816, 834, 848):
  verify the byte slices are guaranteed valid UTF-8 (hex alphabet) — otherwise UB.
- `from_u64`/`as_u64`/`as_u128` conversions: verify truncation semantics are intended.

### T1 — HIGH

#### 1.6 `crypto/txscript/src/data_stack.rs` + `crypto/txscript/src/opcodes/*`
Categories: script VM integer handling, minimal-encoding, DoS.
- `deserialize_i64` / `check_minimal_data_encoding`: verify minimal-encoding enforcement is
  consensus-exact (any divergence = fork). Check `serialize_i64` sign handling and the
  `i64::MIN` 9-byte edge case.
- `OpcodeData<i32>` deserialize uses `.expect("number is within i32 range")` — verify the
  `SizedEncodeInt<4>` bound actually guarantees i32 range (panic = DoS if reachable).
- `pop_items`/`peek_items` use `.expect("Already exact item")` after a length check — verify
  no panic path via `split_off`/`try_from` mismatch.
- `crypto/txscript/src/opcodes/mod.rs` + `standard.rs` + `multisig.rs`: verify opcode cost
  accounting, `OP_CHECKMULTISIG` off-by-one, sig-op counting (`runtime_sig_op_counter.rs`),
  and stack element size limits (DoS).

#### 1.7 `wallet/keys/src/secret.rs`, `privatekey.rs`, `keypair.rs`, `xprv.rs`
Categories: key management, secret handling, zeroization, Debug/serde leakage.
- `Secret` derives `Clone`, `Serialize`, `Deserialize`, `BorshSerialize`, `BorshDeserialize`.
  Verify serialization never writes plaintext secrets to logs/disk/JSON; verify `Clone` copies
  are zeroized on drop (Drop impl exists — confirm all clones dropped).
- `PrivateKey` derives `Debug` — verify `secp256k1::SecretKey`'s Debug is redacted (not raw
  bytes). If not, **secret leakage via Debug**.
- `PrivateKey::to_hex` returns hex of secret bytes — verify not logged.
- `wallet/keys/src/privkeygen.rs` / `pubkeygen.rs`: verify RNG source (OS CSPRNG, not seeded
  PRNG) and no bias.
- `wallet/keys/src/xprv.rs` / `xpub.rs`: verify child-key derivation (hardened vs non-hardened)
  and no key leakage across derivation.

#### 1.8 `wallet/bip32/src/*` (mnemonic, seed, xkey, derivation_path)
Categories: key management, mnemonic entropy, PBKDF2.
- `mnemonic/bits.rs`, `phrase.rs`, `seed.rs`: verify entropy length, checksum, and PBKDF2
  iteration count; verify no weak/empty passphrase default.
- `xprivate_key.rs` / `xpublic_key.rs`: verify serialization of extended keys does not leak
  private material into public key bytes.

#### 1.9 `wallet/psst/src/*` (PSST multisig)
Categories: signature validation, partial-signature aggregation, key handling.
- `input.rs`, `output.rs`, `bundle.rs`, `psst.rs`: verify partial signature validation,
  nonce reuse, and that unsigned inputs cannot be finalized. Check `role.rs` for signer
  authorization.

#### 1.10 `rpc/wrpc/server/*` and `rpc/grpc/server/*`
Categories: authentication, authorization, DoS, input validation.
- wRPC bearer-token auth (F-C-13): `service.rs` extracts token from WebSocket upgrade
  (`Authorization` header / `?token=`). Verify token comparison is constant-time and that
  state-changing methods are actually gated (not just read methods). Verify default
  `rpc_auth_token: None` does not leave state-changing endpoints open on public interfaces.
- gRPC server: verify no unauthenticated state-changing endpoints; verify request size limits
  and rate limits (DoS). Check `connection_handler.rs` for connection exhaustion.
- `rpc/core` + `rpc/service`: verify deserialization of untrusted protobuf/JSON (prost/borsh)
  is bounds-safe.

#### 1.11 `protocol/p2p/src/*` and `protocol/flows/src/*`
Categories: network DoS, deserialization safety, handshake validation.
- `protocol/p2p/src/convert/*`: verify all inbound message conversions are bounds-checked
  (block/tx/header/utxo). Malformed messages must not panic.
- `protocol/p2p/src/handshake.rs`, `core/connection_handler.rs`, `core/router.rs`: verify
  handshake validation, peer identity, and connection limits (DoS).
- `protocol/flows/src/v5/*` and `v6/*`: verify IBD/blockrelay/txrelay request handling has
  size/rate limits; verify orphan pool (`flowcontext/orphans.rs`) and process queue
  (`flowcontext/process_queue.rs`) cannot be memory-exhausted.

### T2 — MEDIUM

#### 1.12 `mining/src/mempool/*`
Categories: transaction validation, RBF, fee accounting, DoS.
- `validate_and_insert_transaction.rs`, `check_transaction_standard.rs`,
  `replace_by_fee.rs`, `populate_entries_and_try_validate.rs`: verify standardness checks,
  RBF rules, orphan pool limits, and that fee/mass math cannot overflow.

#### 1.13 `database/src/*`
Categories: storage safety, key serialization, cache consistency.
- Verify all DB key types use deterministic byte serialization (no padding/uninitialized
  bytes — same class of bug as 1.3). Check `CachedDbAccess` cache eviction correctness.

#### 1.14 `crypto/hashes/src/pow_hashers.rs` + `crypto/hashes/src/lib.rs`
Categories: unsafe FFI, cryptographic correctness.
- `unsafe { KeccakF1600(state) }` (line ~75): verify FFI signature, state size, and that the
  C/asm implementation is correct and constant-time where needed.
- `unsafe { str::from_utf8_unchecked(&hex) }` (line ~117): verify hex alphabet guarantee.

#### 1.15 `zyanya-vm/src/compiler/*` + `zyanya-vm/src/assembler.rs`
Categories: codegen correctness, stack-order consistency.
- Verify compiler codegen (`codegen.rs`) and assembler (`assembler.rs`) produce stack orders
  consistent with VM pop order for `SStore`, `Call`, `Store`, arithmetic (the prior CRIT-01
  false positive was exactly this class of bug — re-verify systematically).
- `assembler.rs` label resolution: verify jump targets resolve to opcode indices correctly
  and that forward/backward references cannot produce out-of-range jumps.

#### 1.16 `consensus/core/src/tx.rs`, `hashing/*`, `mass/*`, `subnets.rs`
Categories: transaction validation, sighash, mass accounting, subnetwork gating.
- `hashing/sighash.rs` + `sighash_type.rs`: verify sighash algorithm matches consensus and
  cannot be malleated.
- `mass/mod.rs`: verify mass computation (storage mass) cannot overflow and is deterministic.
- `subnets.rs`: verify smart-contract subnetwork gating (`is_smart_contract`) and that
  non-contract subnetworks cannot smuggle contract payloads.

### T3 — LOW/INFO

#### 1.17 `zyanya-explorer/src/*`, `zyanya-query/src/*`
Categories: XSS, injection, DoS, input validation.
- Verify HTML/JSON output escaping (XSS), query parameter validation, and result-size limits.

#### 1.18 `utils/*`, `metrics/*`, `notify/*`, `daemon/src/*`, `zyanyad/src/*`
Categories: config validation, resource limits, logging of secrets.
- Verify no secret material is logged; verify config parsing is bounds-safe; verify file
  permissions on keyfiles (prior LOW finding).

---

## 2. Cross-cutting Attack Surfaces (checklist)

1. **Integer overflow/underflow** — `math/src/uint.rs` operators (silent wrap in release);
   VM `Pow`; fee/mass/balance math in `consensus` and `mining`.
2. **Reentrancy / call-depth** — `zyanya-vm/src/vm.rs` `Call` guard (static, diamond-path
   bypass), `MAX_CALL_DEPTH`, gas forwarding.
3. **Key management** — `wallet/keys`, `wallet/bip32`, `wallet/psst`: zeroization, Debug/serde
   leakage, RNG, derivation.
4. **Consensus logic bypass** — `consensus/src/pipeline/virtual_processor/processor.rs`
   (caller derivation, payout after commitment), `consensus/core` (sighash, mass, subnets).
5. **Network DoS** — `protocol/p2p`, `protocol/flows`, `rpc/*`: message size limits, rate
   limits, connection limits, malformed-input panics.
6. **Input validation / deserialization** — `zyanya-vm/src/opcode.rs` (jump targets),
   `protocol/p2p/src/convert/*`, `crypto/txscript` (minimal encoding), borsh/prost.
7. **Unsafe code / raw pointers** — `consensus/src/model/stores/contract.rs` (key serialization),
   `crypto/hashes/src/pow_hashers.rs` (FFI), `math/src/uint.rs` (utf8), `crypto/addresses`
   and `consensus/core/src/tx/script_public_key.rs` (`ref_from_abi`).
8. **Cryptographic correctness** — `crypto/hashes` (Keccak/Blake2b), `crypto/merkle`,
   `crypto/muhash`, `consensus/core/src/hashing` (sighash), `crypto/addresses` (checksum).

---

## 3. Suggested Execution Order (builder agent)

1. T0 files (1.1–1.5) — VM, opcode, contract store, virtual processor, math.
2. T1 files (1.6–1.11) — txscript, wallet keys/bip32/psst, rpc, p2p.
3. T2 files (1.12–1.16) — mempool, database, hashes, compiler, consensus tx/hashing.
4. T3 files (1.17–1.18) — explorer/query/utils/daemon.

For each file: (a) read the code, (b) map to the categories above, (c) attempt a concrete
exploit/divergence, (d) record finding with severity + fix, (e) cross-reference
`audit_reports/` to avoid re-reporting known/fixed items.

## 4. Known Prior Findings to Cross-Check (from `audit_reports/`)

- CRIT-01 SStore operand order — **false positive** (verified 2026-08-01); do not re-apply.
- HIGH-01 HashMap iteration in `ContractStateCache` — marked FIXED; **verify current code
  still uses `HashMap`** (it does) and whether write order is actually consensus-relevant.
- HIGH wallet secret leakage, VM unchecked math, explorer DoS — verify current state.
- F-C-03 (sell payout), F-C-13 (wRPC auth), F-C-16 (metadata binding) — verify applied.
