# Plan: Synthesize Final Zyanya Blockchain Security Audit Report

## Objective

Read the 5 phase findings reports in `audit_reports/` and produce a single
consolidated, deduplicated, cross-referenced report at
`audit_reports/FINAL_AUDIT_REPORT.md`.

The final report must contain:
1. Executive Summary (totals by severity, top risks, overall posture)
2. Deduplication / cross-reference table
3. All CRITICAL findings with full details (file:line, description, code, fix)
4. All HIGH findings with full details
5. All MEDIUM findings (summarized)
6. All LOW findings (listed)
7. Remediation Roadmap (prioritized action plan)
8. Appendix: Files Audited (by phase)

## Inputs (read all five, in full)

- `audit_reports/phase1_findings.md` — Custom Zyanya code (VM, contracts, RPC service, wallet ops, explorer)
- `audit_reports/phase2_findings.md` — Consensus + Crypto
- `audit_reports/phase3_findings.md` — Wallet Core
- `audit_reports/phase4_findings.md` — RPC + Protocol + Network
- `audit_reports/phase5_findings.md` — Explorer + Utils + remaining modules

## Output

- `audit_reports/FINAL_AUDIT_REPORT.md` (single file; do NOT modify any phase report or source code)

---

## Deduplication mapping (MUST apply)

Six findings appear in more than one phase. Merge them into single findings and
cross-reference the original phase IDs. Canonical severity = the higher of the two.

| # | Canonical finding | Phase IDs merged | Canonical severity |
|---|-------------------|------------------|--------------------|
| 1 | RPC fallback bypasses consensus, persisting contract state without a mined tx | P1 C-01 + P4 C-01 | CRITICAL |
| 2 | DEX / bonding-curve math overflows u64 (on-chain ZCL + off-chain explorer client) | P1 H-02 + P5 H-01 + P5 H-02 | HIGH |
| 3 | Ungated `/api/compile` endpoint + parser-recursion DoS | P1 H-04 + P5 H-04 | HIGH |
| 4 | Contract payload validation gaps — no size bounds on bytecode/parameters (borsh OOM) | P1 M-05 + P2 H-01 | HIGH |
| 5 | `check_transaction_outputs_count` reports input fields in error (copy-paste) | P1 L-01 + P2 L-01 | LOW |
| 6 | `unsafe` `AsRef<[u8]>` for `#[repr(C)] ContractStorageKey` | P1 L-02 + P2 M-02 | MEDIUM |

For each merged finding, write ONE entry that:
- Uses the most complete description/code/fix from the contributing phases.
- Lists a `Cross-references:` line naming every contributing phase ID.
- Notes any line-number re-verification (e.g. P4 C-01 re-verified P1 C-01 line numbers).

### Related-but-distinct (cross-reference, do NOT merge)

These share a root cause or pattern but are at different files/layers. Keep them
as separate findings and add a `Related:` line pointing at each other:

