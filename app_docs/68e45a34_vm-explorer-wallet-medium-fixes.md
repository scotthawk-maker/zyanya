# Batch M1: MEDIUM Security Findings — VM, Explorer, and Wallet

**Session:** `68e45a34`
**Scope:** Audit findings F-M-01 through F-M-22 (excluding F-M-08 and F-M-09, verified safe)
**Diff stat:** +728 −125 across 25 files
**Acceptance gate:** `cargo check --workspace --all-targets`

---

## 1. What changed and why it matters

This batch remediates 20 MEDIUM-severity audit findings across three subsystems (VM/consensus, wallet crypto & key management, and the zyanya-explorer API). The unifying theme is **defense in depth against malformed input and memory-safety hazards**: nearly every finding was a place where the code panicked, silently coerced bad input to a default, or used `unsafe` without a provable invariant. The fixes replace panics with propagated errors, replace saturating/unchecked arithmetic with checked operations, add compile-time layout assertions and `SAFETY` documentation to the remaining `unsafe` blocks, and tighten key-material handling (zeroization, deterministic-salt removal, lock-order discipline).

A companion remediation plan was committed at `specs/68e45a34_vm-explorer-wallet-medium-fixes.md`; this document describes what was actually implemented in the code diff.

### Findings addressed

| Finding | Category | One-line summary |
|---------|----------|------------------|
| F-M-01 | Explorer input coercion | `api_contract_state_handler` and DEX client paths now return `400`/`Err` on unparseable keys/addresses instead of `unwrap_or(0)` / `unwrap_or(1)`. |
| F-M-02 | VM gas accounting | `GasMeter::consume` uses `checked_add`; `refund` returns `Result`; failing child `CALL` refunds unused gas before pushing `0`. |
| F-M-03 | ZCL parser recursion | Verify-only — `MAX_PARSE_DEPTH = 100` already present in `parser.rs`; no code change in this diff (regression test noted in plan). |
| F-M-04 | Key input validation | `from_secret_hex` uses `strip_prefix` (one prefix only) + 64-char length check; double-prefixed `"0x0x…"` is rejected. |
| F-M-05 | Unsafe code | `heavy_hash` replaces `MaybeUninit::uninit().assume_init()` + `transmute` with a safe `array_from_fn` nibble expansion. |
| F-M-06 | Unsafe code / layout | `ContractStorageKey` gains compile-time `size_of`/`offset_of` assertions and a `SAFETY` comment on the `AsRef<[u8]>` cast; a unit test verifies the byte layout. |
| F-M-07 | Unsafe UTF-8 | All `from_utf8_unchecked` sites in `crypto/hashes`, `script_public_key`, and `math/uint` replaced with `std::str::from_utf8(…).expect("…")`. |
| F-M-10 | Encryption panic | `decrypt_xchacha20poly1305` checks `data.len() >= 56` (salt+nonce+tag) and returns `Err` instead of panicking. |
| F-M-11 | Go-wallet import panic/UB | `decrypt_mnemonic` validates cipher length, clamps `num_threads`, propagates `?`, and uses `String::from_utf8` instead of `from_utf8_unchecked`. |
| F-M-12 | Deadlock | `derive_pubkey_range` now locks `inner` before `cache`, matching `derive_pubkey`; lock-order invariant documented. |
| F-M-13 | Key zeroization | `Drop` impls added to `Inner`, `KeydataSignerInner`, and `PSSBSignerInner` that iterate key maps and call `zeroize()`. |
| F-M-14 | Division by zero | `calc_storage_mass_output_harmonic_single` and `calc_storage_mass_input_mean_arithmetic` use `checked_div` / zero-guards returning `0`. |
| F-M-15 | WASM panic | `Mnemonic::set_entropy` / `set_phrase` return `Result<(), JsValue>`; phrase is re-validated before storage. |
| F-M-16 | PSST DoS / sig conflict | `PSST::from_hex` enforces `MAX_PSST_SIZE = 10 MB`; `Input::add` detects conflicting partial sigs for the same pubkey → `CombineError::ConflictingPartialSig`. |
| F-M-17 | PSSB signing panic | `pssb_signer_for_address` propagates errors via `?` / `map_err` instead of `unwrap`/`expect`. |
| F-M-18 | Hardcoded mass/fee, empty-output panic | `psst_to_pending_transaction` computes mass via `MassCalculator`, derives `fee = inputs − outputs`, and uses `outputs.first().ok_or(…)?`. |
| F-M-19 | Multisig sig ordering | `finalize_psst_one_or_more_sig_and_redeem_script` parses the redeem script for pubkey order, sorts partial sigs to match, and errors on unknown pubkeys. |
| F-M-20 | Signing panic | `sign_with_multiple_v2` returns `Result<Signed, Error>`; v1 `sign_with_multiple` marked `#[deprecated]`. |
| F-M-21 | Secret `Clone` | `Secret` no longer derives `Clone`; a manual `Clone` impl documents that all copies are zeroized on drop. Known clone call sites refactored to borrow. |
| F-M-22 | Deterministic KDF salt | `encrypt_xchacha20poly1305` generates a random 16-byte salt; payload layout is now `salt(16) ‖ nonce(24) ‖ ciphertext+tag`; `argon2_hash` helper added. |

