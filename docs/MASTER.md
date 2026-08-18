# Zyanya Blockchain — Master Reference

> **The living manifest.** Updated alongside the wiki as work happens. Every section carries a last-verified timestamp; sections older than 7 days are marked ⚠️ stale.

---

**Last Updated**: 2026-08-05
**Phase**: 4a (AMM Graduation + Fee Routing) — complete
**Test Suite**: 101 tests (19 VM unit + 8 VM integration + 50 consensus + 24 consensus-core) — all passing
**Audit**: PASS — all 11 findings remediated
**Origin/main**: `b7ee069`

---

## Manifest

| Module | Package | Key Path | Status | Last Verified |
|--------|---------|----------|--------|---------------|
| Smart Contract VM | `zyanya-vm` | `zyanya-vm/src/` | ✅ Phase 4a | 2026-08-05 |
| Consensus | `zyanya-consensus` | `consensus/src/` | ✅ Phase 4a | 2026-08-05 |
| Consensus Core | `zyanya-consensus-core` | `consensus/core/src/` | ✅ Phase 3 | 2026-08-05 |
| Wallet | `zyanya-wallet` | `zyanya-wallet/src/` | ✅ Phase 2 | 2026-08-05 |
| Explorer | `zyanya-explorer` | `zyanya-explorer/src/` | ✅ Phase 4a | 2026-08-05 |
| Daemon | `zyanyad` | `daemon/src/` | ✅ Phase 1 | 2026-08-04 |
| Database | `zyanya-database` | `database/src/` | ✅ Phase 1 | 2026-08-04 |
| RPC | `zyanya-rpc` | `rpc/src/` | ✅ Phase 3 | 2026-08-04 |
| Crypto | `zyanya-crypto` | `crypto/src/` | ✅ Phase 1 | 2026-08-04 |
| WASM | `zyanya-wasm` | `wasm/src/` | ✅ Phase 1 | 2026-08-04 |
| Mining | `zyanya-mining` | `mining/src/` | ✅ Phase 1 | 2026-08-04 |

---

## 1. System Overview

**Zyanya** ("forever, always") is a UTXO-based blockDAG blockchain forked from Kaspa/Spectre (rusty-spectre). It features an embedded stack-based smart contract VM, a bonding-curve token launchpad with automatic AMM graduation, and a staking system powered by 0.3% protocol fees.

### Key Facts
- **Consensus**: GhostDAG (parallel block inclusion, topological ordering)
- **Block reward**: 50 ZYAN (5,000,000,000 sompi; 1 ZYAN = 10⁸ sompi)
- **Genesis**: Zero premine — 50 ZYAN sent to unspendable `OP_FALSE` script
- **Coinbase maturity**: 100 blocks
- **Native token**: ZYAN (mainnet), ZYNT (testnet)
- **VM**: Stack-based, 64-bit unsigned integers only (no floats — consensus-safe), gas-metered
- **Explorer**: IPv6-only (socket2 with IPV6_V6ONLY=true)

---

## 2. Network Topology

| Network | Prefix | Default Port | Genesis |
|---------|--------|-------------|---------|
| Mainnet | ZYAN-MAINNET | 18110 | 50 ZYAN → OP_FALSE |
| Testnet | ZYNT-TESTNET | 18210 | 50 ZYAN → OP_FALSE |
| Devnet | — | 18610 | — |

### Testnet Nodes

| Node | OS | Tailscale IP | Role | SSH |
|------|-----|-------------|------|-----|
| cachyos | Linux (CachyOS 7.1.5) | 100.124.134.6 | Build + dev | `localhost` |
| minisforum | Windows 11 | 100.83.211.115 | Build + deploy | `ssh windows` |
| scotthawk | Linux | 100.106.22.123 | Testnet peer | — |

Nodes peer over IPv6 via Tailscale.

---

## 3. Smart Contract VM (`zyanya-vm`)

**Tracking ID**: ZYN-P1-001 (initial), ZYN-P2-001 (custody), ZYN-P4a-001 (Pow fix)
**Last Verified**: 2026-08-05
**Commit**: `d02a110`