- P1 H-06 (wallet_ops.rs UTXO unchecked add) ↔ P3 L-07 (utxo/context.rs `sum()` overflow)
- P4 C-02 (RPC unbounded bytecode/calldata) ↔ merged #4 (consensus payload validation)
- P5 M-05 (database registry unsafe cast) ↔ merged #6 (ContractStorageKey unsafe cast)
- P1 M-03 (parser recursion depth) ↔ merged #3 (ungated compile endpoint)
- P4 C-05 (no RPC auth) ↔ merged #1 (RPC consensus bypass)
- P4 C-04 (1 GB message size) ↔ P4 H-09 (inconsistent wRPC size)
- P4 H-03 (no P2P conn limit) ↔ P4 H-08 (no wRPC conn limit)
- P4 H-05 (tx relay no ban) ↔ P4 M-02 (RequestTransactionsFlow no rate limit)
- P5 C-01/C-02 (stored XSS) ↔ P5 H-03 (reflected XSS) ↔ P5 M-01/M-07 (DAG modal XSS) ↔ P5 L-06 (no security headers)
- P3 H-01 (Debug key leak) ↔ P5 L-19 (Rothschild logs private key) ↔ P3 H-04/H-05 (no zeroize)
- P3 M-02 (gen1 decrypt panics/UB) ↔ P3 L-10 (from_utf8_unchecked in gen1)
- P2 M-03 / P4 M-01 / P5 L-07 / P5 L-15 (unsafe `from_utf8_unchecked` hex/display family)
- P5 M-03 (unbounded notify channels) ↔ P5 L-04 (unbounded mining sender)
- P5 M-06 (/tmp fallback) ↔ P3 H-02 (wallet file perms)
- P5 M-09 (DB conn builder panic) ↔ P5 L-09 (delete_db panic)
- P5 L-10 (network() panic) ↔ P5 L-11 (daemon expect)
- P5 L-12 (topological sort assert) ↔ P5 L-14 (mass assert)
- P5 L-17 (hex.rs unwrap) ↔ P5 L-18 (networking unwrap)
- P5 L-20 (CLI wizard secrets) ↔ P3 H-01/H-04/H-05 (key handling)

---

## Final severity counts (deduplicated)

Raw totals across the 5 phases: **135 findings** (17 CRITICAL, 29 HIGH, 40 MEDIUM, 49 LOW).
After merging the 6 duplicates above (7 raw entries collapse into 6 canonical findings),
the consolidated report contains **128 findings**:

| Severity | Count |
|----------|-------|
| CRITICAL | 16 |
| HIGH     | 26 |
| MEDIUM   | 39 |
| LOW      | 47 |
| **Total**| **128** |

### Canonical CRITICAL list (16) — order to use in the report

