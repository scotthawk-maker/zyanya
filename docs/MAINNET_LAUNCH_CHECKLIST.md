# 🚀 Zyanya ($ZYN) Mainnet Pre-Launch Checklist

> **Target Launch Date**: October 1, 2026 (00:00 UTC)  
> **Network Magic**: `ZYAN` (`0x5A, 0x59, 0x41, 0x4E`)  
> **Max Supply**: ~28.7 Billion ZYAN (Zero Premine, 50 ZYAN/block, 1 BPS GhostDAG)  
> **Transport**: Pure Global Unicast IPv6  

---

## 📋 Master Launch Status Overview

| Track | Category | Status | Verified By |
| :--- | :--- | :---: | :--- |
| **Track 1** | Consensus & Core Parameters | 🟢 READY | `verify_preflight_config.py` (51/51) |
| **Track 2** | Security & Determinism Audits | 🟢 READY | `AUDIT.md` (11/11 remediated) |
| **Track 3** | Subnetwork 3 DEX & Smart Contracts | 🟢 READY | 1.65M Swap Stress Test |
| **Track 4** | WebMCP & Agent Faucet Architecture | 🟢 READY | Genesis Spark & Installer Suite |
| **Track 5** | P2P Mesh & Seed Node Infrastructure | 🟡 IN PROGRESS | Multi-region IPv6 Seeds |
| **Track 6** | Mining Fleet & AstroBWTv3 Tooling | 🟢 READY | AstroBWTv3 CPU Miner & Scaling Verified |
| **Track 7** | Release Packaging & Binaries (v1.0.0) | ⚪ PENDING | GitHub Release Assets & Hashes |
| **Track 8** | Community, PR & Launch Blast | ⚪ PENDING | `COMMUNITY_LAUNCH_KIT.md` |

---

## Track 1: Consensus & Core Network Parameters
- [x] **Zero-Premine Genesis Block**:
  - `genesis_txs[0].outputs.is_empty()` verified (0 pre-allocated outputs).
  - Coinbase payload stamped with `OP_FALSE` marker and `ZYAN-MAINNET`.
  - Initial circulating supply at T=0 is strictly `0 ZYAN`.
- [x] **Network Magic & Port Isolation**:
  - Mainnet magic `0x5A, 0x59, 0x41, 0x4E` (`ZYAN`).
  - Mainnet ports: P2P `18111`, RPC `18110`, Borsh RPC `19110`, JSON-RPC `20110`.
  - Strict isolation from testnet (`18210/18211`) and devnet (`18610/18611`).
- [x] **Smooth Deflationary Emission Schedule**:
  - `MAINNET_SUBSIDY_BY_MONTH_TABLE` (727 precomputed monthly steps).
  - Year 1 flat emission: 50 ZYAN/block for 31,449,600 blocks (~1.57B ZYAN).
  - Bit-deterministic integer arithmetic verified across compilers and OS.
  - Hard supply ceiling mathematically bounded at ~28.7 Billion ZYAN.
- [x] **Dual-Stream Coinbase Vesting**:
  - 50% liquid stream spendable after 100 block confirmations.
  - 50% vested stream locked in 12-month linear time-decay UTXO covenant.
- [x] **Address Prefix Enforcement**:
  - Mainnet addresses strictly prefixed with `zyanya:`.
  - Testnet addresses prefixed with `zyanyatest:`.

---

## Track 2: Security & Determinism Audits
- [x] **Consensus Determinism (AUDIT.md)**:
  - CRIT-01: `SStore` stack order verified correct.
  - HIGH-01: Replaced `HashMap` with deterministic `BTreeMap` in state storage.
  - HIGH-02: All VM arithmetic guarded with checked operations (`checked_add`, `checked_mul`, `checked_pow`).
  - MED-01: Dynamic gas metering on `Pow` opcodes to prevent CPU resource exhaustion.
  - MED-02: Subnetwork 3 fixed-point integer math to eliminate float rounding divergence.
- [x] **API & Key Security**:
  - HIGH-03: Wallet CLI masks private keys by default (requires explicit `--show-secret`).
  - HIGH-04: RPC pagination and rate limiting against DoS attacks.
  - MED-04: State-changing endpoints disabled by default on public nodes (`ZYANYA_EXPLORER_ENABLE_WRITE=false`).
  - LOW-01: Mnemonic and keyfiles enforced to POSIX `0600` permissions.

---

## Track 3: Subnetwork 3 DEX & Smart Contracts
- [x] **Core Contracts Validated**:
  - `dex.zcl`: Automated market maker constant-product pool factory.
  - `bonding_curve.zcl`: Autonomous pump.fun-style price discovery curve.
  - `staking.zcl`: Protocol fee routing (0.3% swap fee distributed to sovereign ZYAN stakers).
- [x] **Graduation Thresholds**:
  - Bonding curve auto-migration to AMM pool at 10 ZYAN (1,000,000,000 sompi) reserve.
  - 0% founder access / permanent liquidity locks.
- [x] **High-Concurrency Stress Testing**:
  - >1,650,000 continuous swaps processed on Subnetwork 3 container.
  - Zero state corruption, zero arithmetic panics, 8.0 TPS steady-state throughput.

