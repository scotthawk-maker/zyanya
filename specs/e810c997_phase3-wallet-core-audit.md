# Phase 3 Security Audit Plan — Wallet Core

## Objective

Deep, line-level security audit of the Zyanya wallet core library (28,017 lines in
`wallet/core/src/`, plus `wallet/keys`, `wallet/bip32`, `wallet/psst`, `wallet/macros`,
`wallet/wasm`, `wallet/native`). This is upstream Kaspa wallet code, renamed to `zyanya-*`
crates, with a small number of Zyanya-specific modifications. The goal is to:

1. Identify and verify **Zyanya-specific modifications** to upstream wallet code
   (highest-risk items — upstream code is battle-tested, forks are not).
2. Check the upstream modules for **inherent vulnerabilities** in the listed categories
   (private key storage/leakage, BIP32 derivation, mnemonic handling, signing correctness,
   multisig, PSBT parsing/injection, UTXO selection/fee estimation, address generation,
   key-wiping/zeroization, race conditions).
3. Write a single findings report to:

```
audit_reports/phase3_findings.md
```

The report must contain, for **every** finding:

- **Severity**: `CRITICAL` / `HIGH` / `MEDIUM` / `LOW`
- **Location**: `file:line` (exact line numbers, verified against the current tree)
- **Description**: what the bug is and how it is triggered
- **Code snippet**: the offending lines (verbatim, trimmed)
- **Recommended fix**: concrete, minimal remediation

Order findings by severity (CRITICAL first), then by file path.

---

## How to tell Zyanya code from upstream Kaspa code

The git history is a single source-import commit (`296f86c Initial: Zyanya rusty-spectre
source for audit`), so there is no upstream diff to consult. Identify Zyanya modifications by:

- `grep -rin "zyanya" wallet/` — mostly crate renames (`zyanya_bip32`, `zyanya_wallet_keys`,
  `zyanya_consensus_core`, etc.) and the `zyanya:`/`zyanyatest:` address prefixes. Cosmetic.
- **Substantive Zyanya changes** found so far (verify each):
  1. `wallet/bip32/src/prefix.rs` — new extended-key prefixes `kprv`/`kpub` (mainnet) and
     `ktrv`/`ktub` (testnet/devnet/simnet), versions `0x038f2ef4`/`0x038f332e`/
     `0x03909e07`/`0x0390a241`. `From<NetworkId> for Prefix` maps Mainnet→`KPUB`,
     everything else→`KTUB`.
  2. `wallet/core/src/account/variants/*.rs` — account-kind string constants renamed to
     `zyanya-bip32-standard`, `zyanya-legacy-standard`, `zyanya-multisig-standard`,
     `zyanya-keypair-standard`, `zyanya-bip32-watch-standard`.
  3. `wallet/core/src/derivation.rs` — `create_xpub_from_xprv` and `build_derivate_path`
     contain `panic!` arms for unsupported account kinds (verify whether these are Zyanya
     additions vs upstream).
  4. `wallet/core/src/compat/gen1.rs` — Go-wallet import path (`import_zyanyawallet_golang_*`),
     Argon2id + XChaCha20Poly1305 mnemonic decryption (verify params vs upstream).

Everything else is expected to be upstream Kaspa; audit it for inherent bugs, not diffs.
The BIP32 coin types are upstream Kaspa values: gen1 uses `44'/123456'/...` (and `45'` for
multisig), gen0 legacy uses `44'/972/...`.

---

## Audit scope (files, in priority order)