### VM Architecture
- Stack-based execution engine
- 64-bit unsigned integer arithmetic (no floats)
- Gas metering with per-opcode base costs
- Persistent contract state via `SLOAD`/`SSTORE`
- Inter-contract calls via `Call` opcode
- **All arithmetic uses checked operations** (HIGH-02 fix) — returns `ArithmeticOverflow` on overflow

### Opcode Set

| Category | Opcodes | Base Gas |
|----------|---------|----------|
| Stack | `NOP`, `HALT`, `PUSH(u64)`, `POP`, `DUP`, `SWAP` | 1-2 |
| Arithmetic | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW` | 3-8 (+ dynamic for Pow) |
| Logic | `AND`, `OR`, `XOR`, `NOT` | 2-3 |
| Comparison | `EQ`, `LT`, `GT`, `LTE`, `GTE` | 3 |
| Control Flow | `JUMP(usize)`, `JUMPIF(usize)` | 4 |
| Memory | `LOAD(usize)`, `STORE(usize)` | 3-4 |
| Storage | `SLOAD`, `SSTORE` | 100 / 500 |
| Inter-Contract | `CALL([u8;32])` | 200 |
| Return | `RETURN` | 1 |

### Pow Dynamic Gas
```
extra_gas = 1 + (exponent as u64 / 32)
```
Charged on top of base 8 gas. Prevents resource exhaustion via large exponents (MED-01 fix).

### ZCL Language
Assembly-like language compiled to VM opcodes. Features: labels, jumps (`JUMPIF :label`), comments (`//`), multi-function contracts with entry-point dispatch.

---

## 4. Contract State Key Map

### Bonding Curve Token (`bonding_curve_token.rs`)

| Key | Meaning | Entry Point |
|-----|---------|-------------|
| 0 | Total Supply | EP 3 (query) |
| 1 | Slope (price curve parameter) | EP 0 (init) |
| 2 | Reserve (ZYAN sompi held by curve) | — |
| 3 | Phase (0=bonding, 1=frozen, 2=AMM) | — |
| 4 | AMM X Reserve (ZYAN side) | EP 7 (swap) |
| 5 | AMM Y Supply (token side) | EP 7 (swap) |
| 6 | AMM K (constant product) | EP 7 (swap) |
| <address> | Holder Balance | EP 2 (balance_of) |

### Entry Points

| EP | Operation | Notes |
|----|-----------|-------|
| 0 | Initialize (set slope) | Deploy only |
| 1 | Transfer | `from → to` |
| 2 | Balance of | Returns holder balance |
| 3 | Total supply | Returns key 0 |
| 4 | Buy (bonding curve) | Cost = `slope * supply² / 2` |
| 5 | Sell (bonding curve) | Return = `slope * (supply² - new_supply²) / 2` |
| 6 | Price query | Returns current buy price |
| 7 | AMM swap | `is_x_to_y=1`: ZYAN→token, `=0`: token→ZYAN |

### Staking Contract (`staking.zcl`)

| Key | Meaning |
|-----|---------|
| 0 | Total Staked (sum of all stakes) |
| 1 | totalRewardsDistributed (receives 0.3% protocol fees) |
| 2+ | Per-staker balance (keyed by address) |

| EP | Operation |
|----|-----------|
| 0 | Initialize |
| 1 | Stake (deposit ZYAN) |
| 2 | Unstake (withdraw ZYAN) |
| 3 | Claim rewards |
| 4 | Get total staked |
| 5 | Get staker balance |
| 6 | Get pending rewards |
| 7 | Deposit rewards (internal — called by fee routing) |
| 8 | Get total rewards distributed |

---

## 5. Economic Model

### Bonding Curve → AMM Graduation

**Tracking ID**: ZYN-P4a-002
**Commit**: `c7eed8d`

