# Work Orders

Work orders (WO) track discrete units of work on the Zyanya blockchain. Each WO is a numbered task with a clear scope, acceptance criteria, and completion status.

## WO Log

| WO # | Title | Status | Date |
|------|-------|--------|------|
| #18 | Cross-compiled Windows binaries + runtime deps | ✅ Complete | 2026-08-04 |
| #19 | Security audit remediation — all 11 findings | ✅ Complete | 2026-08-05 |
| #20 | GCM credential store fix + wiki initialization | ✅ Complete | 2026-08-05 |

---

## WO #19: Security Audit Remediation

**Status**: ✅ Complete (2026-08-05)
**Commits**: `d02a110`, `0943c9f`

### Scope
Remediate all open findings from the pre-launch security audit (AUDIT.md).

### Findings Resolved

| ID | Severity | Finding | Resolution |
|----|----------|---------|------------|
| CRIT-01 | Critical | SStore operand transposition | False positive — verified |
| HIGH-01 | High | HashMap non-deterministic iteration | BTreeMap (fixed 2026-08-01) |
| HIGH-02 | High | Unchecked VM arithmetic overflow | `checked_add/sub/mul/pow` — all VM math now checked |
| HIGH-03 | High | Secret keys printed to terminal | `--show-secret` flag gates all secret output |
| HIGH-04 | High | Unbounded API collection queries | `PaginationQuery` with `limit.min(100)` + offset |
| MED-01 | Medium | Unbounded Pow gas | Dynamic gas: `1 + exponent/32` |
| MED-02 | Medium | Float precision in wallet | `parse_zyan_to_sompi()` fixed-point decimal parser |
| MED-03 | Medium | Hex parsing in state handler | `from_str_radix(rest, 16)` for `0x`-prefixed keys |
| MED-04 | Medium | Unauthenticated state-changing endpoints | `check_write_enabled()` env gate, disabled by default |
| LOW-01 | Low | Wallet key file permissions | `0o600` on Unix via `PermissionsExt` |
| INFO-01 | Info | Genesis zero-premine | Verified correct |

### Verification
- 19 VM unit tests — all pass
- 8 VM integration tests — all pass
- 50 consensus tests — all pass
- 24 consensus-core tests — all pass

---

## WO #20: GCM Credential Store Fix + Wiki Initialization

**Status**: ✅ Complete (2026-08-05)

### Scope
1. Fix `git push` failure over SSH on Windows host (GCM wincredman incompatibility)
2. Initialize project wiki with documentation
3. Save working GitHub PAT to master secrets

### Root Cause
Windows GCM defaults to `wincredman` (Windows Credential Manager), which requires an interactive desktop session. SSH sessions don't have one. Confirmed by [official GCM docs](https://github.com/git-ecosystem/git-credential-manager/blob/main/docs/credstores.md#windows-credential-manager):

> ⚠️ Does not work over a network/SSH session. When connecting to a Windows machine over a network session (such as SSH), GCM is unable to persist credentials to the Windows Credential Manager due to limitations in Windows.

### Resolution
- `git config --global credential.credentialStore dpapi` on Windows host
- DPAPI-protected files work in SSH sessions (only needs user identity, not desktop session)
- GitHub PAT saved to `~/.pi/agent/auth.json` master secrets
- Wiki docs created in `docs/wiki/` (6 pages: Home, Architecture, Smart Contracts, Deployment, Work Orders, Audit Results)

---

## How to Create a Work Order

1. Number sequentially (next: #21)
2. Create a section in this page with:
   - Title, status, date
   - Scope (what's being done)
   - Acceptance criteria (observable result)
   - Resolution (what was done, commit hashes)
   - Verification (tests run, results)
3. Reference `WO #N` in commit messages