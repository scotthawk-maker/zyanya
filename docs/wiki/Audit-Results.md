# Audit Results

## Pre-Launch Security Audit

**Date**: July 30, 2026
**Auditor**: Antigravity AI Code Audit Team
**Scope**: `zyanya-wallet`, `zyanya-vm`, `consensus`, `zyanya-explorer`, `database`, `zyanyad`
**Full Report**: [AUDIT.md](../../AUDIT.md)

## Final Verdict: PASS — ALL FINDINGS REMEDIATED (2026-08-05)

| Severity | Count | Status |
|----------|-------|--------|
| 🔴 Critical | 1 | ✅ False positive (CRIT-01) |
| 🟠 High | 4 | ✅ All fixed (HIGH-01/02/03/04) |
| 🟡 Medium | 4 | ✅ All fixed (MED-01/02/03/04) |
| 🔵 Low | 2 | ✅ Fixed + assessed (LOW-01/02) |
| ℹ️ Info | 1 | ✅ Verified (INFO-01) |

## Findings Detail

### CRIT-01: SStore Operand Transposition — FALSE POSITIVE
The auditor claimed `SStore` pops operands in wrong order. Verified that compiler codegen pushes `key` first, `val` second — so `val` is on top of stack. VM popping `val` first then `key` is **correct and consistent**. Applying the proposed fix would break 10+ tests.

### HIGH-01: HashMap Non-Deterministic Iteration — FIXED (2026-08-01)
`ContractStateCache` used `HashMap` for `code`/`storage`/`balances`. Switched to `BTreeMap` for deterministic lexicographic iteration order in `commit_cache_batch`. Prevents consensus forks from non-deterministic write ordering.

### HIGH-02: Unchecked VM Arithmetic — FIXED (2026-08-05)
All VM math opcodes (`Add`, `Sub`, `Mul`, `Pow`) now use `checked_*` arithmetic, returning `VMError::ArithmeticOverflow` on overflow instead of silently wrapping. Prevents attacker from underflowing transfer calculations to obtain `u64::MAX` tokens.

### HIGH-03: Secret Keys in Terminal Output — FIXED
All secret key output gated behind `--show-secret` CLI flag. `--generate-key`, `--generate-mnemonic`, `--import-mnemonic`, `--demo` print address only by default.

### HIGH-04: Unbounded API Queries — FIXED
`/api/contracts`, `/api/tokens`, `/api/dag` now accept `PaginationQuery` with `limit` (default 20, max 100) and `offset`. Prevents DoS via large collection queries.

### MED-01: Unbounded Pow Gas — FIXED
`Pow` opcode charges dynamic gas: `extra_gas = 1 + (exponent / 32)` on top of base 8 gas. Prevents resource exhaustion via large exponents.

### MED-02: Float Precision — FIXED
Transaction amount construction uses `parse_zyan_to_sompi()`, a fixed-point decimal parser. No `f64` involved in sompi conversion. Remaining `f64` usage is display-only (safe).

### MED-03: Hex Parsing — FIXED
`api_contract_state_handler` uses `u64::from_str_radix(rest, 16)` for `0x`-prefixed keys, falling back to decimal for plain numbers. `?key=0x10` correctly parses to 16.

### MED-04: Unauthenticated Endpoints — FIXED
All state-changing endpoints gated behind `check_write_enabled()` which checks `ZYANYA_EXPLORER_ENABLE_WRITE` env var. Disabled by default (returns 503). Deprecated `/api/deploy-token` returns 410 GONE.

### LOW-01: Key File Permissions — FIXED
`save_to_file` sets `0o600` permissions on Unix via `std::os::unix::fs::PermissionsExt`.

### LOW-02: RocksDB FD Limits — Assessed Safe
Connection builder uses `fd_budget::acquire_guard` with default parallelism=1. Safe on Linux and Windows.

### INFO-01: Genesis Zero-Premine — Verified
No pre-allocated balances, team wallets, or initial supply in genesis. 50 ZYAN coinbase → `OP_FALSE` unspendable script.