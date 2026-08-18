# Remediation Plan — F-C-09: `minimum_signatures == 0` anyone-can-spend multisig

## Objective

Remediate CRITICAL finding **F-C-09** from `audit_reports/FINAL_AUDIT_REPORT.md` (§3).

**Root cause:** `minimum_signatures == 0` is never rejected. A 0-of-N multisig redeem
script (`OP_0 <pubkeys...> OP_N OP_CHECKMULTISIG`) is spendable by anyone supplying an
empty signature vector. The value flows from wallet storage / account creation into
`create_address` → `create_multisig_address` → `multisig_redeem_script`, none of which
reject `0`.

**Fix goal:** Enforce `minimum_signatures >= 1` at every multisig address-creation,
deserialization, signing, and transaction-validation path. Reject any multisig
configuration where `minimum_signatures == 0`.

**Scope:** `wallet/core` (primary). `crypto/txscript` (defense-in-depth, root-cause guard).

**Verification command (after changes):**
`source /root/.cargo/env && cargo check -p wallet/core 2>&1`
Fix any compilation errors. Do **NOT** run full build or tests.

---

## Validation rule (apply everywhere)

Reject when **either** condition holds:

1. `minimum_signatures == 0`  (the F-C-09 fix)
2. `minimum_signatures > number_of_public_keys`  (existing behavior, keep it)

Error style:
- In `derivation.rs` use the existing string-`.into()` style (matches current code).
- In `tx/generator/*`, `account/pssb.rs`, `wallet/mod.rs` use
  `Error::InvalidArgument("minimum_signatures must be at least 1".to_string())`.

---

## Files to change (priority order — CRITICAL first)

### 1. `wallet/core/src/derivation.rs` — address creation (CRITICAL)

Add a small `pub(crate)` helper near the top of the file (after imports):

```rust
pub(crate) fn validate_minimum_signatures(minimum_signatures: usize, key_count: usize) -> Result<()> {
    if minimum_signatures == 0 {
        return Err("The minimum amount of signatures must be at least 1".into());
    }
    if key_count < minimum_signatures {
        return Err(format!{"The minimum amount of signatures ({}) is greater than the amount of provided public keys ({key_count})", minimum_signatures}.into());
    }
    Ok(())
}
```

Apply it in three places:

- **`AddressManager::new`** (~line 67): replace the existing `length < minimum_signatures`
  block with `validate_minimum_signatures(minimum_signatures, length)?;`
- **`create_address`** (~line 489): replace the existing `length < minimum_signatures`
  block with `validate_minimum_signatures(minimum_signatures, length)?;`
- **`create_multisig_address`** (~line 443): add
  `validate_minimum_signatures(minimum_signatures, keys.len())?;` at the top
  (defense-in-depth; this is the function that actually builds the redeem script).

### 2. `wallet/core/src/account/variants/multisig.rs` — deserialization (CRITICAL)

- **`Payload::try_load`** (~line 47): after `try_from_slice`, validate before returning:

```rust
pub fn try_load(storage: &AccountStorage) -> Result<Self> {
    let payload = Self::try_from_slice(storage.serialized.as_slice())?;
    payload.validate()?;
    Ok(payload)
}
```

- Add a `validate(&self) -> Result<()>` method on `Payload`:

```rust
fn validate(&self) -> Result<()> {
    if self.minimum_signatures == 0 {
        return Err(Error::InvalidArgument("minimum_signatures must be at least 1".to_string()));
    }
    if self.minimum_signatures as usize > self.xpub_keys.len() {
        return Err(Error::InvalidArgument("minimum_signatures exceeds the number of xpub keys".to_string()));
    }
    Ok(())
}
```

- **`MultiSig::try_new`** (~line 96): call the same validation before `Payload::new`
  (reject at creation time, not just load time). `MultiSig::try_load` is already covered
  by `Payload::try_load`.

### 3. `wallet/core/src/account/variants/watchonly.rs` — deserialization (CRITICAL)

Same pattern as multisig (watch-only accounts can also be multisig):