### CRITICAL tier — key material handling and signing

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 1 | `wallet/core/src/storage/keydata/data.rs` | `Debug`/`Clone` on `PrvKeyDataVariant`/`PrvKeyDataPayload`/`PrvKeyData` leaking mnemonic/seed/xprv/secret-key; `xxh3_64` secret-derived ID; `Zeroize`/`Drop` correctness |
| 2 | `wallet/keys/src/secret.rs` | `Secret` derives `Clone`/`Serialize`/`Deserialize`/`Borsh*` (secret duplication); `as_str` exposure; `From<String>` trim behavior |
| 3 | `wallet/keys/src/privatekey.rs`, `wallet/keys/src/keypair.rs`, `wallet/keys/src/xprv.rs` | `Debug`/`Clone` on types wrapping `secp256k1::SecretKey` (verify whether `SecretKey`'s `Debug` redacts); WASM getters returning plain `String` hex of private key (not zeroized) |
| 4 | `wallet/bip32/src/xprivate_key.rs`, `wallet/bip32/src/xkey.rs` | `ExtendedPrivateKey` derives `Clone` but has **no** `Drop`/`Zeroize` (key not wiped on drop); `ExtendedKey` derives `Clone`; `write_base58` output buffer not zeroized; `from_str` prefix/version cross-validation gap |
| 5 | `wallet/bip32/src/mnemonic/phrase.rs`, `wallet/bip32/src/mnemonic/seed.rs` | `Mnemonic` derives `Clone` + `#[wasm_bindgen(inspectable)]`; `set_entropy`/`set_phrase` panic on invalid input and allow inconsistent state; `phrase_string()` returns non-zeroized `String`; checksum validation correctness (12-word 4-bit checksum) |
| 6 | `wallet/core/src/encryption.rs` | `decrypt_xchacha20poly1305` panics on `<24`-byte ciphertext; `Decrypted<T>`/`Encryptable<T>` derive `Clone`/`Debug` (plaintext duplication + Debug leak); `Encrypted` `Debug` prints ciphertext hex; `argon2_sha256iv_hash` uses `sha256(password)` as salt (deterministic KDF, no per-wallet salt) |
| 7 | `wallet/core/src/tx/generator/signer.rs`, `wallet/core/src/account/pssb.rs` | Signer key caches (`AHashMap<Address,[u8;32]>` / `HashMap<Address,[u8;32]>`) never zeroized on drop; `.unwrap()`/`.expect()` panics on missing keys/addresses |
| 8 | `consensus/core/src/sign.rs` (cross-ref, called by wallet) | `sign_with_multiple_v2` `.unwrap()` on invalid privkey / missing UTXO entry; no multisig support; `sign_with_multiple` (v1) keyed by SEC1 pubkey vs script (dead/broken) |

### HIGH tier — storage, derivation, multisig, PSBT

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 9 | `wallet/core/src/storage/local/wallet.rs` | `try_store` uses `std::fs::File::create` with **no** 0600 permissions (wallet file world-readable on default umask) |
| 10 | `wallet/core/src/storage/local/transaction/fsio.rs` | Transaction records written as `Encryptable::Plain` (secret `None`); no file permissions; `read`/`read_sync` decrypt path |
| 11 | `wallet/core/src/compat/gen1.rs` | `decrypt_mnemonic`: `split_at(24)` panic on short cipher; `num_threads` from untrusted file → `p_cost(0)`/huge → panic/DoS; `String::from_utf8_unchecked` UB risk; `.expect("num_threads must present")` panic |
| 12 | `wallet/keys/src/derivation/gen0/hd.rs` | Lock-order inversion deadlock (`derive_pubkey_range` locks cache→inner, `derive_pubkey` locks inner→cache); `unreachable!()` in `from_extended_public_key*`; `.expect()`/`.unwrap()` on uninitialized state; `index()+1` u32 overflow |
| 13 | `wallet/keys/src/derivation/gen1/hd.rs` | Hardened-vs-unhardened derivation correctness; `create_extended_key` path construction; `derive_child_pubkey_manager` non-hardened cosigner/address-type derivation |
| 14 | `wallet/core/src/derivation.rs` | `create_address`/`create_multisig_address` do **not** validate `minimum_signatures > 0` (0-of-N multisig = anyone-can-spend — verify `multisig_redeem_script` behavior); `AddressManager::new_address` `index()+1` overflow; `update_address_to_index_map` `offset+index` overflow; `panic!` arms in `create_xpub_from_xprv`/`build_derivate_path` |
| 15 | `wallet/core/src/account/variants/multisig.rs` | `minimum_signatures: u16` not validated (>0, ≤ xpub count); `sig_op_count` `u8::try_from(xpub_keys.len()).unwrap()` panic; `cosigner_index` handling |
| 16 | `wallet/psst/src/psst.rs`, `wallet/psst/src/input.rs`, `wallet/psst/src/output.rs`, `wallet/psst/src/global.rs`, `wallet/psst/src/bundle.rs`, `wallet/psst/src/convert.rs` | PSBT deserialization with no size limits (memory DoS); `Input::add` silently overwrites conflicting partial sigs (BTreeMap `extend`); `PSST<Combiner>::add` combine macro keeps extra inputs from longer list without count check; `finalize_internal` does not verify signatures; `extract_tx_unchecked` vs `extract_tx` |
| 17 | `wallet/core/src/account/pssb.rs` | `pssb_signer_for_address` `.expect()`/`.unwrap()` panics; `finalize_psst_one_or_more_sig_and_redeem_script` signature ordering (BTreeMap order vs redeem-script pubkey order) for multisig; `psst_to_pending_transaction` hardcoded `mass=10`, `fee=0`, `output[0]` indexing panic |

### MEDIUM tier — UTXO selection, fee estimation, address generation

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 18 | `wallet/core/src/tx/generator/generator.rs` | **Double `transaction_fees += change_output_value`** (lines ~797 and ~809) in `absorb_change_to_fees` branch — fee over-counting; underpayment of receiver in `ReceiverPays` mode; unchecked `+`/`sum()` on u64; `unreachable!`/`expect` panics |
| 19 | `wallet/core/src/tx/mass.rs` | `calc_storage_mass_output_harmonic_single` and `calc_storage_mass_input_mean_arithmetic` divide by zero (panic); `calc_minimum_required_transaction_relay_fee` `mass*1000` overflow; `transaction_serialized_byte_size` plain `sum()` |
| 20 | `wallet/core/src/tx/fees.rs` | `Fees::try_from(&str)` silently maps invalid fee string to 0; `From<i64>` `i64::MIN` handling |
| 21 | `wallet/core/src/utxo/context.rs` | `panic!`/`unreachable!` on invariant violations (`revive`, `promote`); `calculate_balance` plain `sum()` overflow; `.expect()` on outgoing-transaction lookups |
| 22 | `wallet/core/src/account/descriptor.rs`, `wallet/core/src/account/mod.rs` | `create_private_keys` derivation correctness (gen0 hardened vs gen1 non-hardened); `unsafe { from_utf8_unchecked }` at `account/mod.rs:856` |
| 23 | `wallet/bip32/src/derivation_path.rs`, `wallet/bip32/src/child_number.rs` | `DerivationPath::from_str` no length limit (memory DoS); `ChildNumber::from_bytes`/`From<u32>` skip validation; `Deserialize` visitor `expecting` copy-paste message |
| 24 | `wallet/bip32/src/prefix.rs` | `ExtendedKey::from_str` does not cross-validate prefix string vs version bytes (version ignored for `is_private`/`is_public`); `From<NetworkId>` maps all non-mainnet to `KTUB` |

### LOW tier — WASM/native bindings, macros, misc

| # | File | Vulnerability categories |
|---|------|--------------------------|
| 25 | `wallet/wasm/src/lib.rs`, `wallet/native/src/main.rs` | Thin CLI wrappers; confirm no key material logged |
| 26 | `wallet/macros/src/*.rs` | Macro-generated WASM interfaces; confirm no `Debug`/serialization of secrets |
| 27 | `wallet/core/src/wasm/**` | WASM bindings for signer/encryption/keydata — confirm no plaintext key leakage to JS, no panics on malformed JS input |
| 28 | `wallet/core/src/storage/local/mod.rs` | `static mut` storage-folder/file globals (unsafe, not thread-safe); `set_default_storage_folder` creates dir |
| 29 | `wallet/core/src/storage/local/transaction/indexdb.rs` | IndexedDB transaction handling; no size limits |

---

## Findings to validate and expand

For each item: confirm the exact `file:line`, reproduce the trigger mentally (or with a
minimal test where feasible), and write it up in the report format. Items marked **[NEW]**
are specific to this phase (not in Phase 1/2 reports).

### 1. Private key material leaked via derived `Debug` impls — HIGH [NEW]

- **File**: `wallet/core/src/storage/keydata/data.rs:20` (`PrvKeyDataVariant`),
  `:118` (`PrvKeyDataPayload`), `:190` (`PrvKeyData`); also `wallet/keys/src/keypair.rs:28`
  (`Keypair`), `wallet/keys/src/privatekey.rs:12` (`PrivateKey`)
- **What to verify**:
  - `PrvKeyDataVariant` is `#[derive(Clone, Debug, Serialize, Deserialize)]` and its
    variants hold the mnemonic / BIP39 seed / xprv / secret key as a plain `String`.
    `{:?}` of a `PrvKeyDataVariant` (or of `PrvKeyDataPayload`/`PrvKeyData`, which
    transitively contain it) prints the secret in full. Any `log_*!`/`println!`/`dbg!`
    of these types leaks the key to logs.
  - `Keypair` and `PrivateKey` derive `Debug` and wrap `secp256k1::SecretKey`. **Verify**
    whether `secp256k1` 0.29.1's `SecretKey` `Debug` impl redacts or prints the secret
    (check `display_secret()`). If it prints, `{:?}` of `Keypair`/`PrivateKey` leaks the key.
  - `Encryptable<T>`/`Decrypted<T>` in `encryption.rs` derive `Debug`; `Decrypted<PrvKeyDataPayload>`
    Debug prints the plaintext payload.
- **Fix direction**: replace derived `Debug` with a redacting manual impl
  (`f.debug_struct(...).field("secret", &"********")`), or remove `Debug` entirely from
  secret-bearing types; add a `#[deny(clippy::dbg_macro)]`-style lint and grep for
  `{:?}`/`dbg!`/`println!` on these types.

### 2. Wallet file written with default (world-readable) permissions — HIGH [NEW]

- **File**: `wallet/core/src/storage/local/wallet.rs:65-72` (`WalletStorage::try_store`)
- **What to verify**:
  - Native path uses `std::fs::File::create(store.filename())` with no explicit mode.
    `File::create` uses `0o666 & !umask` → `0o644` on default umask 022, i.e. the wallet
    file (containing the encrypted mnemonic/private keys) is world-readable. An attacker
    with local FS access can read the encrypted blob and brute-force the payment secret
    offline.
  - Compare with `zyanya-wallet/src/key_management.rs` (Phase 1) which sets 0600 — the
    core wallet storage does not.
  - Also check `fsio.rs` transaction-record writes (plaintext, no permissions).
- **Fix direction**: create the file with `OpenOptions::new().write(true).create(true)
  .mode(0o600)` (or `set_permissions(0o600)` after create) on non-WASM; document the
  WASM/localStorage equivalent.

### 3. `decrypt_xchacha20poly1305` panics on short ciphertext — MEDIUM [NEW]

- **File**: `wallet/core/src/encryption.rs:248` (`let nonce = &data[0..24];`)
- **What to verify**: if the stored encrypted payload is truncated/corrupted to `< 24`
  bytes, the slice panics (DoS) instead of returning an error. This is reachable from
  any stored `Encrypted` payload (wallet file, transaction records, keydata).
- **Fix direction**: check `data.len() >= 24 + 16` (nonce + tag) and return
  `Error::Chacha20poly1305`/a length error before slicing.

### 4. Go-wallet import path: panics and UB on malformed wallet files — MEDIUM [NEW]

- **File**: `wallet/core/src/compat/gen1.rs:9-18` (`decrypt_mnemonic`), `:181/:192`
  (`.expect("num_threads must present in case of v0")`)
- **What to verify**:
  - `cipher.as_ref().split_at(24)` panics if `cipher.len() < 24`.
  - `num_threads: u32` comes from the imported wallet file; `p_cost(num_threads)` with
    `num_threads == 0` (or huge) makes `ParamsBuilder::build().unwrap()` panic or
    allocate unbounded memory (DoS). `num_threads` is `Option<u8>` in the JSON but is
    cast to `u32`; a v0 file missing `numThreads` panics via `.expect(...)`.
  - `String::from_utf8_unchecked(decrypted)` is UB if the AEAD-decrypted plaintext is
    not valid UTF-8 (only "safe" because AEAD authenticates the correct password; still
    fragile — prefer `String::from_utf8`).
- **Fix direction**: validate `cipher.len() >= 24`, clamp/validate `num_threads` to a
  sane range (e.g. 1..=64), use `String::from_utf8(...)?`, and return errors instead of
  `.expect()`/`.unwrap()`.

### 5. Double addition of `change_output_value` to `transaction_fees` — HIGH [NEW]

- **File**: `wallet/core/src/tx/generator/generator.rs:797` and `:809`
- **What to verify**:
  - In `try_finish_standard_stage_processing`, the `absorb_change_to_fees || change_output_value == 0`
    branch executes `transaction_fees += change_output_value;` **twice** (once at :797,
    once at :809 after the mass recompute).
  - For `Fees::SenderPays`, the change is absorbed into the actual fee (no change output
    is created), so the double-count only over-reports `transaction_fees` by
    `change_output_value` (misleading fee metric, wrong `PendingTransaction` fee).
  - For `Fees::ReceiverPays`, `generate_transaction` reduces the receiver's output by
    `transaction_fees` (`output.value -= transaction_fees`), so the receiver is
    **underpaid by `change_output_value`** (fund loss). Note the `ReceiverPays` path is
    currently marked "unreachable at the API level" but is still a latent fund-loss bug.
- **Fix direction**: remove the duplicate `transaction_fees += change_output_value;`
  (keep exactly one); add a unit test asserting `transaction_fees == inputs - outputs`
  for the absorb-change case.

### 6. Division-by-zero panics in mass calculation — MEDIUM [NEW]

- **File**: `wallet/core/src/tx/mass.rs:346` (`calc_storage_mass_output_harmonic_single`:
  `self.storage_mass_parameter / output_value`) and `:350`
  (`calc_storage_mass_input_mean_arithmetic`: `total_input_value / number_of_inputs`)
- **What to verify**:
  - `calc_storage_mass_output_harmonic_single(0)` panics (integer div-by-zero). It is
    called from `generate_edge_transaction` with
    `data.aggregate_input_value.saturating_sub(compute_fees)` which can be 0 when the
    aggregated input value ≤ edge compute fees (tiny-UTXO wallet) → panic.
  - `calc_storage_mass_input_mean_arithmetic` divides by `number_of_inputs`; verify every
    call site guarantees `number_of_inputs > 0` (currently `data.inputs.len()` after at
    least one `aggregate_utxo`), and harden anyway.
- **Fix direction**: return `Option`/`Result` (or `checked_div`) and propagate
  `Error::MassCalculationError`; guard `number_of_inputs == 0`.

### 7. Lock-order inversion deadlock in `PubkeyDerivationManagerV0` — MEDIUM [NEW]

- **File**: `wallet/keys/src/derivation/gen0/hd.rs:124-125` (`derive_pubkey_range` locks
  `cache` then `inner`) vs `:162-175` (`derive_pubkey` locks `inner` then `cache`)
- **What to verify**:
  - `derive_pubkey_range` holds the `cache` mutex while calling `opt_inner()` (locks
    `inner`). `derive_pubkey` holds the `inner` mutex (via `opt_inner()`) while calling
    `self.cache.lock()`. Concurrent calls from two threads deadlock (classic ABBA).
  - Confirm the two functions can run concurrently (e.g. `get_range` vs `current_pubkey`
    on the same `Arc<PubkeyDerivationManagerV0>` from different tasks).
- **Fix direction**: establish a single lock order (always `inner` before `cache`, or
  use one combined mutex); drop the `cache` guard before `opt_inner()` in
  `derive_pubkey_range`, or clone the needed data under the cache lock first.

### 8. Signer key caches never zeroized — MEDIUM [NEW]

- **File**: `wallet/core/src/tx/generator/signer.rs:17` (`Signer::Inner.keys`),
  `:62` (`KeydataSignerInner.keys`), `wallet/core/src/account/pssb.rs:31`
  (`PSSBSignerInner.keys`)
- **What to verify**: all three caches are `Mutex<AHashMap<Address,[u8;32]>>` /
  `HashMap<Address,[u8;32]>` holding raw private keys, with no `Drop`/`Zeroize`. After
  signing, `keys_for_signing.zeroize()` wipes the temporary vector but the cached map
  entries persist in memory until the signer is dropped (and even then are not wiped).
- **Fix direction**: implement `Drop` (or `Zeroize`) on the `Inner` structs that
  iterates and zeroizes each `[u8;32]` value; consider `zeroize::Zeroizing<[u8;32]>`
  values in the map.

### 9. `Secret`/`ExtendedPrivateKey`/`Mnemonic`/`ExtendedKey` derive `Clone`; `ExtendedPrivateKey` lacks `Drop` — MEDIUM [NEW]

- **File**: `wallet/keys/src/secret.rs:8`, `wallet/bip32/src/xprivate_key.rs:18`,
  `wallet/bip32/src/mnemonic/phrase.rs:47`, `wallet/bip32/src/xkey.rs:14`
- **What to verify**:
  - `Secret` derives `Clone` (and `Serialize`/`Deserialize`/`Borsh*`) — every clone
    duplicates the secret bytes; only the clone's own `Drop` zeroizes its copy, and the
    transient copy during `clone()` is not wiped.
  - `ExtendedPrivateKey<K>` derives `Clone` but implements **no** `Drop`/`Zeroize`, so
    the wrapped `secp256k1::SecretKey` is not wiped when the struct is dropped (verify
    whether `secp256k1::SecretKey` zeroizes on drop — it does not by default).
  - `Mnemonic` derives `Clone` (phrase + entropy duplicated); `ExtendedKey` derives
    `Clone` (private `key_bytes` duplicated).
- **Fix direction**: remove `Clone` from secret-bearing types where possible, or implement
  manual `Clone` that is explicit about the copy; add `Drop`/`Zeroize` to
  `ExtendedPrivateKey`; prefer `Zeroizing<...>` wrappers.

### 10. `Mnemonic` WASM surface: panics, inconsistent state, non-zeroized getters — MEDIUM [NEW]

- **File**: `wallet/bip32/src/mnemonic/phrase.rs:73-100`
- **What to verify**:
  - `set_entropy` panics on invalid hex or wrong length (`panic!("invalid entropy ...")`,
    `panic!("Invalid entropy ...")`) — a JS caller can trigger a WASM panic (DoS).
  - `set_phrase` sets `self.phrase` without re-validating or recomputing entropy, so a
    `Mnemonic` can be left in an inconsistent state (phrase ≠ entropy); a subsequent
    `to_seed` derives from the arbitrary phrase.
  - `phrase_string()` returns a plain `String` (clone of the mnemonic) that is not
    zeroized; `#[wasm_bindgen(inspectable)]` exposes fields to JS.
- **Fix direction**: return `Result` from setters instead of panicking; re-validate on
  `set_phrase`; return `Zeroizing<String>` where possible (or document the JS copy).

### 11. PSBT/PSSB deserialization: no size limits, silent partial-sig overwrite, combine count mismatch — MEDIUM [NEW]

- **File**: `wallet/psst/src/psst.rs:150-156` (`from_hex`), `wallet/psst/src/bundle.rs:60-66`
  (`deserialize`), `wallet/psst/src/input.rs:88-90` (`partial_sigs.extend`),
  `wallet/psst/src/psst.rs:250-270` (`combine!` macro)
- **What to verify**:
  - `serde_json::from_slice` on untrusted `PSST`/`PSSB` hex with no bounds on
    `inputs`/`outputs`/`unknowns`/`proprietaries` → memory DoS (multi-GB JSON).
  - `Input::add` does `self.partial_sigs.extend(rhs.partial_sigs)` — a conflicting
    signature for the same pubkey silently overwrites (BTreeMap insert) instead of
    erroring (BIP-174 requires conflict detection).
  - `PSST<Combiner>::add` `combine!` macro zips the shorter into the longer and keeps the
    longer's extra elements without verifying `input_count`/`output_count` match the
    `Global` counts (the `Global::add` takes `max` of counts).
  - `finalize_internal` checks sig count/non-empty but does **not** verify signatures;
    only `extract_tx` (checked) runs the script engine.
- **Fix direction**: add explicit limits (max inputs/outputs, max JSON size) before
  deserialization; detect conflicting partial sigs; reject count mismatches in combine;
  document `extract_tx_unchecked` as unsafe-by-contract.

### 12. `psst_to_pending_transaction` hardcoded mass/fee and panics — MEDIUM [NEW]

- **File**: `wallet/core/src/account/pssb.rs:290-360`
- **What to verify**:
  - `let mass = 10;` and `let fee_u: u64 = 0;` are hardcoded placeholders — the
    reconstructed `PendingTransaction` has wrong mass/fee (fee estimation and relay
    acceptance break).
  - `output[0]` indexing panics on an empty-output PSST; `extract_script_pub_key_address(...).unwrap()`
    panics on an unparseable script pub key.
- **Fix direction**: compute mass via `MassCalculator` and fee from the extracted tx;
  return errors instead of indexing/`unwrap()`.

### 13. `pssb_signer_for_address` panics on malformed input — MEDIUM [NEW]

- **File**: `wallet/core/src/account/pssb.rs:150-200`
- **What to verify**: `addresses.get(idx).expect("Input indexed address")`,
  `signer.public_key(address).expect(...)`, `signer.sign_schnorr(address, msg).unwrap()`,
  `Message::from_digest_slice(...).unwrap()` — all panic on malformed/mismatched input
  (e.g. more inputs than addresses, missing key coverage).
- **Fix direction**: propagate errors; validate `addresses.len() == inputs.len()` up front.

### 14. Multisig signature ordering in finalize — MEDIUM [NEW]

- **File**: `wallet/core/src/account/pssb.rs:220-245`
  (`finalize_psst_one_or_more_sig_and_redeem_script`)
- **What to verify**: signatures are emitted in `BTreeMap` (pubkey-sorted) order, which
  may not match the pubkey order in the multisig redeem script → invalid scriptSig for
  multisig. Confirm whether this path is only used for single-sig (1-of-1) and whether
  multisig finalization orders signatures correctly elsewhere.
- **Fix direction**: order signatures to match the redeem script's pubkey order (or use
  the script engine to assemble); add a multisig finalize test.