1. RPC fallback consensus bypass — P1 C-01 + P4 C-01
2. Failed contract txs still commit state (commit-on-failure) — P1 C-02
3. Bonding-curve custody accounting gap (buy no deduct / sell destroys refund) — P1 C-03
4. No `msg.sender` / no auth in token, bonding-curve, DEX — P1 C-04
5. VM `CALL` no reentrancy guard / no call-depth limit — P1 C-05
6. `f64::powf` in consensus subsidy schedule (fork risk) — P2 C-01
7. Coinbase output-count limit too small for 13× vesting split — P2 C-02
8. Transaction mass calculation overflow → block-mass bypass — P2 C-03
9. `minimum_signatures == 0` → anyone-can-spend multisig — P3 C-01
10. Unbounded bytecode/calldata in contract RPC handlers — P4 C-02
11. Panic vectors: `.try_into().unwrap()` on contract address — P4 C-03
12. 1 GB message size limits across all transports — P4 C-04
13. No authentication on any RPC method — P4 C-05
14. Stored XSS via token metadata social links — P5 C-01
15. Stored XSS via token name/symbol in token list — P5 C-02
16. Unsigned tx metadata injection (signatures don't cover metadata) — P5 C-03

### Canonical HIGH list (26) — order to use in the report

1. `POW` opcode uses `wrapping_pow` (silent overflow) — P1 H-01
2. DEX/bonding-curve math overflows u64 (on-chain + off-chain) — P1 H-02 + P5 H-01 + P5 H-02
3. Jump-target byte-offset vs opcode-index confusion — P1 H-03
4. Ungated `/api/compile` + parser-recursion DoS — P1 H-04 + P5 H-04
5. Wallet `holder_u64` 8-byte truncation → balance collision/theft — P1 H-05
6. UTXO selection unchecked u64 addition — P1 H-06
7. Contract payload validation gaps (no size bounds / borsh OOM) — P1 M-05 + P2 H-01
8. Muhash `U3072::mul` non-debug `assert!` (panic DoS) — P2 H-02
9. `debug_assert!` overflow checks in `Uint` arithmetic wrap in release — P2 H-03
10. GhostDAG non-debug `assert!` sanity check (panic DoS) — P2 H-04
11. Private key leaked via derived `Debug` impls — P3 H-01
12. Wallet file written with world-readable permissions — P3 H-02
13. Double addition of `change_output_value` to `transaction_fees` — P3 H-03
14. `ExtendedPrivateKey` derives `Clone`, no `Drop`/`Zeroize` — P3 H-04
15. `Decrypted<T>` derives `Debug`/`Clone`, no `Drop`/`Zeroize` — P3 H-05
16. P2P `user_agent` not bounded on receive — P4 H-01
17. P2P timestamp `i64`→`u64` cast + time_offset wraparound — P4 H-02
18. No P2P connection limit / per-IP rate limiting — P4 H-03
19. Peer identity spoofing (self-declared PeerId, duplicate eviction) — P4 H-04
20. Tx relay: no ban/disconnect for spam peers — P4 H-05
21. Address manager poisoning (no source validation) — P4 H-06
22. Address manager `is_banned` u64 underflow (ban bypass) — P4 H-07
23. wRPC server no connection limit — P4 H-08
24. Inconsistent wRPC message size limits (proxy 1 GB vs server 128 MB) — P4 H-09
25. gRPC conversion `expect()`/`unwrap()` on untrusted input — P4 H-10
26. Reflected XSS via API error messages — P5 H-03

### Canonical MEDIUM list (39) — summarize each in 1–3 sentences

Phase 1 (4): M-01 invalid keys coerced to 0/1; M-02 gas saturating math + failing CALL burns all forwarded gas; M-03 parser recursion unbounded; M-04 `trim_start_matches("0x")` over-strips.
Phase 2 (5): M-01 `MaybeUninit`+`transmute` in PoW heavy_hash; M-02 ContractStorageKey unsafe AsRef (merged #6); M-03 unsafe `from_utf8_unchecked` hex display; M-04 bech32 `conv5to8` truncation (VERIFIED SAFE — note as such); M-05 PoW `compute_rank` f64 (VERIFIED SAFE — note as such).
Phase 3 (13): M-01 decrypt panics on <24-byte ciphertext; M-02 gen1 import panics/UB; M-03 lock-order inversion deadlock; M-04 signer key caches never zeroized; M-05 div-by-zero in mass calc; M-06 Mnemonic WASM panics/inconsistent state; M-07 PSBT/PSSB no size limits + partial-sig overwrite; M-08 pssb_signer panics; M-09 psst_to_pending hardcoded mass/fee + panics; M-10 multisig sig ordering mismatch; M-11 sign_with_multiple_v2 panics; M-12 `Secret` derives Clone/serializable; M-13 deterministic Argon2 salt.
Phase 4 (7): M-01 unsafe `from_utf8_unchecked` hex; M-02 RequestTransactionsFlow no rate limit; M-03 RandomWeightedIterator panic; M-04 wRPC disconnect poisoned-mutex unwrap; M-05 wRPC notification serialization unwrap; M-06 RPC error info leak; M-07 MAX_INV_PER_TX_INV_MSG = 131,072.
Phase 5 (10): M-01 reflected XSS DAG modal; M-02 no CORS on explorer API; M-03 unbounded notify channels; M-04 token icon upload no validation; M-05 unsafe pointer cast in database registry; M-06 /tmp fallback for metadata; M-07 reflected XSS DAG modal error; M-08 DAG `limit + offset` usize overflow; M-09 DB conn builder panics; M-10 unbounded `ram_scale` f64.

### Canonical LOW list (47) — one line each

Phase 1 (3): L-01 outputs-count error (merged #5); L-03 assembler label-on-same-line mis-parse + numeric JUMP byte offset; L-04 stack bounds OK / cross-call depth unbounded (see C-05).
Phase 2 (4): L-02 merkle `next_power_of_two` overflow (verified safe); L-03 `is_builtin` excludes smart-contract subnet (by design); L-04 coinbase `red_reward` unchecked add; L-05 `with_capacity` hint only (supports C-02).
Phase 3 (13): L-01 Fees::try_from maps invalid to 0; L-02 DerivationPath no length limit; L-03 ChildNumber from_bytes/From<u32> skip validation; L-04 ExtendedKey::from_str no prefix/version cross-check; L-05 `static mut` storage globals not thread-safe; L-06 utxo/context panic/unreachable; L-07 calculate_balance plain sum overflow; L-08 create_xpub/build_derivate_path panic; L-09 sig_op_count panics >255 keys; L-10 from_utf8_unchecked gen1/test; L-11 Encrypted Debug prints ciphertext; L-12 From<NetworkId> maps all non-mainnet to KTUB; L-13 DerivationPath expecting message copy-paste.
Phase 4 (7): L-01 TPS throttle overflow; L-02 broadcast_to_some_peers assert; L-03 Router::enqueue assert; L-04 connection_failed_count +1 overflow; L-05 hub_sender.send().expect(); L-06 server panic on serve error; L-07 DaaScoreTimestampEstimate div-by-zero.
Phase 5 (20): L-01 unsafe impl Send for WASM Sink; L-02 no rate limiting explorer; L-03 non-atomic metadata write; L-04 unbounded sender mining; L-05 no uacomment validation; L-06 no security headers; L-07 unsafe from_utf8_unchecked WASM hex; L-08 DbKey Display bounds; L-09 delete_db panic; L-10 network() panic; L-11 daemon expect; L-12 topological sort assert; L-13 orphan eviction not random; L-14 mass assert; L-15 unsafe from_utf8_unchecked math uint; L-16 Display LUT indexing; L-17 hex.rs unwrap; L-18 networking unwrap; L-19 Rothschild logs private key; L-20 CLI wizard secrets not zeroized.

---

## Report structure (write in this exact order)

### Title + metadata block
- Title: `Zyanya Blockchain — Consolidated Security Audit Report`
- Date: 2026-08-18
- Scope: all 5 phases; note this is a synthesis of `phase1_findings.md` … `phase5_findings.md`.
- Methodology note: source was NOT modified; line numbers re-verified per phase; note the
  phase-2 count discrepancy (see Notes below).

### 1. Executive Summary
- Deduplicated totals table (16/26/39/47 = 128) AND raw totals table (17/29/40/49 = 135)
  so the reader sees both.
- Top risks (thematic, 5 bullets):
  1. Consensus integrity (RPC bypass, commit-on-failure, f64 subsidy fork, coinbase limit, mass overflow)
  2. Smart-contract / VM security (no msg.sender, reentrancy, custody accounting, math overflow, jump targets)
  3. Network / RPC DoS (1 GB messages, no auth, unbounded payloads, no connection limits)
  4. Wallet key management (Debug leak, file perms, 0-of-N multisig, no zeroize, fee double-add)
  5. Explorer web security (stored/reflected XSS chain, metadata injection, no CORS/headers)
- Overall posture: fork of Kaspa/rusty-spectre; upstream code largely sound, but Zyanya-specific
  additions (VM/contracts, bonding curve, DEX, explorer) introduce the majority of CRITICALs.
  Verdict: **NOT production-ready** — all 16 CRITICALs must be fixed before any mainnet launch.

### 2. Deduplication & Cross-Reference Table
- Render the 6-row merge table from above (canonical finding → phase IDs → severity).
- Add a short "related-but-distinct" list (the cross-reference pairs above).

### 3. CRITICAL Findings (full details)
- 16 entries in the canonical order above.
- Each entry MUST include: canonical ID (e.g. `F-C-01`), title, severity, `File:` (file:line),
  `Cross-references:` (phase IDs), `Description:`, `Code:` (verbatim snippet from the phase
  report), `Recommended fix:`.
- For merged findings, combine the best code/fix from both phases and list both phase IDs.

### 4. HIGH Findings (full details)
- 26 entries, same per-entry format as CRITICAL (ID `F-H-01` … `F-H-26`).

### 5. MEDIUM Findings (summarized)
- 39 entries, one short paragraph each: ID (`F-M-01`…), title, file:line, 1–3 sentence summary,
  fix in one sentence. Mark P2 M-04 and P2 M-05 as "VERIFIED SAFE — no action".

### 6. LOW Findings (listed)
- 47 entries, one line each: ID (`F-L-01`…), title, file:line, one-line fix.

### 7. Remediation Roadmap
- Prioritized action plan. Suggested structure:
  - **P0 — Block mainnet (CRITICAL, 16):** grouped into 5 workstreams with the specific IDs:
    - Consensus integrity: F-C-01, F-C-02, F-C-06, F-C-07, F-C-08
    - VM/contract: F-C-03, F-C-04, F-C-05
    - RPC/network: F-C-10, F-C-11, F-C-12, F-C-13
    - Wallet: F-C-09
    - Explorer: F-C-14, F-C-15, F-C-16
  - **P1 — High priority (HIGH, 26):** grouped by theme (VM math, consensus hardening, network, wallet, explorer).
  - **P2 — Medium (39)** and **P3 — Low (47):** hardening backlog.
  - Add a short "definition of done" per workstream (e.g. regression test + cross-platform golden vector for the f64 subsidy fix).

### 8. Appendix A: Files Audited (by phase)
- List the modules/files covered in each phase (copy the scope lines from each phase report's header).

### 9. Appendix B: Verified-Safe / Informational items
- P2 H-05 (PoW-before-GhostDAG ordering — verified safe), P2 M-04, P2 M-05, and the P2
  "Verified Safe" list (13 items), plus P5 I-01 … I-14 (informational). One line each.

---

## Canonical ID scheme

- CRITICAL: `F-C-01` … `F-C-16`
- HIGH: `F-H-01` … `F-H-26`
- MEDIUM: `F-M-01` … `F-M-39`
- LOW: `F-L-01` … `F-L-47`

Every finding keeps its original phase ID(s) in a `Cross-references:` (or `Source:`) line so
the reader can trace back to the phase report.

---

## Notes & discrepancies to record in the report

1. **Phase 2 count discrepancy:** the task brief says phase 2 has "18 findings (3 CRITICAL,
   5 HIGH, 5 MEDIUM, 5 LOW)", but the phase-2 report's own summary table lists 17
   (3 CRITICAL, 4 HIGH, 5 MEDIUM, 5 LOW) because its H-05 is "Verified safe (no finding)".
   Use the report's actual numbers (4 HIGH) and note this in the methodology.
2. **Phase 2 M-04 and M-05** are listed under MEDIUM but marked "verified safe / no change".
   Keep them in the MEDIUM section with an explicit "VERIFIED SAFE" tag; do not silently drop them.
3. **Phase 5** has 3 CRITICAL, 4 HIGH, 10 MEDIUM, 20 LOW, plus 14 Informational (I-01…I-14).
   The informational items go in Appendix B, not in the severity counts.
4. Preserve verbatim code snippets and `file:line` references from the phase reports — do not
   paraphrase code or invent line numbers.

## Execution steps for the builder

1. Read all 5 phase reports in full (they are already in `audit_reports/`).
2. Build the merged/canonical finding list using the mapping and counts above.
3. Write `audit_reports/FINAL_AUDIT_REPORT.md` following the structure above.
4. For CRITICAL and HIGH, copy the `Code:` and `Recommended fix:` blocks verbatim from the
   phase reports (for merged findings, use the most complete version and note both sources).
5. For MEDIUM, write concise summaries; for LOW, one-liners.
6. Fill in the Remediation Roadmap and both appendices.
7. Do NOT modify any phase report or any source file.