---

## 2. Files that carry the changes

### Consensus / VM / PoW
- `consensus/core/src/sign.rs` — F-M-20: `sign_with_multiple_v2` returns `Result`; v1 deprecated.
- `consensus/core/src/tx/script_public_key.rs` — F-M-07: safe UTF-8 in `Serialize`.
- `consensus/pow/src/matrix.rs` — F-M-05: safe `array_from_fn` replaces `MaybeUninit`/`transmute`.
- `consensus/src/model/stores/contract.rs` — F-M-06: layout assertions + `SAFETY` doc + layout test.
- `crypto/hashes/src/lib.rs` — F-M-07: safe UTF-8 in `Display`.
- `math/src/uint.rs` — F-M-07: safe UTF-8 in four `Display`/`Serialize` sites.
- `zyanya-vm/src/gas.rs` — F-M-02: `consume`/`refund` checked arithmetic.
- `zyanya-vm/src/vm.rs` — F-M-02: child `CALL` refunds unused gas on both `Ok` and `Err` branches.

### Wallet crypto & keys
- `wallet/keys/src/secret.rs` — F-M-21: manual `Clone` + safety doc.
- `wallet/keys/src/derivation/gen0/hd.rs` — F-M-12: lock-order fix + invariant comment.
- `wallet/core/src/encryption.rs` — F-M-10, F-M-22: random-salt KDF, short-ciphertext guard, salt-uniqueness & short-ciphertext tests.
- `wallet/core/src/compat/gen1.rs` — F-M-11: input validation + `?` propagation.
- `wallet/core/src/tx/generator/signer.rs` — F-M-13: `Drop` zeroization; F-M-20: propagate `Result`.
- `wallet/core/src/tx/generator/pending.rs` — F-M-20: `?` on `sign_with_multiple_v2`.
- `wallet/core/src/tx/mass.rs` — F-M-14: `checked_div` / zero-guards.
- `wallet/core/src/wasm/wallet/storage.rs` — F-M-21: borrow `&Secret` instead of cloning.
- `wallet/core/src/account/pssb.rs` — F-M-13, F-M-17, F-M-18, F-M-19: key zeroization, error propagation, computed mass/fee, redeem-script pubkey ordering.
- `wallet/bip32/src/mnemonic/phrase.rs` — F-M-15: `Result`-returning setters.
- `wallet/psst/src/psst.rs` — F-M-16: `MAX_PSST_SIZE` guard.
- `wallet/psst/src/error.rs` — F-M-16: `PsstSizeLimitExceeded` variant.
- `wallet/psst/src/input.rs` — F-M-16: `ConflictingPartialSig` detection.
- `zyanya-wallet/src/key_management.rs` — F-M-04: `strip_prefix` + length validation + double-prefix test.

