# Security & Architecture Review: Section 4 (Cryptography & Agent Session Keys)

**Date:** September 24, 2026  
**Target:** Zyanya Blockchain (Track 9)  
**Scope:** `wallet/core/src/session.rs`, `cli/src/modules/session.rs`, `zyanya-explorer/src/api.rs`, `wallet/core/src/message.rs`  
**Verdict:** ⛔ NO-GO (Critical Vulnerabilities Identified)

---

## Executive Summary & Audit Scorecard

Ahead of the October 1, 2026 Mainnet Launch, a comprehensive line-by-line security review of the Scoped Agent Session Keys (Track 9) was conducted. The audit revealed **two CRITICAL** flaws in the architectural enforcement of session policies, one **HIGH** severity canonicalization vulnerability, and one **LOW** severity operational hygiene issue. 

The most severe finding allows an autonomous agent to bypass all spending limits and destination whitelists by spoofing JSON-RPC arguments, leading to potential full hot-wallet compromise. Due to these findings, the current implementation is deemed **unsafe for production**.

| Category | Finding | Severity |
|----------|---------|----------|
| Architecture / Policy | L1 Bypass via RPC Argument Spoofing | **CRITICAL** |
| Cryptography / State | Session Limit Reset via Unsigned Volatile Fields | **CRITICAL** |
| Cryptography / Tamper | Canonical Payload Collision (Newline Injection) | **HIGH** |
| Memory Safety / UX | Ephemeral Private Key Exposure via `stdout` | **LOW / ADVISORY** |

---

## Deep-Dive Findings & Recommendations

### 1. [CRITICAL] L1 Bypass via RPC Argument Spoofing
**Location:** `zyanya-explorer/src/api.rs` (in `zyanya_send_transaction`)

**Vulnerability:**  
The `zyanya_send_transaction` WebMCP RPC endpoint enforces session policies by comparing the limits against the user-supplied `amount_sompi` and `recipient` JSON arguments. Once the policy check passes, the endpoint blindly submits the accompanying `tx_hex` (containing the signed UTXO transaction) to the L1 network. 
The system does not verify that the actual outputs inside `tx_hex` match the validated `amount_sompi` and `recipient`. An agent can pass an authorized amount/recipient in the JSON-RPC wrapper to pass the policy check, but embed a `tx_hex` that spends millions of ZYAN to a completely unauthorized address. The L1 consensus network inherently lacks session policy awareness and will process the validly signed transaction.

**Recommendation:**  
Before submission, the RPC node must parse `rpc_tx` from `tx_hex` and strictly calculate the total Sompi output intended for the requested `recipient`. 
- Ensure that the sum of the outputs directed to the `recipient` exactly matches `amount_sompi`.
- Ensure that any other outputs strictly return change back to the ephemeral public key or master key. 
- The policy enforcement MUST be applied against the cryptographically parsed transaction outputs, not the unauthenticated JSON-RPC arguments.

---

### 2. [CRITICAL] Agent Session Spend Reset via Unsigned Velocity Fields
**Location:** `wallet/core/src/session.rs` and `zyanya-explorer/src/api.rs`

**Vulnerability:**  
The `SessionCertificate` contains `current_daily_spent` and `daily_window_start`. These stateful variables are incorrectly excluded from the `payload_for_signing` canonical representation. Since they are unsigned, a malicious agent can modify them (e.g., setting `current_daily_spent = 0`) without invalidating the master wallet's Schnorr signature.
While the RPC endpoint attempts to track session velocity in an in-memory lock (`client.session_velocities.lock().await`), this map is entirely volatile. In a load-balanced, multi-node deployment—or following a simple service restart—the map resolves to `None`. The node then defaults to trusting the forged values within the JSON certificate, allowing agents to continuously reset their 24-hour spend limits.

**Recommendation:**  
- **Remove** `current_daily_spent` and `daily_window_start` from the `SessionCertificate` struct definition. Certificates must only hold static policy constraints (limits, TTL, whitelist).
- **Persist** session velocities server-side. The RPC layer must rely on a persistent data store (e.g., PostgreSQL, LevelDB) for `vel_map` rather than volatile memory, ensuring state continuity across restarts and load-balanced requests.

---

### 3. [HIGH] Canonical Payload Collision (Newline Injection)
**Location:** `wallet/core/src/session.rs` (in `payload_for_signing`)

**Vulnerability:**  
The `payload_for_signing` method constructs a canonical string for Schnorr signing by directly concatenating fields with newline `\n` characters:
```rust
format!("session_id:{}\nagent_label:{}\npublic_key:{}...", self.session_id, self.agent_label, self.public_key)
```
There is no sanitization of `session_id` or `agent_label`. An attacker who can influence the agent label during creation can inject newlines to spoof subsequent payload fields. 
For example, an `agent_label` of `"evil\npublic_key:02abcd...\nmax_per_tx:999999999999"` will overwrite the parser's intended field boundaries. If the master wallet blindly signs this generated payload, the policy limits are permanently subverted.

**Recommendation:**  
- Implement strict input sanitization to reject `\n` and `:` characters in `session_id` and `agent_label`.
- Alternatively, transition from string formatting to a robust, deterministic serialization format (such as Borsh or Canonical JSON) for signature payloads to prevent injection attacks natively.

---

### 4. [LOW] Ephemeral Private Key Exposure via `stdout`
**Location:** `cli/src/modules/session.rs`

**Vulnerability:**  
While the `Secret` struct excellently implements the `Zeroize` and `Drop` traits for memory hygiene, the `session create` CLI command actively subverts this protection by printing the `privkey_hex` in cleartext to `stdout`.
Printing key material directly to the terminal guarantees exposure to shell history, CI/CD pipeline logs, and screen scraping malware, breaking the containerized isolation of the `ScopedSessionKey`.

**Recommendation:**  
- Instead of printing to `stdout`, the CLI should write the ephemeral private key directly into a restricted file with strict permissions (e.g., `chmod 600`), alongside or within the `cert_json`.
- If standard output must be used, add a distinct warning and require the user to pass a `--show-secret` flag.

---

## Cryptographic Mathematical Soundness Validation
- **Arithmetic:** The usage of `checked_add` and `saturating_add/sub` effectively mitigates buffer overflows and underflows in velocity tracking and TTL bounding.
- **Fail-Closed Verification:** `verify_message` accurately applies `secp256k1` XOnly Schnorr signature validation, appropriately mapping all anomalies to an `InvalidSignature` error. Master keys are securely maintained within `cli/src/modules/session.rs` and do not leak into the `ScopedSessionKey` container.

## Official Verdict
The Section 4 Cryptography & Agent Session Keys implementation receives a **NO-GO** status. The critical issues mapping to L1 policy bypass and state-tampering must be patched, heavily re-tested, and audited before considering Mainnet deployment.