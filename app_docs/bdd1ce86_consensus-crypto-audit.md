# Document — Phase 2 Consensus + Crypto Security Audit

## What changed

This session executed a deep line-level security audit of the Zyanya blockchain's consensus and cryptographic modules (31,000+ lines across 11 module groups) and produced a findings report. No source code was modified — the work is audit and report only.

### Primary deliverable

**`audit_reports/phase2_findings.md`** (775 lines) — 17 numbered security findings plus 13 verified-safe items:

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 3 | C-01, C-02, C-03 |
| HIGH | 4 | H-01, H-02, H-03, H-04 |
| MEDIUM | 5 | M-01, M-02, M-03, M-04, M-05 |
| LOW | 5 | L-01, L-02, L-03, L-04, L-05 |

### Secondary deliverables

- **`audit_reports/phase1_findings.md`** (685 lines) — Phase 1 audit of custom Zyanya modules (RPC fallback bypass, contract state store, custody accounting). Included in this diff because it shipped alongside Phase 2.
- **`specs/bdd1ce86_phase2-consensus-crypto-audit.md`** (443 lines) — the audit plan/spec: 23 file/module scope entries, 16 prioritized finding items to validate, and a report skeleton.

### Session infrastructure

The diff also adds ADWS session metadata for two sessions:
- **`bdd1ce86`** — the Phase 2 audit session (planner → builder → reviewer pipeline with `ollama/deepseek-v4-pro:cloud` and `ollama/glm-5.2:cloud` models).
- **`e167222d`** — the Phase 1 audit session (builder + reviewer).

These include agent maps, envelope JSONs, prompt files, raw output logs, event logs, and a SQLite database update. They are operational artifacts, not audit content.

---

## Key findings and why they matter

### CRITICAL findings (Zyanya-specific, require immediate remediation)

1. **C-01 — `f64::powf` in consensus-critical subsidy schedule** (`consensus/src/processes/coinbase.rs:88-95`): Mainnet computes a 727-entry subsidy table using `f64::powf`, which is not guaranteed bit-identical across platforms/libm. A 1-ULP difference can flip `.round()` to a different integer → two nodes compute different subsidy tables → consensus fork on mainnet launch. Non-mainnet correctly uses a pre-computed integer table. **Fix**: replace with a constant integer table or fixed-point recurrence; add cross-platform golden-vector tests.

2. **C-02 — Coinbase output-count limit too small for 13× vesting split** (`consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs:46-48`): The limit is `(ghostdag_k + 2) * 13 = 260` on mainnet, but the coinbase emits 13 outputs per DAA-window blue, and the number of such blues is bounded by `mergeset_size_limit = 180` — not `ghostdag_k`. A block with > 19 DAA-window blues produces a coinbase with > 260 outputs that every node rejects → chain halt. **Fix**: derive limit from `(mergeset_size_limit + 1) * 13 = 2353`.

3. **C-03 — Transaction mass calculation overflow** (`consensus/core/src/mass/mod.rs:87-105`): `calc_tx_compute_mass` uses plain `*` and `+` (wrapping in release). With mainnet `max_tx_inputs/outputs = 1e9`, the estimated size can reach ~10^18 bytes, and the mass sum can overflow u64 → mass wraps to a small value → block-mass limit bypass. **Fix**: use `checked_*`/`saturating_*` arithmetic; lower `max_tx_*` limits.

### HIGH findings

4. **H-01 — Contract payload validation gaps** (`tx_validation_in_isolation.rs:157-200`): No bounds on `bytecode.len()`, `parameters.len()`, or `deposit_amount`. Borsh deserialization of untrusted `tx.payload` with a `0xFFFFFFFF` length prefix attempts multi-GB allocation → OOM DoS. **Fix**: add explicit max bounds to consensus params; bound `tx.payload` before Borsh decode.

5. **H-02 — Muhash non-debug `assert!` invariants** (`crypto/muhash/src/u3072.rs:123,143,144`): Three `assert_eq!`/`assert!` in `mul` execute in release builds. If an invariant is violated (e.g. via a crafted serialized MuHash), the node panics → remote DoS. **Fix**: prove unreachability or convert to `debug_assert!` + error return.

6. **H-03 — `debug_assert!` overflow in Uint arithmetic** (`math/src/uint.rs:533-582`): `Add`/`Sub`/`Mul` for `Uint192/256/320/3072` silently wrap in release. No currently-reachable consensus path triggers overflow, but the pattern is fragile. **Fix**: use `overflowing_*`/`checked_*` in consensus-critical paths.

