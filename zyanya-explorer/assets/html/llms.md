# Zyanya ,  LLM/Agent API Documentation

> Zyanya is an IPv6-native, agent-native, CPU-mineable blockDAG. This document provides detailed API documentation for AI agents to interact with the Zyanya blockchain programmatically.

## Quick Start for Agents

```
# Query the chain state
curl https://testnet.zyanya.scottcloudhawk.org/api/info

# Get recent blocks
curl https://testnet.zyanya.scottcloudhawk.org/api/blocks

# Deploy a bonding-curve token
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/deploy-token \
  -H 'Content-Type: application/json' \
  -d '{"name":"MyToken","symbol":"MTK","supply":1000000,"slope":2,"owner":"100","description":"A test token","twitter":"@mytoken"}'

# Buy tokens on the bonding curve (entry_point 4 = buy)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/invoke-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":4,"calldata":"100,10"}'

# Check the price (entry_point 6 = price)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/call-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":6,"calldata":""}'

# Sell tokens (entry_point 5 = sell)
curl -X POST https://testnet.zyanya.scottcloudhawk.org/api/invoke-contract \
  -H 'Content-Type: application/json' \
  -d '{"contract_address":"<TOKEN_ADDRESS>","entry_point":5,"calldata":"100,5"}'

# Get token metadata
curl https://testnet.zyanya.scottcloudhawk.org/api/token/<TOKEN_ADDRESS>/metadata
```

## API Reference

### GET /api/info
Returns the current chain state.
- Response: `{block_count, header_count, difficulty, network, is_synced, server_version, virtual_daa_score, past_median_time, sink_hash, peer_count, mempool_size, coin_supply_zyan, max_supply_zyan}`

### GET /api/blocks
Returns the 20 most recent blocks.
- Response: `[{hash, blue_score, daa_score, timestamp, tx_count, selected_parent}]`

### GET /api/block/:hash
Returns details for a specific block by its 64-character hex hash.

### GET /api/dag
Returns parallel GHOSTDAG structure and nodes.
- Query parameters: `limit` (number, default 20, max 100), `offset` (number, default 0)
- Response: `{nodes: [{hash, blue_score, parents, selected_parent}], sink_hash, total_blocks}`

### GET /api/contracts
Returns a list of all deployed contracts.

### GET /api/contract/:address/state
Query persistent storage key-value state of a ZCL smart contract.
- Query parameters: `key` (u64 string or 0x hex, default 0)
- Response: `{address, key, value}`

### GET /api/contract/:address/code
Query deployed ZCL bytecode hex and size for a smart contract address.
- Response: `{address, bytecode, size}`

### GET /api/tokens
Returns a list of all deployed tokens with their metadata (name, symbol, total_supply, owner_address).

### GET /api/token/:address/metadata
Returns the off-chain metadata for a token: `{name, symbol, description, twitter, telegram, website, icon_uri}`

### GET /api/token-balance
Query custom token balance for a holder address.
- Query parameters: `token=<contract_address>&holder=<holder_id>`
- Response: `{balance, holder, token}`

### GET /api/dex-reserves
Query DEX liquidity pool reserves (Reserve A, Reserve B, LP Supply).
- Query parameters: `dex=<dex_address>`
- Response: `{dex, reserve_a, reserve_b, lp_supply}`

### POST /api/deploy-token
Deploy a bonding-curve token with metadata.
- Body: `{name, symbol, supply, slope, owner, description, twitter, telegram, website, icon_base64}`
- Response: `{contract_address, name, symbol, description, socials, icon_uri, slope, supply, gasUsed}`

### POST /api/token-transfer
Transfer custom tokens from sender to recipient.
- Body: `{token, from, to, amount, gas}`
- Response: `{success, txHash, gasUsed}`

### POST /api/compile-contract
Compile ZCL contract source code into executable VM bytecode hex.
- Body: `{source: string}`
- Response: `{bytecode, gas_estimate}`

### POST /api/invoke-contract
Execute a state-changing contract call (buy, sell, transfer, init).
- Body: `{contract_address, entry_point, calldata, gas}`
- calldata format: comma-separated u64 values (e.g., "100,10" for caller=100, amount=10)
- Entry points for bonding-curve tokens: 0=init(slope), 1=transfer(from,to,amt), 4=buy(caller,k), 5=sell(caller,k)
- Response: `{success, returnValue, gasUsed, transactionId}`

### POST /api/call-contract
Execute a read-only contract call (balance_of, total_supply, price).
- Body: `{contract_address, entry_point, calldata, gas}`
- Entry points for bonding-curve tokens: 2=balance_of(addr), 3=total_supply(), 6=price()
- Response: `{executionSuccess, gasUsed, returnValue}`

### GET /token-icons/:filename
Returns a token icon PNG image.