### 15. `minimum_signatures` not validated (>0, ≤ key count) — HIGH [NEW]

- **File**: `wallet/core/src/derivation.rs:400-430` (`create_address`/
  `create_multisig_address`), `wallet/core/src/account/variants/multisig.rs:30-40`
  (`Payload`)
- **What to verify**:
  - `create_address` checks `length < minimum_signatures` but not `minimum_signatures == 0`.
    A 0-of-N multisig redeem script (if `multisig_redeem_script` in `crypto/txscript`
    permits `minimum_signatures == 0`) is spendable by anyone → fund loss. Verify the
    txscript `multisig_redeem_script`/`multisig_redeem_script_ecdsa` behavior for 0.
  - `Payload::minimum_signatures: u16` is deserialized from storage with no validation;
    `sig_op_count` does `u8::try_from(xpub_keys.len()).unwrap()` (panic >255 keys).
- **Fix direction**: reject `minimum_signatures == 0` and `> xpub_keys.len()` at
  construction/deserialization; return error instead of `unwrap()` in `sig_op_count`.

### 16. `decrypt_mnemonic`/`decrypt_xchacha20poly1305` and `Encryptable` Debug/Clone — LOW/MEDIUM [NEW]

- **File**: `wallet/core/src/encryption.rs:20-30` (`Encryptable`), `:90-100` (`Decrypted`),
  `:150-160` (`Encrypted`), `:230-250` (encrypt/decrypt)
