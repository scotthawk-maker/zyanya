# 🛡️ Specification: Scoped Agent Session Keys & Spend Policy

> **Status**: APPROVED (P0 Mainnet Blocker)  
> **Target Delivery**: September 21, 2026 (Testnet Verification)  
> **Mainnet Enforced**: October 1, 2026 (Genesis)  
> **Tracker**: #330 (Pre-Launch Security Sprint)  

---

## 1. Problem Statement

Autonomous AI agents executing WebMCP directives (`zyanya_send_transaction`) risk catastrophic fund loss if provisioned with unconstrained root private keys:
1. **Prompt Injection Attacks**: Malicious text injected into agent context (poisoned websites, untrusted inputs) can coerce the model into signing unauthorized transfers.
2. **Agent Hallucinations**: Model reasoning bugs can trigger unexpected transaction volume, looping trades, or oversized slippage tolerance.
3. **Hot-Wallet Exposure**: Storing raw root private keys in memory or agent runtime environment variables creates a severe exfiltration surface.

---

## 2. Architecture: Ephemeral Session Certificates

Rather than granting agents raw root keys, the reference wallet (`zyanya-wallet`) and WebMCP gateway enforce cryptographically scoped session keys.

### Session Key Structure
A user generates a scoped session key from their master seed:
```bash
zyanya-wallet session-key create \
  --max-per-tx 50000000 \
  --daily-cap 500000000 \
  --whitelist "zyanya:qq8f...dex,zyanya:qq2a...bonding" \
  --expiry 86400
```

### Policy Constraints
1. **`max_spend_per_tx`**: Strict ceiling on total Sompi value moved in a single UTXO transaction. Any transaction exceeding this limit fails client-side verification before signature generation.
2. **`daily_velocity_limit`**: Rolling 24-hour spend meter tracked in the local wallet database. Exceeding the cumulative budget halts automated signing until human re-authorization.
3. **`contract_whitelist`**: The session key is restricted to signing transactions where outputs/calldata target pre-authorized Subnetwork 3 contract addresses.
4. **`ephemeral_lifetime`**: Hard UNIX timestamp expiration after which the session key becomes cryptographically invalid.

---

## 3. Implementation Milestones

* **Milestone 1 (Sept 14 - Sept 17)**:
  - Implement `SessionPolicy` and `SessionKey` structs in `wallet/core` and `wallet/keys`.
  - Add CLI command `zyanya-wallet session-key create` with interactive confirmation.
* **Milestone 2 (Sept 18 - Sept 21)**:
  - Integrate session policy validation into the local WebMCP RPC dispatcher (`/mcp/rpc`).
  - Enforce fail-closed policy checks: rejected transactions return JSON-RPC error `-32003 (Policy Violation)`.
* **Milestone 3 (Sept 22 - Sept 27)**:
  - Adversarial prompt-injection testnet trials on Testnet-10.
  - Verification of automated rate-limiting and daily velocity resets.
* **Milestone 4 (October 1)**:
  - Mainnet genesis activation with default session key requirement for all autonomous agent tooling.
