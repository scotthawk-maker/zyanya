# Remediation Plan — Batch M1: MEDIUM findings in VM, Explorer, and Wallet

**Session:** `68e45a34`
**Date:** 2026-08-18
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md` §5 (MEDIUM findings)
**Scope:** F-M-01 through F-M-22, **excluding** F-M-08 and F-M-09 (verified safe — no action).
**Acceptance gate:** `cargo check --workspace --all-targets` must pass.

---

## 1. Summary

This batch remediates 20 MEDIUM findings across three subsystems:

- **VM / consensus-adjacent:** F-M-02, F-M-03, F-M-05, F-M-06, F-M-07, F-M-14, F-M-20
- **Wallet key management / crypto:** F-M-04, F-M-10, F-M-11, F-M-12, F-M-13, F-M-15, F-M-21, F-M-22
- **Wallet PSST / transaction signing:** F-M-16, F-M-17, F-M-18, F-M-19
- **Explorer:** F-M-01

**Important pre-flight note:** F-M-03 (ZCL parser depth limit) is **already implemented** in the current tree
(`zyanya-vm/src/compiler/parser.rs` has `MAX_PARSE_DEPTH = 100` and a `depth` counter in `parse_expression`).
Treat F-M-03 as **verify-only**: confirm the guard is present and add a regression test if none exists; do not re-implement.

All other findings were verified against the current tree and are **not yet fixed** (see per-finding status).

---

## 2. Priority order

All findings are MEDIUM, but within the batch they are ordered by security impact and blast radius:

| Priority | Findings | Rationale |
|----------|----------|-----------|
| P0 | F-M-21, F-M-22, F-M-13, F-M-04, F-M-11, F-M-10, F-M-12 | Secret/key material handling, KDF weakness, deadlock, panic-on-corrupt-input in wallet crypto |
| P1 | F-M-02, F-M-05, F-M-06, F-M-07, F-M-14, F-M-20 | VM gas accounting, unsafe blocks, division-by-zero, signing panics |
| P2 | F-M-16, F-M-17, F-M-18, F-M-19, F-M-15 | PSST deserialization/signing correctness, WASM panic surface |
| P3 | F-M-01 | Explorer input coercion (fail-open) |
| Verify | F-M-03 | Already fixed — verify + test only |

---

## 3. Files to audit and per-finding remediation

### P0 — Key material, KDF, wallet crypto

#### F-M-21 — `Secret` derives `Clone` (secret duplication)
- **File:** `wallet/keys/src/secret.rs:8`
- **Category:** Key management / secret handling
- **Status:** NOT fixed — `#[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]` present.
- **Fix:**
  1. Remove `Clone` from the derive list.
  2. Implement `Clone` **manually** with a `// SAFETY`/documentation comment stating that cloning is an explicit, audited duplication and that every `Secret` (including clones) is zeroized on drop by the existing `Drop` impl. (A `Clone` impl cannot zeroize its `&self` source; the mitigation is that all copies are zeroized on drop.)
  3. Keep `Serialize`/`Deserialize`/`Borsh*` for now — they are required by the `#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]` API message structs in `wallet/core/src/api/message.rs`. Add a code comment flagging plaintext-serialization risk as a follow-up (out of scope for this batch).
  4. Refactor the known `Secret` clone call sites to avoid the clone where trivial:
     - `wallet/core/src/wasm/wallet/storage.rs:202,211` — `decrypt(wallet_secret.clone())` → pass the reference directly (`decrypt(&wallet_secret)` or `decrypt(wallet_secret)` depending on the resolved type; `Encrypted::decrypt` takes `&Secret`).
     - `wallet/core/src/account/mod.rs:391,399` — reorder: call `keydata.get_xprv(payment_secret.as_ref())` **first**, then move `payment_secret` into `PSSBSigner::new(...)` instead of `payment_secret.clone()`.
  5. `cargo check` will surface any remaining `Secret` clone sites — fix each by borrowing or reordering.