- **What to verify**:
  - `Decrypted<T>` derives `Debug` (prints plaintext `T`) and `Clone` (duplicates
    plaintext) and has no `Drop`/`Zeroize`.
  - `Encrypted` derives `Debug` printing `payload.to_hex()` (ciphertext) — lower risk but
    still leaks ciphertext to logs.
  - `argon2_sha256iv_hash` uses `sha256(data)` as the Argon2 salt (deterministic, no
    per-wallet salt) — same password ⇒ same key across wallets; note as a KDF weakness.
- **Fix direction**: redact/remove `Debug` on `Decrypted`/`Encrypted`; add `Zeroize`/`Drop`
  to `Decrypted`; consider a random per-wallet salt stored alongside the ciphertext.

### 17. `Fees::try_from(&str)` silently maps invalid fee to 0 — LOW [NEW]

- **File**: `wallet/core/src/tx/fees.rs:60-70`
- **What to verify**: `try_zyanya_str_to_sompi_i64(fee)?.unwrap_or(0)` — an unparseable
  fee string becomes `Fees::SenderPays(0)` instead of an error, silently disabling fees.
- **Fix direction**: propagate the parse error.

### 18. `ExtendedKey::from_str` prefix/version cross-validation gap — LOW [NEW]

- **File**: `wallet/bip32/src/xkey.rs:70-90` (`from_str`), `wallet/bip32/src/prefix.rs`
- **What to verify**: `from_str` validates the 4-char prefix string is alphabetic but
  stores the decoded `version` via `from_parts_unchecked` without checking they agree.
  `is_private()`/`is_public()` are decided from the prefix **string** only, so the
  version bytes are effectively ignored for the private/public decision. Confirm no
  security impact (a mismatched version is only re-serialized), and note the
  `TryFrom<&str> for Prefix` (known-prefix) path is stricter.
