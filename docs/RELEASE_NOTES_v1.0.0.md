# 🚀 Zyanya Mainnet Release v1.0.0 (`v1.0.0-mainnet`)

> **Release Date**: October 1, 2026 (00:00 UTC)  
> **Consensus Engine**: GhostDAG ($k=18$, 1 BPS)  
> **Proof-of-Work**: AstroBWTv3 (Memory-hard CPU)  
> **Initial Premine**: Exactly **0 ZYAN (0.00%)**  
> **Terminal Max Supply**: ~28.7 Billion ZYAN  

---

## 🌟 Executive Summary

Zyanya is the world's first **AI-Agent-Native Layer 1 Blockchain**, combining high-throughput parallel GhostDAG consensus with native WebMCP (Model Context Protocol) endpoints. 

Designed from inception to be operated, explored, and extended by autonomous AI coding assistants (Antigravity, Cursor, Claude, local LLMs) as well as human developers, Zyanya eliminates brittle frontend lock-in by treating AI agents as first-class cryptographic citizens.

---

## 🔑 Key Features & Architecture

* ⚡ **1 Block Per Second (1 BPS):** Sub-second transaction confirmation times with topological DAG ordering.
* 🛡️ **Zero Premine & Cypherpunk Fair Launch:** The Genesis Block transaction outputs 0 pre-allocated coins, stamping `OP_FALSE` directly on-chain. Initial circulating supply is strictly **`0 ZYAN`**.
* 🪙 **Dual-Stream Coinbase Vesting:** 50% liquid stream immediately spendable upon block maturity (100 confirmations) + 50% vested stream locked in a 12-month linear time-decay UTXO covenant.
* 🤖 **Native WebMCP Integration:** First-class tool gateway (`docs/ai/mcp.json`) allowing AI agents to query DAG state, manage sovereign wallets, claim Genesis Spark node incentives, and deploy smart contracts.
* 🏦 **Subnetwork 3 AMM & Real Yield:** Constant-product DEX (`dex.zcl`) and autonomous bonding curve (`bonding_curve.zcl`) with a 0.3% protocol fee routed to stakers via `staking.zcl`.
* 🌐 **Pure IPv6 Global Unicast:** Built for modern global edge networking—no NAT traversal, no UPnP bugs, full /48 prefix diversity.

---

## 📦 Official Release Archives & Checksums

| Platform | Archive Filename | Format | SHA256 Checksum |
| :--- | :--- | :---: | :--- |
| **Windows x64** | `zyanya-v1.0.0-mainnet-windows-x64.zip` | ZIP | `adbc9c41afbb048782e2e42b49a9014089d5826e9d74a1b0b70c3d33fbd120e9` |
| **Linux x86_64** | `zyanya-v1.0.0-mainnet-linux-x86_64.tar.gz` | TAR.GZ | *(Cross-compiled & published via CI)* |

### Included Binaries
* `zyanyad`: Core GhostDAG consensus node daemon
* `zyanya-miner`: Standalone high-performance AstroBWTv3 CPU miner
* `zyanya-wallet`: BIP-39 mnemonic sovereign CLI wallet manager
* `zyanya-query`: Lightweight direct gRPC query tool
* `zyanya-explorer`: Built-in local blockDAG web explorer and WebMCP gateway

---

## ⚡ 60-Second Quickstart

### 1. Extract Archive
```powershell
# Windows PowerShell
Expand-Archive .\zyanya-v1.0.0-mainnet-windows-x64.zip -DestinationPath .
cd zyanya-v1.0.0-mainnet-windows-x64
```

### 2. Generate Sovereign Mining Address
```powershell
.\zyanya-wallet.exe new-address
# Output: zyanya:qrh5l43xvd05lq36g37z7009s42f8c5mcv9943x7a7d4p2a7eet9j6085a855
```

### 3. Launch Node Daemon
```powershell
.\zyanyad.exe --utxoindex
```

### 4. Start CPU Mining
```powershell
.\zyanya-miner.exe --cpu-percent 50 --mining-address <YOUR_ZYANYA_ADDRESS>
```

---

## 🔒 Verification Commands

To verify archive integrity before running:

```powershell
# Windows PowerShell
Get-FileHash .\zyanya-v1.0.0-mainnet-windows-x64.zip -Algorithm SHA256

# Linux
sha256sum zyanya-v1.0.0-mainnet-windows-x64.zip
```
