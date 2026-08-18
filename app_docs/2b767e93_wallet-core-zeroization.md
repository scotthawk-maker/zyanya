# Batch 3 Remediation — Wallet Core & Secret Zeroization

**Session:** `2b767e93`
**Findings remediated:** F-H-05, F-H-06, F-H-11, F-H-12, F-H-13, F-H-14, F-H-15
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md`
**Verification gate:** `cargo check --workspace --all-targets` passes.

---

## 1. What changed and why it matters

Seven HIGH-severity audit findings were addressed across the wallet core, the
BIP-32 key layer, the transaction generator, and the zyanya-wallet CLI. The
fixes fall into three themes: preventing balance collision/theft, preventing
fund loss from incorrect fee math, and preventing secret-key/plaintext leakage
through debug output, missing zeroization, and world-readable files.

### F-H-05 — `holder_u64` truncated the address to its first 8 bytes (balance collision / theft)

The wallet's `holder_u64` derived a u64 storage/balance key from the **first 8
bytes** of an address payload. Two addresses sharing those 8 leading bytes would
collide to the same balance key, enabling theft by an attacker who crafts a
colliding address. The fix replaces the truncation with a **blake2b-256 hash of
the full address payload**, taking the first 8 bytes of the digest as a
little-endian u64.

This is a **cross-cutting** change: the consensus
`derive_caller_from_script_pub_key` (in
`consensus/src/pipeline/virtual_processor/processor.rs`) used the identical
"first 8 bytes" derivation to compute the on-chain `CALLER` identity. If only
the wallet side were changed, every real `from` key would differ from `CALLER`
and all token transfers would silently fail-closed. Both functions were updated
in lockstep to the same blake2b-256-of-full-payload derivation, and
cross-reference comments were updated so the two remain obviously paired.
`blake2b_simd` was added as a direct dependency to both `zyanya-wallet/Cargo.toml`
and `consensus/Cargo.toml`.

### F-H-06 — Unchecked addition in UTXO selection (u64 overflow)

`get_zyan_balance` and `send_zyan` accumulated `total_balance` / `selected_amount`
with plain `+=`. On overflow this silently wraps, which could produce a
wildly incorrect (and spendable) balance. Both accumulations now use
`checked_add(...).ok_or(WalletOpsError::BalanceOverflow)?`, and a new
`WalletOpsError::BalanceOverflow` variant was added.

### F-H-11 — Private key material leaked via derived `Debug`

`PrvKeyDataVariant`, `PrvKeyDataPayload`, and `PrvKeyData` derived `Debug`, so
any `{:?}` formatting (logs, error chains, `unwrap` panics) printed the raw
mnemonic / secret-key bytes. `Debug` was removed from all three derives and
replaced with manual redacting impls that emit `[REDACTED]` for the secret
fields (only non-secret metadata like `id` and `name` remain visible on
`PrvKeyData`). The `Serialize`/`Deserialize`/`Borsh*` derives are unchanged —
only `Debug` was the leak. A unit test asserts `format!("{:?}", variant)` does
not contain the secret and does contain `REDACTED`.

### F-H-12 — Wallet file written with world-readable permissions

`WalletStorage::try_store` created the wallet file with `File::create`, which
inherits the process umask and is typically `0644` (world-readable). The
non-WASM branch now uses `OpenOptions` with `mode(0o600)` under
`#[cfg(unix)]`, falling back to `File::create` under `#[cfg(not(unix))]`. This
mirrors the pattern already used in `zyanya-wallet/src/key_management.rs`.

### F-H-13 — Double addition of `change_output_value` to `transaction_fees`

In the absorb-change-to-fees branch, `transaction_fees += change_output_value`
appeared **twice** (once before the mass computation, once after), so the change
was charged as fee twice — direct fund loss for the user. The second addition
(line 813) was removed; the first (line 801) remains.

### F-H-14 — `ExtendedPrivateKey` derived `Clone` with no `Drop`/`Zeroize`

`ExtendedPrivateKey<K>` held a private key and chain code in memory but had no
`Drop`/`Zeroize`, so the key material lingered until the allocator reused the
memory. The fix:

- Removed `#[derive(Clone)]` and replaced it with a **manual** `Clone` (callers
  in `wallet/keys` still require `Clone`).
- Added `Zeroize` (zeroizes `private_key` via a new `PrivateKey::zeroize` trait
  method and the `[u8; 32]` chain code), a `Drop` that calls `zeroize`, and a
  `ZeroizeOnDrop` marker impl.
- Added `fn zeroize(&mut self)` to the `PrivateKey` trait in
  `wallet/bip32/src/private_key.rs`, implemented for `secp256k1::SecretKey` via
  `non_secure_erase()` (the workspace's `secp256k1` has no `zeroize` feature, so
  `SecretKey` does not implement `zeroize::Zeroize`; the trait method bridges
  that gap).

### F-H-15 — `Decrypted<T>` derived `Debug`/`Clone` with no `Drop`/`Zeroize`

`Decrypted<T>` is the transient guard around decrypted plaintext. It derived
`Debug` (leaking plaintext) and had no zeroization. The fix:

- Removed `Debug` from the derive (kept `Clone`, `BorshSerialize`,
  `BorshDeserialize`) and added a manual redacting `Debug` that prints
  `Decrypted([REDACTED])`.
- Added `Zeroize`, `Drop`, and `ZeroizeOnDrop` impls, all bounded on
  `T: Zeroize` (in addition to the existing borsh bounds). Types whose inner `T`
  does not implement `Zeroize` (e.g. `Decrypted<PrvKeyDataMap>`) simply do not
  get the zeroizing impls — this is intentional and does not break compilation;
  the `PrvKeyData` values inside already zeroize via their own `Drop`.

---

## 2. Files that carry the change

| File | Finding(s) | Nature |
|------|-----------|--------|
| `zyanya-wallet/src/wallet_ops.rs` | F-H-05, F-H-06 | blake2b `holder_u64`, `checked_add` balance math, `BalanceOverflow` error, two new tests |
| `zyanya-wallet/Cargo.toml` | F-H-05 | add `blake2b_simd.workspace = true` |
| `consensus/src/pipeline/virtual_processor/processor.rs` | F-H-05 | `derive_caller_from_script_pub_key` uses blake2b of full payload |
| `consensus/Cargo.toml` | F-H-05 | add `blake2b_simd.workspace = true` |
| `wallet/core/src/tx/generator/generator.rs` | F-H-13 | remove duplicate `transaction_fees += change_output_value` |
| `wallet/core/src/storage/keydata/data.rs` | F-H-11 | remove derived `Debug` on 3 types, add redacting `Debug` impls, redaction unit test |
| `wallet/core/src/storage/local/wallet.rs` | F-H-12 | `OpenOptions` with `mode(0o600)` under `#[cfg(unix)]` |
| `wallet/bip32/src/xprivate_key.rs` | F-H-14 | manual `Clone`, `Zeroize`/`Drop`/`ZeroizeOnDrop` |
| `wallet/bip32/src/private_key.rs` | F-H-14 | add `zeroize` to `PrivateKey` trait + `SecretKey` impl |
| `wallet/core/src/encryption.rs` | F-H-15 | manual redacting `Debug`, `Zeroize`/`ZeroizeOnDrop` for `Decrypted<T>` |
| `specs/2b767e93_wallet-core-zeroization.md` | (plan) | full remediation plan written before implementation |

---

## 3. How to verify

1. **Build gate (the required gate for this batch):**
   ```sh
   cargo check --workspace --all-targets
   ```
   Must pass with no new errors or warnings.

2. **Spot-check the cross-cutting pairing (F-H-05):** both `holder_u64` and
   `derive_caller_from_script_pub_key` must use the **identical** blake2b-256
   derivation — same hash length (32), same byte range (`[0..8]`), same
   endianness (`from_le_bytes`). The comment in each function points at the
   other.
   ```sh
   grep -n "blake2b_simd" zyanya-wallet/src/wallet_ops.rs \
             consensus/src/pipeline/virtual_processor/processor.rs
   ```

3. **Confirm no derived `Debug` remains on key types (F-H-11):**
   ```sh
   grep -n "derive(Clone, Debug" wallet/core/src/storage/keydata/data.rs
   ```
   Should return no matches for the three secret-bearing types.

4. **Confirm no plain `File::create` in the wallet writer (F-H-12):**
   ```sh
   grep -n "File::create" wallet/core/src/storage/local/wallet.rs
   ```
   Should only appear under the `#[cfg(not(unix))]` fallback.

5. **Unit tests (if time permits):**
   ```sh
   cargo test -p zyanya-tui-wallet  # wallet_ops tests incl. holder_u64
   cargo test -p zyanya-wallet-core # keydata/encryption redaction tests
   ```

6. **Runtime wallet-file mode (unix, optional):** after a `try_store`, assert
   the file mode is `0o600` via `std::os::unix::fs::MetadataExt::mode()`.

---

## 4. Notes for the next agent

- **F-H-05 is load-bearing for token transfers.** `holder_u64` (wallet) and
  `derive_caller_from_script_pub_key` (consensus) MUST stay byte-for-byte
  identical in derivation. If either is changed in the future, change both.
- `secp256k1` in this workspace has **no** `zeroize` feature; `SecretKey` does
  not implement `zeroize::Zeroize`. The `PrivateKey::zeroize` trait method
  (calling `non_secure_erase()`) is the bridge — do not try to replace it with
  a `zeroize::Zeroize` impl on `SecretKey` without enabling the feature.
- `Decrypted<T>` keeps its `Clone` derive intentionally (the prompt scoped
  F-H-15 to `Debug` + `Drop`/`Zeroize`). A manual zeroizing `Clone` is a future
  hardening item, not part of this batch.
- `zeroize` is v1.9.0 in the lockfile; `ZeroizeOnDrop` is used as a manual marker
  impl (no derive macro needed).
- The remediation plan document `specs/2b767e93_wallet-core-zeroization.md`
  shipped alongside the code and is the authoritative design rationale for
  every change above.