- **Fix direction**: cross-check `Prefix::from_version(version).as_str() == chars` in
  `from_str`, or derive the prefix from the version.

### 19. `DerivationPath::from_str` unbounded length — LOW [NEW]

- **File**: `wallet/bip32/src/derivation_path.rs:120-130`
- **What to verify**: no limit on the number of path components; a huge path string
  allocates unbounded memory (DoS). `ExtendedPrivateKey::derive_child` caps depth at 255
  (`checked_add`), but the `Vec<ChildNumber>` is built before derivation.
- **Fix direction**: cap path length (e.g. ≤ 255) at parse time.

### 20. `ChildNumber::from_bytes`/`From<u32>` skip validation — LOW [NEW]

- **File**: `wallet/bip32/src/child_number.rs:40-60`
- **What to verify**: `ChildNumber::new` validates `index < HARDENED_FLAG`, but
  `from_bytes` (used when parsing xprv/xpub) and `From<u32>` do not. A serialized key
  with `index >= 2^31` and no hardened flag is accepted. Confirm this is benign (the
  hardened bit is the top bit; `index()` masks it) and document.
- **Fix direction**: validate in `from_bytes` or document the invariant.

### 21. `utxo/context.rs` non-debug panics — LOW/MEDIUM [NEW]

- **File**: `wallet/core/src/utxo/context.rs:382` (`unreachable!("promotion of the
  outgoing transaction!")`), `:403` (`panic!("non-stasis utxo revival!")`), plus
  `.expect("outgoing transaction ...")` at `:250/:280`