```
Phase 0 (Bonding)
  Buy: cost = slope × supply² / 2
  Sell: return = slope × (supply² - new_supply²) / 2
  ↓
  Reserve ≥ 1,000,000,000 sompi (10 ZYAN) after a buy
  ↓
Consensus auto-fires graduation:
  Phase → 2 (AMM)
  x_reserve = reserve
  y_supply = supply
  k = reserve × supply
  ↓
Phase 2 (AMM)
  x_to_y: send ZYAN, receive tokens
  y_to_x: send tokens, receive ZYAN
  Constant product: x × y ≥ k
```

### Fee Routing

- **0.3% of buy cost** → staking contract key 1 (`totalRewardsDistributed`)
- **0.3% of AMM trade value** → staking contract key 1
- Fee routed atomically in the same transaction
- Routed via `ContractProcessor` in consensus, not via VM contract code

### Contract Execution Flow

```
TX (Deploy/Invoke)
  → ContractProcessor.process_contract_tx()
    → VM.execute_stateful(code, contract_address, state_cache)
      → Opcode dispatch loop with gas metering
      → SLOAD/SSTORE against ContractStateCache (BTreeMap)
      → Return value + gas used
    → ContractExecutionOutcome (success/fail, gas, fees)
  → State changes remain in cache
  → Committed atomically to RocksDB on virtual block acceptance
```

### Fee Split (Consensus)
- 50% burned
- 50% to miner

---

## 6. Consensus Architecture

**Tracking ID**: ZYN-P1-002
**Last Verified**: 2026-08-05

### State Cache
- `ContractStateCache` uses `BTreeMap` for `code`, `storage`, `balances` (HIGH-01 fix)
- Deterministic lexicographic iteration order prevents consensus forks
- In-memory during block execution, committed atomically to RocksDB on virtual block acceptance

### RocksDB Column Families

| Column Family | Key | Value | Purpose |
|---------------|-----|-------|---------|
| `cf_contract_code` | ContractAddress (32 bytes) | Bytecode | Contract code |
| `cf_contract_storage` | ContractAddress + StorageKey | StorageValue | Key-value storage |
| `cf_contract_balance` | ContractAddress | u64 (sompi) | Native coin held by contract |
| `cf_contract_meta` | ContractAddress | ContractMetadata | Owner, block height, revision |

---

## 7. Wallet (`zyanya-wallet`)

**Tracking ID**: ZYN-P2-002
**Last Verified**: 2026-08-05

### Key Management
- **Key generation**: `OsRng` + secp256k1 (Schnorr signatures)
- **Mnemonic**: BIP-39 24-word seed phrase with optional passphrase
- **Key file**: `~/.zyanya/wallet.key`, permissions `0600` on Unix (LOW-01 fix)
- **Secret output**: Gated behind `--show-secret` flag (HIGH-03 fix)

### Amount Parsing
- `parse_zyan_to_sompi()`: fixed-point decimal parser (MED-02 fix)
- Splits on `.`, parses whole/fractional parts as integers
- Uses `checked_mul`/`checked_add` with `SOMPI_PER_ZYANYA`
- No `f64` in sompi conversion (display-only f64 for balance formatting is safe)

### CLI Commands
```bash
zyanya-wallet --generate-key                    # Address only (no secret)
zyanya-wallet --generate-key --show-secret      # Include secret key
zyanya-wallet --generate-mnemonic               # 24-word BIP-39
zyanya-wallet --import-mnemonic "<phrase>"      # Restore from seed
zyanya-wallet --devnet --rpcserver 127.0.0.1:18610  # Launch TUI
```

---

## 8. Explorer (`zyanya-explorer`)

**Tracking ID**: ZYN-P4a-003
**Last Verified**: 2026-08-05

### Architecture
- Axum web server, IPv6-only (`socket2` with `IPV6_V6ONLY=true`)
- RPC client connects to `zyanyad` gRPC
- Default listen: `[::]:8098`

### API Endpoints

