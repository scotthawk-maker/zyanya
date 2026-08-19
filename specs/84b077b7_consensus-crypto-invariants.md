# Remediation Plan — Batch 2: Consensus & Crypto Invariants (F-H-08, F-H-09, F-H-10)

**Date:** 2026-08-18
**Repo:** `/root/zyanya-audit` (Zyanya blockchain — Kaspa/rusty-spectre fork)
**Source of truth:** `audit_reports/FINAL_AUDIT_REPORT.md` (§4 HIGH findings)
**Goal:** Remediate three HIGH findings that allow remote peers / release-build arithmetic to
panic a node or silently wrap consensus-critical math. After the changes,
`cargo check --workspace --all-targets` must pass.

---

## 1. Summary

| Finding | Severity | File | Root cause | Fix |
|---------|----------|------|------------|-----|
| F-H-08 | HIGH | `crypto/muhash/src/u3072.rs` | Non-debug `assert!`/`assert_eq!` in `U3072::mul` → remote panic DoS | Convert to `debug_assert!` |
| F-H-09 | HIGH | `math/src/uint.rs` | `debug_assert!` overflow checks in `Add`/`Sub`/`Mul` are compiled out in release → silent wrap | Add `checked_*` API; make operators panic on overflow |
| F-H-10 | HIGH | `consensus/src/processes/ghostdag/protocol.rs` | Non-debug `assert!` sanity check in GhostDAG coloring → remote panic DoS | Convert to `debug_assert!` |

All three are **HIGH** (no CRITICAL findings in this batch). Priority order below is by
consensus-criticality and ease of remote triggering.

---

## 2. Priority order (highest first)

1. **F-H-10 — GhostDAG `assert!` (protocol.rs:214)** — runs on every block's GhostDAG
   processing; a malformed DAG or reachability-store corruption panics the node in release.
   Consensus-critical, remotely triggerable.
2. **F-H-08 — MuHash `U3072::mul` asserts (u3072.rs:123,143,144)** — MuHash state is updated
   from every UTXO-set element during block processing; a panic here halts the node.
   Consensus-critical.
3. **F-H-09 — `Uint` arithmetic silent wrap (uint.rs:533-582)** — no currently-reachable
   consensus path overflows, but the pattern is fragile and any future use silently wraps in
   production. Fix is a hardening change with the widest blast radius (shared `math` crate),
   so it is done last and verified with the full workspace build.

---

## 3. Files to audit (with paths) and vulnerability categories

| File | Vulnerability categories to check |
|------|-----------------------------------|
| `crypto/muhash/src/u3072.rs` | Non-debug `assert!`/`assert_eq!` invariants; panic-on-untrusted-input; `expect()` on `mod_inverse`; deserialization of arbitrary 384-byte arrays feeding `mul` |
| `math/src/uint.rs` | Integer overflow/underflow in `Add`/`Sub`/`Mul`/`Shl`/`Shr`; `debug_assert!` compiled out in release; silent wraparound in consensus math |
| `consensus/src/processes/ghostdag/protocol.rs` | Non-debug `assert!` sanity checks; `panic!` in `blue_anticone_size`; consensus logic bypass / DoS via malformed DAG |

---

## 4. Detailed remediation steps

### 4.1 F-H-08 — MuHash `U3072::mul` non-debug asserts

**File:** `crypto/muhash/src/u3072.rs`

**Current code (verified line numbers):**
- Line 123: `assert_eq!(carry_highest, 0);`
- Line 143: `assert_eq!(carry_high, 0);`
- Line 144: `assert!(carry_low == 0 || carry_low == 1);`

**Fix:** Convert the three non-debug assertions to debug assertions:

```rust
// line 123
debug_assert_eq!(carry_highest, 0);
// line 143
debug_assert_eq!(carry_high, 0);
// line 144
debug_assert!(carry_low == 0 || carry_low == 1);
```

**Rationale:** These are *internal algorithm invariants* of the schoolbook-multiplication
carry propagation, not properties of the input operands. The carry values are bounded by the
algorithm regardless of the (possibly deserialized) `U3072` values, so they are provably
unreachable from any input — including the arbitrary 384-byte arrays accepted by
`MuHash::deserialize` (`crypto/muhash/src/lib.rs:111-118`). `mul` returns `()` and is called
from `MulAssign` (`*=`), `div` (`/=`), and the debug-only `inverse` check, so converting to
`debug_assert!` is the minimal, correct fix that removes the release-build panic vector
without changing the API.