- **What to verify**: these are non-debug panics on invariant violations. A reorg/event
  sequence that violates the invariant crashes the wallet (DoS). Confirm reachability
  from network events and downgrade to logged errors.
- **Fix direction**: return `Result`/log instead of `panic!`/`unreachable!`.

### 22. `calculate_balance` and mass/fee arithmetic overflow — LOW [NEW]

- **File**: `wallet/core/src/utxo/context.rs:430-460` (`calculate_balance`),
  `wallet/core/src/tx/mass.rs:20-30` (`calc_minimum_required_transaction_relay_fee`),
  `:150-170` (`transaction_serialized_byte_size`)
- **What to verify**: plain `sum()`/`+`/`*` on u64. Bounded by `MAX_SOMPI`/mass limits in
  practice, but confirm no path (e.g. crafted UTXO set, huge payload) can overflow in
  release (silent wrap) and produce a wrong balance/fee/mass.
- **Fix direction**: use `checked_*`/`saturating_*` in balance and size accumulation.

### 23. `sign_with_multiple_v2` panics and multisig gap (cross-ref) — MEDIUM [NEW]

- **File**: `consensus/core/src/sign.rs:126-160` (called by `wallet/core/src/tx/generator/signer.rs`)
- **What to verify**:
  - `Keypair::from_seckey_slice(...).unwrap()` panics on an invalid/zero private key;
    `mutable_tx.entries[i].as_ref().unwrap()` panics on a missing UTXO entry.
  - The map is keyed by the P2PK script (`0x20 || xonly || 0xac`); P2SH/multisig inputs
    are never matched → `Partially` → `fully_signed()` errors. Confirm the wallet routes
    multisig through the PSST path and never through `Signer`.
  - `sign_with_multiple` (v1) keys the map by SEC1 pubkey but looks up by full script —
    never matches (dead/broken code).