#### F-M-22 — `argon2_sha256iv_hash` uses `sha256(data)` as salt (deterministic KDF)
- **File:** `wallet/core/src/encryption.rs:224` (function at `:243`), encrypt at `:251`, decrypt at `:264`
- **Category:** Cryptographic implementation correctness / KDF
- **Status:** NOT fixed.
- **Fix:**
  1. Add a core KDF helper `argon2_hash(data: &[u8], salt: &[u8], byte_length: usize) -> Result<Secret>` that runs Argon2 with the **explicit** salt.
  2. Keep `argon2_sha256iv_hash` as a thin wrapper (it is used by the WASM `js_argon2_sha256iv_phash_*` password-hash API in `wallet/core/src/wasm/encryption.rs`), but **stop using it in the encrypt/decrypt path**. Document that it is a deterministic password-hash helper, not an encryption KDF.
  3. `encrypt_xchacha20poly1305`: generate a random 16-byte salt via `OsRng`; compute `key = argon2_hash(secret, &salt, 32)`; output `salt || nonce || ciphertext`.
  4. `decrypt_xchacha20poly1305`: read `salt` (first 16 bytes), `nonce` (next 24 bytes), `ciphertext` (remainder); compute `key = argon2_hash(secret, &salt, 32)`; decrypt.
  5. Update the round-trip test; add a test asserting two encryptions of the same plaintext/password yield different ciphertexts (salt uniqueness).
  6. **Format note:** this changes the `Encrypted.payload` layout. The codebase is pre-production (audit verdict: NOT production-ready), so a silent format change is acceptable, but add a code comment documenting the new layout `salt(16) || nonce(24) || ciphertext+tag`. If backward compatibility is required, version the format — flag this in the commit message.

#### F-M-13 — Signer key caches never zeroized on drop
- **Files:** `wallet/core/src/tx/generator/signer.rs:17,62` (`Inner.keys`, `KeydataSignerInner.keys`), `wallet/core/src/account/pssb.rs:31` (`PSSBSignerInner.keys`)
- **Category:** Key management / zeroization
- **Status:** NOT fixed — all three are `Mutex<AHashMap<Address, [u8; 32]>>` / `HashMap<Address, [u8; 32]>` with no `Drop`.
- **Fix:**
  1. Implement `Drop` for `Inner`, `KeydataSignerInner`, and `PSSBSignerInner` that iterates the map and calls `.zeroize()` on each `[u8; 32]` value (import `zeroize::Zeroize`).
  2. Prefer `Zeroizing<[u8; 32]>` as the map value type if it does not ripple too widely; otherwise the explicit `Drop` loop is sufficient.
  3. Note: `KeydataSignerInner.keys` is a plain `HashMap` (not behind a `Mutex`) — a `Drop` impl on the struct is still straightforward.

#### F-M-04 — `from_secret_hex` uses `trim_start_matches("0x")`
- **File:** `zyanya-wallet/src/key_management.rs:96`
- **Category:** Key management / input validation
- **Status:** NOT fixed.
- **Fix:**
  1. Replace `hex_str.trim().trim_start_matches("0x")` with `hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim())` (strip at most one prefix).
  2. After stripping, reject input whose length is not 64 hex chars (return `KeyManagementError::InvalidHex` or a new length error) before `faster_hex::hex_decode`.
  3. Add a unit test: `"0x0x<64hex>"` must be rejected (or at least not silently accepted as a valid key).

#### F-M-11 — `decrypt_mnemonic` panics / UB on malformed Go-wallet files
- **File:** `wallet/core/src/compat/gen1.rs:9,15,18`
- **Category:** Input validation / deserialization safety / panic
- **Status:** NOT fixed.
- **Fix:**
  1. Validate `cipher.as_ref().len() >= 24` before `split_at(24)`; return `Err` otherwise.
  2. Clamp `num_threads` to `1..=64` before `p_cost(...)`; replace `.build().unwrap()` with `?`.
  3. Replace `.hash_password_into(...).unwrap()` with `?`.
  4. Replace `String::from_utf8_unchecked(decrypted)` with `String::from_utf8(decrypted)?`.
  5. Replace the `num_threads.expect(...)` in the v0 deserialization path (`into_wallet_type`) with a proper error (`?` / `ok_or`).

#### F-M-10 — `decrypt_xchacha20poly1305` panics on short ciphertext
- **File:** `wallet/core/src/encryption.rs:248` (current `:268` `let nonce = &data[0..24];`)
- **Category:** Input validation / panic
- **Status:** NOT fixed.
- **Fix:**
  1. Before slicing, check `data.len() >= 16 + 24 + 16` (salt + nonce + minimum AEAD tag) and return `Err(...)` otherwise.
  2. This check is naturally folded into the F-M-22 payload-layout change; if F-M-22 is implemented first, ensure the length check accounts for the new `salt(16)` prefix.