### Explorer
- `zyanya-explorer/src/api.rs` — F-M-01: `400` on invalid key.
- `zyanya-explorer/src/client.rs` — F-M-01: `?` propagation / fail-closed on unparseable keys & unknown `token_in`.

### Spec
- `specs/68e45a34_vm-explorer-wallet-medium-fixes.md` — the full remediation plan (pre-implementation).

---

## 3. How to use and verify

### Build gate (mandatory)
```sh
cargo check --workspace --all-targets
```
This must pass from the repo root. All `Result`-returning signature changes (`sign_with_multiple_v2`, `GasMeter::refund`, `Mnemonic` setters, `pssb_signer_for_address`) have their call sites updated in the same diff, so the workspace compiles cleanly.

### Targeted tests
```sh
cargo test -p zyanya-vm              # F-M-02 gas metering
cargo test -p zyanya-pow             # F-M-05 heavy_hash
cargo test -p zyanya-consensus       # F-M-06 ContractStorageKey layout
cargo test -p zyanya-wallet-core    # F-M-10, F-M-11, F-M-13, F-M-14, F-M-18, F-M-19
cargo test -p zyanya-wallet-psst     # F-M-16 size limit + sig conflict
cargo test -p zyanya-wallet          # F-M-04 double-prefix rejection
cargo test -p zyanya-bip32           # F-M-15 mnemonic setters
cargo test -p zyanya-explorer        # F-M-01 input validation
```

### Key behavioural changes to be aware of

- **Encrypted payload format change (F-M-22):** `Encrypted.payload` is now `salt(16) ‖ nonce(24) ‖ ciphertext+tag` instead of `nonce(24) ‖ ciphertext+tag`. Ciphertexts produced by the old code are **not** decryptable by the new code. This is acceptable for a pre-production codebase but must be called out in the commit message; a format version byte should be added if backward compatibility is later required.
- **`sign_with_multiple_v2` is now fallible (F-M-20):** callers must add `?`. The old v1 `sign_with_multiple` is `#[deprecated]` (its key lookup never matched pay-to-public-key) but retained so the existing `#[allow(deprecated)]` test compiles.
- **`GasMeter::refund` is now fallible (F-M-02):** the `CALL` error branch deliberately ignores the refund `Result` (`let _ = …`) so a failed child still pushes `0` onto the stack even if the refund underflows; the `Ok` branch propagates with `?`.
- **`Secret` clone is explicit (F-M-21):** borrowing is preferred; where a clone is unavoidable the manual `Clone` impl documents that every copy is zeroized on drop.
- **Mnemonic WASM setters throw (F-M-15):** JS callers that previously ignored the `void` return must now handle a thrown `JsValue`.

### Regression tests added in this diff
- `test_from_secret_hex_rejects_double_prefix` (F-M-04)
- `test_wallet_encrypt_salt_uniqueness` (F-M-22)
- `test_wallet_decrypt_short_ciphertext_errors` (F-M-10)
- `test_contract_storage_key_as_ref_layout` (F-M-06)

---

## 4. Notes for the next agent

- F-M-03 (ZCL parser depth limit) required **no code change** — the `MAX_PARSE_DEPTH = 100` guard already exists in `zyanya-vm/src/compiler/parser.rs`. The plan recommends adding a regression test with 200-level nested parentheses; that test is **not** present in this diff and remains a follow-up.
- F-M-21 notes that plaintext `Serialize`/`Deserialize` of `Secret` is a residual risk (API message structs require it). This is flagged as out-of-scope follow-up.
- The failing-child-`CALL` gas refund (F-M-02 `Err` branch) uses `let _ = self.gas_meter.refund(unused);` so a refund underflow does not abort the parent VM step. If stricter accounting is desired later, promote that to `?`.
- No changes were made to `audit_reports/` or to the `crypto/addresses` bech32 / `compute_rank` code (F-M-08, F-M-09 verified safe).