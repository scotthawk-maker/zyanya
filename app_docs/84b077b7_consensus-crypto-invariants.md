# Document — Batch 2: Consensus & Crypto Invariants (F-H-08, F-H-09, F-H-10)

**Session:** 84b077b7
**Base commit:** e9d7265
**Diff stat:** +300 −19 across 4 tracked files (1 new spec, 3 source edits)
**Diff path:** `adws/adw_data/sessions/84b077b7/context_handoff/changes.diff`

---

## What changed and why it matters

Three HIGH audit findings were remediated. All three share a common root pattern: invariants
that were meant to catch internal bugs were written as release-active panics (or as
`debug_assert!` checks that are compiled out in release), turning a malformed input or a latent
overflow into either a remote node-crash DoS or a silent wrap in consensus-critical math.

### F-H-08 — MuHash `U3072::mul` non-debug asserts (remote panic DoS)

**File:** `crypto/muhash/src/u3072.rs`

Three `assert!` / `assert_eq!` statements inside the schoolbook-multiplication carry
propagation of `U3072::mul` were converted to `debug_assert!` / `debug_assert_eq!`:

| Line | Before | After |
|------|--------|-------|
| 123  | `assert_eq!(carry_highest, 0);` | `debug_assert_eq!(carry_highest, 0);` |
| 143  | `assert_eq!(carry_high, 0);` | `debug_assert_eq!(carry_high, 0);` |
| 144  | `assert!(carry_low == 0 \|\| carry_low == 1);` | `debug_assert!(carry_low == 0 \|\| carry_low == 1);` |

These are *internal algorithm invariants* of the carry propagation, not properties of the
operands. The carry values are bounded by the algorithm regardless of input — including
arbitrary 384-byte arrays accepted by `MuHash::deserialize`. Because `mul` runs on every
UTXO-set element during block processing, a release-build panic here would halt the node on
remote-supplied data. Converting to `debug_assert!` removes the panic vector while preserving
the debug-mode safety net.

**Not changed:** the `assert_eq!(one, Self::one())` at line 170 (already guarded by
`if cfg!(debug_assertions)`) and the `mod_inverse(...).expect(...)` in `inverse()` (provably
unreachable given the `a == Self::zero()` early-return and `is_overflow()` reduction).

### F-H-09 — `Uint` arithmetic `debug_assert!` overflow checks silently wrap in release

**File:** `math/src/uint.rs` (inside the `construct_uint!` macro, which generates
`Uint192`, `Uint256`, `Uint320`, `Uint3072`)

Two-part fix:

**(a) New `checked_*` API.** Five methods were added next to the existing
`overflowing_*` / `saturating_*` methods, each returning `Option<Self>` (`None` on overflow):

- `checked_add(self, other: Self) -> Option<Self>`
- `checked_add_u64(self, other: u64) -> Option<Self>`
- `checked_sub(self, other: Self) -> Option<Self>`
- `checked_mul(self, other: Self) -> Option<Self>`
- `checked_mul_u64(self, other: u64) -> Option<Self>`

**(b) Operator impls now panic on overflow in both debug and release.** The five
`core::ops` impls (`Add`, `Add<u64>`, `Sub`, `Mul`, `Mul<u64>`) previously did:

```rust
let (sum, carry) = self.overflowing_add(other);
debug_assert!(!carry, "attempt to add with overflow"); // compiled out in release
sum
```

They now do:

```rust
self.checked_add(other).expect("attempt to add with overflow")
```

The operator traits have fixed signatures returning `Self`, so they cannot return a
`Result`. The fix therefore (1) exposes a `checked_*` API that *does* return `Option<Self>`
for graceful handling, and (2) makes the operators fail loudly (panic) on overflow in both
build modes instead of silently wrapping in release. This directly closes the "overflows
silently wrap in release" finding.

**Blast radius (verified safe):** `Sum`/`Product` impls use `+`/`*` and now also panic on
overflow — intended, since no reachable consensus path overflows. `difficulty.rs`
(`targets_sum`, `average_target`, `new_target`), `calc_work` (`(!target / (target + 1)) + 1`,
where `from_compact_target_bits` rejects the `MAX` target so `target + 1` cannot overflow), and
`div_rem` (guarded by `sub_copy >= shift_copy` and shift ≤ `my_bits − your_bits`) were all
confirmed in-range.

