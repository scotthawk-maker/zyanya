# Phase 5 Security Audit — Explorer + Utils + Remaining Modules

**Session:** `9eec6464`
**Date:** 2026-08-18
**Deliverable:** `audit_reports/phase5_findings.md` (1,051 lines, 51 findings)
**Spec:** `specs/9eec6464_phase5-explorer-utils-audit.md`

---

## What Changed and Why It Matters

This session produced a comprehensive Phase 5 security audit of all Zyanya
blockchain modules not covered in Phases 1–4: the block explorer (web/HTTP
attack surface), database layer, daemon/node entry points, math/utils, indexes,
notification system, mining (block template + mempool), metrics, WASM bindings,
CLI, simulation, rothschild, and integration tests.

The audit identified **51 findings** across five severity tiers:

| Severity | Count | Scope |
|----------|-------|-------|
| CRITICAL | 3 | Stored XSS chain via token metadata (C-01, C-02, C-03) |
| HIGH | 4 | Bonding-curve integer overflow (H-01, H-02), reflected XSS (H-03), ungated compile endpoint (H-04) |
| MEDIUM | 10 | Unbounded channels DoS, unsafe DB pointer cast, unvalidated file uploads, /tmp fallback, pagination overflow, DB open panics, unbounded ram_scale |
| LOW | 20 | Unsafe `from_utf8_unchecked` in math/WASM, panics on misconfiguration, non-atomic writes, missing security headers, plaintext private key logging, incomplete secret zeroization |
| Info | 14 | Confirmed-safe areas (path traversal protection, RBF correctness, orphan pool bounds, serde deserializers, mining template delegation, math overflow correctness) |

### Most Critical Finding: Stored XSS Chain (C-01 + C-02 + C-03)

The three CRITICAL findings form a single exploitable chain:

1. **C-03** — `submit_signed_tx` (`client.rs:1180-1245`) deserializes a
   `SignableTxData` struct from client-supplied hex. Schnorr signatures cover
   only `data.tx` and `data.entries`, **not** the metadata fields (`name`,
   `symbol`, `description`, `twitter`, `telegram`, `website`, `icon_uri`).
   An attacker can tamper metadata freely after receiving an unsigned tx.

2. **C-01** — Tampered metadata (e.g. `twitter:
   '"><img src=x onerror=alert(document.cookie)>'`) is saved server-side via
   `save_token_metadata()` and then rendered into `innerHTML` in the token
   page's social links (`web.rs:3334-3339`), executing arbitrary JS in every
   visitor's browser.

3. **C-02** — Token `name`/`symbol` are likewise rendered unescaped into the
   token list table (`web.rs:1382-1383` → `innerHTML` at 1389), affecting all
   explorer visitors.

### Key Non-Obvious Finding: No Stratum Server Exists

The prompt's "Stratum protocol vulnerabilities" focus area is **N/A** — there
is no Stratum server in `mining/src/`. The only `stratum` references are a doc
comment in `consensus/pow/src/wasm.rs:77` and marketing text in the explorer's
embedded HTML referencing an external `zyanya-pool` binary not present in this
repo. This is documented as I-01. The audit instead covered block-template
building, mempool management, and the `get_block_template` RPC path.

### Notable N/A / Safe Areas (Informational)

- **I-02/I-03** — `token_icon_handler` uses `Path::file_name()` and
  `brand_asset_handler` uses a static match: **no path traversal**.
- **I-05** — Mining block template builder delegates all coinbase/mass/fee
  arithmetic to the consensus layer; no direct arithmetic vulnerabilities.
- **I-06** — Math `overflowing_*` implementations reviewed and found correct.
- **I-13** — RBF implementation correctly validates all double-spends before
  removing any transaction.
- **I-14** — Orphan pool is bounded by `maximum_orphan_transaction_count` (500)
  and per-orphan mass limits.

---

## Files That Carry the Work

### Primary deliverable

| File | Lines | Description |
|------|-------|-------------|
| `audit_reports/phase5_findings.md` | 1,051 | Full audit report with 51 findings (3 CRITICAL, 4 HIGH, 10 MEDIUM, 20 LOW, 14 Info) |

### Planning / review artifacts

