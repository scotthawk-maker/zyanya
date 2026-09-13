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
| **Track 5** | P2P Mesh & Seed Node Infrastructure | 🟢 READY | Multi-Region Seeds, DNS & Port Matrix (64/64) |
| **Track 6** | Mining Fleet & AstroBWTv3 Tooling | 🟢 READY | AstroBWTv3 CPU Miner & Scaling Verified |
| **Track 7** | Release Packaging & Binaries (v1.0.0) | 🟢 READY | Windows ZIP Bundled, Hashes & Release Notes |
| **Track 8** | Satoshi Stealth Launch | 🟢 READY | GitHub & SourceForge Only (Zero Socials/Hype) |
| **Track 9** | Scoped Agent Session Keys | 🟡 IN PROGRESS | `specs/SESSION_KEYS_SPEC.md` (P0 Blocker, Target: Sept 21) |

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
- [x] **Multi-Region Seed Deployments**:
  - US East Node (Core Homelab / `10.10.1.250` / `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18111`).
  - London Seed Node (Vultr Canary Wharf - IPv6 port `18111`).
  - Tokyo Seed Node (Vultr Minamishinagawa - IPv6 port `18111`).
- [x] **DNS Seeders & AAAA Records**:
  - Official DNS seeders defined in `consensus/core/src/config/params.rs`: `mainnet-dnsseed-1.zyanya-network.org`, `mainnet-dnsseed-2.zyanya-network.org`, `mainnet-dnsseed-3.zyanya-network.xyz`.
  - Canonical port matrix and dual-stack isolation verified.
- [x] **Firewall & IPv6 Validation**:
  - Canonical ports defined and verified: P2P `18111`, gRPC `18110`, Borsh wRPC `19110`, JSON wRPC `20110`.
  - Pure IPv6 Global Unicast transport invariant and firewall rules (`ufw`, `netsh`) fully documented in `docs/NETWORK.md`.
  - Filtering for IPv4-mapped addresses (`::ffff:0:0/96`), bogons, and link-local active.

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
- [x] **Cross-Platform Compilation**:
  - [x] Windows x64 (`x86_64-pc-windows-msvc`): `zyanyad.exe`, `zyanya-miner.exe`, `zyanya-wallet.exe`, `zyanya-query.exe`, `zyanya-explorer.exe`.
  - [x] Automated release packaging engine: `scripts/package_release.py`.
- [x] **Release Archive Generation**:
  - [x] Packaged `zyanya-v1.0.0-mainnet-windows-x64.zip` (32.96 MB).
  - [x] Generated SHA256 checksums (`sha256sums-v1.0.0-mainnet.txt`): `adbc9c41afbb048782e2e42b49a9014089d5826e9d74a1b0b70c3d33fbd120e9`.
- [x] **GitHub Release Notes & Documentation**:
  - [x] Drafted official release notes in `docs/RELEASE_NOTES_v1.0.0.md` with SHA256 verification commands and 60-second quickstarts.

---

## Track 8: Satoshi-Style Stealth Launch (GitHub & SourceForge)
- [x] **Zero Social Media Policy Enforced**:
  - Strictly no Twitter/X hype threads, paid influencers, or vanity social campaigns.
  - Cypherpunk release notice drafted in `docs/COMMUNITY_LAUNCH_KIT.md` (modeled on Satoshi's 2009 Cryptography Mailing List announcement).
- [x] **Primary Distribution Channels**:
  - [x] GitHub Releases: Tag `v1.0.0-mainnet`, official binaries, and SHA256 verification hashes.
  - [x] SourceForge Mirror: Independent FOSS mirror project configuration (`sourceforge.net/projects/zyanya`).
- [x] **Public Explorer & WebMCP Gateway**:
  - [x] Independent explorer live at `https://zyanya.scottcloudhawk.org`.
  - [x] Read-only safety default (`ZYANYA_EXPLORER_ENABLE_WRITE=false`).
- [x] **Launch Day Sequence (Oct 1, 00:00 UTC)**:
  - Publish GitHub release & SourceForge tarballs/zips.
  - Seed nodes start generating initial blocks with zero premine.
  - Community miners connect and begin discovering blocks organically.
