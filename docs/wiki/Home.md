# Zyanya Blockchain

> **"Forever, Always"** — a high-performance Spectre/GhostDAG fork featuring an embedded smart-contract VM and custom token ecosystem.

## Quick Links

- [Architecture](Architecture.md) — BlockDAG consensus, crate structure, node architecture
- [Smart Contracts](Smart-Contracts.md) — VM opcodes, ZCL language, bonding curve, AMM graduation
- [Deployment](Deployment.md) — Build instructions, cross-compilation, 3-node testnet
- [Work Orders](Work-Orders.md) — Task tracking, work order history, current status
- [Audit Results](Audit-Results.md) — Security audit findings and remediation status

## What is Zyanya?

Zyanya is a UTXO-based blockDAG blockchain (forked from Kaspa/Spectre) with:

- **GhostDAG consensus** — parallel block inclusion with topological ordering
- **Embedded smart contract VM** — stack-based VM with gas metering, persistent state storage
- **Bonding curve token launch** — tokens launch on a bonding curve, automatically graduate to an AMM at 1B sompi reserve
- **0.3% protocol fee** — routed to staking rewards on every buy/sell/swap
- **ZYAN staking** — stake ZYAN to earn protocol fees from DEX activity
- **Zero premine** — genesis sends 50 ZYAN to an unspendable `OP_FALSE` script
- **IPv6-first** — explorer and node enforce IPv6-only listener semantics

## Network Info

| Network | Prefix | Default Port | Genesis |
|---------|--------|-------------|---------|
| Mainnet | ZYAN-MAINNET | 18110 | 50 ZYAN → OP_FALSE (unspendable) |
| Testnet | ZYNT-TESTNET | 18210 | 50 ZYAN → OP_FALSE (unspendable) |
| Devnet | — | 18610 | — |

## Testnet Nodes

| Node | OS | Tailscale IP | Role |
|------|-----|-------------|------|
| cachyos | Linux (CachyOS) | 100.124.134.6 | Build + dev |
| minisforum | Windows 11 | 100.83.211.115 | Build + deploy |
| scotthawk | Linux | 100.106.22.123 | Testnet peer |

## Current Status

**Phase 4a complete** — AMM graduation + fee routing live. All 11 security audit findings remediated. Ready for public testnet launch.