> **Note on Testnet Write Access:** On the Zyanya testnet, `ZYANYA_EXPLORER_ENABLE_WRITE=1` is intentionally set so users can test the `/launch` token creator + bonding-curve trades without authentication. No authentication is required on the testnet. For mainnet, state-changing endpoints will require authentication + rate limiting.

## Bonding Curve Math
- price(supply) = slope * supply
- buy cost = slope * (2 * S * k + k^2) / 2  (where S = current supply, k = tokens to mint)
- sell refund = slope * (2 * S * k - k^2) / 2  (where S = supply before burn, k = tokens to burn)
- All arithmetic is checked (overflow/underflow reverts the transaction)

## Web MCP
The explorer serves `/webmcp.js` which registers 15 blockchain tools on `navigator.modelContext`:

### Query Tools (Read-Only)
1. **`get-chain-info`**: Query chain state (block count, DAA score, difficulty, supply, peers).
   - Input Schema: `{ type: "object", properties: {} }`
2. **`get-block`**: Query block details by 64-char hex hash or recent blocks.
   - Input Schema: `{ type: "object", properties: { blockHash: { type: "string" } } }`
3. **`get-dag-info`**: Query parallel GHOSTDAG structure and sink block.
   - Input Schema: `{ type: "object", properties: {} }`
4. **`get-contract-state`**: Query storage key-value state of a ZCL contract.
   - Input Schema: `{ type: "object", properties: { contractAddress: { type: "string" }, key: { type: "string" } }, required: ["contractAddress"] }`
5. **`get-contract-code`**: Query deployed bytecode hex and size.
   - Input Schema: `{ type: "object", properties: { contractAddress: { type: "string" } }, required: ["contractAddress"] }`
6. **`get-token-balance`**: Query custom token balance for holder.
   - Input Schema: `{ type: "object", properties: { tokenAddress: { type: "string" }, holder: { type: "string" } }, required: ["tokenAddress"] }`
7. **`get-dex-reserves`**: Query DEX liquidity pool reserves.
   - Input Schema: `{ type: "object", properties: { dexAddress: { type: "string" } }, required: ["dexAddress"] }`
8. **`ipv6-safety`**: Return IPv6 peer-to-peer security & firewall guidance.
   - Input Schema: `{ type: "object", properties: {} }`

### Deploy & Invoke Tools (Network Operations)
9. **`deploy-token`**: Deploy a bonding-curve token with metadata and socials.
   - Input Schema: `{ type: "object", properties: { name: { type: "string" }, symbol: { type: "string" }, supply: { type: "number" }, owner: { type: "string" }, slope: { type: "number" }, description: { type: "string" }, twitter: { type: "string" }, telegram: { type: "string" }, website: { type: "string" }, icon_base64: { type: "string" } }, required: ["supply"] }`
10. **`deploy-contract`**: Deploy compiled ZCL bytecode hex.
    - Input Schema: `{ type: "object", properties: { bytecode: { type: "string" }, gas: { type: "number" } }, required: ["bytecode"] }`
11. **`compile-contract`**: Compile ZCL source code into VM bytecode hex.
    - Input Schema: `{ type: "object", properties: { source: { type: "string" } }, required: ["source"] }`
12. **`invoke-contract`**: Invoke smart contract entry point with calldata (state-changing).
    - Input Schema: `{ type: "object", properties: { contractAddress: { type: "string" }, entryPoint: { type: "number" }, calldata: { type: "string" }, gas: { type: "number" } }, required: ["contractAddress"] }`
13. **`call-contract`**: Read-only virtual execution of smart contract function.
    - Input Schema: `{ type: "object", properties: { contractAddress: { type: "string" }, calldata: { type: "string" }, entryPoint: { type: "number" }, gas: { type: "number" } }, required: ["contractAddress"] }`
14. **`swap-on-dex`**: Swap tokens on a DEX liquidity pool.
    - Input Schema: `{ type: "object", properties: { dexAddress: { type: "string" }, tokenIn: { type: "string" }, amountIn: { type: "number" } }, required: ["dexAddress", "tokenIn", "amountIn"] }`
15. **`token-transfer`**: Transfer custom tokens to a recipient.
    - Input Schema: `{ type: "object", properties: { tokenAddress: { type: "string" }, from: { type: "string" }, to: { type: "string" }, amount: { type: "number" } }, required: ["tokenAddress", "to", "amount"] }`

Browser-based agents auto-discover these tools when visiting the explorer page.

## Network
- Testnet seed (P2P): `[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18211`
- Testnet RPC: `[2606:8ac0:2615:79aa:1a66:daff:fe99:31f7]:18210`
- Explorer: `https://testnet.zyanya.scottcloudhawk.org/`
- GitHub: `https://github.com/scotthawk-maker/zyanya`
- All connections are IPv6-only.