#### F-M-12 — Lock-order inversion deadlock in `PubkeyDerivationManagerV0`
- **File:** `wallet/keys/src/derivation/gen0/hd.rs:124,162`
- **Category:** Concurrency / deadlock
- **Status:** NOT fixed — `derive_pubkey_range` locks `cache` then `inner`; `derive_pubkey` locks `inner` then `cache`.
- **Fix:**
  1. Establish a single lock order: always acquire `inner` (via `opt_inner()`) **before** `cache`.
  2. In `derive_pubkey_range`, restructure so the `cache` guard is not held while calling `opt_inner()` (or acquire `inner` first, then `cache`).
  3. Verify `derive_pubkey` follows the same order (it already locks `inner` first, then `cache` — keep it).
  4. Add a comment documenting the lock order invariant.

---

### P1 — VM gas accounting, unsafe blocks, division-by-zero, signing panics

#### F-M-02 — Gas metering saturates; failing child `CALL` burns all forwarded gas
- **Files:** `zyanya-vm/src/gas.rs:28-36` (`consume`, `refund`), `zyanya-vm/src/vm.rs:240,267-270` (`CALL` path)
- **Category:** VM gas accounting / DoS / griefing
- **Status:** NOT fixed.
- **Fix:**
  1. `GasMeter::consume`: replace `saturating_add` with `checked_add`; on overflow return `VMError::OutOfGas { limit, requested }` (use `amount` or `u64::MAX` for `requested` — pick one and be consistent).
  2. `GasMeter::refund`: change signature to `Result<(), VMError>` and replace `saturating_sub` with `checked_sub`; return an error (e.g. `VMError::OutOfGas` or a new `GasRefundUnderflow` variant) on underflow instead of silently masking negative gas.
  3. `vm.rs` `CALL` path: in the `Err(_)` branch of the child execution result, compute `unused = child_vm.gas_meter.gas_limit() - child_vm.gas_meter.used_gas()` and call `self.gas_meter.refund(unused)?` **before** pushing `0`, so a failing child does not burn the entire forwarded gas. Update the `Ok` branch to handle the new `refund` return type.
  4. Add a VM test: a child `CALL` that fails early must not consume the full `forward_gas`.

#### F-M-05 — Unsafe `MaybeUninit` + `transmute` in PoW `heavy_hash`
- **File:** `consensus/pow/src/matrix.rs:97-103`
- **Category:** Unsafe code / raw pointer handling
- **Status:** NOT fixed.
- **Fix:**
  1. Replace the `MaybeUninit::uninit().assume_init()` + `transmute` block with a safe `array_from_fn` (already used elsewhere in the same file for `product`):
     ```rust
     let vec: [u8; 64] = array_from_fn(|i| {
         let element = hash.as_bytes()[i / 2];
         if i % 2 == 0 { element >> 4 } else { element & 0x0F }
     });
     ```
  2. Confirm the output is bit-identical to the old code (nibble expansion: `vec[2*i] = byte >> 4`, `vec[2*i+1] = byte & 0x0F`).
  3. Add a golden-vector test comparing `heavy_hash` output against a known hash.

#### F-M-06 — `unsafe` `AsRef<[u8]>` for `#[repr(C)] ContractStorageKey`
- **File:** `consensus/src/model/stores/contract.rs:22-44`
- **Category:** Unsafe code / memory layout
- **Status:** NOT fixed.
- **Fix (minimal, per prompt "add safety documentation"):**
  1. Add a compile-time assertion at module scope: `const _: () = assert!(std::mem::size_of::<ContractStorageKey>() == 40);`
  2. Add a `// SAFETY:` comment on the `AsRef` impl documenting the invariant: `#[repr(C)]` with `[u8; 32]` (align 1) followed by `u64` (align 8) at offset 32 → no padding, total 40 bytes, so the raw byte slice is exactly `contract_address || key.to_le_bytes()`.
  3. Add a unit test asserting `ContractStorageKey::new(addr, key).as_ref()` equals `addr.iter().chain(key.to_le_bytes().iter())`.
  4. (Optional, preferred) Replace the unsafe cast with safe serialization if the DB layer can accept an owned key; otherwise the assertion + comment is the accepted fix.