- **Fix direction**: return errors instead of `unwrap()`; document/remove v1.

### 24. `create_xpub_from_xprv`/`build_derivate_path` `panic!` arms — LOW [NEW]

- **File**: `wallet/core/src/derivation.rs:520-560`
- **What to verify**: `_ => panic!("create_xpub_from_xprv not supported for account kind: ...")`
  and `_ => panic!("build derivate path not supported ...")`. If `account_kind` is
  attacker-influenced (e.g. from a crafted account descriptor), this is a DoS. Confirm
  `AccountKind` is validated before reaching these functions.
- **Fix direction**: return `Error::AccountKindFeature` instead of panicking.

### 25. `unsafe` blocks inventory — LOW [NEW]

- **Files**: `wallet/core/src/storage/local/mod.rs:41-153` (`static mut` globals),
  `wallet/core/src/compat/gen1.rs:18` (`from_utf8_unchecked`),
  `wallet/core/src/account/mod.rs:856` (`from_utf8_unchecked`),
  `wallet/core/src/wallet/mod.rs:1741` (commented-out `from_utf8_unchecked`)
- **What to verify**: run `grep -rn "unsafe" wallet/` and audit each block. The `static mut`
  storage globals are not thread-safe (documented, but confirm no concurrent mutation);
  the two `from_utf8_unchecked` sites must be proven to only ever receive valid UTF-8.
