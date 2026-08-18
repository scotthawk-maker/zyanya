# Phase 3 — Wallet Core Security Audit

## What changed and why it matters

Phase 3 of the Zyanya security audit completed a deep, line-level review of the
wallet core library (~28,000 lines across `wallet/core`, `wallet/keys`,
`wallet/bip32`, `wallet/psst`, `wallet/macros`, `wallet/wasm`, and
`wallet/native`). This is upstream Kaspa / rusty-spectre wallet code renamed to
`zyanya-*` crates, with a small set of Zyanya-specific modifications (BIP32
extended-key prefixes `kprv`/`kpub`/`ktrv`/`ktub`, account-kind string
constants, the Go-wallet import path in `compat/gen1.rs`, and the
`zyanya:`/`zyanyatest:` address prefixes).

The audit produced a single findings report with **32 findings** spanning
private-key handling, BIP32 derivation, mnemonic handling, Schnorr/BIP-340
signing correctness, multisig wallet security, PSBT parsing/injection, UTXO
selection and fee estimation, address generation, key zeroization, and
concurrency. Findings are ordered by severity (CRITICAL → HIGH → MEDIUM → LOW)
with stable IDs so later audit phases can reference them.

The headline finding (C-01) is that `minimum_signatures == 0` is never rejected
in `create_address` / `create_multisig_address`, and the txscript
`multisig_redeem_script` also does not reject `required == 0`, producing a
0-of-N multisig address that is spendable by anyone. Because
`Payload::minimum_signatures` is deserialized from wallet storage with no
validation, a crafted wallet file can trigger this and make funds immediately
stealable. The HIGH tier covers mnemonic/private-key `Debug` leaks (H-01),
world-readable wallet file permissions (H-02), a double
`transaction_fees += change_output_value` over-count / underpayment bug (H-03),
`ExtendedPrivateKey` lacking `Drop`/`Zeroize` (H-04), and `Decrypted<T>`
deriving `Debug`/`Clone` with no zeroization (H-05). The MEDIUM and LOW tiers
address panics reachable from malformed/corrupted input (DoS), a lock-order
inversion deadlock, non-zeroized signer key caches, a deterministic Argon2 salt,
PSBT deserialization with no size limits and silent partial-sig overwrite,
hardcoded mass/fee in `psst_to_pending_transaction`, multisig signature
ordering, and several `unsafe`/overflow/panic issues.

This matters because the wallet core is the component that handles user key
material and constructs and signs transactions — bugs here directly translate
into key leakage, fund loss, or denial of service. The report gives the
engineering team a prioritized, verified remediation list.

## Files that carry the work

| File | Purpose | Size |
|------|---------|------|
| `audit_reports/phase3_findings.md` | The deliverable: 32 findings with severity, file:line, description, verbatim code snippet, and recommended fix, plus the Zyanya-vs-upstream analysis and BIP32/BIP39 correctness spot-checks. | ~39 KB / 842 lines |
| `specs/e810c997_phase3-wallet-core-audit.md` | The audit plan/spec: scope, priority tiers, 26 pre-identified findings to validate, execution steps, and report skeleton. Identical content to `context_handoff/plan.md`. | ~516 lines |
| `adws/adw_data/sessions/e810c997/context_handoff/plan.md` | Planner output — the structured plan the builder executed. | ~516 lines |
| `adws/adw_data/sessions/e810c997/context_handoff/review.md` | Reviewer verdict (APPROVED): requirement-by-requirement verification table, spot-check accuracy notes, and confirmation that all 26 plan items plus 3 additional findings are covered. | ~79 lines |

The remaining changed files in the diff are session metadata
(`agent_map.json`, `envelope.json`, `events.jsonl`, `pi_sessions/`,
`raw_output.jsonl`, `prompts/`, `command.log`) and the `sssf.db` session
database — they record the orchestration run, not the audit content itself.

## How to use or verify it

1. **Read the report**: open `audit_reports/phase3_findings.md`. Start with the
   CRITICAL finding (C-01) and work down. Each finding is self-contained with
   its own severity, location, description, code snippet, and fix.
2. **Verify a finding**: the report cites exact `file:line` locations. Open the
   cited source file at that line and confirm the snippet. The reviewer
   independently spot-checked 8 findings against the tree and found line
   numbers accurate within ±2 lines and snippets verbatim. Example checks:
   - C-01: `wallet/core/src/derivation.rs` around line 70 — confirm only
     `length < minimum_signatures` is checked, no `== 0` guard; also
     `crypto/txscript/src/standard/multisig.rs:18` for the redeem-script path.
   - H-03: `wallet/core/src/tx/generator/generator.rs` lines ~797 and ~809 —
     confirm `transaction_fees += change_output_value;` appears twice.
   - M-03: `wallet/keys/src/derivation/gen0/hd.rs` lines ~124 and ~162 —
     confirm `derive_pubkey_range` locks cache→inner while `derive_pubkey`
     locks inner→cache.
3. **Reproduce the plan**: `specs/e810c997_phase3-wallet-core-audit.md` (or the
   identical `context_handoff/plan.md`) lists all 26 pre-identified findings
   and the execution steps; the review at `context_handoff/review.md`
   cross-references each plan item to its finding ID and confirms coverage.
4. **Track remediation**: use the stable finding IDs (`C-01`, `H-01`…`H-05`,
   `M-01`…`M-13`, `L-01`…`L-13`) as ticket references when opening fix issues.
5. **Scope note**: the audit is read-only — no source code was modified. All
   remediation is recommended fixes in the report, not applied patches.

### Finding summary

| Severity | Count | IDs |
|----------|-------|-----|
| CRITICAL | 1     | C-01 |
| HIGH     | 5     | H-01–H-05 |
| MEDIUM   | 13    | M-01–M-13 |
| LOW      | 13    | L-01–L-13 |
| **Total**| **32** | |