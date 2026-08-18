# Remediation Plan — Batch 3: Wallet Core & Secret Zeroization

**Findings:** F-H-05, F-H-06, F-H-11, F-H-12, F-H-13, F-H-14, F-H-15
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md`
**Verification gate:** `cargo check --workspace --all-targets` must pass.

---

## 0. Scope & Priority Order

All seven findings are HIGH. Implementation order (highest impact first):

| # | Finding | File(s) | Risk |
|---|---------|---------|------|
| 1 | F-H-05 | `zyanya-wallet/src/wallet_ops.rs` **+ consensus mirror** | balance collision / theft |
| 2 | F-H-13 | `wallet/core/src/tx/generator/generator.rs` | fund loss (double fee) |
| 3 | F-H-06 | `zyanya-wallet/src/wallet_ops.rs` | u64 overflow in balance math |
| 4 | F-H-11 | `wallet/core/src/storage/keydata/data.rs` | private key leak via `Debug` |
| 5 | F-H-14 | `wallet/bip32/src/xprivate_key.rs` (+ `private_key.rs`) | key not zeroized on drop |
| 6 | F-H-15 | `wallet/core/src/encryption.rs` | decrypted plaintext not zeroized |
| 7 | F-H-12 | `wallet/core/src/storage/local/wallet.rs` | world-readable wallet file |

---

## 1. F-H-05 — `holder_u64` truncates address to first 8 bytes (balance collision / theft)

### Files
- `zyanya-wallet/src/wallet_ops.rs:279-285` (`holder_u64`)
- **REQUIRED COORDINATION:** `consensus/src/pipeline/virtual_processor/processor.rs:1356-1368` (`derive_caller_from_script_pub_key`)
- `zyanya-wallet/Cargo.toml` and `consensus/Cargo.toml` (add `blake2b_simd`)

### Why the consensus file is in scope
The token contract's `transfer` (see `zyanya-vm/src/token.rs:65-70`) enforces `from == CALLER`, where:
- `from` is the holder key computed by the wallet via `holder_u64(address)`.
- `CALLER` (`msg.sender`) is computed by consensus via `derive_caller_from_script_pub_key(script_public_key)` (see `processor.rs:498` and `:545`).

Both currently use "first 8 bytes of address payload, little-endian". If only the wallet is changed, `from != CALLER` for every real user and **all token transfers/balance queries break** (fail-closed). The two functions MUST be updated in lockstep to the identical derivation.

### Fix
Replace the "first 8 bytes" logic with a blake2b-256 hash of the **full** address payload, then take the first 8 bytes of the digest as a little-endian u64.

`zyanya-wallet/src/wallet_ops.rs`:
```rust
pub fn holder_u64(address: &Address) -> u64 {
    let mut hasher = blake2b_simd::Params::new().hash_length(32).to_state();
    hasher.update(address.payload.as_slice());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}
```

`consensus/src/pipeline/virtual_processor/processor.rs` (`derive_caller_from_script_pub_key`):
```rust
fn derive_caller_from_script_pub_key(script_public_key: &zyanya_consensus_core::tx::ScriptPublicKey) -> u64 {
    use zyanya_addresses::Prefix;
    match zyanya_txscript::extract_script_pub_key_address(script_public_key, Prefix::Mainnet) {
        Ok(addr) => {
            let mut hasher = blake2b_simd::Params::new().hash_length(32).to_state();
            hasher.update(addr.payload.as_slice());
            let hash = hasher.finalize();
            u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap())
        }
        Err(_) => 0,
    }
}
```

Update the cross-reference comments in both files so they still say they mirror each other.

### Dependency changes
Add to **both** `zyanya-wallet/Cargo.toml` and `consensus/Cargo.toml`:
```toml
blake2b_simd.workspace = true
```
(`blake2b_simd = "1.0.2"` is already a workspace dependency in the root `Cargo.toml`.)

### Tests
- Update `test_holder_u64_derivation` in `wallet_ops.rs` to assert the result equals `u64::from_le_bytes(blake2b(payload)[0..8])` computed inline (determinism + correct algorithm).
- Add a test that two addresses whose payloads share the first 8 bytes but differ later produce **different** keys (construct two `Address` values with crafted payloads via `Address::new(Prefix::Devnet, Version::PubKey, &payload)` — note `Address::new` asserts payload length for non-test prefixes, so use a test prefix or a 32-byte payload).

---

## 2. F-H-13 — Double addition of `change_output_value` to `transaction_fees`

### File
`wallet/core/src/tx/generator/generator.rs:801` and `:813`

### Fix
Remove the **second** `transaction_fees += change_output_value;` (line 813). Keep the first (line 801). The fee must include the change exactly once.

Before:
```rust
if absorb_change_to_fees || change_output_value == 0 {
    transaction_fees += change_output_value;   // line 801 — keep

    let compute_mass = ...;
    let storage_mass = ...;
    data.aggregate_mass = calc.combine_mass(compute_mass, storage_mass);

    transaction_fees += change_output_value;   // line 813 — REMOVE
    data.transaction_fees = transaction_fees;
    ...
}
```

After: delete line 813 only.

### Test
Add a unit test (in `generator.rs` tests module) asserting `transaction_fees == aggregate_input_value - final_transaction.value_no_fees` for the absorb-change case (`absorb_change_to_fees == true` or `change_output_value == 0`). If a full generator harness is impractical, at minimum assert the single-addition invariant via a focused test of the branch.

---

## 3. F-H-06 — Unchecked addition in UTXO selection

### File
`zyanya-wallet/src/wallet_ops.rs:158` and `:204`

### Fix
Use `checked_add` and return an error on overflow.

Add an error variant to `WalletOpsError`:
```rust
#[error("Balance overflow")]
BalanceOverflow,
```

Line 158 (`get_zyan_balance`):
```rust
total_balance = total_balance
    .checked_add(entry.utxo_entry.amount)
    .ok_or(WalletOpsError::BalanceOverflow)?;
