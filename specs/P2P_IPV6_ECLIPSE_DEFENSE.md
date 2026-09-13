# 🛡️ Specification: IPv6 P2P Anti-Eclipse & Inbound Slot Defense

> **Status**: APPROVED (P0 Mainnet Blocker)  
> **Target Delivery**: September 23, 2026 (Testnet Verification)  
> **Mainnet Enforced**: October 1, 2026 (Genesis)  
> **Tracker**: #331 (Pre-Launch Security Sprint)  

---

## 1. Threat Model: IPv6 Inbound Starvation & Eclipse

In a pure IPv6 peer-to-peer network, an adversary controlling a single `/64` prefix possesses $2^{64}$ (18.4 quintillion) distinct IP addresses:
1. **Inbound Slot Exhaustion**: An attacker spins up virtual sockets across one `/64` to occupy 100% of a target node's inbound peer slots (default 128 connections).
2. **Information Eclipse**: By dominating inbound connections and combining with targeted outbound connection pressure, the attacker isolates the victim node (especially mining nodes).
3. **Mempool & Block Asymmetry**: The eclipsed miner only receives transactions relayed by the attacker, allowing the attacker to selectively censor counter-orders and manipulate batch settlement input sets.

---

## 2. Architectural Defenses

### A. Inbound Prefix Capping (`/64` and `/48`)
* **Strict Per-Prefix Limits**:
  * A node will accept a maximum of **1 active inbound connection per `/64` subnet**.
  * A node will accept a maximum of **4 active inbound connections per `/48` routing prefix**.
  * Additional connection handshakes from an already-represented prefix are rejected during initial TCP negotiation with zero buffer allocation.

### B. Inbound Eviction Policy (Prefix Diversity)
* When inbound slots reach capacity:
  * Incoming connections from previously unseen `/48` prefixes trigger eviction of an existing connection from the most heavily represented prefix family.
  * Ensures a diverse global topology naturally displaces localized clusters.

### C. AddressManager Netgroup Bucketing
* Addresses stored in the address manager table are partitioned into discrete netgroup buckets keyed by `(prefix_48, hash(prefix_64))`.
* Random selection for outbound attempts samples across buckets, guaranteeing that having millions of addresses in a single `/64` provides zero statistical advantage in peer selection.

---

## 3. Implementation Milestones

* **Milestone 1 (Sept 18 - Sept 20)**:
  - Implement prefix extraction helpers (`ipv6_to_netgroup_64` and `ipv6_to_netgroup_48`) in `components/addressmanager`.
  - Add `InboundPrefixLimiter` state to `components/connectionmanager`.
* **Milestone 2 (Sept 21 - Sept 23)**:
  - Integrate prefix checks into `p2p/src/adaptor.rs` connection listener.
  - Implement heuristic eviction prioritizing unrepresented ASNs and prefixes.
* **Milestone 3 (Sept 24 - Sept 27)**:
  - Sybil eclipse simulation on Testnet-10: spin up 1,000 virtual nodes within a single `/64` and verify victim node maintains 100% diversity.
* **Milestone 4 (October 1)**:
  - Mainnet genesis activation.