**Out of scope (left as-is):** `Shl`/`Shr` operator impls (lines ~669/681) also use
`debug_assert!`. They are outside the finding's line range (533–582) and `Shl` is fed by
`from_compact_target_bits`; changing it to panic could introduce a new panic on malformed
`bits`. No `checked_shl`/`checked_shr` were added in this batch.

### F-H-10 — GhostDAG `assert!` sanity check (remote panic DoS)

**File:** `consensus/src/processes/ghostdag/protocol.rs`

Line 214, inside `check_blue_candidate_with_chain_block`:

```rust
// before
assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k,
        "found blue anticone larger than K");
// after
debug_assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k,
              "found blue anticone larger than K");
```

The preceding check (line 206) already returns `ColoringState::Red` when the size `== self.k`,
so the assert can only fire on store-corruption / internal-bug conditions, not on valid DAG
state. Since the function returns `ColoringState` (not `Result`), `debug_assert!` is the
minimal fix that removes the release-build panic without an API change.

**Not changed:** `blue_anticone_size` (line 231) contains a `panic!` driven by internal store
state; converting it to a graceful error would require a signature change and is out of scope.

---

## Files that carry the change

| File | Change |
|------|--------|
| `consensus/src/processes/ghostdag/protocol.rs` | F-H-10: `assert!` → `debug_assert!` (1 line) |
| `crypto/muhash/src/u3072.rs` | F-H-08: 3× `assert!`/`assert_eq!` → `debug_assert!`/`debug_assert_eq!` |
| `math/src/uint.rs` | F-H-09: +5 `checked_*` methods; 5 operator impls rewritten to panic on overflow |
| `specs/84b077b7_consensus-crypto-invariants.md` | New 261-line remediation plan (spec, not shipped code) |

---

## How to use and verify

### Build check (acceptance gate)

```bash
cargo check --workspace --all-targets
```

Must exit 0 with no new warnings/errors.

### Targeted tests (recommended)

```bash
cargo test -p zyanya-math -p zyanya-muhash -p zyanya-consensus
```

### Acceptance criteria

1. `cargo check --workspace --all-targets` exits 0.
2. No remaining non-debug `assert!`/`assert_eq!` in `U3072::mul` or in
   `check_blue_candidate_with_chain_block`.
3. `math/src/uint.rs` exposes `checked_add`, `checked_add_u64`, `checked_sub`, `checked_mul`,
   `checked_mul_u64`, and the `Add`/`Sub`/`Mul`/`Add<u64>`/`Mul<u64>` operators no longer use
   `debug_assert!` for overflow detection.
4. Existing unit tests (`test_overflow_bug`, `test_saturating_ops`, `test_mul`,
   `test_inverse`, `test_mul_div`, `test_mul_max`, difficulty/ghostdag tests) still pass.

### Manual grep to confirm no regressions

```bash
grep -n 'assert!\|assert_eq!' crypto/muhash/src/u3072.rs
# expect only debug_assert* and the cfg-guarded assert_eq!(one, Self::one())

grep -n 'debug_assert!' math/src/uint.rs | grep -i overflow
# expect zero matches in the Add/Sub/Mul operator impls (lines 533-582)
```

---

## Risks & rollback

- **F-H-09 is the highest-risk change** because `math` is shared across consensus, wallet, and
  crypto. If `cargo check`/`cargo test` reveals a reachable overflow that now panics, do **not**
  paper over it — the correct response is to fix the caller to use `checked_*`/`overflowing_*`
  explicitly, not to revert to silent wrapping.
- The three changes are independent and can be committed separately (one commit per finding is
  recommended).
- No consensus rule changes are introduced: `debug_assert!` is a no-op in release, and the
  `checked_*`/panic changes only alter behavior on overflow, which the audit confirms is
  unreachable on valid inputs.

---

## Suggested commit structure

1. `fix(muhash): convert U3072::mul invariants to debug_assert! (F-H-08)`
2. `fix(math): add checked_* API and panic on Uint arithmetic overflow (F-H-09)`
3. `fix(consensus): convert GhostDAG blue-anticone sanity check to debug_assert! (F-H-10)`