```

Line 204 (`send_zyan`):
```rust
selected_amount = selected_amount
    .checked_add(entry.amount)
    .ok_or(WalletOpsError::BalanceOverflow)?;
```

---

## 4. F-H-11 — Private key material leaked via derived `Debug`

### File
`wallet/core/src/storage/keydata/data.rs`

### Current state
`PrvKeyDataVariant`, `PrvKeyDataPayload`, and `PrvKeyData` already have `Zeroize` + `Drop` (+ `ZeroizeOnDrop` for the first two). The remaining leak is the **derived `Debug`**.

### Fix
Remove `Debug` from the `#[derive(...)]` on these three types and add manual redacting `Debug` impls:

- `PrvKeyDataVariant` (line 20): remove `Debug` from derive.
- `PrvKeyDataPayload` (line 117): remove `Debug` from derive.
- `PrvKeyData` (line 191): remove `Debug` from derive.

Add:
```rust
impl std::fmt::Debug for PrvKeyDataVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PrvKeyDataVariant([REDACTED])")
    }
}

impl std::fmt::Debug for PrvKeyDataPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrvKeyDataPayload").field("prv_key_variant", &"[REDACTED]").finish()
    }
}

impl std::fmt::Debug for PrvKeyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrvKeyData")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("payload", &"[REDACTED]")
            .finish()
    }
}
```

Notes:
- `PrvKeyDataVariantKind` (line 12) is a plain kind enum (no secret) — keep its `Debug`.
- Keep `Serialize`/`Deserialize`/`BorshSerialize`/`BorshDeserialize` derives unchanged (serde is required for storage; only `Debug` leaks).
- `PrvKeyData` has `Zeroize` + `Drop` but no `ZeroizeOnDrop`; optionally add `impl ZeroizeOnDrop for PrvKeyData {}` for consistency (not required by the finding).

### Test
Add a test asserting `format!("{:?}", variant)` does **not** contain the secret string, e.g.:
```rust
let v = PrvKeyDataVariant::Mnemonic("abandon abandon ...".to_string());
let dbg = format!("{:?}", v);
assert!(!dbg.contains("abandon"));
assert!(dbg.contains("REDACTED"));
```

---

## 5. F-H-14 — `ExtendedPrivateKey` derives `Clone`, no `Drop`/`Zeroize`

### Files
- `wallet/bip32/src/xprivate_key.rs:18-19`
- `wallet/bip32/src/private_key.rs` (add a `zeroize` method to the `PrivateKey` trait)

### Fix

**1. Remove `#[derive(Clone)]`** from `ExtendedPrivateKey` (line 18). `Clone` is still required by callers (`wallet/keys/src/xprv.rs:56`, `wallet/keys/src/privkeygen.rs:31,34`), so add a manual, explicit `Clone`:
```rust
impl<K> Clone for ExtendedPrivateKey<K>
where
    K: PrivateKey + Clone,
{
    fn clone(&self) -> Self {
        Self { private_key: self.private_key.clone(), attrs: self.attrs.clone() }
    }
}
```

**2. Add `zeroize` to the `PrivateKey` trait** (`wallet/bip32/src/private_key.rs`):
```rust
pub trait PrivateKey: Sized {
    type PublicKey: PublicKey;
    fn from_bytes(bytes: &PrivateKeyBytes) -> Result<Self>;
    fn to_bytes(&self) -> PrivateKeyBytes;
    fn derive_child(&self, other: PrivateKeyBytes) -> Result<Self>;
    fn public_key(&self) -> Self::PublicKey;
    /// Zeroize the private key material.
    fn zeroize(&mut self);
}
```
Implement for `secp256k1::SecretKey` (the only implementor):
```rust
fn zeroize(&mut self) {
    self.non_secure_erase();
}
```
(`secp256k1::SecretKey::non_secure_erase` exists in secp256k1 0.29; the workspace does **not** enable a `zeroize` feature, so `SecretKey` does not impl `zeroize::Zeroize` — hence the trait method.)

