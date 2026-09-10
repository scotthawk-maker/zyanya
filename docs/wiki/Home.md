# Zyanya Sovereign Blockchain ($ZYN)

> **"Forever, Always"**  
> *The sovereign, agent-native Layer 1 blockchain powered by GhostDAG 1 BPS consensus, AstroBWTv3 democratic CPU proof-of-work, Subnetwork 3 ZCL stack smart contracts, and pure-IPv6 mesh transport.*

---

### 📚 Sovereign Documentation Chapters

- 🏛️ **[System Architecture](Architecture.md)**  
  Topological GhostDAG ordering ($k=18$), parallel block DAG engine, virtual processor, and RocksDB UTXO state storage.

- ⛏️ **[AstroBWTv3 Proof-of-Work](Proof-of-Work.md)**  
  Mathematical foundations of Burrows-Wheeler Transform sorting, branch-heavy memory hardness, ASIC/FPGA immunity, and CPU hardware tuning.

- 🛠️ **[Mining Operations Guide](Mining-Operations.md)**  
  60-second solo CPU mining setup, v0.4.0 release binaries, dynamic thread scaling, HiveOS / mmpOS flight sheets, and Stratum pool configuration.

- 🖥️ **[Node Deployment Runbook](Node-Deployment.md)**  
  Production node administration, systemd service units, pure-IPv6 socket binding, UTXO indexing, memory scaling, and health verification.

- 💎 **[Tokenomics & Monetary Policy](Tokenomics.md)**  
  Zero premine (`OP_FALSE` genesis), 50 ZYAN coinbase emission (50% liquid + 50% 12-month linear vest), and 0.3% protocol fee staking rewards.

- 📜 **[Subnetwork 3 ZCL Smart Contracts](Smart-Contracts.md)**  
  64-bit deterministic stack VM, gas metering, continuous bonding curves, constant-product AMM, and UTXO staking vaults.

- 🤖 **[WebMCP Agent Integration](WebMCP-Agent-Guide.md)**  
  The zero-GUI machine-native protocol: `/mcp.json` tool discovery, JSON-RPC 2.0 execution at `/mcp/rpc`, and LLM autonomous pairing.

- 🌐 **[Network Parameters & Wire Specs](Network-Parameters.md)**  
  October 1 Mainnet production configuration vs live Testnet-10 parameters, ports, seed nodes, and pure-IPv6 transport rules.

- 🛡️ **[Security Audit Scorecard](Audit-Results.md)**  
  Comprehensive audit findings breakdown and cryptographic verification for all 112 remediated vulnerabilities.

---

### 🌐 Dual Network Quick Reference

- 🦅 **Mainnet (Public Launch: October 1, 2026)**:
  • Network ID: `mainnet`  
  • Address Prefix: `zyanya:`  
  • Default Ports: P2P `18111` | gRPC `18110` | wRPC Borsh `19110` | wRPC JSON `20110`  
  • Launch Command: `./zyanyad --utxoindex`  

- 🧪 **Testnet-10 (Live Active Staging Mesh)**:
  • Network ID: `testnet-10`  
  • Address Prefix: `zyanyatest:`  
  • Current State: Block Height > 210,000  
  • Default Ports: P2P `18211` | gRPC `18210` | wRPC Borsh `19210` | wRPC JSON `20210`  
  • Launch Command: `./zyanyad --testnet --utxoindex`  
  • Canonical Seed Peer: `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211`  

---

### 📦 Official Pre-Compiled Releases (v0.4.0)

- 🐧 **Linux x86_64**: [`zyanya-v0.4.0-linux-x86_64.tar.gz`](https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-linux-x86_64.tar.gz)  
  `SHA256: cfa8cbdc75267298613999b907db8ebec7e07148431d181e6c7cfc723f9201b1`
- 🪟 **Windows x64**: [`zyanya-v0.4.0-windows-x64.zip`](https://zyanya.scottcloudhawk.org/releases/zyanya-v0.4.0-windows-x64.zip)  
  `SHA256: 9f62b1bd33d4569b07e7695743f5ec6b0732339332d2355c1a09a2b7b4a3dd19`
- 🔗 **GitHub Releases**: [v0.4.0 on GitHub](https://github.com/scotthawk-maker/zyanya/releases/tag/v0.4.0)

---

### 🌐 Official Network Resources

- 🌐 **Sovereign Portal**: [https://zyanya.scottcloudhawk.org](https://zyanya.scottcloudhawk.org)
- 📊 **Block Explorer**: [https://testnet.zyanya.scottcloudhawk.org](https://testnet.zyanya.scottcloudhawk.org)
- 🤖 **WebMCP Gateway**: [https://zyanya.scottcloudhawk.org/mcp/rpc](https://zyanya.scottcloudhawk.org/mcp/rpc)
- 📦 **Core Node Source**: [https://github.com/scotthawk-maker/zyanya](https://github.com/scotthawk-maker/zyanya)
- ⛏️ **Standalone Miner Source**: [https://github.com/scotthawk-maker/zyanya-miner](https://github.com/scotthawk-maker/zyanya-miner)