| File | Description |
|------|-------------|
| `specs/9eec6464_phase5-explorer-utils-audit.md` | Audit spec with per-file vulnerability categories, pre-verified leads, and prioritized attack surfaces |
| `adws/adw_data/sessions/9eec6464/context_handoff/plan.md` | Detailed plan covering all 23 module groups with specific file:line leads |
| `adws/adw_data/sessions/9eec6464/context_handoff/review.md` | Second-pass review: APPROVED with notes — all 15 plan priority areas met |

### Source files audited (not modified — audit only)

- `zyanya-explorer/src/web.rs` (5,881 lines) — ~70 `innerHTML` sinks, XSS vectors
- `zyanya-explorer/src/client.rs` (1,572 lines) — bonding curve overflow, metadata injection
- `zyanya-explorer/src/api.rs` (592 lines) — ungated endpoints, pagination overflow
- `zyanya-explorer/src/main.rs` (127 lines) — no CORS, no rate limiting, no security headers
- `database/src/` (1,597 lines) — unsafe pointer cast, DB open panics, delete_db expect
- `zyanyad/src/` (1,288 lines) — unbounded ram_scale, network() panic
- `daemon/src/` (834 lines) — expect() on missing daemons
- `math/src/` (1,560 lines) — unsafe from_utf8_unchecked in Display/LowerHex
- `utils/src/` (3,213 lines) — hex.rs unwrap, networking.rs unwrap
- `notify/src/` (5,980 lines) — unbounded channels, DoS vectors
- `mining/src/` (8,438 lines) — UnboundedSender, topological sort assert, orphan eviction
- `wasm/core/src/` (506 lines) — unsafe impl Send, from_utf8_unchecked
- `cli/src/` + `node-cli/src/` (5,745 lines) — incomplete secret zeroization
- `rothschild/src/main.rs` (566 lines) — plaintext private key logging
- `metrics/core/src/`, `simpa/src/`, `testing/integration/src/` — reviewed, found safe

---

## How to Use or Verify

### Reading the report

Open `audit_reports/phase5_findings.md`. Findings are severity-ordered
(CRITICAL → HIGH → MEDIUM → LOW → Info) with stable IDs (`C-01`, `H-01`, …).
Each finding includes:
- **File:line** reference
- **Description** of the vulnerability
- **Code snippet** from the actual source
- **Recommended fix** with concrete code examples

### Verifying findings

Every `file:line` was verified against the current source tree. The review
(`review.md`) confirms all 15 plan priority areas were met. Seven line-number
references were off by 6–50 lines (see review.md accuracy table) — the
vulnerability descriptions and code snippets are accurate in all cases.

To verify a finding, e.g. C-01:
```bash
sed -n '3330,3345p' zyanya-explorer/src/web.rs  # socialsHtml innerHTML
sed -n '1220,1240p' zyanya-explorer/src/client.rs  # save_token_metadata
```

### Prioritized remediation order

1. **C-01/C-02/C-03** (stored XSS chain) — escape all metadata before
   `innerHTML`, sign the full `SignableTxData` payload or store metadata
   server-side at unsigned-tx build time.
2. **H-01/H-02** (bonding curve overflow) — use `u128` intermediates for
   `2 * S * k + k * k` and `2 * S * k - k * k`.
3. **H-03** (reflected XSS) — use `textContent` for error display.
4. **H-04** (ungated compile) — add `check_write_enabled()` gate.
5. **M-03** (unbounded channels) — switch to bounded channels with backpressure.
6. Remaining MEDIUM/LOW findings in severity order.

---

## Review Notes

The builder's envelope inaccurately reported "No specific task was provided"
with empty artifacts/changed_files, but the deliverable exists on disk and is
comprehensive. The review (second pass) verified all 15 plan priority areas and
marked the audit APPROVED. Minor gaps (non-blocking):
- `access.rs`/`set_access.rs` `delete_range` bounds not explicitly classified
- `async_threads`/`max_tracked_addresses` upper-bound enforcement not verified
- `address/tracker.rs:367` counter underflow panic not mentioned
- Shell command construction in `cli/src/modules/rpc.rs`, `modules/node.rs` not
  analyzed
- Seven line-number references off by 6–50 lines (content correct)