**3. Add `Zeroize` + `Drop` + `ZeroizeOnDrop`** for `ExtendedPrivateKey` in `xprivate_key.rs`:
```rust
impl<K> Zeroize for ExtendedPrivateKey<K>
where
    K: PrivateKey,
{
    fn zeroize(&mut self) {
        self.private_key.zeroize();
        self.attrs.chain_code.zeroize(); // [u8; 32] — zeroize provides the impl
    }
}

impl<K> Drop for ExtendedPrivateKey<K>
where
    K: PrivateKey,
{
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl<K> ZeroizeOnDrop for ExtendedPrivateKey<K> where K: PrivateKey {}
```
`zeroize::{Zeroize, Zeroizing}` is already imported in `xprivate_key.rs`; add `ZeroizeOnDrop` to that import.

Notes:
- `ExtendedKeyAttrs` (chain code, depth, fingerprint, child number) is not secret; zeroizing `chain_code` is harmless hygiene. No change to `attrs.rs` required.
- The existing manual `Debug` impl already redacts `private_key` — leave it.

---

## 6. F-H-15 — `Decrypted<T>` derives `Debug`/`Clone`, no `Drop`/`Zeroize`

### File
`wallet/core/src/encryption.rs:98-99`

### Fix

**1. Remove `Debug` from the derive** (keep `Clone`, `BorshSerialize`, `BorshDeserialize`):
```rust
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Decrypted<T>(pub(crate) T)
where
    T: BorshSerialize + BorshDeserialize;
```

**2. Add manual redacting `Debug`:**
```rust
impl<T> std::fmt::Debug for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Decrypted").field(&"[REDACTED]").finish()
    }
}
```

**3. Add `Zeroize` + `Drop` + `ZeroizeOnDrop`** (bounded on `T: Zeroize`):
```rust
impl<T> Zeroize for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize + Zeroize,
{
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<T> Drop for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize + Zeroize,
{
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl<T> ZeroizeOnDrop for Decrypted<T>
where
    T: BorshSerialize + BorshDeserialize + Zeroize,
{}
```

**4. Update the import** at the top of `encryption.rs`:
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};
```

Notes:
- `Clone` is intentionally kept (the prompt does not require removing it for F-H-15; `Decrypted<T>` is a transient guard and `Clone` is used by `Encryptable` paths). If desired, a manual `Clone` can be added later, but it is out of scope for this batch.
- `Decrypted<PrvKeyDataMap>` (where `PrvKeyDataMap = HashMap<...>`) will not get the `Drop`/`Zeroize` impls because `HashMap` does not impl `Zeroize` — this is fine and does not break compilation; the `PrvKeyData` values inside already zeroize via their own `Drop`.

---

## 7. F-H-12 — Wallet file written with world-readable permissions

### File
`wallet/core/src/storage/local/wallet.rs:69`

### Fix
Replace the plain `File::create` in the non-WASM branch with a Unix `OpenOptions` that sets mode `0600`, falling back to `File::create` on non-Unix (the `cfg_if!` `else` branch also covers Windows).

```rust
} else {
    // make this platform-specific to avoid creating
    // a buffer containing serialization
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(store.filename())?;
        BorshSerialize::serialize(self, &mut file)?;
    }
    #[cfg(not(unix))]
    {
        let mut file = std::fs::File::create(store.filename())?;
        BorshSerialize::serialize(self, &mut file)?;
    }
}
```

Alternative (mirrors `zyanya-wallet/src/key_management.rs:129-131`): create the file, then `fs::set_permissions(path, fs::Permissions::from_mode(0o600))` under `#[cfg(unix)]`.

### Test (optional, unix-only)
After `try_store`, assert the file mode is `0o600` via `std::os::unix::fs::MetadataExt::mode()`.

---

## 8. Verification

1. `cargo check --workspace --all-targets` — must pass with no new warnings/errors.
2. `cargo test -p zyanya-tui-wallet` (wallet_ops tests) and `cargo test -p zyanya-wallet-core` (keydata/encryption tests) if time permits.
3. Grep to confirm no remaining derived `Debug` on the three key types and no remaining `File::create` in `wallet.rs`:
   - `grep -n "derive(Clone, Debug" wallet/core/src/storage/keydata/data.rs`
   - `grep -n "File::create" wallet/core/src/storage/local/wallet.rs`
4. Confirm `holder_u64` and `derive_caller_from_script_pub_key` use the **identical** blake2b derivation (same hash length, same byte range, same endianness).

---

## 9. Notes for the builder

- **F-H-05 is cross-cutting.** Do not change `holder_u64` without also changing `derive_caller_from_script_pub_key` in `consensus/src/pipeline/virtual_processor/processor.rs`. They must produce identical u64 keys or the wallet's token operations will silently fail.
- `blake2b_simd` must be added as a **direct** dependency to both `zyanya-wallet/Cargo.toml` and `consensus/Cargo.toml` (transitive deps are not usable in Rust).
- `secp256k1` in this workspace has **no** `zeroize` feature, so `SecretKey` does not impl `zeroize::Zeroize`; use `non_secure_erase()` via a new `PrivateKey::zeroize` trait method (F-H-14).
- `zeroize` is v1.9.0 in the lockfile; `ZeroizeOnDrop` is available and is implemented manually (no derive macro needed).
- Keep serde/borsh derives intact on the key types — only `Debug` is the leak.