**Read endpoints** (no auth):
| Endpoint | Purpose | Pagination |
|----------|---------|------------|
| `/api/blocks` | Recent blocks | Fixed 20 |
| `/api/block/:hash` | Block detail | — |
| `/api/contracts` | Contract list | `limit` (max 100) + `offset` |
| `/api/tokens` | Token list | `limit` (max 100) + `offset` |
| `/api/dag` | DAG graph | `limit` (max 100) + `offset` |
| `/api/contract/:addr/state?key=` | Contract storage | — |
| `/api/contract/:addr/code` | Contract bytecode | — |
| `/api/staking-info` | Staking info | — |
| `/api/token/:addr/graduation` | Graduation status | — |

**Write endpoints** (gated behind `ZYANYA_EXPLORER_ENABLE_WRITE=1`):
| Endpoint | Purpose |
|----------|---------|
| `/api/deploy-contract` | Deploy bytecode |
| `/api/invoke-contract` | Invoke entry point |
| `/api/call-contract` | Read-only call |
| `/api/unsigned-deploy-token` | Build unsigned deploy TX |
| `/api/unsigned-buy` | Build unsigned buy TX |
| `/api/unsigned-sell` | Build unsigned sell TX |
| `/api/unsigned-stake` | Build unsigned stake TX |
| `/api/unsigned-unstake` | Build unsigned unstake TX |
| `/api/unsigned-claim-rewards` | Build unsigned claim TX |
| `/api/submit-signed-tx` | Submit signed TX |
| `/api/token-transfer` | Token transfer |
| `/api/swap-on-dex` | DEX swap |

**Deprecated**: `/api/deploy-token` returns 410 GONE.

---

## 9. Build & Deployment

