# Phase 3 Findings — Wallet Core

Security audit of the Zyanya wallet core library (~28,000 lines across
`wallet/core`, `wallet/keys`, `wallet/bip32`, `wallet/psst`, `wallet/macros`,
`wallet/wasm`, `wallet/native`).

This codebase is upstream Kaspa/rusty-spectre wallet code renamed to
`zyanya-*` crates. Zyanya-specific modifications are limited to:
crate renames, address prefixes (`zyanya:`/`zyanyatest:`), BIP32 extended-key
prefixes (`kprv`/`kpub`/`ktrv`/`ktub`), and account-kind string constants
(`zyanya-bip32-standard`, `zyanya-multisig-standard`, etc.). All findings
below apply to the code as it exists in the Zyanya tree.

Findings are ordered by severity (CRITICAL → HIGH → MEDIUM → LOW), then
by file path.

---

## CRITICAL

### [C-01] `minimum_signatures == 0` produces an anyone-can-spend multisig address

- **File**: `wallet/core/src/derivation.rs:400` (`create_address`), `:428`
  (`create_multisig_address`); `wallet/core/src/account/variants/multisig.rs:117`
  (`Payload` deserialization)
- **Description**: `create_address` validates only `length < minimum_signatures`
  — it never rejects `minimum_signatures == 0`. When `minimum_signatures` is 0
  and `keys.len() > 1`, the function calls `create_multisig_address(0, keys, ...)`,
  which calls `multisig_redeem_script(..., 0)`. The txscript
  `multisig_redeem_script` function (`crypto/txscript/src/standard/multisig.rs:18`)
  does **not** reject `required == 0` (it only rejects `required > count` and
  `count == 0`). The resulting redeem script is `OP_0 <pubkeys...> OP_N
  OP_CHECKMULTISIG` — a 0-of-N multisig that is spendable by anyone who supplies
  an empty signature vector. `Payload::minimum_signatures: u16` is deserialized
  from wallet storage with no validation, so a crafted or corrupted wallet file
  can set it to 0. Any funds sent to such an address are immediately stealable.
- **Code**:
  ```rust
  // wallet/core/src/derivation.rs:400
  pub fn create_address(minimum_signatures: usize, keys: Vec<secp256k1::PublicKey>, ...) -> Result<Address> {
      let length = keys.len();
      if length < minimum_signatures {   // does NOT check minimum_signatures == 0
          return Err(...);
      }
      if length > 1 {
          return create_multisig_address(minimum_signatures, keys, prefix, ecdsa);
      }
      ...
  }

  // crypto/txscript/src/standard/multisig.rs:18
  pub fn multisig_redeem_script(pub_keys: ..., required: usize) -> Result<Vec<u8>, Error> {
      // no check for required == 0
      builder.add_i64(required as i64)?;
      ...
  }
  ```
- **Recommended fix**: Reject `minimum_signatures == 0` in `create_address`,
  `AddressManager::new`, and `Payload` deserialization (add a
  `try_version`/magic-style guard or a `BorshDeserialize` wrapper that returns
  `Err` when `minimum_signatures == 0 || minimum_signatures > xpub_keys.len()`).
  Also add the guard to `multisig_redeem_script` in `crypto/txscript`.

---

## HIGH

### [H-01] Private key material leaked via derived `Debug` impls on `PrvKeyDataVariant` and `PrvKeyDataPayload`

- **File**: `wallet/core/src/storage/keydata/data.rs:20` (`PrvKeyDataVariant`),
  `:117` (`PrvKeyDataPayload`), `:191` (`PrvKeyData`)
- **Description**: `PrvKeyDataVariant` is `#[derive(Clone, Debug, Serialize,
  Deserialize)]` and its variants hold the mnemonic phrase, BIP39 seed hex,
  xprv string, or secret key hex as a plain `String`. `{:?}` of
  `PrvKeyDataVariant` (or of `PrvKeyDataPayload`/`PrvKeyData` which transitively
  contain it) prints the secret in full. Any `log_*!`/`dbg!`/`println!` of these
  types — or of `Encryptable::Plain(PrvKeyDataPayload)` before encryption —
  leaks the key to logs. `PrvKeyDataPayload` and `PrvKeyData` also derive
  `Debug` and `Clone`.
- **Code**:
  ```rust
  // wallet/core/src/storage/keydata/data.rs:20
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub enum PrvKeyDataVariant {
      Mnemonic(String),      // <-- Debug prints the mnemonic phrase
      Bip39Seed(String),     // <-- Debug prints the seed hex
      ExtendedPrivateKey(String),
      SecretKey(String),
  }

  // data.rs:117
  #[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
  pub struct PrvKeyDataPayload { prv_key_variant: PrvKeyDataVariant }
  ```
- **Recommended fix**: Replace the derived `Debug` with a manual redacting impl:
  ```rust
  impl std::fmt::Debug for PrvKeyDataVariant {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          f.debug_tuple("PrvKeyDataVariant").field(&"********").finish()
      }
  }
  ```
  Do the same for `PrvKeyDataPayload` and `PrvKeyData`. Consider removing `Debug`
  entirely. Add a CI lint (`#![deny(clippy::dbg_macro)]`) and grep for `{:?}` on
  these types.

### [H-02] Wallet file written with default (world-readable) permissions

- **File**: `wallet/core/src/storage/local/wallet.rs:69`
  (`WalletStorage::try_store`)
