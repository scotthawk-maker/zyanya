# Native IPv6 P2P Network Specification

> **Transport Protocol**: Pure IPv6 Global Unicast (TCP)  
> **IPv4 Policy**: Strictly Disabled at Socket and Peer Discovery Layer  
> **Target Launch**: October 1, 2026  

---

## 1. Architectural Rationale

Zyanya requires native IPv6 connectivity for all network nodes, miners, and validators:
- **Global End-to-End Addressability**: Eliminates NAT traversal bottlenecks, STUN/TURN relays, and UPnP vulnerabilities. Every node has a distinct global address.
- **Sybil Resistance via Sparse Addressing**: The vast IPv6 address space ($2^{128}$) combined with AstroBWTv3 proof-of-work renders IP-spoofing and eclipse attacks economically impractical.
- **Agent Mesh Topology**: Autonomous AI coding agents (Antigravity, Cursor, Claude, Ollama) interact directly with local or remote nodes without fragile port-forwarding.

---

## 2. Canonical Network Port Matrix

| Network | Network Magic | P2P Port (TCP) | gRPC Port (TCP) | Borsh wRPC | JSON wRPC | Address Prefix |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Mainnet** | `ZYAN` (`0x5A59414E`) | **18111** | **18110** | **19110** | **20110** | `zyanya:` |
| **Testnet-10** | `ZYNT` (`0x5A594E54`) | **18211** | **18210** | **19210** | **20210** | `zyanyatest:` |
| **Devnet** | `ZYND` (`0x5A594E44`) | **18611** | **18610** | **19610** | **20610** | `zyanyadev:` |
| **Simnet** | `ZYNS` (`0x5A594E53`) | **18511** | **18510** | **19510** | **20510** | `zyanyasim:` |

---

## 3. Official DNS Seeders & AAAA Records

Nodes automatically discover initial peers at boot by querying the hardcoded DNS seeders in `consensus/core/src/config/params.rs`:

### Mainnet Seeders
- `mainnet-dnsseed-1.zyanya-network.org`
- `mainnet-dnsseed-2.zyanya-network.org`
- `mainnet-dnsseed-3.zyanya-network.xyz`

### Testnet Seeders
- `testnet-dnsseed-1.zyanya-network.org`
- `testnet-dnsseed-2.zyanya-network.org`
- `testnet-dnsseed-3.zyanya-network.xyz`

Each domain resolves exclusively to IPv6 `AAAA` records of active high-uptime seed nodes.

---

## 4. Tri-Region Seed Node Infrastructure

Zyanya operates dedicated anchor nodes strategically positioned across 3 global regions:

1. 🇺🇸 **US East Anchor (Homelab Core / Virginia)**:
   - Primary Seed Node: `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18111`
   - Latency to EU: ~96 ms
2. 🇬🇧 **London Relay (Vultr Canary Wharf - Western Europe)**:
   - European Seed Node: Port `18111` (P2P)
   - Cross-connect to US and Asia
3. 🇯🇵 **Tokyo Relay (Vultr Minamishinagawa - Asia-Pacific)**:
   - Asia-Pacific Seed Node: Port `18111` (P2P)
   - Latency to US: ~199 ms

---

## 5. Socket Configuration & Peering Commands

### Mainnet Node Launch
```bash
# Launch with explicit IPv6 binding and bootstrap seed
./zyanyad \
  --listen=[::]:18111 \
  --rpclisten=[::]:18110 \
  --connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18111 \
  --utxoindex
```

### Testnet Node Launch
```bash
./zyanyad --testnet \
  --listen=[::]:18211 \
  --rpclisten=[::]:18210 \
  --connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211 \
  --utxoindex
```

---

## 6. Firewall & Network Security Configuration

### Linux (UFW)
Open the required P2P and RPC ports for your role:

```bash
# Allow P2P traffic from any IPv6 host
sudo ufw allow in proto tcp to any port 18111 comment "Zyanya Mainnet P2P"

# Restrict RPC port to localhost / trusted mesh subnet
sudo ufw allow in proto tcp from ::1 to any port 18110 comment "Zyanya Mainnet gRPC Local"

# Public Explorer / WebMCP gateway (if hosting public surface)
sudo ufw allow in proto tcp to any port 8098 comment "Zyanya Explorer Web"
```

### Windows (PowerShell Administrator)
```powershell
# Inbound P2P
New-NetFirewallRule -DisplayName "Zyanya Mainnet P2P" -Direction Inbound -LocalPort 18111 -Protocol TCP -Action Allow

# Local gRPC RPC
New-NetFirewallRule -DisplayName "Zyanya Mainnet RPC" -Direction Inbound -LocalPort 18110 -Protocol TCP -Action Allow
```

---

## 7. Filtering & Address Validation

To preserve IPv6 integrity and protect node pools:
- **IPv4-Mapped Banning**: All IPv4-mapped IPv6 addresses (`::ffff:0:0/96`) are automatically rejected during peer discovery.
- **Bogon & Loopback Drops**: Private, link-local (`fe80::/10`), and loopback (`::1`) addresses are blocked from the external peer table.
- **Outbound Peer Diversity**: Nodes enforce subnet diversity (/48 IPv6 prefix isolation) when selecting outbound peers to prevent routing centralization.

---

## 8. Mesh Quality Assurance & Verification

To verify that your node is peering properly with the global mesh:

```bash
# Check connected peer count and IDs
./zyanya-query -r 127.0.0.1:18110 get-connected-peer-info

# Check synchronization and DAG sink blue score
./zyanya-query -r 127.0.0.1:18110 get-sink-blue-score

# Run automated tri-region mesh QA probe
python scripts/verify_mesh.py
```
