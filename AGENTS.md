# Autonomous Agent Integration Guide (AGENTS.md)

> **Mandate**: Zyanya is an Agent-Native Layer 1 blockchain designed for autonomous machine-to-machine operation. Autonomous AI agents interact directly with the network via WebMCP without requiring human intervention or graphical user interfaces.

---

## 1. System Prompt Context for AI Agents

When configuring autonomous coding, trading, or governance agents to interact with Zyanya, inject the following system prompt context:

```text
You are an autonomous agent connected to the Zyanya ($ZYN) Layer 1 blockchain.
Zyanya is an IPv6-native, parallel Block-DAG (GhostDAG 1 BPS) with Subnetwork 3 ZCL smart contracts.
You interact with the node via the sovereign WebMCP JSON-RPC interface at:
  Endpoint: https://zyanya.scottcloudhawk.org/mcp/rpc
  Discovery: https://zyanya.scottcloudhawk.org/mcp.json
Your capabilities include querying consensus state, estimating mining difficulty, compiling and deploying ZCL smart contracts, and executing UTXO transactions.
Always adhere to gas limits, verify nonces, and enforce fail-closed execution safety.
```

---

## 2. WebMCP Protocol Discovery

Agents can dynamically discover live tools, schemas, and parameter constraints at runtime:

```bash
curl -6 https://zyanya.scottcloudhawk.org/mcp.json
```

Or query the full OpenAPI 3.1 specification:

```bash
curl -6 https://zyanya.scottcloudhawk.org/openapi.json
```

---

## 3. Tool Calling Contracts

All tool calls are dispatched via JSON-RPC 2.0 to `/mcp/rpc`:

### `zyanya_get_dag_info`
Fetches real-time consensus parameters, block counts, tip hashes, and DAA score.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "zyanya_get_dag_info",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "network_name": "testnet-10",
    "block_count": 482910,
    "header_count": 482910,
    "tip_hashes": ["8f1d3c..."],
    "difficulty": 1.0,
    "past_median_time": 1725916000000,
    "virtual_parent_hashes": ["a2e4b1..."],
    "daa_score": 482910
  }
}
```

---

### `zyanya_compile_contract`
Compiles high-level ZCL source code into deterministic bytecode for Subnetwork 3.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "zyanya_compile_contract",
    "arguments": {
      "source_code": "contract Counter { state val: u64; pub fn inc() { self.val = self.val + 1; } }"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "bytecode_hex": "030100000000000001015b",
    "gas_estimate": 12500,
    "abi": {
      "functions": [{"name": "inc", "params": [], "return": "void"}]
    }
  }
}
```

---

### `zyanya_send_transaction`
Submits a signed UTXO transaction directly to the mempool for 1 BPS GhostDAG inclusion.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "zyanya_send_transaction",
    "arguments": {
      "raw_tx_hex": "01000000..."
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "tx_id": "4b2c1f8a9e0d...",
    "status": "ACCEPTED",
    "subnetwork_id": 3
  }
}
```

---

## 4. Agent Safety Invariants

Autonomous agents interacting with Zyanya must uphold the following safety rules:

1. **Deterministic Gas Budgets**: Always set an explicit `gas_limit` on contract interactions. Never leave gas unmetered.
2. **Fail-Closed Execution**: If an RPC call returns a timeout or ambiguous result, re-query the transaction by hash (`tx_id`) before submitting a duplicate order to avoid double-spend or double-mint scenarios.
3. **Local Key Protection**: Private keys must remain in local secure memory with zeroization upon process exit. Never transmit private keys over RPC.
