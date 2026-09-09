# Native IPv6 P2P Network Specification

> **Transport Protocol**: Pure IPv6 Global Unicast (TCP)  
> **IPv4 Policy**: Strictly Disabled at Socket and Discovery Layer

---

## 1. Architectural Rationale

Zyanya requires native IPv6 connectivity for all network participants:
- **Global Addressability**: Eliminates NAT traversal complexities, UPnP vulnerabilities, and centralized relay servers.
- **Sybil Resistance**: The vast IPv6 address space ($2^{128}$) combined with AstroBWTv3 proof-of-work makes address spoofing and eclipse attacks economically prohibitive.
- **Agent Mesh Readiness**: Autonomous AI agents operate with end-to-end cryptographic addressability across global cloud and edge infrastructure.

---

## 2. Socket Configuration & Peering

When launching `zyanyad`, specify IPv6 socket bindings:

```bash
zyanyad --testnet \
  --listen=[::]:18211 \
  --rpclisten=[::]:18210 \
  --connect=[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211
```

### Filtering & Address Validation
- **IPv4-Mapped Banning**: All IPv4-mapped IPv6 addresses (`::ffff:0:0/96`) are automatically rejected during peer discovery.
- **Bogons & Loopbacks**: Private, link-local (`fe80::/10`), and loopback addresses are blocked from the external peer table.
- **Canonical Seed Node**: `[2606:8ac0:2615:79aa:5a47:caff:fe7b:d473]:18211`