#### F-M-07 — Unsafe `str::from_utf8_unchecked` in hex display impls
- **Files:** `crypto/hashes/src/lib.rs:117`, `consensus/core/src/tx/script_public_key.rs:90`, `math/src/uint.rs:777,836,854,868`
- **Category:** Unsafe code / UTF-8 safety
- **Status:** NOT fixed.
- **Fix:**
  1. Replace each `unsafe { str::from_utf8_unchecked(&hex) }` with a safe conversion:
     - `f.write_str(...)` sites → `f.write_str(std::str::from_utf8(&hex).expect("hex output is valid UTF-8"))`.
     - `serializer.serialize_str(...)` sites → `serializer.serialize_str(std::str::from_utf8(&hex).map_err(serde::ser::Error::custom)?)`.
     - `math/src/uint.rs` `f.pad_integral(...)` sites → `std::str::from_utf8(&buf[..]).expect("...")`.
  2. Do **not** touch other `from_utf8_unchecked` sites outside these files (F-M-23 and the `utils/` family are separate findings/batches).

#### F-M-14 — Division-by-zero panics in mass calculation
- **File:** `wallet/core/src/tx/mass.rs:346,350`
- **Category:** Integer arithmetic / panic
- **Status:** NOT fixed.
- **Fix:**
  1. `calc_storage_mass_output_harmonic_single`: use `self.storage_mass_parameter.checked_div(output_value)` and return `0` (or propagate an error) when `output_value == 0`.
  2. `calc_storage_mass_input_mean_arithmetic`: guard `number_of_inputs == 0` (return `0` or error) and use `checked_div` for `total_input_value / number_of_inputs` and `storage_mass_parameter / mean_input_value`.
  3. Audit call sites of both functions and ensure they cannot pass `0` (or now handle the `0`/error return).

#### F-M-20 — `sign_with_multiple_v2` panics on invalid key / missing UTXO entry
- **File:** `consensus/core/src/sign.rs:129,138`
- **Category:** Input validation / panic
- **Status:** NOT fixed.
- **Fix:**
  1. `sign_with_multiple_v2`: replace `Keypair::from_seckey_slice(...).unwrap()` with `?` (the function already returns `Signed`; change it to return `Result<Signed, Error>` or map the error — note the existing `Error::Secp256k1Error` variant).
  2. Replace `mutable_tx.entries[i].as_ref().unwrap()` with a checked access returning an error (e.g. `Error::Message("missing UTXO entry")`).
  3. Update callers (`wallet/core/src/tx/generator/signer.rs`, `wallet/core/src/account/pssb.rs` if applicable) to propagate the new `Result`.
  4. Document/remove the dead v1 `sign_with_multiple` (keys map by SEC1 pubkey but looks up by full script — never matches). Prefer marking it `#[deprecated]` or removing it if no callers remain; route multisig through the PSST path.

---

### P2 — PSST deserialization/signing, WASM panic surface

#### F-M-16 — PSST deserialization has no size limits; silent partial-sig overwrite
- **Files:** `wallet/psst/src/psst.rs:171-177` (`from_hex`), `wallet/psst/src/input.rs:108` (`Input::add`)
- **Category:** Deserialization safety / DoS / signature correctness
- **Status:** NOT fixed.
- **Fix:**
  1. Add `const MAX_PSST_SIZE: usize = 10 * 1024 * 1024;` (10 MB) in `psst.rs`.
  2. In `from_hex`, after `hex::decode`, check `decoded.len() <= MAX_PSST_SIZE` before `serde_json::from_slice`; return `Error::Custom("PSST exceeds maximum size")` (or a new `Error` variant) on violation.
  3. In `Input::add`, before `self.partial_sigs.extend(rhs.partial_sigs)`, detect conflicting signatures for the same pubkey (same key, different `Signature`) and return a new `CombineError::ConflictingPartialSig { pubkey }` variant (add it to `wallet/psst/src/input.rs` `CombineError`).
  4. Add a test: combining two inputs with a conflicting partial sig for the same pubkey returns an error.