- **Description**: The native (non-WASM) storage path uses
  `std::fs::File::create(store.filename())` with no explicit file mode.
  `File::create` creates the file with mode `0o666 & !umask` → `0o644` on the
  default umask 022, meaning the wallet file — which contains the encrypted
  mnemonic/private keys — is world-readable. An attacker with local filesystem
  access can copy the encrypted blob and brute-force the payment secret offline.
- **Code**:
  ```rust
  // wallet/core/src/storage/local/wallet.rs:69
  let mut file = std::fs::File::create(store.filename(), )?;
  BorshSerialize::serialize(self, &mut file)?;
  ```
- **Recommended fix**: On Unix, create the file with restrictive permissions:
  ```rust
  use std::os::unix::fs::OpenOptionsExt;
  std::fs::OpenOptions::new()
      .write(true).create(true).truncate(true)
      .mode(0o600)
      .open(store.filename())?;
  ```
  Or call `file.set_permissions(Permissions::from_mode(0o600))` immediately after
  create. Apply the same to transaction-record writes in `fsio.rs`.

### [H-03] Double addition of `change_output_value` to `transaction_fees`

- **File**: `wallet/core/src/tx/generator/generator.rs:797` and `:809`
- **Description**: In `try_finish_standard_stage_processing`, the
  `absorb_change_to_fees || change_output_value == 0` branch executes
  `transaction_fees += change_output_value;` **twice** — once at line 797 and
  again at line 809 after the mass recompute. For `Fees::SenderPays`, the
  change is absorbed into the fee, so the double-count over-reports
  `transaction_fees` by `change_output_value`, producing a wrong fee in the
  `PendingTransaction`. For `Fees::ReceiverPays` (currently marked "unreachable
  at the API level" but still a latent bug), `generate_transaction` reduces the
  receiver's output by `transaction_fees`, so the receiver is **underpaid by
  `change_output_value`** — a fund-loss bug.
- **Code**:
  ```rust
  // wallet/core/src/tx/generator/generator.rs:795-810
  if absorb_change_to_fees || change_output_value == 0 {
      transaction_fees += change_output_value;       // line 797 — first add

      let compute_mass = ...;
      let storage_mass = self.calc_storage_mass(...);
      data.aggregate_mass = calc.combine_mass(compute_mass, storage_mass);

      transaction_fees += change_output_value;       // line 809 — DUPLICATE
      data.transaction_fees = transaction_fees;
      ...
  }
  ```
- **Recommended fix**: Remove the duplicate `transaction_fees +=
  change_output_value;` at line 809 (keep exactly one). Add a unit test
  asserting `transaction_fees == inputs - outputs` for the absorb-change case.

### [H-04] `ExtendedPrivateKey` derives `Clone` but has no `Drop`/`Zeroize`

- **File**: `wallet/bip32/src/xprivate_key.rs:19`
- **Description**: `ExtendedPrivateKey<K>` is `#[derive(Clone)]` and wraps a
  `secp256k1::SecretKey` (which does **not** zeroize on drop by default). There
  is no `Drop` or `Zeroize` impl on `ExtendedPrivateKey`, so when the struct is
  dropped the private key bytes remain in memory. The `Clone` derive also
  creates transient copies that are never wiped. While the `Debug` impl is
  manual and redacts (`field("private_key", &"...")`), and `to_string()`
  returns a `Zeroizing<String>`, the core key material in the struct itself is
  not zeroized on drop.
- **Code**:
  ```rust
  // wallet/bip32/src/xprivate_key.rs:19
  #[derive(Clone)]
  pub struct ExtendedPrivateKey<K: PrivateKey> {
      private_key: K,   // secp256k1::SecretKey — not zeroized on drop
      attrs: ExtendedKeyAttrs,
  }
  // no impl Drop, no impl Zeroize
  ```
- **Recommended fix**: Implement `Drop` (or `Zeroize`) on `ExtendedPrivateKey`
  that calls `self.private_key.to_bytes()` and zeroizes the result. Consider
  wrapping `private_key` in `Zeroizing<K>`. Remove `Clone` where possible or
  implement a manual `Clone` that is explicit about the copy.

### [H-05] `Decrypted<T>` derives `Debug` and `Clone` with no `Drop`/`Zeroize`

- **File**: `wallet/core/src/encryption.rs:98`
- **Description**: `Decrypted<T>` is `#[derive(Clone, Debug, BorshSerialize,
  BorshDeserialize)]` and holds the plaintext `T`. When `T` is
  `PrvKeyDataPayload` (which itself leaks via `Debug` — see H-01),
  `{:?}` on `Decrypted<PrvKeyDataPayload>` prints the full mnemonic/seed/xprv.
  `Clone` duplicates the plaintext without zeroizing the transient copy. There
  is no `Drop`/`Zeroize`, so decrypted plaintext persists in memory after the
  `Decrypted` guard is dropped.
- **Code**:
  ```rust
  // wallet/core/src/encryption.rs:98
  #[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
  pub struct Decrypted<T>(pub(crate) T)
  where T: BorshSerialize + BorshDeserialize;
  ```
- **Recommended fix**: Remove `Debug` or replace with a redacting impl. Add
  `Drop`/`Zeroize` bound (`impl<T: Zeroize> Drop for Decrypted<T>`). Remove
  `Clone` or make it explicit.

---

## MEDIUM

### [M-01] `decrypt_xchacha20poly1305` panics on ciphertext shorter than 24 bytes

- **File**: `wallet/core/src/encryption.rs:248`
- **Description**: `let nonce = &data[0..24];` panics with an index-out-of-bounds
  if the stored encrypted payload is truncated or corrupted to fewer than 24
  bytes. This is reachable from any stored `Encrypted` payload (wallet file,
  transaction records, keydata). Instead of returning a clean error, the wallet
  crashes (DoS).
- **Code**:
  ```rust
  // wallet/core/src/encryption.rs:246-250
  pub fn decrypt_xchacha20poly1305(data: &[u8], secret: &Secret) -> Result<Secret> {
      ...
      let nonce = &data[0..24];   // panics if data.len() < 24
      let mut buffer = data[24..].to_vec();
      ...
  }
  ```
- **Recommended fix**: Check `data.len() >= 24 + 16` (nonce + minimum AEAD tag)
  and return `Err(...)` before slicing:
  ```rust
  if data.len() < 24 + 16 { return Err("ciphertext too short".into()); }
  ```

### [M-02] `decrypt_mnemonic`: panics and UB on malformed Go-wallet import files

- **File**: `wallet/core/src/compat/gen1.rs:9` (`p_cost(num_threads).build().unwrap()`),
  `:15` (`split_at(24)` panic), `:18` (`from_utf8_unchecked` UB)
- **Description**: Three issues in the Go-wallet import path:
  1. `cipher.as_ref().split_at(24)` panics if `cipher.len() < 24`.
  2. `num_threads: u32` comes from the imported wallet file;
     `ParamsBuilder::new()...p_cost(num_threads).build().unwrap()` panics if
     `num_threads == 0` or is too large (DoS). The `.unwrap()` never returns an
     error.
  3. `String::from_utf8_unchecked(decrypted)` is undefined behavior if the
     AEAD-decrypted plaintext is not valid UTF-8 (only "safe" because AEAD
     authenticates with the correct password, but still fragile).
  Additionally, `num_threads.expect("num_threads must present in case of v0")`
  at `:181`/`:192` panics on a v0 file missing the `numThreads` field.
- **Code**:
  ```rust
  // wallet/core/src/compat/gen1.rs:9-18
  let params = argon2::ParamsBuilder::new().t_cost(1).m_cost(64*1024)
      .p_cost(num_threads).output_len(32).build().unwrap();   // panic if num_threads==0
  ...
  let (nonce, ciphertext) = cipher.as_ref().split_at(24);     // panic if <24
  let decrypted = aead.decrypt(nonce.into(), ciphertext)?;
  Ok(unsafe { String::from_utf8_unchecked(decrypted) })       // UB if not valid UTF-8
  ```
- **Recommended fix**: Validate `cipher.len() >= 24`; clamp `num_threads` to a
  sane range (e.g. `1..=64`); use `String::from_utf8(decrypted)?` instead of
  `from_utf8_unchecked`; replace `.unwrap()`/`.expect()` with `?` error
  propagation.

### [M-03] Lock-order inversion deadlock in `PubkeyDerivationManagerV0`

- **File**: `wallet/keys/src/derivation/gen0/hd.rs:124` (`derive_pubkey_range`)
  and `:162` (`derive_pubkey`)
- **Description**: `derive_pubkey_range` locks `cache` (line 124) then locks
  `inner` via `self.opt_inner()` (line 125). `derive_pubkey` locks `inner` via
  `self.opt_inner()` (line 162) then locks `cache` (line 172/175). Concurrent
  calls from two threads on the same `Arc<PubkeyDerivationManagerV0>` create a
  classic ABBA deadlock.
- **Code**:
  ```rust
  // derive_pubkey_range — locks cache THEN inner
  let mut cache = self.cache.lock()?;      // line 124 — cache first
  let locked = self.opt_inner();           // line 125 — inner second

  // derive_pubkey — locks inner THEN cache
  let locked = self.opt_inner();           // line 162 — inner first
  ...
  self.cache.lock()?.insert(index, key);   // line 172 — cache second
  ```
- **Recommended fix**: Establish a single lock order (always `inner` before
  `cache`). In `derive_pubkey_range`, drop the `cache` guard before calling
  `opt_inner()`, or clone the needed data under the cache lock first. Better:
  use a single combined mutex or `tokio::sync::Mutex` with `try_lock` fallback.

### [M-04] Signer key caches never zeroized on drop

- **File**: `wallet/core/src/tx/generator/signer.rs:17` (`Inner.keys`),
  `:62` (`KeydataSignerInner.keys`), `wallet/core/src/account/pssb.rs:31`
  (`PSSBSignerInner.keys`)
- **Description**: All three signer caches are `Mutex<AHashMap<Address,
  [u8;32]>>` or `HashMap<Address, [u8;32]>` holding raw 32-byte private keys.
  After signing, `keys_for_signing.zeroize()` wipes the temporary `Vec`, but
  the cached map entries persist in memory until the signer is dropped — and
  even then they are not wiped (no `Drop`/`Zeroize` on the `Inner` structs).
- **Code**:
  ```rust
  // wallet/core/src/tx/generator/signer.rs:17
  struct Inner {
      keydata: PrvKeyData,
      account: Arc<dyn Account>,
      payment_secret: Option<Secret>,
      keys: Mutex<AHashMap<Address, [u8; 32]>>,  // never zeroized
  }
  ```
- **Recommended fix**: Implement `Drop` on `Inner`/`KeydataSignerInner`/
  `PSSBSignerInner` that iterates and zeroizes each `[u8;32]` value. Or use
  `Zeroizing<[u8; 32]>` as the map value type.

### [M-05] Division-by-zero panics in mass calculation

- **File**: `wallet/core/src/tx/mass.rs:346`
  (`calc_storage_mass_output_harmonic_single`) and `:350`
  (`calc_storage_mass_input_mean_arithmetic`)
- **Description**: `calc_storage_mass_output_harmonic_single(0)` panics
  (integer division by zero). It is called from `generate_edge_transaction` with
  `data.aggregate_input_value.saturating_sub(compute_fees)`, which can be 0 when
  the aggregated input value ≤ edge compute fees (tiny-UTXO wallet).
  `calc_storage_mass_input_mean_arithmetic` divides by `number_of_inputs`; if
  called with 0 inputs it panics.
- **Code**:
  ```rust
  // wallet/core/src/tx/mass.rs:346
  pub fn calc_storage_mass_output_harmonic_single(&self, output_value: u64) -> u64 {
      self.storage_mass_parameter / output_value   // panic if output_value == 0
  }
  // :350
  pub fn calc_storage_mass_input_mean_arithmetic(&self, total_input_value: u64, number_of_inputs: u64) -> u64 {
      let mean_input_value = total_input_value / number_of_inputs;  // panic if 0
      ...
  }
  ```
- **Recommended fix**: Use `checked_div` and return `0` or propagate an
  `Error::MassCalculationError` on zero. Guard `number_of_inputs == 0` at all
  call sites.

### [M-06] `Mnemonic` WASM surface: panics on invalid input and inconsistent state

- **File**: `wallet/bip32/src/mnemonic/phrase.rs:79` (`set_entropy` panic),
  `:99` (`set_phrase` no re-validation), `:91` (`phrase_string` non-zeroized)
- **Description**: `set_entropy` panics on invalid hex or wrong length
  (`panic!("invalid entropy ...")`, `panic!("Invalid entropy ...")`) — a JS
  caller can trigger a WASM panic (DoS). `set_phrase` sets `self.phrase` without
  re-validating or recomputing entropy, leaving the `Mnemonic` in an
  inconsistent state (phrase ≠ entropy); a subsequent `to_seed` derives from the
  arbitrary phrase. `phrase_string()` returns a plain `String` (clone of the
  mnemonic) that is not zeroized.
- **Code**:
  ```rust
  // wallet/bip32/src/mnemonic/phrase.rs:77-80
  #[wasm_bindgen(setter, js_name = entropy)]
  pub fn set_entropy(&mut self, entropy: String) {
      let vec = Vec::<u8>::from_hex(&entropy).unwrap_or_else(|err| panic!("invalid entropy `{entropy}`: {err}"));
      ...
  }
  // :97-99
  #[wasm_bindgen(setter, js_name = phrase)]
  pub fn set_phrase(&mut self, phrase: &str) {
      self.phrase = phrase.to_string();   // no validation, no entropy recompute
  }
  ```
- **Recommended fix**: Return `Result` from setters instead of panicking.
  Re-validate on `set_phrase` (or recompute entropy). Return
  `Zeroizing<String>` where possible from `phrase_string()` (or document the JS
  copy risk).

### [M-07] PSBT/PSSB deserialization: no size limits and silent partial-sig overwrite

- **File**: `wallet/psst/src/psst.rs:151` (`from_hex` → `serde_json::from_slice`
  with no bounds), `wallet/psst/src/input.rs:108` (`partial_sigs.extend`)
- **Description**: `PSST::from_hex` and `Bundle::deserialize` call
  `serde_json::from_slice` on untrusted hex with no bounds on the number of
  inputs/outputs/unknowns/proprietaries — a multi-GB JSON payload causes memory
  exhaustion (DoS). `Input::add` does `self.partial_sigs.extend(rhs.
  partial_sigs)` — a conflicting signature for the same pubkey silently
  overwrites (BTreeMap `insert`) instead of erroring. BIP-174 requires conflict
  detection on partial signature merge.
- **Code**:
  ```rust
  // wallet/psst/src/input.rs:108
  self.partial_sigs.extend(rhs.partial_sigs);  // silent overwrite of conflicting sigs

  // wallet/psst/src/psst.rs:151
  Ok(serde_json::from_slice(hex::decode(hex_data)?.as_slice())?)  // no size limit
  ```
- **Recommended fix**: Add explicit limits (max inputs/outputs, max JSON byte
  size) before deserialization. Detect conflicting partial sigs by checking
  for existing keys before `extend` and returning a `CombineError` on conflict.

### [M-08] `pssb_signer_for_address` panics on malformed input

- **File**: `wallet/core/src/account/pssb.rs:168` (`addresses.get(idx).expect`),
  `:170` (`signer.public_key(...).expect`), `:171`
  (`signer.sign_schnorr(...).unwrap()`), `:163`
  (`Message::from_digest_slice(...).unwrap()`)
- **Description**: The signing closure uses `.expect()` and `.unwrap()` on
  operations that can fail with malformed/mismatched input (more inputs than
  addresses, missing key coverage, invalid digest). These panics are
  unreachable from normal flow but trigger on a crafted PSST bundle, causing a
  wallet crash (DoS).
- **Code**:
  ```rust
  // wallet/core/src/account/pssb.rs:163-171
  let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice()).unwrap();
  let address: &Address = match sign_for_address {
      Some(address) => address,
      None => addresses.get(idx).expect("Input indexed address"),
  };
  let public_key = signer.public_key(address).expect("Public key for input indexed address");
  Ok(SignInputOk { signature: Signature::Schnorr(signer.sign_schnorr(address, msg).unwrap()), ... })
  ```
- **Recommended fix**: Propagate errors instead of `.unwrap()`/`.expect()`.
  Validate `addresses.len() == inputs.len()` up front. Change the closure return
  type to `Result<Vec<SignInputOk>, Error>`.

### [M-09] `psst_to_pending_transaction` uses hardcoded mass and fee, panics on empty outputs

- **File**: `wallet/core/src/account/pssb.rs:291` (`let mass = 10;`),
  `:324` (`let fee_u: u64 = 0;`), `:323`/`:329` (`output[0]` indexing),
  `:339` (`.unwrap()`)
- **Description**: `mass = 10` and `fee_u = 0` are hardcoded placeholders — the
  reconstructed `PendingTransaction` has wrong mass/fee, breaking fee estimation
  and relay acceptance. `output[0]` indexing panics on an empty-output PSST.
  `extract_script_pub_key_address(...).unwrap()` panics on an unparseable script
  public key.
- **Code**:
  ```rust
  // wallet/core/src/account/pssb.rs:291
  let mass = 10;
  // :323
  let recipient = extract_script_pub_key_address(&output[0].script_public_key, network_id.into())?;
  // :324
  let fee_u: u64 = 0;
  ```
- **Recommended fix**: Compute mass via `MassCalculator` and fee from the
  extracted transaction. Return errors instead of indexing/`unwrap()`: check
  `output.is_empty()` first, use `output.get(0).ok_or(...)`.

### [M-10] Multisig signature ordering may not match redeem script pubkey order

- **File**: `wallet/core/src/account/pssb.rs:237-245`
  (`finalize_psst_one_or_more_sig_and_redeem_script`)
- **Description**: Signatures are emitted in `BTreeMap` (pubkey-sorted) order,
  which may not match the pubkey order in the multisig redeem script. For
  multisig inputs, `OP_CHECKMULTISIG` evaluates signatures in the order they
  appear on the stack, which must correspond to the pubkey order in the redeem
  script. Mismatched ordering produces an invalid scriptSig. This path appears
  to be used for both single-sig and multisig finalization.
- **Code**:
  ```rust
  // wallet/core/src/account/pssb.rs:237-242
  let signatures: Vec<_> = input
      .partial_sigs.clone()
      .into_iter()   // BTreeMap iteration — pubkey-sorted, NOT redeem-script-ordered
      .flat_map(|(_, signature)| iter::once(OpData65).chain(signature.into_bytes()).chain([input.sighash_type.to_u8()]))
      .collect();
  ```
- **Recommended fix**: Order signatures to match the redeem script's pubkey
  order (parse the redeem script, extract pubkeys, then sort `partial_sigs` by
  their position in the redeem script). Add a multisig finalize test.

### [M-11] `sign_with_multiple_v2` panics on invalid key or missing UTXO entry

- **File**: `consensus/core/src/sign.rs:129` (`Keypair::from_seckey_slice(...).
  unwrap()`), `:138` (`mutable_tx.entries[i].as_ref().unwrap()`)
- **Description**: `from_seckey_slice(...).unwrap()` panics on an invalid or
  zero private key. `mutable_tx.entries[i].as_ref().unwrap()` panics on a
  missing UTXO entry. The map is keyed by the P2PK script
  (`0x20 || xonly || 0xac`); P2SH/multisig inputs are never matched, resulting
  in `Partially` → `fully_signed()` error. The v1 function
  `sign_with_multiple` (line 99) keys the map by SEC1 pubkey but looks up by
  full script — it never matches (dead/broken code).
- **Code**:
  ```rust
  // consensus/core/src/sign.rs:129
  let schnorr_key = secp256k1::Keypair::from_seckey_slice(secp256k1::SECP256K1, privkey).unwrap();
  // :138
  let script = mutable_tx.entries[i].as_ref().unwrap().script_public_key.script();
  ```
- **Recommended fix**: Return errors instead of `.unwrap()`. Document/remove v1
  `sign_with_multiple`. Route multisig inputs through the PSST path exclusively.

### [M-12] `Secret` derives `Clone` and serializable traits — secret duplication

- **File**: `wallet/keys/src/secret.rs:8`
- **Description**: `Secret` is `#[derive(Clone, Serialize, Deserialize,
  BorshSerialize, BorshDeserialize)]`. Every `clone()` duplicates the secret
  bytes; only the clone's own `Drop` zeroizes its copy, and the transient copy
  during `clone()` is not wiped. `Serialize`/`Deserialize`/`Borsh*` allow the
  secret to be serialized to plaintext (e.g. JSON, borsh) without any guard.
  The `Debug` impl is correctly redacting (manual impl at line 47).
- **Code**:
  ```rust
  // wallet/keys/src/secret.rs:8
  #[derive(Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
  pub struct Secret(Vec<u8>);
  ```
- **Recommended fix**: Remove `Clone` from `Secret` where possible, or
  implement a manual `Clone` that is explicit about the copy. Remove
  `Serialize`/`Deserialize`/`Borsh*` or gate them behind a feature flag to
  prevent accidental serialization of secret material.

### [M-13] `argon2_sha256iv_hash` uses `sha256(data)` as salt — deterministic KDF

- **File**: `wallet/core/src/encryption.rs:224`
- **Description**: `argon2_sha256iv_hash` computes `salt = sha256_hash(data)`
  where `data` is the password/secret itself. This makes the Argon2 salt
  deterministic — the same password always produces the same Argon2 salt and
  therefore the same encryption key across all wallets. An attacker who knows
  the password can precompute the key. There is no per-wallet random salt.
- **Code**:
  ```rust
  // wallet/core/src/encryption.rs:223-228
  pub fn argon2_sha256iv_hash(data: &[u8], byte_length: usize) -> Result<Secret> {
      let salt = sha256_hash(data);   // deterministic — same password → same salt
      let mut key = vec![0u8; byte_length];
      Argon2::default().hash_password_into(data, salt.as_ref(), &mut key)?;
      Ok(key.into())
  }
  ```
- **Recommended fix**: Generate a random per-wallet salt (16+ bytes) and store
  it alongside the ciphertext. Use the password + salt as Argon2 inputs.

---

## LOW

### [L-01] `Fees::try_from(&str)` silently maps invalid fee string to 0

- **File**: `wallet/core/src/tx/fees.rs:85`
- **Description**: `try_zyanya_str_to_sompi_i64(fee)?.unwrap_or(0)` — if the
  parse function returns `Ok(None)` (empty string → `Fees::None`, which is
  correct) but if `try_zyanya_str_to_sompi_i64` returns `Ok(Some(0))` for a
  non-numeric input that `parse::<f64>()` fails on, the `?` propagates the
  error. However, the `.unwrap_or(0)` means if the inner function returns
  `Ok(None)` for a non-empty but whitespace-only string, the fee silently
  becomes `Fees::SenderPays(0)`, disabling fees.
- **Code**:
  ```rust
  // wallet/core/src/tx/fees.rs:83-86
  impl TryFrom<&str> for Fees {
      fn try_from(fee: &str) -> Result<Self> {
          if fee.is_empty() {
              Ok(Fees::None)
          } else {
              let fee = crate::utils::try_zyanya_str_to_sompi_i64(fee)?.unwrap_or(0);
              Ok(Fees::from(fee))
          }
      }
  }
  ```
- **Recommended fix**: Propagate the `Option<i64>` explicitly — if `None` is
  returned for a non-empty string, return an error rather than defaulting to 0.

### [L-02] `DerivationPath::from_str` has no length limit

- **File**: `wallet/bip32/src/derivation_path.rs:117`
- **Description**: `DerivationPath::from_str` parses an unbounded number of path
  components into a `Vec<ChildNumber>`. A huge path string (e.g. millions of
  components) allocates unbounded memory (DoS). `ExtendedPrivateKey::derive_child`
  caps depth at 255 via `checked_add`, but the `Vec` is built before derivation
  begins.
- **Code**:
  ```rust
  // wallet/bip32/src/derivation_path.rs:115-121
  fn from_str(path: &str) -> Result<DerivationPath> {
      let mut path = path.split('/');
      if path.next() != Some(PREFIX) { return Err(...); }
      Ok(DerivationPath { path: path.map(str::parse).collect::<Result<_>>()? })
  }
  ```
- **Recommended fix**: Cap path length (e.g. ≤ 255 components) at parse time:
  ```rust
  let parts: Vec<_> = path.collect();
  if parts.len() > 255 { return Err(Error::Depth); }
  ```

### [L-03] `ChildNumber::from_bytes` and `From<u32>` skip validation

- **File**: `wallet/bip32/src/child_number.rs:40` (`from_bytes`), `:55`
  (`From<u32>`)
- **Description**: `ChildNumber::new` validates `index < HARDENED_FLAG`, but
  `from_bytes` (used when parsing xprv/xpub serialized keys) and `From<u32>` do
  not. A serialized key with `index >= 2^31` and no hardened flag is accepted.
  This is technically benign (the hardened bit is the top bit; `index()` masks
  it), but it violates the invariant that `ChildNumber::new` enforces.
- **Code**:
  ```rust
  // wallet/bip32/src/child_number.rs:40-42
  pub fn from_bytes(bytes: [u8; Self::BYTE_SIZE]) -> Self {
      u32::from_be_bytes(bytes).into()   // no validation
  }
  // :55-57
  impl From<u32> for ChildNumber {
      fn from(n: u32) -> ChildNumber { ChildNumber(n) }   // no validation
  }
  ```
- **Recommended fix**: Validate in `from_bytes` or document the invariant that
  callers must ensure the hardened bit is set correctly.

### [L-04] `ExtendedKey::from_str` does not cross-validate prefix string vs version bytes

- **File**: `wallet/bip32/src/xkey.rs:74`
- **Description**: `from_str` validates the 4-char prefix string is alphabetic
  but constructs the `Prefix` via `from_parts_unchecked(chars, version)` without
  checking they agree. `is_private()`/`is_public()` are decided from the prefix
  **string** only, so the version bytes are effectively ignored for the
  private/public decision. A key with prefix `xpub` but a private-key version
  would be treated as public. The `TryFrom<&str> for Prefix` path (known-prefix
  lookup) is stricter.
- **Code**:
  ```rust
  // wallet/bip32/src/xkey.rs:74-80
  let prefix = base58.get(..4).ok_or(Error::DecodeIssue).and_then(|chars| {
      Prefix::validate_str(chars)?;
      let b: [u8; 4] = bytes[..4].try_into()?;
      let version = Version::from_be_bytes(b);
      Ok(Prefix::from_parts_unchecked(chars, version))   // no cross-check
  })?;
  ```
- **Recommended fix**: Cross-check that `Prefix::from_version(version).as_str()
  == chars` in `from_str`, or derive the prefix from the version bytes.

### [L-05] `static mut` storage globals are not thread-safe

- **File**: `wallet/core/src/storage/local/mod.rs:32-34`
- **Description**: `DEFAULT_STORAGE_FOLDER`, `DEFAULT_WALLET_FILE`, and
  `DEFAULT_SETTINGS_FILE` are `static mut Option<String>`. The `get_or_insert`
  accessors and setters use `unsafe` blocks that are not thread-safe. Concurrent
  calls to `set_default_storage_folder` and `default_storage_folder` from
  multiple threads can cause a data race. The code comments acknowledge this
  ("not thread-safe") but the functions are `pub` and exported via
  `#[wasm_bindgen]`.
- **Code**:
  ```rust
  // wallet/core/src/storage/local/mod.rs:32-34
  static mut DEFAULT_STORAGE_FOLDER: Option<String> = None;
  static mut DEFAULT_WALLET_FILE: Option<String> = None;
  static mut DEFAULT_SETTINGS_FILE: Option<String> = None;
  ```
- **Recommended fix**: Replace `static mut` with `OnceLock<String>` (for
  one-time init) or `Mutex<String>` for mutable access.

### [L-06] `utxo/context.rs` uses `panic!`/`unreachable!` on invariant violations

- **File**: `wallet/core/src/utxo/context.rs:382` (`unreachable!("promotion of
  the outgoing transaction!")`), `:403` (`panic!("non-stasis utxo revival!")`)
- **Description**: These are non-debug panics on invariant violations. A reorg
  or event sequence that violates the invariant crashes the wallet (DoS). The
  `unreachable!` at line 382 is reached when `outgoing.get(&txid).is_some()`
  during promotion — a reorg can cause this.
- **Code**:
  ```rust
  // wallet/core/src/utxo/context.rs:382
  unreachable!("Error: promotion of the outgoing transaction!");
  // :403
  panic!("Error: non-stasis utxo revival!");
  ```
- **Recommended fix**: Return `Result` and log the error instead of
  `panic!`/`unreachable!`.

### [L-07] `calculate_balance` uses plain `sum()` — potential u64 overflow

- **File**: `wallet/core/src/utxo/context.rs:480-481`
- **Description**: `context.mature.iter().map(|e| e.as_ref().amount).sum()`
  and `context.pending.values().map(|e| e.as_ref().amount).sum()` use plain
  `sum()` on `u64`. Bounded by `MAX_SOMPI` in practice, but a crafted UTXO set
  with amounts summing beyond `u64::MAX` would silently wrap.
- **Code**:
  ```rust
  // wallet/core/src/utxo/context.rs:480-481
  let mature: u64 = context.mature.iter().map(|e| e.as_ref().amount).sum();
  let pending: u64 = context.pending.values().map(|e| e.as_ref().amount).sum();
  ```
- **Recommended fix**: Use `checked_sum()` or `try_fold` with `checked_add` and
  propagate an overflow error.

### [L-08] `create_xpub_from_xprv`/`build_derivate_path` `panic!` on unsupported account kind

- **File**: `wallet/core/src/derivation.rs:540` and `:561`
- **Description**: These functions have `_ => panic!(...)` arms for unsupported
  account kinds. If `account_kind` is attacker-influenced (e.g. from a crafted
  account descriptor in a wallet file), this is a DoS. `AccountKind` should be
  validated before reaching these functions, but the panic is a defense-in-depth
  failure.
- **Code**:
  ```rust
  // wallet/core/src/derivation.rs:540
  _ => panic!("create_xpub_from_xprv not supported for account kind: {:?}", account_kind),
  // :561
  panic!("build derivate path not supported for account kind: {:?}", account_kind);
  ```
- **Recommended fix**: Return `Error::AccountKindFeature` instead of panicking.

### [L-09] `sig_op_count` panics on >255 xpub keys

- **File**: `wallet/core/src/account/variants/multisig.rs:195`
- **Description**: `u8::try_from(self.xpub_keys.len()).unwrap()` panics if the
  multisig has more than 255 xpub keys. While unlikely in practice, a crafted
  wallet file with >255 keys causes a crash.
- **Code**:
  ```rust
  // wallet/core/src/account/variants/multisig.rs:195
  fn sig_op_count(&self) -> u8 {
      u8::try_from(self.xpub_keys.len()).unwrap()
  }
  ```
- **Recommended fix**: Validate `xpub_keys.len() <= 255` at construction/
  deserialization; return an error instead of `unwrap()`.

### [L-10] `unsafe { from_utf8_unchecked }` in gen1 import and test code

- **File**: `wallet/core/src/compat/gen1.rs:18`, `wallet/core/src/account/mod.rs:856`
- **Description**: `gen1.rs:18` uses `String::from_utf8_unchecked(decrypted)`
  on AEAD-decrypted plaintext — UB if the plaintext is not valid UTF-8 (only
  "safe" with the correct password). `account/mod.rs:856` uses
  `from_utf8_unchecked` on a hex-encoded buffer in test code (safe but
  fragile). `wallet/core/src/wallet/mod.rs:1741` has a commented-out
  `from_utf8_unchecked`.
- **Code**:
  ```rust
  // wallet/core/src/compat/gen1.rs:18
  Ok(unsafe { String::from_utf8_unchecked(decrypted) })
  ```
- **Recommended fix**: Use `String::from_utf8(decrypted)?` in `gen1.rs`. Replace
  the test-code `from_utf8_unchecked` with `from_utf8` for safety.

### [L-11] `Encrypted` `Debug` prints ciphertext hex

- **File**: `wallet/core/src/encryption.rs:152`
- **Description**: `Encrypted` has a manual `Debug` impl that prints
  `self.payload.to_hex()` — the ciphertext. While lower risk than plaintext
  leakage (the ciphertext is encrypted), it still leaks ciphertext to logs,
  which could aid an attacker in offline brute-force attempts.
- **Code**:
  ```rust
  // wallet/core/src/encryption.rs:150-154
  impl std::fmt::Debug for Encrypted {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          f.debug_struct("Encrypted").field("encryption_kind", &self.encryption_kind)
              .field("payload", &self.payload.to_hex()).finish()
      }
  }
  ```
- **Recommended fix**: Replace `&self.payload.to_hex()` with `&"********"` or a
  length-only field.

### [L-12] `From<NetworkId> for Prefix` maps all non-mainnet to `KTUB`

- **File**: `wallet/bip32/src/prefix.rs:215`
- **Description**: `From<NetworkId>` maps `Mainnet → KPUB`, and all other
  network types (`Testnet`, `Devnet`, `Simnet`) to `KTUB`. This means testnet,
  devnet, and simnet all share the same extended-key prefix, making them
  indistinguishable in serialized form. A devnet xpub could be accidentally
  imported as a testnet xpub (or vice versa) without detection.
- **Code**:
  ```rust
  // wallet/bip32/src/prefix.rs:213-222
  impl From<NetworkId> for Prefix {
      fn from(value: NetworkId) -> Self {
          match value.network_type() {
              NetworkType::Mainnet => Prefix::KPUB,
              NetworkType::Devnet => Prefix::KTUB,
              NetworkType::Simnet => Prefix::KTUB,
              NetworkType::Testnet => Prefix::KTUB,
          }
      }
  }
  ```
- **Recommended fix**: Consider separate prefixes for devnet/simnet if network
  isolation is required, or document this as intentional.

### [L-13] `DerivationPath` deserialize visitor has copy-paste error in `expecting` message

- **File**: `wallet/bip32/src/derivation_path.rs:35`
- **Description**: The `expecting` message says `"a string containing list of
  permissions separated by a '+'"` — this is a copy-paste error from another
  module. It should say something like `"a BIP32 derivation path string"`.
- **Code**:
  ```rust
  // wallet/bip32/src/derivation_path.rs:34-36
  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a string containing list of permissions separated by a '+'")
  }
  ```
- **Recommended fix**: Fix the message to `"a BIP32 derivation path string (e.g. m/44'/123456'/0')"`.

---

## Summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 1     | C-01 |
| HIGH     | 5     | H-01–H-05 |
| MEDIUM   | 13    | M-01–M-13 |
| LOW      | 13    | L-01–L-13 |
| **Total**| **32** | |

### Zyanya-specific modifications

The following Zyanya-specific changes were identified and are **not** findings
themselves (they are cosmetic or intentional), but they are the points where
Zyanya diverges from upstream Kaspa:

1. **BIP32 prefixes** (`wallet/bip32/src/prefix.rs`): `kprv`/`kpub` (mainnet)
   and `ktrv`/`ktub` (testnet/devnet/simnet) — new prefix constants with
   versions `0x038f2ef4`/`0x038f332e`/`0x03909e07`/`0x0390a241`. Verified
   against test vectors in `mnemonic/phrase.rs`.
2. **Account-kind strings** (`wallet/core/src/account/variants/*.rs`):
   `zyanya-bip32-standard`, `zyanya-legacy-standard`, `zyanya-multisig-standard`,
   `zyanya-keypair-standard`, `zyanya-bip32-watch-standard`.
3. **Go-wallet import path** (`wallet/core/src/compat/gen1.rs`):
   `import_zyanyawallet_golang_*` functions with Argon2id + XChaCha20Poly1305
   mnemonic decryption.
4. **Address prefixes**: `zyanya:` (mainnet) and `zyanyatest:` (testnet).

### BIP32/BIP39 correctness spot-checks (no findings)

- `ExtendedPrivateKey::new` accepts seed lengths 16/32/64 and uses the BIP39
  domain separator `"Bitcoin seed"` (correct for BIP32 master-key derivation).
- `derive_child` hardened vs non-hardened HMAC input (`0x00 || key` vs `pubkey`)
  is correct. The "no retry on invalid tweak" approach is acceptable (prob <
  2^-127).
- `Mnemonic::new` 12-word checksum: `actual_checksum = entropy[16]` vs
  `expected_checksum = sha256[0] & 0b11110000` — the bottom 4 bits of the 17th
  byte are always 0 (BitWriter pads), so the comparison is exact. 24-word
  checksum uses the full first byte of `sha256(entropy)`. Both are correct.
- gen0 legacy uses `44'/972/<account>'` with hardened address indices derived
  from a pre-seeded HMAC (custom Kaspa scheme) — matches in-repo test vectors.
- gen1 uses `44'/123456'/<account>'` (multisig `45'`) with non-hardened
  cosigner and address-type children — matches Kaspa upstream.