- **Fix direction**: replace `static mut` with `OnceLock`/`Mutex`; use checked UTF-8
  conversion.

### 26. BIP32/BIP39 correctness spot-checks — verify (no finding unless broken)

- **Files**: `wallet/bip32/src/xprivate_key.rs` (CKD), `wallet/bip32/src/mnemonic/phrase.rs`
  (checksum), `wallet/keys/src/derivation/gen0/hd.rs` + `gen1/hd.rs` (path derivation)
- **What to verify**:
  - `ExtendedPrivateKey::new` accepts seed lengths 16/32/64 and uses the BIP39 domain
    separator `"Bitcoin seed"` (correct for BIP32 master-key derivation).
  - `derive_child` hardened vs non-hardened HMAC input (`0x00 || key` vs `pubkey`) is
    correct; the "no retry on invalid tweak" comment is acceptable (prob < 2^-127).
  - `Mnemonic::new` 12-word checksum: `actual_checksum = entropy[16]` (full byte) vs
    `expected_checksum = sha256[0] & 0b11110000` — confirm the bottom 4 bits of the
    17th byte are always 0 (BitWriter) so the comparison is exact.
  - gen0 legacy uses hardened address indices derived from a pre-seeded HMAC (custom
    Kaspa scheme) — confirm it matches the in-repo test vectors (it does; tests pass).
  - gen1 uses `44'/123456'/account'` (multisig `45'`) with non-hardened cosigner and
    address-type children — confirm matches Kaspa upstream.

---

## Execution steps for the builder

1. `audit_reports/` already exists. Write findings to `audit_reports/phase3_findings.md`.
2. Work through the files in the priority order above. For each file:
   - Read the full file (use `offset`/`limit` for large files — `generator.rs` is 1152
     lines, `utxo/context.rs` 731, `account/mod.rs` 896, `wasm/api/message.rs` 1775).
   - Verify every finding's `file:line` against the current tree (line numbers above are
     from the audit snapshot and may drift — re-grep to confirm).
   - Reproduce the trigger mentally or with a minimal test where feasible.
   - Record the finding in the report format (severity, file:line, description, snippet, fix).
3. For the Zyanya-vs-upstream question: for each wallet-critical file, sanity-check
   against known upstream Kaspa/rusty-spectre behavior (the in-repo tests and the
   `kprv`/`ktrv` test vectors in `mnemonic/phrase.rs` and `derivation/*/hd.rs` are the
   reference). The `zyanya-*` account-kind strings and `kprv`/`kpub`/`ktrv`/`ktub`
   prefixes are the main Zyanya deltas.
4. Run `grep -rn "unsafe" wallet/`, `grep -rn "unwrap()\|expect(\|panic!\|unreachable!" wallet/`
   and `grep -rn "derive(.*Debug" wallet/` to catch additional instances of the
   categories above (key leakage, panics, unsafe).
5. Add any additional findings discovered in the listed categories (integer overflow,
   key leakage/side channels, BIP32 derivation, mnemonic handling, signing correctness,
   multisig, PSBT injection, UTXO selection/dust, address generation, key wiping,
   race conditions).
6. Sort the final report by severity (CRITICAL → HIGH → MEDIUM → LOW), then by file path.
7. Do **not** modify source code — this phase is audit + report only.

## Report skeleton

```markdown
# Phase 3 Findings — Wallet Core

## CRITICAL
### [C-01] <title>
- **File**: <path>:<line>
- **Description**: ...
- **Code**:
  ```rust
  ...
  ```
- **Recommended fix**: ...

## HIGH
### [H-01] ...

## MEDIUM
### [M-01] ...

## LOW
### [L-01] ...
```

Use stable IDs (`C-01`, `H-01`, ...) so later phases can reference them. Phase 3 IDs are
independent of Phase 1/2.
