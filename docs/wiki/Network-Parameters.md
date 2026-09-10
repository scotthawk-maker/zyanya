# Zyanya Network Parameters & Wire Specifications

Zyanya operates a dual-network topology: **Testnet-10** (live testing ground at block 210,000+) and **Mainnet** (launching October 1, 2026). Both networks enforce pure-IPv6 transport and identical GHOSTDAG consensus rules.

---

### 1. Dual Network Overview

- 🦅 **Mainnet (Production Sovereign Chain)**:
  • Launch Target: October 1, 2026  
  • Network Identifier: `mainnet`  
  • Address Prefix: `zyanya:`  
  • Genesis Output: 50 ZYAN to `OP_FALSE`  
  • P2P Default Port: `18111`  
  • RPC gRPC Default Port: `18110`  
  • wRPC Borsh Port: `19110`  
  • wRPC JSON Port: `20110`  

- 🧪 **Testnet-10 (Live Active Testnet)**:
  • Status: Active (Block Height > 210,000)  
  • Network Identifier: `testnet-10`  
  • Address Prefix: `zyanyatest:`  
  • P2P Default Port: `18211`  
  • RPC gRPC Default Port: `18210`  
  • wRPC Borsh Port: `19210`  
  • wRPC JSON Port: `20210`  
  • Canonical Seed Peer: `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211`  

---

### 2. Pure IPv6 Transport Enforcement

Zyanya is built for the post-exhaustion Internet. IPv4 address space is vulnerable to carrier-grade NAT (CGNAT), centralized gateway filtering, and IP spoofing.

- **Socket Layer Policy**:
  • Nodes reject incoming IPv4 connection attempts at the TCP socket binding layer.  
  • Peer discovery packets containing IPv4 mapped addresses (`::ffff:0:0/96`) are dropped immediately.  
  • All nodes, validators, and miners must advertise Global Unicast IPv6 addresses (`2000::/3`).  
- **Firewall & Security**:
  • Open inbound TCP ports `18111` (Mainnet) or `18211` (Testnet) on your host firewall.  
  • No port forwarding or NAT traversal (UPnP) is required when using native IPv6.

---

### 3. Consensus Engine Constants

- **Block Formation Rate**: 1.0 Block Per Second (1 BPS)
- **GhostDAG G-Parameter (k-cluster)**: $k = 18$
- **Difficulty Adjustment Algorithm (DAA)**: Past median time window of 2,641 blocks
- **Pruning Point Depth**: 185,411 blocks (~2.1 days of continuous DAG history)
- **Coinbase Maturity**: 10 confirmations (~10 seconds)
- **Subnetwork ID for Smart Contracts**: Subnetwork `3` (ZCL Stack Virtual Machine)

---

### 4. Connection Strings & Seed Nodes

#### Connect to Testnet-10
```bash
./zyanyad --testnet \
  --listen=[::]:18211 \
  --rpclisten=[::]:18210 \
  --connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211 \
  --utxoindex
```

#### Connect to Mainnet (October 1)
```bash
./zyanyad \
  --listen=[::]:18111 \
  --rpclisten=[::]:18110 \
  --utxoindex
```