**Do NOT change:** the `assert_eq!(one, Self::one())` at line 170 — it is already inside
`if cfg!(debug_assertions) { ... }`.

**Related (out of scope, note only):** `inverse()` at line 166 uses
`Uint3072::mod_inverse(...).expect("Cannot fail, 0 < a < prime")`. This is guarded by the
`a == Self::zero()` early-return and the `is_overflow()` reduction, so `0 < a < prime` holds
and the `expect` is unreachable. Leave as-is for this batch; do not expand scope.

---

### 4.2 F-H-09 — `Uint` arithmetic `debug_assert!` overflow checks

**File:** `math/src/uint.rs` (inside the `construct_uint!` macro, which generates
`Uint192`, `Uint256`, `Uint320`, `Uint3072` — see `math/src/lib.rs:9-12`)

**Current code (verified line numbers):**
- `Add` impl (lines 533-542): `debug_assert!(!carry, "attempt to add with overflow");` (line 534)
- `Add<u64>` impl (lines 544-552): `debug_assert!(!carry, "attempt to add with overflow");` (line 546)
- `Sub` impl (lines 554-562): `debug_assert!(!carry, "attempt to subtract with overflow");` (line 558)
- `Mul` impl (lines 564-572): `debug_assert!(!carry, "attempt to multiply with overflow");` (line 570)
- `Mul<u64>` impl (lines 574-582): `debug_assert!(!carry, "attempt to multiply with overflow");` (line 582)

**Fix (two parts):**

**(a) Add `checked_*` methods to the macro** (place them next to the existing
`overflowing_*`/`saturating_*` methods, e.g. after `saturating_add`). Each returns
`Option<Self>` (`None` on overflow):

```rust
#[inline]
pub fn checked_add(self, other: Self) -> Option<Self> {
    let (sum, carry) = self.overflowing_add(other);
    if carry { None } else { Some(sum) }
}

#[inline]
pub fn checked_add_u64(self, other: u64) -> Option<Self> {
    let (sum, carry) = self.overflowing_add_u64(other);
    if carry { None } else { Some(sum) }
}

#[inline]
pub fn checked_sub(self, other: Self) -> Option<Self> {
    let (sum, carry) = self.overflowing_sub(other);
    if carry { None } else { Some(sum) }
}

#[inline]
pub fn checked_mul(self, other: Self) -> Option<Self> {
    let (product, carry) = self.overflowing_mul(other);
    if carry { None } else { Some(product) }
}

#[inline]
pub fn checked_mul_u64(self, other: u64) -> Option<Self> {
    let (product, carry) = self.overflowing_mul_u64(other);
    if carry { None } else { Some(product) }
}
```

**(b) Rewrite the five operator impls to use the checked methods and panic on overflow**
(replacing the `debug_assert!`, which is compiled out in release):

```rust
impl core::ops::Add<$name> for $name {
    type Output = $name;
    #[inline]
    #[track_caller]
    fn add(self, other: $name) -> $name {
        self.checked_add(other).expect("attempt to add with overflow")
    }
}
// ... same pattern for Add<u64>, Sub, Mul, Mul<u64> using
//     checked_add_u64 / checked_sub / checked_mul / checked_mul_u64
```

**Rationale:** The operator traits (`core::ops::Add/Sub/Mul`) have fixed signatures returning
`Self`, so they cannot return a `Result`. The fix therefore (1) exposes a `checked_*` API that
*does* return errors (`Option<Self>`) for callers that want graceful handling, and (2) makes
the operators fail loudly (panic) on overflow in **both** debug and release instead of
silently wrapping. This directly addresses "overflows silently wrap in release".

**Blast-radius notes (verified safe):**
- `Sum`/`Product` impls (lines ~684-720) use `+`/`*` and will now panic on overflow too — this
  is intended; the audit confirms no reachable consensus path overflows.
- `difficulty.rs` uses `Uint320` with a bounded window (`targets_sum`, `average_target`,
  `new_target`) — within range.
