# WebMCP Gateway Specification

> **Specification**: WebMCP Protocol v2.0 (OpenAPI 3.1 & Model Context Protocol JSON-RPC)  
> **Target**: Sovereign Autonomous AI Agent Infrastructure

---

## 1. Overview

The Zyanya WebMCP Gateway bridges standard JSON-RPC 2.0 clients and autonomous AI agents to the native Zyanya Layer 1 node. It serves two primary interfaces:

1. **Manifest Discovery (`/mcp.json`)**: Machine-readable catalog describing available tools, parameter types, capabilities, and system instructions.
2. **Execution Endpoint (`/mcp/rpc` & `/api/mcp/execute`)**: High-throughput JSON-RPC 2.0 handler executing node queries, smart contract compilation, transaction dispatch, and mining estimation.

---

## 2. HTTP Endpoints

| Path | Method | Purpose |
| :--- | :--- | :--- |
| `/mcp.json` | `GET` | Returns MCP tool manifest and capabilities |
| `/mcp/rpc` | `POST` | Standard JSON-RPC 2.0 tool execution endpoint |
| `/openapi.json` | `GET` | OpenAPI 3.1 schema for standard REST/agent clients |
| `/llms.txt` | `GET` | Curated machine-readable overview for LLM context injection |

---

## 3. Standard Tools Catalog

### Consensus & Node State
- `zyanya_get_dag_info`: Query block counts, tip hashes, difficulty, and DAA score.
- `zyanya_get_block`: Retrieve complete block header, parent DAG vertices, and transactions by hash.
- `zyanya_get_current_network`: Query network identifier, protocol version, and active magic.

### Smart Contracts & Subnetwork 3
- `zyanya_compile_contract`: Compile ZCL high-level contracts to deterministic 64-bit stack bytecode.
- `zyanya_deploy_contract`: Inscribe and instantiate a contract on Subnetwork 3.
- `zyanya_call_contract`: Execute a state-mutating or view method on a deployed contract.

### Mining & Proof-of-Work
- `zyanya_estimate_mining_target`: Estimate mining difficulty and yields based on AstroBWTv3 CPU thread counts.

---

## 4. Error Codes

WebMCP conforms to standard JSON-RPC 2.0 error specifications:

| Code | Meaning | Description |
| :--- | :--- | :--- |
| `-32700` | Parse Error | Invalid JSON received by the gateway |
| `-32600` | Invalid Request | JSON sent is not a valid JSON-RPC 2.0 payload |
| `-32601` | Method Not Found | Tool or method requested does not exist |
| `-32602` | Invalid Params | Missing required arguments or type mismatch |
| `-32603` | Internal Error | Upstream node communication error |
| `-32001` | Contract Revert | ZCL VM execution reverted during execution |
| `-32002` | Out of Gas | Gas limit exceeded before execution finished |