### Build Host
- **Primary**: minisforum (Windows 11, MSVC toolchain)
- **Path**: `C:\Users\Shawn\zyanya-build\rusty-spectre-git`
- **Cross-compile from Linux**: `x86_64-pc-windows-msvc` (NOT `gnu` — causes crash, Issue #1)

### ⚠️ Critical Build Notes
- Always pass `-p <package>` (ambiguous `zyanya-wallet` binary target)
- MSVC only — GNU toolchain causes NULL deref on Windows 11 build 26200
- GCM over SSH: `git config --global credential.credentialStore dpapi` (wincredman fails over SSH)

### Test Counts
| Suite | Count | Command |
|-------|-------|---------|
| VM unit | 19 | `cargo test -p zyanya-vm` |
| VM integration | 8 | (same command, separate binaries) |
| Consensus | 50 | `cargo test -p zyanya-consensus --lib` |
| Consensus core | 24 | `cargo test -p zyanya-consensus-core --lib` |
| **Total** | **101** | |

---

## 10. Audit Status

**Final Verdict**: PASS — ALL FINDINGS REMEDIATED (2026-08-05)

| ID | Severity | Finding | Status | Tracking ID |
|----|----------|---------|--------|-------------|
| CRIT-01 | Critical | SStore operand order | ✅ False positive | ZYN-AUD-01 |
| HIGH-01 | High | HashMap iteration | ✅ Fixed (BTreeMap) | ZYN-AUD-02 |
| HIGH-02 | High | Unchecked arithmetic | ✅ Fixed (checked_pow) | ZYN-AUD-03 |
| HIGH-03 | High | Secret keys in terminal | ✅ Fixed (--show-secret) | ZYN-AUD-04 |
| HIGH-04 | High | Unbounded API queries | ✅ Fixed (pagination) | ZYN-AUD-05 |
| MED-01 | Medium | Unbounded Pow gas | ✅ Fixed (dynamic gas) | ZYN-AUD-06 |
| MED-02 | Medium | Float precision | ✅ Fixed (fixed-point) | ZYN-AUD-07 |
| MED-03 | Medium | Hex parsing | ✅ Fixed (from_str_radix) | ZYN-AUD-08 |
| MED-04 | Medium | Unauthenticated endpoints | ✅ Fixed (env gate) | ZYN-AUD-09 |
| LOW-01 | Low | Key file permissions | ✅ Fixed (0o600) | ZYN-AUD-10 |
| LOW-02 | Low | RocksDB FD limits | ✅ Assessed safe | ZYN-AUD-11 |
| INFO-01 | Info | Genesis zero-premine | ✅ Verified | ZYN-AUD-12 |

Full report: [AUDIT.md](../../AUDIT.md) | Wiki: [Audit Results](wiki/Audit-Results.md)

---

## 11. Phase History

| Phase | Title | Status | Date | Commit |
|-------|-------|--------|------|--------|
| 1a | VM Seed Engine | ✅ Complete | 2026-07-29 | `90f7d40` |
| 1b | VM Expansion (opcodes, serialization) | ✅ Complete | 2026-07-29 | `90f7d40` |
| 1c | Consensus TX Payload Integration | ✅ Complete | 2026-07-29 | `90f7d40` |
| 1d | Contract State DB & Virtual Processor | ✅ Complete | 2026-07-29 | `90f7d40` |
| 2 | Real ZYAN custody in bonding curve | ✅ Complete | 2026-07-31 | `e1c198f` |
| 3 | ZYAN Staking + AMM graduation tracking | ✅ Complete | 2026-08-04 | `6f29c17` |
| 4a | AMM graduation + fee routing | ✅ Complete | 2026-08-05 | `c7eed8d` |
| — | Audit remediation (all 11 findings) | ✅ Complete | 2026-08-05 | `d02a110` |
| — | Wiki + master reference init | ✅ Complete | 2026-08-05 | `b7ee069` |

---

## 12. Incident Decision Tree

### Node won't start
1. Check FD limit: `ulimit -n` (needs ≥8192; Windows may show error 203 — non-fatal)
2. Check data dir: `--appdir` path exists and is writable
3. Check peer connectivity: `tailscale status` (all 3 nodes online?)

### Consensus fork suspected
1. Check BTreeMap: `grep BTreeMap consensus/src/model/stores/contract.rs` (should NOT contain HashMap)
2. Check checked arithmetic: `grep wrapping_ zyanya-vm/src/vm.rs` (should return nothing)
3. Compare node blue scores via RPC

### Can't push to GitHub from Windows
1. Check GCM store: `git config --global --get credential.credentialStore` (should be `dpapi`)
2. If SSH session: PAT required (wincredman doesn't work over SSH)
3. See `github-push` skill for non-interactive push procedure

### Smart contract execution fails
1. Check gas limit: ensure `max_gas` ≥ sum of opcode costs
2. Check storage keys: verify key numbers match contract state key map (Section 4)
3. Check entry point: verify EP number matches contract dispatch table
4. Check phase: bonding curve EP 4/5 rejected if phase ≠ 0; EP 7 rejected if phase ≠ 2

### Explorer returns 503 on write endpoints
1. Check env: `ZYANYA_EXPLORER_ENABLE_WRITE` must be `1` or `true`
2. This is by design — write endpoints are disabled on public deployments (MED-04 fix)

---

## 13. Customs Changelog

Every update to this document is logged here. Each entry declares what changed, its origin, and any "dutiable" impact.

| Date | Change | Origin | Impact |
|------|--------|--------|--------|
| 2026-08-05 | Initial creation — all sections | WO #20 | None (new doc) |
| 2026-08-05 | Added Phase 4a, audit remediation, GCM fix | WO #19, #20 | Updated sections 3-11 |

---

## Tracking ID Format

```
ZYN-P<phase>-<NN>      — Feature tracking (e.g., ZYN-P4a-001)
ZYN-AUD-<NN>           — Audit finding tracking (e.g., ZYN-AUD-03)
ZYN-WO-<NN>            — Work order (maps to WO # in Work-Orders.md)
```

Each tracking ID links the doc section to the commit SHA that implemented it. When a section is updated, the tracking ID is preserved and the commit SHA is updated.

---

## Staleness Rule

Each section header carries a "Last Verified" date. If the date is older than 7 days from today, the section should be marked ⚠️ stale and re-verified against source code using the `audit-check` skill.