---

## Track 4: WebMCP & Agent-Native Ecosystem
- [x] **WebMCP Specification & Tools**:
  - `docs/ai/mcp.json` published with full tool schemas.
  - `docs/ai/llms.txt` and `docs/ai/llms-full.txt` updated for AI agents (Claude, Cursor, Antigravity, Ollama).
  - Tools registered: `zyanya_claim_genesis_spark`, `zyanya_get_pioneer_node_status`.
- [x] **Genesis Spark Node Reward**:
  - 2.0 ZYAN liquid gas + 8.0 ZYAN auto-staked in AMM fee vault.
  - Sybil defenses: Hardware P2P node ID binding, single-claim nonce, 72-hour staking cooldown.
- [x] **Universal Installers**:
  - PowerShell (`install.ps1`): Windows automated node bootstrap & wallet generation.
  - Bash (`install.sh`): Linux automated node bootstrap & systemd service setup.
  - Terminal completion cards formatted with WebMCP copy-paste prompts.

---

## Track 5: P2P Mesh & Seed Node Infrastructure
- [ ] **Multi-Region Seed Deployments**:
  - [x] US East Node (Core Homelab / `10.10.1.250`).
  - [ ] London Seed Node (Vultr Canary Wharf - IPv6).
  - [ ] Tokyo Seed Node (Vultr Minamishinagawa - IPv6).
- [ ] **DNS Seeders & AAAA Records**:
  - Set up `seed.zyanya.org` and `seed2.zyanya.org` DNS AAAA records pointing directly to IPv6 seeds.
  - Hardcode fallback bootstrap peers in `consensus/core/src/config/params.rs`.
- [ ] **Firewall & IPv6 Validation**:
  - Ensure TCP port `18111` (P2P) and `18110` (RPC) open on all seeds.
  - Verify `IPV6_V6ONLY=true` socket option is active and blocks `::ffff:0:0/96`.

---

## Track 6: Mining Fleet & AstroBWTv3 Tooling
- [x] **CPU Miner Validation**:
  - Verified standalone `zyanya-miner` against `zyanyad --mining-address zyanya:...`.
  - Tested mining thread scaling (`--threads N`, `--cpu-percent N`, `--dynamic`) on multi-core Ryzen architecture (AMD Ryzen 7 8700G, 16 CPUs).
- [x] **Initial Difficulty Calibration**:
  - Genesis block target bits: `536999497` (`0x2001fc49`).
  - DAA sampling window calibrated to smoothly absorb early mining hashrate without deadlocks.
- [x] **Mining Documentation & Community Guide**:
  - Hardened commands in `docs/MINING_QUICKSTART.md` (Windows PowerShell + Linux bash).
  - Added systemd service unit (`zyanya-miner.service`) for 24/7 background CPU mining on Ubuntu/Debian/CachyOS.
  - Documented Dual-Stream coinbase payout (25 ZYAN liquid + 25 ZYAN 12-month lock).

---

## Track 7: Release Packaging & Binaries (v1.0.0-mainnet)
- [ ] **Cross-Platform Compilation**:
  - [ ] Linux x86_64 (`x86_64-unknown-linux-gnu`): `zyanyad`, `zyanya-miner`, `zyanya-wallet`, `zyanya-explorer`.
  - [ ] Windows x64 (`x86_64-pc-windows-msvc`): `zyanyad.exe`, `zyanya-miner.exe`, `zyanya-wallet.exe`, `zyanya-explorer.exe`.
- [ ] **Release Archive Generation**:
  - Package `zyanya-v1.0.0-linux-x86_64.tar.gz`.
  - Package `zyanya-v1.0.0-windows-x64.zip`.
  - Generate and verify SHA256 checksums (`sha256sum.txt`).
- [ ] **GitHub Release & Git Tagging**:
  - Create annotated git tag `v1.0.0-mainnet`.
  - Draft GitHub release notes with binary download links, release hashes, and verification commands.

---

## Track 8: Community, PR & Launch Broadcast
- [ ] **Social Media & Developer Outreach**:
  - Twitter/X launch thread from `docs/COMMUNITY_LAUNCH_KIT.md`.
  - Discord `#announcements` publication.
  - Reddit r/CryptoTechnology and Hacker News "Show HN: Zyanya — GhostDAG L1 Driven by AI Coding Agents" submissions.
- [ ] **Public Explorer Deployment**:
  - Deploy public block explorer at `https://explorer.zyanya.org` (or IPv6 direct).
  - Enforce `ZYANYA_EXPLORER_ENABLE_WRITE=false` for public safety.
- [ ] **T-Minus Countdown Schedule**:
  - **T-14 Days (Sept 17)**: Staging rehearsal with external miners on isolated test subnet.
  - **T-7 Days (Sept 24)**: Seed node infrastructure freeze and DNS verification.
  - **T-3 Days (Sept 28)**: Final binary release publishing on GitHub.
  - **T-1 Day (Sept 30)**: Community announcements and node operator preparation.
  - **T-0 (Oct 1, 00:00 UTC)**: Genesis launch, block generation commences, mining goes live.