7. **H-04 — GhostDAG non-debug `assert!`** (`consensus/src/processes/ghostdag/protocol.rs:220`): `assert!(... <= k)` runs in release; a maliciously constructed DAG could panic the node. **Fix**: convert to `debug_assert!` + `RuleError`.

### MEDIUM / LOW findings

- **M-01**: Unsafe `MaybeUninit` + `transmute` in PoW matrix `heavy_hash` (upstream, sound but fragile).
- **M-02**: Unsafe `from_raw_parts` in `ContractStorageKey::as_ref` (Phase 1 finding 17, still open).
- **M-03**: Unsafe `str::from_utf8_unchecked` in hex display impls (upstream, sound).
- **M-04**: Bech32 `conv5to8` truncation — verified safe for well-formed addresses.
- **M-05**: PoW `compute_rank` uses `f64` Gaussian elimination — verified exact for matrix entries 0..15.
- **L-01**: `TooManyOutputs` error reports `inputs.len()`/`max_tx_inputs` instead of `outputs.len()`/`max_tx_outputs` (copy-paste bug).
- **L-02**: Merkle `next_power_of_two` overflow — verified safe (OOM first).
- **L-03**: `SUBNETWORK_ID_SMART_CONTRACT` not in `is_builtin()` — by design.
- **L-04**: Unchecked `red_reward` accumulation — safe in practice but should use `checked_add`.
- **L-05**: `Vec::with_capacity` capacity hint — supports C-02 (code expects up to 2353 outputs).

### Verified-safe items (13)

PoW target comparison, `from_compact_target_bits`, difficulty calculation, `calc_work`, script VM opcode safety (all limits enforced, arithmetic checked, disabled opcodes, multisig bounds, conditional stack balance), Bech32 decoding, Muhash `data_to_element`, Merkle root, sighash correctness, header validation ordering, coinbase vesting value preservation, CSV lock semantics, Keccak F1600 assembly.

---

## Files carrying this work

| File | Lines | Purpose |
|------|-------|---------|
| `audit_reports/phase2_findings.md` | 775 | Phase 2 audit report — the primary deliverable |
| `audit_reports/phase1_findings.md` | 685 | Phase 1 audit report (custom Zyanya modules) |
| `specs/bdd1ce86_phase2-consensus-crypto-audit.md` | 443 | Audit plan/spec with 23 scope entries and 16 finding items |
| `adws/adw_data/sessions/bdd1ce86/**` | ~7,000 | Session metadata: planner/builder/reviewer prompts, envelopes, logs |
| `adws/adw_data/sessions/e167222d/**` | ~8,500 | Phase 1 session metadata |
| `adws/adw_data/sssf.db*` | binary | SQLite session store |

---

## How to verify

1. **Read the report**: `audit_reports/phase2_findings.md` — findings are sorted CRITICAL → HIGH → MEDIUM → LOW, each with `file:line`, code snippet, and recommended fix.

2. **Spot-check line references**: the reviewer verified 7 findings against source. To re-verify:
   ```bash
   # C-01: f64::powf in coinbase
   sed -n '86,95p' consensus/src/processes/coinbase.rs
   # C-02: output limit
   sed -n '46,48p' consensus/src/processes/transaction_validator/tx_validation_in_isolation.rs
   # C-03: mass overflow
   sed -n '87,105p' consensus/core/src/mass/mod.rs
   # H-02: muhash asserts
   sed -n '123p;143p;144p' crypto/muhash/src/u3072.rs
   ```

3. **Check the plan**: `specs/bdd1ce86_phase2-consensus-crypto-audit.md` lists all 23 scope entries and 16 finding items — all 16 are covered in the report (verified by the reviewer's requirement-by-requirement table).

4. **Reviewer verdict**: `adws/adw_data/sessions/bdd1ce86/context_handoff/review.md` — APPROVED, all 16 plan items met with accurate line references.

5. **No source modifications**: `git diff` should show only `audit_reports/`, `specs/`, and `adws/` files — no `consensus/` or `crypto/` source changes.

---

## Next steps

The builder's notes for the next agent indicate Phase 3 should audit:
- The VM (`zyanya-vm`) — smart contract execution engine
- P2P network layer
- RPC endpoints

Top 3 CRITICAL findings requiring immediate code fixes before mainnet launch:
1. C-01: Replace `f64::powf` with integer table
2. C-02: Fix coinbase output limit to `(mergeset_size_limit + 1) * 13`
3. C-03: Add overflow checks to mass calculation

Source code must not be modified during audit phases — fixes are tracked as findings for a remediation phase.