- **`Payload::try_load`** (~line 47): validate after deserialize.
- Add `validate(&self)` on `Payload` (same two checks).
- **`WatchOnly::try_new`** (~line 88): validate `minimum_signatures` in the multisig
  branch (`xpub_keys.len() > 1`) before `Payload::new`. `WatchOnly::try_load` is covered
  by `Payload::try_load`.

### 4. `wallet/core/src/wallet/mod.rs` — account creation entry points (HIGH)

- **`create_account_multisig`** (~line 707): at the top, reject early:

```rust
if minimum_signatures == 0 {
    return Err(Error::InvalidArgument("minimum_signatures must be at least 1".to_string()));
}
```

- **`import_multisig_with_mnemonic`** (~line 1613): no direct change needed — it calls
  `MultiSig::try_new`, which now validates. (Optionally add the same early check for a
  clearer error.)

### 5. `wallet/core/src/account/pssb.rs` — PSBT signing (HIGH)

- **`PSSBSigner::new`** (~line 40): reject a 0-of-N account before any signing:

```rust
if account.minimum_signatures() == 0 {
    return Err(Error::InvalidArgument("minimum_signatures must be at least 1".to_string()));
}
```

  (Alternatively/additionally guard `pssb_signer_for_address`; `PSSBSigner::new` is the
  single choke point for the signer.)

### 6. `wallet/core/src/tx/generator/generator.rs` — transaction validation (HIGH)

- **`Generator::try_new`** (~line 340): after destructuring `settings`, add:

```rust
if minimum_signatures == 0 {
    return Err(Error::InvalidArgument("minimum_signatures must be at least 1".to_string()));
}
```

  This is the single choke point for all `GeneratorSettings` constructors
  (`try_new_with_account`, `try_new_with_context`, `try_new_with_iterator`).

### 7. `wallet/core/src/tx/generator/pending.rs` — transaction validation (HIGH)

- **`PendingTransaction::try_new`** (~line 90): add the same `minimum_signatures == 0`
  rejection at the top (defense-in-depth; `psst_to_pending_transaction` hardcodes `1`,
  but this guards all callers).

### 8. `crypto/txscript/src/standard/multisig.rs` — root-cause guard (defense-in-depth)

- Add a new error variant to the `Error` enum (~line 7):

```rust
#[error("at least one signature is required")]
ErrZeroRequiredSigs,
```

- In **`multisig_redeem_script`** (~line 18) and **`multisig_redeem_script_ecdsa`**
  (~line 44), add at the very top:

```rust
if required == 0 {
    return Err(Error::ErrZeroRequiredSigs);
}
```

- **Update `test_empty_keys`** (~line 116): it currently calls
  `multisig_redeem_script(empty::<[u8; 32]>(), 0)` and expects `Err(Error::EmptyKeys)`.
  With the new guard this now returns `ErrZeroRequiredSigs`. Change the test to use
  `required = 1` with empty keys (still exercises `EmptyKeys`), and add a new test that
  `required = 0` with non-empty keys returns `ErrZeroRequiredSigs`.

---

## Notes / edge cases

- **`compat/gen1.rs`** `minimum_signatures` occurrences are inside `#[cfg(test)] mod test`
  (legacy wallet-format test fixtures) — out of scope for the production fix.
- **No exhaustive matches** on `zyanya_txscript::standard::multisig::Error` exist outside
  the crate's own tests (verified via grep), so adding `ErrZeroRequiredSigs` is safe.
- `wallet/core/src/error.rs` already re-exports the txscript error as
  `MultisigCreateError(#[from] zyanya_txscript::MultisigCreateError)`; no change needed.
- Single-key accounts (bip32/legacy/keypair/resident/bip32watch) always pass
  `minimum_signatures = 1`, so the new `== 0` checks do not affect them.
- `tx/mass.rs` already uses `minimum_signatures.max(1)` for mass math; the new checks are
  about rejecting the configuration, not changing mass behavior.

## Verification checklist

1. `source /root/.cargo/env && cargo check -p wallet/core 2>&1` — must pass.
2. Confirm no new warnings/errors from the added `validate` methods or the new txscript
   error variant.
3. Do **not** run `cargo build` or `cargo test` (per instructions).