- `calc_work` (`consensus/src/processes/difficulty.rs:274-283`) computes
  `(!target / (target + 1)) + 1`; `from_compact_target_bits` rejects the `MAX` target
  (`mant > 0x7FFFFF` → `ZERO`), so `target + 1` cannot overflow.
- `div_rem` (uint.rs) uses `-`/`<<`/`>>` internally but is bounded (`sub_copy >= shift_copy`
  guard; shift ≤ `my_bits - your_bits`).

**Out of scope (same pattern, different lines — do NOT change in this batch):**
- `Shl` impl (line 669) and `Shr` impl (line 681) also use `debug_assert!`. They are outside
  the finding's line range (533-582). `Shl` is used by `from_compact_target_bits`
  (`math/src/lib.rs:44`) and `level_work` (`difficulty.rs:335`); changing `Shl` to panic could
  introduce a panic if `from_compact_target_bits` is ever fed malformed `bits` (expt > 32).
  Leave them as-is; optionally add `checked_shl`/`checked_shr` methods for completeness, but do
  not change the `Shl`/`Shr` operator semantics in this batch.

---

### 4.3 F-H-10 — GhostDAG `assert!` sanity check

**File:** `consensus/src/processes/ghostdag/protocol.rs`

**Current code (verified line number — note: audit report cites line 220; current file has it
at line 214):**

```rust
// line 214
assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k, "found blue anticone larger than K");
```

**Fix:** Convert to a debug assertion:

```rust
debug_assert!(*candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k, "found blue anticone larger than K");
```

**Rationale:** This is a sanity check inside `check_blue_candidate_with_chain_block`. The
immediately preceding check (line 206) already returns `ColoringState::Red` when the size
`== self.k`, so the assert can only fire on a store-corruption / internal-bug condition, not on
valid DAG state. The function returns `ColoringState` (not `Result`), so `debug_assert!` is the
minimal fix that removes the release-build panic without an API change.

**Related (out of scope, note only):** `blue_anticone_size` (line 231) contains
`panic!("block {block} is not in blue set of the given context")`. This is an internal
invariant driven by the reachability/ghostdag store, not by peer-supplied data, and the
function returns `KType` (not `Result`). Converting it to a graceful error would require a
signature change and is not part of this batch. Leave as-is.

---

## 5. Verification

Run from `/root/zyanya-audit`:

```bash
cargo check --workspace --all-targets
```

Then, to exercise the changed code paths (recommended, may be slow):

```bash
cargo test -p zyanya-math -p zyanya-muhash -p zyanya-consensus
```

**Acceptance criteria:**
1. `cargo check --workspace --all-targets` exits 0 with no new warnings/errors.
2. No remaining non-debug `assert!`/`assert_eq!` in `U3072::mul` (u3072.rs) or in
   `check_blue_candidate_with_chain_block` (protocol.rs).
3. `math/src/uint.rs` exposes `checked_add`, `checked_add_u64`, `checked_sub`, `checked_mul`,
   `checked_mul_u64` and the `Add`/`Sub`/`Mul`/`Add<u64>`/`Mul<u64>` operators no longer use
   `debug_assert!` for overflow detection.
4. Existing unit tests in `math` (e.g. `test_overflow_bug`, `test_saturating_ops`),
   `crypto/muhash` (`test_mul`, `test_inverse`, `test_mul_div`, `test_mul_max`), and
   `consensus` difficulty/ghostdag tests still pass.

---

## 6. Risks & rollback

- **F-H-09 is the highest-risk change** because `math` is shared across consensus, wallet, and
  crypto. If `cargo check`/`cargo test` reveals a reachable overflow that now panics, do **not**
  paper over it — report it back; the correct response is to fix the caller to use
  `checked_*`/`overflowing_*` explicitly, not to revert to silent wrapping.
- The three changes are independent and can be committed separately (one commit per finding is
  recommended for clean review).
- No consensus rule changes are introduced: `debug_assert!` is a no-op in release, and the
  `checked_*`/panic changes only alter behavior on overflow, which the audit confirms is
  unreachable on valid inputs.

---

## 7. Suggested commit structure

1. `fix(muhash): convert U3072::mul invariants to debug_assert! (F-H-08)`
2. `fix(math): add checked_* API and panic on Uint arithmetic overflow (F-H-09)`
3. `fix(consensus): convert GhostDAG blue-anticone sanity check to debug_assert! (F-H-10)`