#### F-M-17 — `pssb_signer_for_address` panics on malformed input
- **File:** `wallet/core/src/account/pssb.rs:163-171`
- **Category:** Input validation / panic
- **Status:** NOT fixed.
- **Fix:**
  1. Validate `addresses.len() == inputs.len()` up front (when `sign_for_address` is `None`); return `Err` on mismatch.
  2. Replace `.expect("Input indexed address")`, `.expect("Public key for input indexed address")`, `.unwrap()` on `Message::from_digest_slice`, `.unwrap()` on `sign_schnorr`, and `.unwrap()` on `pass_signature_sync` with `?` / error propagation.
  3. Change the signing closure to return `Result<Vec<SignInputOk>, String>` (or the crate `Error`) and propagate.

#### F-M-18 — `psst_to_pending_transaction` hardcoded mass/fee, panics on empty outputs
- **File:** `wallet/core/src/account/pssb.rs:291,323-339`
- **Category:** Correctness / panic
- **Status:** NOT fixed.
- **Fix:**
  1. Replace `let mass = 10;` with a computed mass via `MassCalculator` (the same calculator used by the generator) for the extracted transaction.
  2. Replace `let fee_u: u64 = 0;` with the fee derived from the extracted transaction (inputs − outputs, or the generator's fee estimate).
  3. Replace `output[0]` indexing with a checked `outputs.first().ok_or(...)?` (return error on empty outputs).
  4. Replace `extract_script_pub_key_address(...).unwrap()` with `?`.

#### F-M-19 — Multisig signature ordering may not match redeem script pubkey order
- **File:** `wallet/core/src/account/pssb.rs:237-245` (`finalize_psst_one_or_more_sig_and_redeem_script`)
- **Category:** Signature correctness / multisig
- **Status:** NOT fixed.
- **Fix:**
  1. In the finalizer, when a `redeem_script` is present, parse it to extract the pubkeys **in script order** (the multisig redeem script is `OP_1..OP_16 <32-byte pubkey>... OP_N OP_CHECKMULTISIG`; use the txscript opcode parser or a small local parser).
  2. Order `input.partial_sigs` by each pubkey's position in the redeem script before emitting signatures. If a partial-sig pubkey is not present in the redeem script, return an error.
  3. Add a multisig finalize test that verifies the emitted signature order matches the redeem script pubkey order.

#### F-M-15 — `Mnemonic` WASM surface panics on invalid input / inconsistent state
- **File:** `wallet/bip32/src/mnemonic/phrase.rs:79,91,99`
- **Category:** WASM panic surface / state consistency
- **Status:** NOT fixed.
- **Fix:**
  1. `set_entropy`: return `Result<(), JsValue>` (wasm-bindgen setters may return `Result<(), JsValue>` to throw) instead of `panic!`; validate hex decode and length (16 or 32) and return an error.
  2. `set_phrase`: re-validate the phrase (call `Mnemonic::new(phrase, language)` or equivalent) and recompute entropy; return `Result<(), JsValue>` on invalid input instead of silently storing an unvalidated phrase.
  3. `phrase_string`: return `Zeroizing<String>` where the wasm-bindgen binding permits, or at minimum document the plaintext-return risk. (If `Zeroizing<String>` is not wasm-bindgen-compatible, keep `String` but add a comment.)
  4. Verify the non-WASM `Mnemonic::new`/`from_entropy` paths already validate (they do) and are unaffected.

---

### P3 — Explorer

#### F-M-01 — Explorer silently coerces invalid storage keys / addresses to 0 or 1
- **Files:** `zyanya-explorer/src/api.rs:155-159`, `zyanya-explorer/src/client.rs:1080,1236,1472`
- **Category:** Input validation / fail-open
- **Status:** NOT fixed.
- **Fix:**
  1. `api.rs` `api_contract_state_handler`: parse the `key` query param with `parse_u64_key` (or equivalent) and return `400 Bad Request` on parse failure instead of `unwrap_or(0)` / `unwrap_or(0)` on the outer `Option`.
  2. `client.rs:1080` (`buyer_u64`) and `:1236` (`seller_u64`): replace `parse_u64_key(&user_address.to_string()).unwrap_or(1)` with `parse_u64_key(&user_address.to_string())?` (propagate the error; the enclosing functions already return `Result<_, String>`).
  3. `client.rs:1472` (`swap_on_dex`): replace `token_in.parse::<u64>().unwrap_or(0)` with an error return for unknown `token_in` values (keep the explicit `"a"|"0"|"zyan"` and `"b"|"1"|"ghost"` mappings; fail closed otherwise).
  4. Add/adjust tests asserting malformed keys/addresses produce errors, not silent `0`/`1`.

---

### Verify-only

#### F-M-03 — ZCL parser recursion depth limit
- **File:** `zyanya-vm/src/compiler/parser.rs:143-262`
- **Status:** ALREADY FIXED (`MAX_PARSE_DEPTH = 100`, `depth` counter in `parse_expression`, `ParserError::MaxRecursionDepth`).
- **Action:** Confirm the guard covers the `LParen → parse_expression` recursion path (it does, since `parse_primary` calls `parse_expression`). Add a regression test that feeds deeply nested parentheses (e.g. 200 levels) and asserts `MaxRecursionDepth` is returned rather than a stack overflow.

---

## 4. Vulnerability categories per file (summary)

| File | Categories |
|------|-----------|
| `zyanya-vm/src/gas.rs`, `zyanya-vm/src/vm.rs` | Gas accounting overflow/underflow, DoS/griefing |
| `zyanya-vm/src/compiler/parser.rs` | Recursion DoS (verify-only) |
| `consensus/pow/src/matrix.rs` | Unsafe code, raw pointer/transmute |
| `consensus/src/model/stores/contract.rs` | Unsafe code, memory layout |
| `crypto/hashes/src/lib.rs`, `consensus/core/src/tx/script_public_key.rs`, `math/src/uint.rs` | Unsafe UTF-8 conversion |
| `consensus/core/src/sign.rs` | Panic on invalid input, dead code |
| `wallet/keys/src/secret.rs` | Secret duplication, zeroization |
| `wallet/keys/src/derivation/gen0/hd.rs` | Lock-order deadlock |
| `wallet/core/src/encryption.rs` | KDF weakness, panic on short ciphertext |
| `wallet/core/src/compat/gen1.rs` | Panic/UB on malformed import |
| `wallet/core/src/tx/generator/signer.rs`, `wallet/core/src/account/pssb.rs` | Key zeroization, panic on malformed input, mass/fee correctness, multisig ordering |
| `wallet/core/src/tx/mass.rs` | Division-by-zero |
| `wallet/bip32/src/mnemonic/phrase.rs` | WASM panic, state consistency |
| `wallet/psst/src/psst.rs`, `wallet/psst/src/input.rs` | Deserialization size limits, signature conflict |
| `zyanya-wallet/src/key_management.rs` | Key input validation |
| `zyanya-explorer/src/api.rs`, `zyanya-explorer/src/client.rs` | Input coercion / fail-open |

---

## 5. Implementation order & verification

1. Implement P0 fixes (F-M-21, F-M-22, F-M-13, F-M-04, F-M-11, F-M-10, F-M-12).
2. Implement P1 fixes (F-M-02, F-M-05, F-M-06, F-M-07, F-M-14, F-M-20).
3. Implement P2 fixes (F-M-16, F-M-17, F-M-18, F-M-19, F-M-15).
4. Implement P3 fix (F-M-01).
5. Verify F-M-03 and add its regression test.
6. Run `cargo check --workspace --all-targets` and fix all compile errors/warnings introduced.
7. Run targeted tests for the touched crates (`cargo test -p zyanya-vm -p zyanya-wallet-keys -p zyanya-wallet-core -p zyanya-wallet-psst -p zyanya-pow -p zyanya-explorer` as applicable).

---

## 6. Notes for the builder

- **Do not modify** `audit_reports/` or any existing `specs/` file.
- F-M-08 and F-M-09 are **verified safe** — do not touch `crypto/addresses/src/bech32.rs` or `consensus/pow/src/matrix.rs` `compute_rank`.
- F-M-07 is scoped to exactly the four listed files; leave `rpc/core/src/model/hex_cnv.rs` (F-M-23) and the `utils/` serde hex helpers for their own batches.
- F-M-22 changes the encrypted payload format; call this out in the commit message and add a code comment documenting the new layout.
- F-M-21: prefer borrowing/reordering over cloning; if a `Secret` clone is unavoidable, the manual `Clone` impl must carry a safety comment.
- The `cargo check --workspace --all-targets` gate is mandatory; run it from the repo root.
