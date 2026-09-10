# WebMCP Agent Integration Guide

Zyanya is the world's first **Agent-Native Layer 1 Blockchain**. While human-centric blockchains prioritize flashy browser extensions and heavy React dashboards, Zyanya provides a sovereign, high-velocity **WebMCP (Model Context Protocol over HTTP)** gateway designed specifically for autonomous AI agents, LLM copilots, and robotic traders.

---

### 1. The Zero-GUI Philosophy

Modern blockchains suffer from frontend fragility: DNS hijacking, malicious browser extensions, CDN compromises, and abandoned graphical interfaces.

Zyanya eliminates this attack surface:
- **Machine Primitives Over Human GUIs**: The core consensus daemon (`zyanyad`) and gateway expose pure machine interfaces.
- **Agent Self-Assembly**: An AI agent can discover the network schema, inspect the DAG state, write a Subnetwork 3 ZCL smart contract, compile bytecode, and broadcast signed transactions without touching a human interface.
- **Frontend Agnosticism**: If a human wants a dashboard, their personal AI agent can generate an ephemeral, bespoke interface tailored to their exact specifications in seconds.

---

### 2. Protocol Endpoints & Discovery

Every sovereign Zyanya appliance hosts standardized machine endpoints:

- 📋 **`/mcp.json`**: Standardized Model Context Protocol manifest defining all callable tools, input JSON schemas, descriptions, and authentication parameters.
- ⚡ **`/mcp/rpc`**: JSON-RPC 2.0 execution endpoint supporting stateless batch and single tool dispatches.
- 📜 **`/llms.txt`**: Markdown summary structured for system prompts, RAG ingest, and context windows.
- 📐 **`/openapi.json`**: OpenAPI 3.1 schema for SDK autogeneration and REST integrations.

---

### 3. Core Machine Tools

Autonomous agents interact with the blockchain through native WebMCP tools:

- 🔍 **`zyanya_get_dag_info`**:
  • Returns current block height, virtual DAA score, tip hashes, network difficulty, and BPS velocity.
- 📦 **`zyanya_get_block`**:
  • Fetches full block headers, transactions, selected parent relations, and blue score metrics.
- 💰 **`zyanya_get_balance`**:
  • Queries UTXO set for confirmed and unconfirmed sompi balances for any `zyanya:` or `zyanyatest:` address.
- 💸 **`zyanya_send_transaction`**:
  • Constructs, signs, and broadcasts a standard UTXO payment transaction.
- 🛠️ **`zyanya_compile_contract`**:
  • Passes ZCL high-level contract syntax to the embedded Subnetwork 3 compiler, returning validated bytecode.
- 🚀 **`zyanya_deploy_contract`**:
  • Submits contract bytecode to Subnetwork 3 and instantiates a persistent contract address.
- 🔄 **`zyanya_call_contract`**:
  • Invokes a stateful method on an active contract (e.g. AMM swap, bonding curve buy/sell, staking deposit).

---

### 4. Interactive Execution Examples

#### Query DAG Info (cURL)
```bash
curl -X POST https://zyanya.scottcloudhawk.org/mcp/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "zyanya_get_dag_info",
      "arguments": {}
    }
  }'
```

#### Autonomous Python Integration
```python
import requests

MCP_ENDPOINT = "https://zyanya.scottcloudhawk.org/mcp/rpc"

def call_tool(tool_name: str, arguments: dict):
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }
    response = requests.post(MCP_ENDPOINT, json=payload, timeout=10)
    response.raise_for_status()
    return response.json().get("result")

# Get live consensus status
dag_info = call_tool("zyanya_get_dag_info", {})
print("Current DAA Score:", dag_info["virtual_daa_score"])
print("Network Hashrate Target:", dag_info["difficulty"])
```

---

### 5. Pairing with AI Assistants

Developers and miners can hook their favorite coding assistants directly into Zyanya:

- 🤖 **Google Antigravity / Claude Code / Cursor**:
  • Add `https://zyanya.scottcloudhawk.org/mcp.json` to your local MCP server configuration.
- 🤖 **Oh My Pi (OMP)**:
  • Delegate contract generation and live testnet verification directly through the local OMP agent CLI.
