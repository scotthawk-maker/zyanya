# 🏛️ Zyanya Cypherpunk Launch Kit (Satoshi Edition)

> **Launch Policy**: Zero Social Media, Zero Marketing Hype, 100% Code & Mathematics  
> **Primary Distribution**: GitHub Releases & SourceForge Project Mirror  
> **Launch Date**: October 1, 2026 (00:00 UTC)  

---

## 1. The Cypherpunk Announcement

Modeled directly on Satoshi Nakamoto's January 9, 2009 Bitcoin release notice to the Cryptography Mailing List. Understated, factual, and strictly technical.

```text
Subject: Announcing Zyanya v1.0.0 — an open-source, peer-to-peer GhostDAG system

I've developed a new open-source P2P blockDAG cryptocurrency called Zyanya ($ZYN).
It is completely decentralized, with no central server or trusted parties,
and is engineered from inception to be operated directly by autonomous AI agents.

Key properties:
- Parallel GhostDAG consensus running at 1 block per second with sub-second finality
- Zero premine (the genesis transaction outputs 0 coins; initial circulating supply is 0)
- AstroBWTv3 memory-hard CPU proof-of-work (optimized for consumer CPUs)
- Dual-stream coinbase emission: 50 ZYAN/block (25 liquid, 25 12-month linear vesting lock)
- Subnetwork 3 constant-product AMM with 0.3% protocol fee staking rewards
- Native WebMCP (Model Context Protocol) gateway for autonomous coding agents
- Pure global unicast IPv6 transport (no NAT traversal or UPnP vulnerabilities)
- Mathematical lifetime cap of ~28.7 Billion ZYAN (smooth geometric halving)

Source code and standalone binaries (Windows x64 & Linux x86_64) are available at:
GitHub:      https://github.com/scotthawk-maker/zyanya
SourceForge: https://sourceforge.net/projects/zyanya/

The network is live and open to anyone who wishes to run a node or mine.
```

---

## 2. SourceForge Project Configuration

Hosting Zyanya on SourceForge preserves the historical open-source tradition and provides a secondary, independent mirror for miners and node operators.

### Project Metadata
* **Project Name**: `Zyanya`
* **Unix Name**: `zyanya`
* **Short Description**: Agent-Native GhostDAG Layer 1 Blockchain with Zero Premine
* **Categories**: 
  - Communications > Telephony / P2P
  - Security > Cryptography
  - Software Development > Build Tools
* **License**: MIT License / Apache License 2.0
* **Programming Language**: Rust (100%)

### File Distribution Structure on SourceForge
```text
/zyanya/
  ├── v1.0.0/
  │   ├── zyanya-v1.0.0-mainnet-windows-x64.zip
  │   ├── zyanya-v1.0.0-mainnet-windows-x64.zip.sha256
  │   ├── zyanya-v1.0.0-mainnet-linux-x86_64.tar.gz
  │   ├── zyanya-v1.0.0-mainnet-linux-x86_64.tar.gz.sha256
  │   ├── sha256sums-v1.0.0-mainnet.txt
  │   └── README.txt
  └── latest -> v1.0.0/
```

---

## 3. GitHub Distribution Checklist

On October 1, the launch requires exactly three actions on GitHub:

1. **Tag & Release:**
   - Tag: `v1.0.0-mainnet`
   - Attach binaries: `zyanya-v1.0.0-mainnet-windows-x64.zip` and `zyanya-v1.0.0-mainnet-linux-x86_64.tar.gz`
   - Attach `sha256sums-v1.0.0-mainnet.txt`
2. **Release Body:**
   - Copy contents from [`docs/RELEASE_NOTES_v1.0.0.md`](file:///C:/Users/Shawn/zyanya-build/rusty-spectre-git/docs/RELEASE_NOTES_v1.0.0.md)
3. **Genesis Mining Start:**
   - Begin block generation on seed nodes and open public P2P mesh on port `18111`

---

## 4. Why Zero Socials?

1. **Immunity to Regulatory Scrutiny:** No promotion, no securities claims, no marketing claims, no entity. Pure free software.
2. **Organic Meritocracy:** Only genuine developers, miners, and agent builders join the early network.
3. **AI-First Discovery:** Autonomous coding agents crawl GitHub, parse `llms.txt`, and interact with MCP servers. They do not get influenced by Twitter hype.
